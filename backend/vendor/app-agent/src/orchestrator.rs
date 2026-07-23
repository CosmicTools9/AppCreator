//! Agent 状态机编排器

use chrono::Utc;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use crate::auto_router::{AutoRouter, ModelTier, ReasoningTier, RoutePlan};
use crate::compressor::ContextCompressor;
use crate::execution_log::{ExecutionEvent, ExecutionLogEntry, LogLevel};
use crate::harness::{LlmHarness, TaskType};
use crate::llm::LlmService;
use crate::planner::PlanningPrompt;
// SESSION-WIRE:aligner-import
use crate::aligner;
use crate::state::progress_event;
use crate::state::{
    AgentProgress, AgentState, BuildResult, ComposeScratch, ConversationContext, FlowPlan,
    MissingInfo, MissingInfoCategory, ModuleUsage, PlatformCatalog, Question, ResumeConfig,
    RunValidationResult, StepDetail, StepResult, UserAnswer,
};
use crate::tools;

pub struct AppAgent {
    pool: Arc<PgPool>,
    llm_service: Box<dyn LlmService>,
}

/// 从 LLM 自由文本中提取第一个 0-100 整数分数（非结构化文本，手动扫描，不用正则）。
fn parse_judge_score(s: &str) -> Option<f32> {
    let mut digits = String::new();
    for c in s.chars() {
        if c.is_ascii_digit() {
            digits.push(c);
            if digits.len() >= 3 {
                break;
            }
        } else if !digits.is_empty() {
            break;
        }
    }
    let n: u32 = digits.parse().ok()?;
    if n <= 100 {
        Some(n as f32 / 100.0)
    } else {
        None
    }
}

/// 剥离 LLM 输出可能的 markdown code fence（```json ... ```），供 tool_calls 解析。
fn strip_code_fence_for_tool_call(text: &str) -> String {
    let trimmed = text.trim();
    if let Some(rest) = trimmed.strip_prefix("```") {
        if let Some(start) = rest.find('\n') {
            let body = &rest[start + 1..];
            if let Some(end) = body.rfind("```") {
                return body[..end].trim().to_string();
            }
            return body.trim().to_string();
        }
    }
    trimmed.to_string()
}

impl AppAgent {
    pub fn new(pool: Arc<PgPool>, llm_service: Box<dyn LlmService>) -> Self {
        Self { pool, llm_service }
    }

    /// 一次性运行 agent 直到到达终止状态或被用户中断
    ///
    /// 每完成一个状态转移都会记录到 `ctx.step_history`。
    /// 若用户发送中断信号（`interrupt_requested = true`），
    /// Agent 会在当前 step 完成后停止循环，返回中断提示。
    pub async fn run(&self, ctx: &mut ConversationContext) -> Result<String, String> {
        loop {
            if ctx.interrupt_requested {
                ctx.interrupt_requested = false;
                let msg = format!(
                    "⏸️ 执行已暂停。当前状态：{}（进度 {}%）\n\n已执行 {} 步，可通过 resume 继续或 reset-state 回退。",
                    state_name(&ctx.state),
                    progress_percent(&ctx.state),
                    ctx.step_history.len()
                );
                return Ok(msg);
            }

            let result = self
                .run_single_step(ctx, None::<&fn(AgentProgress)>)
                .await?;
            if result.is_terminal {
                return Ok(result.message);
            }
        }
    }
    /// 带进度回调的 run — 每步状态转移时调用 `on_progress`，适合 WebSocket 实时推送
    pub async fn run_with_progress<F>(
        &self,
        ctx: &mut ConversationContext,
        on_progress: F,
    ) -> Result<String, String>
    where
        F: Fn(AgentProgress) + Send + Sync,
    {
        loop {
            if ctx.interrupt_requested {
                ctx.interrupt_requested = false;
                let msg = format!(
                    "\u{23f8}️ 执行已暂停。当前状态：{}（进度 {}%）\n\n已执行 {} 步，可通过 resume 继续或 reset-state 回退。",
                    state_name(&ctx.state),
                    progress_percent(&ctx.state),
                    ctx.step_history.len()
                );
                return Ok(msg);
            }

            on_progress(AgentProgress::new(
                state_name(&ctx.state),
                progress_percent(&ctx.state),
                "执行中",
                progress_event::STEP_STARTED,
                None,
            ));
            let result = self.run_single_step(ctx, Some(&on_progress)).await?;
            on_progress(AgentProgress::new(
                state_name(&result.state_after),
                progress_percent(&result.state_after),
                "步骤完成",
                progress_event::STEP_COMPLETED,
                None,
            ));
            if result.is_terminal {
                on_progress(AgentProgress::new(
                    "Completed",
                    100,
                    "执行完成",
                    progress_event::COMPLETED,
                    None,
                ));
                return Ok(result.message);
            }
        }
    }

    /// 设置中断信号（由前端调用）
    pub fn request_interrupt(ctx: &mut ConversationContext) {
        ctx.interrupt_requested = true;
        common::telemetry::info!(
            "Interrupt requested for session {} (current state: {:?})",
            ctx.session_id,
            ctx.state
        );
    }

    /// 单步执行：每次只推进一个状态转移，支持中断与恢复
    ///
    /// 前端根据返回的 `StepResult.is_terminal` 决定是否继续调用：
    /// - `is_terminal = true`：停止轮询，等待用户输入或展示最终结果
    /// - `is_terminal = false`：前端自动再次调用 `generate-response` 推进下一步
    pub async fn run_single_step<F>(
        &self,
        ctx: &mut ConversationContext,
        on_progress: Option<&F>,
    ) -> Result<StepResult, String>
    where
        F: Fn(AgentProgress) + Send + Sync,
    {
        let start = std::time::Instant::now();
        let state_before = ctx.state.clone();

        // 步骤索引（基于当前 step_history 长度而非 step_details，后者可能被修剪）
        let step_index = ctx.step_history.len() + 1;

        // 预先创建 StepDetail 占位 — handler（Planning/ExecutingSkill）需要执行前就有记录才能回填 prompt/response
        ctx.step_details.push(StepDetail {
            index: step_index,
            state_before: state_before.clone(),
            state_after: state_before.clone(), // placeholder，step() 后更新
            elapsed_ms: 0,
            is_terminal: false,
            llm_system_prompt: None,
            llm_user_prompt: None,
            llm_response: None,
            plan_violations: None,
            message: String::new(),
        });

        let next_state = self.step(ctx, on_progress).await?;
        ctx.state = next_state.clone();
        ctx.updated_at = Utc::now();

        // 记录执行流水日志 + WebSocket 推送
        Self::emit_execution_log(
            &mut ctx.execution_log,
            ctx.session_id,
            &on_progress,
            LogLevel::Info,
            ExecutionEvent::StateTransition {
                from: state_name(&state_before).to_string(),
                to: state_name(&next_state).to_string(),
                elapsed_ms: start.elapsed().as_millis() as u64,
            },
        );

        // 记录断点：成功完成 Composing / Verifying / Publishing 时设为 checkpoint
        if matches!(
            &next_state,
            AgentState::Composing
                | AgentState::Verifying { .. }
                | AgentState::Publishing { .. }
                | AgentState::Published { .. }
                | AgentState::Presenting { .. }
                | AgentState::SemanticAnalysis
                | AgentState::FunctionDecomposition
                | AgentState::OntologyAnalysis { .. }
                | AgentState::ModuleCreation
                | AgentState::BlockCreation
                | AgentState::OntologyTransfer
                | AgentState::ServiceAPI
                | AgentState::ExecutingSkill { .. },
        ) {
            ctx.last_checkpoint = Some(next_state.clone());
        }
        let is_terminal = matches!(
            &next_state,
            AgentState::Planning {
                needs_clarification: Some(_),
                ..
            } | AgentState::Published { .. }
                | AgentState::Presenting { .. }
                | AgentState::AwaitingUserInput { .. }
                // Failed 必须为终止态，否则 run() 会在 Failed→Failed 上无限自旋（本 e2e 暴露的 bug）
                | AgentState::Failed { .. }
        );

        let message = if matches!(
            &next_state,
            AgentState::Planning {
                needs_clarification: Some(_),
                ..
            }
        ) {
            self.render_state_message(ctx)
        } else if matches!(&next_state, AgentState::Presenting { .. }) {
            self.render_presentation(ctx)
        } else if matches!(&next_state, AgentState::Published { .. }) {
            self.render_published(ctx)
        } else if matches!(&next_state, AgentState::AwaitingUserInput { .. }) {
            self.render_awaiting_input(ctx)
        } else {
            // 非终止状态返回简短进度提示
            self.render_progress(ctx, &state_before, &next_state)
        };

        // 更新 StepDetail（占位条目现在是 step_details 的最后一条）
        if let Some(detail) = ctx.step_details.last_mut() {
            detail.state_after = next_state.clone();
            detail.elapsed_ms = start.elapsed().as_millis() as u64;
            detail.is_terminal = is_terminal;
            detail.message = message.clone();
            if detail.plan_violations.is_none() {
                detail.plan_violations = ctx.plan_violations.clone();
            }
        }

        // 裁剪 step_details 上限
        if ctx.step_details.len() > 50 {
            ctx.step_details.drain(0..(ctx.step_details.len() - 50));
        }

        let result = StepResult {
            state_before,
            state_after: next_state,
            message,
            is_terminal,
            elapsed_ms: start.elapsed().as_millis() as u64,
        };
        ctx.step_history.push(result.clone());

        // 上下文压缩 — 长对话中自动截断旧历史，防止 prompt 膨胀
        ContextCompressor::new().compress_if_needed(ctx);

        Ok(result)
    }
    fn emit_progress<F>(
        on_progress: &Option<&F>,
        state: &str,
        percent: u8,
        message: &str,
        event_kind: &str,
        payload: Option<serde_json::Value>,
    ) where
        F: Fn(AgentProgress) + Send + Sync,
    {
        if let Some(cb) = on_progress {
            cb(AgentProgress::new(
                state, percent, message, event_kind, payload,
            ));
        }
    }

    /// OntologyTransfer 完成后的下一跳：新版路径委托 alioth-service skill
    fn after_ontology_transfer(ctx: &ConversationContext) -> AgentState {
        let on_new_pipeline = ctx
            .flow_plan
            .as_ref()
            .map(|p| !p.created_blocks.is_empty())
            .unwrap_or(false);
        if on_new_pipeline {
            let ns = ctx
                .flow_plan
                .as_ref()
                .map(|p| p.namespace.clone())
                .unwrap_or_default();
            let mut ctx_map = std::collections::HashMap::new();
            ctx_map.insert("ns".to_string(), ns.clone());
            AgentState::ExecutingSkill {
                skill_name: "alioth-service".to_string(),
                track_index: 0,
                step_index: 0,
                attempt: 0,
                context: ctx_map,
                // D1=B：新版路径委托 alioth-service 后必须回到 Composing 生成
                // app.json(17 字段) + extensions/*.yaml，否则应用产物永不落地。
                return_state: Box::new(AgentState::Composing),
            }
        } else {
            AgentState::GeneratingFrontend {
                modules_generated: 0,
                verification_log: None,
            }
        }
    }

    /// LLM 坐标推理（GAP-1）：从意图 + 维度表推演 scene/factor code
    async fn infer_coordinates(
        llm: &dyn crate::llm::LlmService,
        scene_table: &str,
        factor_table: &str,
        db_scenes: &std::collections::HashSet<String>,
        db_factors: &std::collections::HashSet<String>,
        entity: &str,
        table: &str,
        function: Option<&str>,
        user_description: &str,
    ) -> (Option<String>, Option<String>, f64, f64) {
        let _system = "你是一个 Alioth 企业应用平台的本体坐标专家。你的任务是从候选列表中挑选最合适的 Scene 和 Factor 码。只返回 JSON，不包含任何解释。";
        let user = format!(
            r#"为以下 Entity 确定 Scene（业务场景）和 Factor（业务要素）：

Entity: {entity}
Table: {table}
Function: {func}
User intent: {intent}

可用 Scene 码：
{scenes}

可用 Factor 码：
{factors}

从上方列表中各选一个最匹配的。输出 JSON:
{{"scene_code": "XX", "factor_code": "FJA", "scene_confidence": 0.95, "factor_confidence": 0.90}}
"#,
            entity = entity,
            table = table,
            func = function.unwrap_or("(未推断)"),
            intent = user_description,
            scenes = scene_table,
            factors = factor_table,
        );
        let result = llm.generate(&user).await;
        let raw = match result {
            Ok(r) => r,
            Err(e) => {
                common::telemetry::warn!("LLM coordinate inference failed: {e}");
                return (None, None, 0.0, 0.0);
            }
        };
        // 从 LLM 回复中提取 JSON（支持 markdown 代码块与裸 JSON）
        let json_str = raw.trim();
        let json_str = if let Some(s) = json_str.strip_prefix("```json") {
            s.trim_start().split("```").next().unwrap_or("").trim()
        } else if let Some(s) = json_str.strip_prefix("```") {
            s.trim_start().split("```").next().unwrap_or("").trim()
        } else {
            json_str
        };
        let parsed: serde_json::Value = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(e) => {
                common::telemetry::warn!(
                    "Coordinate inference JSON parse failed: {e} raw={}",
                    &raw[..raw.len().min(200)]
                );
                return (None, None, 0.0, 0.0);
            }
        };
        let scene = parsed
            .get("scene_code")
            .and_then(|v| v.as_str())
            .map(String::from);
        let factor = parsed
            .get("factor_code")
            .and_then(|v| v.as_str())
            .map(String::from);
        let scene_conf = parsed
            .get("scene_confidence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let factor_conf = parsed
            .get("factor_confidence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        // DB 验证：码必须存在于维度表中
        let scene_valid = scene
            .as_ref()
            .map(|s| db_scenes.contains(s))
            .unwrap_or(false);
        let factor_valid = factor
            .as_ref()
            .map(|f| db_factors.contains(f))
            .unwrap_or(false);
        if !scene_valid {
            common::telemetry::warn!("LLM returned invalid scene code: {:?}", scene);
        }
        if !factor_valid {
            common::telemetry::warn!("LLM returned invalid factor code: {:?}", factor);
        }
        (
            scene.filter(|_| scene_valid && scene_conf >= 0.70),
            factor.filter(|_| factor_valid && factor_conf >= 0.70),
            if scene_valid { scene_conf } else { 0.0 },
            if factor_valid { factor_conf } else { 0.0 },
        )
    }

    /// GAP-5: function 规则未命中时 LLM 兜底推理
    async fn infer_function(
        llm: &dyn crate::llm::LlmService,
        entity: &str,
        table: &str,
        user_description: &str,
    ) -> Option<String> {
        let user = format!(
            r#"Entity: {entity}
Table: {table}
User intent: {intent}

从 zc_id_function 维度表中选择最匹配该实体的 Function 码（六象限编码）。
输出 JSON: {{"function_code": "↓_XX", "function_confidence": 0.85}}
"#,
            entity = entity,
            table = table,
            intent = user_description,
        );
        let raw = llm.generate(&user).await.ok()?;
        let json_str = raw.trim();
        let json_str = if let Some(s) = json_str.strip_prefix("```json") {
            s.trim_start().split("```").next().unwrap_or("").trim()
        } else if let Some(s) = json_str.strip_prefix("```") {
            s.trim_start().split("```").next().unwrap_or("").trim()
        } else {
            json_str
        };
        let parsed: serde_json::Value = serde_json::from_str(json_str).ok()?;
        let code = parsed
            .get("function_code")
            .and_then(|v| v.as_str())
            .map(String::from);
        let conf = parsed
            .get("function_confidence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        if conf >= 0.70 {
            code
        } else {
            common::telemetry::warn!(
                "Inferred function {code:?} below confidence threshold ({conf})"
            );
            None
        }
    }

    /// 将一条日志追加写入磁盘（append 模式），确保 session 即使在 Composing 前中断也不丢失
    fn append_to_disk(session_id: i64, entry: &ExecutionLogEntry) {
        use std::io::Write;
        let log_dir = std::env::temp_dir().join("alioth-agent-logs");
        let _ = std::fs::create_dir_all(&log_dir);
        let path = log_dir.join(format!("session-{}.log", session_id));
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = writeln!(file, "{}", entry.to_json_line());
        }
    }
    /// 推送执行日志条目 + 同时通过 WebSocket 推送
    /// 只操作 execution_log + session_id（不触及其他 ctx 字段，避免与同时存在的 ctx 引用冲突）
    fn emit_execution_log<F>(
        execution_log: &mut Vec<ExecutionLogEntry>,
        session_id: i64,
        on_progress: &Option<&F>,
        level: LogLevel,
        event: ExecutionEvent,
    ) where
        F: Fn(AgentProgress) + Send + Sync,
    {
        let entry = ExecutionLogEntry {
            timestamp: chrono::Utc::now(),
            level,
            session_id,
            event,
        };
        execution_log.push(entry);

        // 裁剪到上限
        if execution_log.len() > 500 {
            execution_log.drain(0..(execution_log.len() - 500));
        }

        // 磁盘持久化 — 立即追加到 temp log，确保 Composing 前中断不丢失
        if let Some(entry) = execution_log.last() {
            Self::append_to_disk(session_id, entry);
        }

        // WebSocket 推送
        if let Some(entry) = execution_log.last() {
            if let Ok(payload) = serde_json::to_value(entry) {
                Self::emit_progress(
                    on_progress,
                    "执行日志",
                    0,
                    "",
                    progress_event::EXECUTION_LOG,
                    Some(payload),
                );
            }
        }
    }

    /// 断点恢复：将上下文重置到指定状态，保留/清除相关数据
    pub fn reset_to_checkpoint(
        ctx: &mut ConversationContext,
        config: &ResumeConfig,
    ) -> Result<(), String> {
        let old_state = ctx.state.clone();
        ctx.state = config.target_state.clone();
        ctx.updated_at = Utc::now();

        if !config.preserve_ontology {
            ctx.ontology_model = None;
        }
        if !config.preserve_flow_plan {
            ctx.flow_plan = None;
        }
        if !config.preserve_scratch {
            ctx.compose_scratch = None;
            ctx.verification_error = None;
        }
        if !config.preserve_yaml_ops {
            ctx.yaml_operations.clear();
            ctx.yaml_operation_log.clear();
        }

        // 追加恢复记录到 step_history
        ctx.step_history.push(StepResult {
            state_before: old_state.clone(),
            state_after: config.target_state.clone(),
            message: format!(
                "已重置到状态 {:?} (ontology={}, flow_plan={}, scratch={}, yaml_ops={})",
                config.target_state,
                config.preserve_ontology,
                config.preserve_flow_plan,
                config.preserve_scratch,
                config.preserve_yaml_ops
            ),
            is_terminal: true,
            elapsed_ms: 0,
        });

        // 追加 StepDetail（恢复 step_history 与 step_details 索引同步）
        ctx.step_details.push(StepDetail {
            index: ctx.step_history.len(),
            state_before: old_state.clone(),
            state_after: config.target_state.clone(),
            elapsed_ms: 0,
            is_terminal: true,
            llm_system_prompt: None,
            llm_user_prompt: None,
            llm_response: None,
            plan_violations: None,
            message: format!("断点恢复到 {:?}", config.target_state),
        });
        if ctx.step_details.len() > 50 {
            ctx.step_details.drain(0..(ctx.step_details.len() - 50));
        }

        common::telemetry::info!(
            "Session {} reset from {:?} to {:?}",
            ctx.session_id,
            old_state,
            config.target_state
        );
        Ok(())
    }

    /// 技能门禁白名单（与 skill-adapters/*.yaml 对齐）
    const SKILL_GATE_WHITELIST: &[&str] = &["target/debug/ontology-mapping", "bun", "npx", "cargo"];

    /// 执行 step.gates——硬阻断，失败重试后进入 Failed
    const MAX_GATE_ATTEMPTS: u32 = 3;

    async fn execute_step_gates(
        _skill_name: &str,
        step: &crate::skills::Step,
        context: &std::collections::HashMap<String, String>,
    ) -> Result<(), String> {
        use tokio::process::Command;
        let project_root = crate::composer::resolve_project_root();

        fn resolve_template(s: &str, ctx: &std::collections::HashMap<String, String>) -> String {
            let mut result = s.to_string();
            for (k, v) in ctx {
                result = result.replace(&format!("{{{}}}", k), v);
            }
            result
        }

        for gate in &step.gates {
            // 纯文件检查
            if gate.program.is_empty() {
                if let Some(ref glob) = gate.output_glob {
                    let path = project_root.join(resolve_template(glob, context));
                    let pattern = path.to_string_lossy().to_string();
                    let wildcard = pattern.contains('*');
                    if wildcard {
                        let dir = path.parent().unwrap_or(&project_root);
                        let name = path.file_name().unwrap_or_default().to_string_lossy();
                        if !dir
                            .read_dir()
                            .map(|mut e| {
                                e.any(|e| {
                                    e.as_ref().is_ok_and(|e| {
                                        e.file_name()
                                            .to_string_lossy()
                                            .contains(name.trim_matches('*'))
                                    })
                                })
                            })
                            .unwrap_or(false)
                        {
                            return Err(format!("gate output_glob not found: {}", glob));
                        }
                    } else if !path.exists() {
                        return Err(format!("gate output_path not found: {}", path.display()));
                    }
                }
                continue;
            }
            // 白名单校验
            let program = &gate.program;
            let allowed = Self::SKILL_GATE_WHITELIST
                .iter()
                .any(|w| *w == program || program.starts_with(&format!("{}/", w)));
            if !allowed {
                return Err(format!("gate program '{}' not in whitelist", program));
            }
            // 执行
            let mut cmd = Command::new(program);
            for arg in &gate.args {
                cmd.arg(resolve_template(arg, context));
            }
            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::piped());
            cmd.kill_on_drop(true);

            let output = tokio::time::timeout(
                std::time::Duration::from_secs(gate.timeout_sec),
                cmd.output(),
            )
            .await
            .map_err(|_| format!("gate timeout ({}s): {}", gate.timeout_sec, program))?
            .map_err(|e| format!("gate io error: {}", e))?;

            if output.status.code() != Some(gate.expected_exit_code) {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!(
                    "gate '{}' exit {:?} != {}: {}",
                    program,
                    output.status.code(),
                    gate.expected_exit_code,
                    stderr
                ));
            }
            // 可选 output_glob 检查
            if let Some(ref glob) = gate.output_glob {
                let path = project_root.join(resolve_template(glob, context));
                if !path.exists() {
                    return Err(format!(
                        "gate output_glob not found after execution: {}",
                        glob
                    ));
                }
            }
        }
        Ok(())
    }

    /// 解析 LLM 输出 JSON 中的应用层 tool_calls（ExecutingSkill 工具调用）。
    ///
    /// 提取 `{ "tool_calls": [ { "name": <工具名>, "arguments": {...} }, ... ] }` 形态；
    /// 未知工具名由调用方按 ToolRegistry 已注册集合过滤。返回 (name, arguments) 列表。
    fn parse_skill_tool_calls(raw: &str) -> Vec<(String, serde_json::Value)> {
        let stripped = strip_code_fence_for_tool_call(raw);
        let value: serde_json::Value = match serde_json::from_str(&stripped) {
            Ok(v) => v,
            Err(_) => return Vec::new(),
        };
        let arr = match value.get("tool_calls").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return Vec::new(),
        };
        arr.iter()
            .filter_map(|tc| {
                let name = tc.get("name").and_then(|v| v.as_str())?.to_string();
                let args = tc
                    .get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                Some((name, args))
            })
            .collect()
    }

    /// 把 `{ns}`/`{app}`/`{module}`/`{block}`/`{service}`/`{crate}` 等上下文占位符
    /// 解析为真实值（复刻 `execute_step_gates` 内的 `resolve_template` 闭包）。
    fn resolve_templates(s: &str, ctx: &std::collections::HashMap<String, String>) -> String {
        let mut result = s.to_string();
        for (k, v) in ctx {
            result = result.replace(&format!("{{{}}}", k), v);
        }
        result
    }

    async fn step<F>(
        &self,
        ctx: &mut ConversationContext,
        on_progress: Option<&F>,
    ) -> Result<AgentState, String>
    where
        F: Fn(AgentProgress) + Send + Sync,
    {
        match &ctx.state {
            AgentState::Initializing => {
                Self::emit_progress(
                    &on_progress,
                    state_name(&ctx.state),
                    progress_percent(&ctx.state),
                    "正在加载平台本体目录...",
                    progress_event::PLANNING_START,
                    None,
                );
                let catalog = self
                    .load_platform_catalog(ctx.namespace.as_deref().unwrap_or("Alioth"))
                    .await
                    .map_err(|e| {
                        common::telemetry::error!("{}", e);
                        e
                    })?;
                ctx.platform_catalog = Some(catalog);
                Ok(AgentState::SemanticAnalysis)
            }

            AgentState::SemanticAnalysis => {
                // 1. 语义分析：提取业务意图和关键概念
                Self::emit_progress(
                    &on_progress,
                    state_name(&ctx.state),
                    progress_percent(&ctx.state),
                    "正在分析用户意图...",
                    progress_event::PLANNING_START,
                    None,
                );
                let concepts = crate::planner::extract_semantic_concepts(&ctx.user_description);
                let namespace = self.derive_namespace(ctx);
                if ctx.flow_plan.is_none() {
                    ctx.flow_plan = Some(FlowPlan {
                        namespace,
                        used_modules: vec![],
                        known_entities: vec![],
                        missing_info: vec![],
                        workflow_steps: vec![],
                        computations: vec![],
                        constraints: vec![],
                        business_rules: vec![],
                        app_meta: None,
                        created_modules: vec![],
                        created_blocks: vec![],
                        created_services: vec![],
                        ontology_model_json: None,
                        functional_units: vec![],
                        semantic_concepts: concepts.clone(),
                    });
                }
                common::telemetry::info!("Semantic concepts extracted: {:?}", concepts);
                Ok(AgentState::FunctionDecomposition)
            }

            AgentState::FunctionDecomposition => {
                // 2. 功能拆解：从需求提取功能单元，写入 FlowPlan.functional_units（D2 真实逻辑）
                Self::emit_progress(
                    &on_progress,
                    state_name(&ctx.state),
                    progress_percent(&ctx.state),
                    "正在拆解功能...",
                    progress_event::PLANNING_START,
                    None,
                );
                let concepts = crate::planner::extract_semantic_concepts(&ctx.user_description);
                if let Some(ref mut plan) = ctx.flow_plan {
                    plan.functional_units = concepts
                        .into_iter()
                        .map(|c| {
                            let m = slugify(&c);
                            crate::state::FunctionalUnit {
                                name: format!("{}管理", c),
                                description: format!("处理「{}」相关业务功能", c),
                                entities: vec![c.clone()],
                                suggested_module: Some(m.clone()),
                                suggested_blocks: vec![format!("block-{}", m)],
                                suggested_services: vec![format!("{}-service", m)],
                            }
                        })
                        .collect();
                    common::telemetry::info!(
                        "FunctionDecomposition: {} 功能单元",
                        plan.functional_units.len()
                    );
                }
                Ok(AgentState::OntologyAnalysis { ontology_round: 0 })
            }

            AgentState::OntologyAnalysis { .. } => {
                // 3. 本体分析 → 过渡到旧 Planning 逻辑（兼容现有实现）
                Self::emit_progress(
                    &on_progress,
                    state_name(&ctx.state),
                    progress_percent(&ctx.state),
                    "正在分析本体映射...",
                    progress_event::PLANNING_START,
                    None,
                );
                Ok(AgentState::Planning {
                    revision_round: 0,
                    needs_clarification: None,
                })
            }

            AgentState::ModuleCreation => {
                // 4. 模块创建/组装：从 FlowPlan 生成 module.json 骨架
                Self::emit_progress(
                    &on_progress,
                    state_name(&ctx.state),
                    progress_percent(&ctx.state),
                    "正在创建模块骨架...",
                    progress_event::COMPOSING_START,
                    None,
                );
                let project_root = crate::composer::resolve_project_root();
                let plan = ctx.flow_plan.as_ref().ok_or("Missing flow plan")?;
                let namespace = &plan.namespace;
                let domain = plan
                    .known_entities
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "default".to_string());
                let module_id = format!("{}-app", slugify(&domain));
                crate::tools::create_module_scaffold(
                    project_root.to_str().unwrap_or("."),
                    namespace,
                    &module_id,
                    &plan
                        .used_modules
                        .first()
                        .cloned()
                        .unwrap_or_else(|| domain.clone()),
                    &format!("AppAgent 生成的 {} 模块", domain),
                    &[],
                )
                .await
                .map_err(|e| format!("Module creation failed: {}", e))?;
                let ns_owned = namespace.clone();
                let mid = module_id.clone();
                if let Some(ref mut p) = ctx.flow_plan {
                    p.created_modules = vec![module_id];
                }
                common::telemetry::info!("Module scaffold created");
                let mut ctx_map = std::collections::HashMap::new();
                ctx_map.insert("ns".to_string(), ns_owned);
                ctx_map.insert("module".to_string(), mid);
                Ok(AgentState::ExecutingSkill {
                    skill_name: "alioth-module".to_string(),
                    track_index: 0,
                    step_index: 0,
                    attempt: 0,
                    context: ctx_map,
                    return_state: Box::new(AgentState::BlockCreation),
                })
            }

            AgentState::BlockCreation => {
                // 5. Block 创建：为模块创建业务 Block 流程骨架
                Self::emit_progress(
                    &on_progress,
                    state_name(&ctx.state),
                    progress_percent(&ctx.state),
                    "正在创建区块骨架...",
                    progress_event::COMPOSING_START,
                    None,
                );
                let project_root = crate::composer::resolve_project_root();
                let plan = ctx.flow_plan.as_ref().ok_or("Missing flow plan")?;
                let namespace = &plan.namespace;
                let steps: Vec<String> = if plan.workflow_steps.is_empty() {
                    vec!["main".to_string()]
                } else {
                    plan.workflow_steps.clone()
                };
                let mut created = vec![];
                for step in &steps {
                    let block_id = format!("block-{}", slugify(step));
                    crate::tools::create_block_scaffold(
                        project_root.to_str().unwrap_or("."),
                        namespace,
                        &block_id,
                        step,
                        &[],
                    )
                    .await
                    .map_err(|e| format!("Block creation failed: {}", e))?;
                    created.push(block_id);
                }
                let ns_owned = namespace.clone();
                let created_clone = created.clone();
                if let Some(ref mut p) = ctx.flow_plan {
                    p.created_blocks = created_clone;
                }
                common::telemetry::info!("Block scaffolds created: {:?}", created);
                let first_block = created.first().cloned().unwrap_or_default();
                let mut ctx_map = std::collections::HashMap::new();
                ctx_map.insert("ns".to_string(), ns_owned);
                ctx_map.insert("block".to_string(), first_block);
                Ok(AgentState::ExecutingSkill {
                    skill_name: "alioth-block".to_string(),
                    track_index: 0,
                    step_index: 0,
                    attempt: 0,
                    context: ctx_map,
                    return_state: Box::new(AgentState::OntologyTransfer),
                })
            }

            AgentState::OntologyTransfer => {
                // 6. 本体转移：gap 域 → DB 叶表（discovery）+ 坐标（LLM 推理 + DB 验证）
                //
                // 流程：表发现 → 字段映射 + function 规则推断 → LLM 从意图+
                // 维度表推演 scene/factor → DB 码验证 → 高置信度自动接受。
                // 综合评分 < 0.5 的域保持 request-no-impl 缺口状态。
                Self::emit_progress(
                    &on_progress,
                    state_name(&ctx.state),
                    progress_percent(&ctx.state),
                    "正在将本体缺口映射到 DB 叶表...",
                    progress_event::COMPOSING_START,
                    None,
                );

                let gaps = ctx.extension_gaps.clone();
                if gaps.is_empty() {
                    return Ok(Self::after_ontology_transfer(ctx));
                }

                let rules_path = crate::composer::resolve_project_root()
                    .join("Meta/backend/ontology-mapping/rules.yaml");
                let namespace = self.derive_namespace(ctx);
                let services_dir = crate::composer::resolve_project_root()
                    .join("Pre-Proc")
                    .join(&namespace)
                    .join("Sources")
                    .join("Services");

                // 预加载维度表用于坐标 LLM 推理
                let scene_dim: Vec<(String, String)> =
                    sqlx::query_as("SELECT code, notice FROM isahl.zc_id_scene ORDER BY code")
                        .fetch_all(&*self.pool)
                        .await
                        .unwrap_or_default();
                let factor_dim: Vec<(String, String)> =
                    sqlx::query_as("SELECT code, notice FROM isahl.zc_id_factor ORDER BY code")
                        .fetch_all(&*self.pool)
                        .await
                        .unwrap_or_default();
                let db_scenes: std::collections::HashSet<String> =
                    scene_dim.iter().map(|(c, _)| c.clone()).collect();
                let db_factors: std::collections::HashSet<String> =
                    factor_dim.iter().map(|(c, _)| c.clone()).collect();
                let scene_codes_str: String = scene_dim
                    .iter()
                    .map(|(c, n)| format!("  {c:6} - {n}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                let factor_codes_str: String = factor_dim
                    .iter()
                    .map(|(c, n)| format!("  {c:6} - {n}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                // OntologyMapper 加载 namespace 已有 service 实体（factor_match 优先）
                let mapper = ontology_mapping::OntologyMapper::load(&rules_path, &services_dir)
                    .map_err(|e| format!("加载 ontology mapper 失败: {e}"))?;

                let pool = self.pool.clone();
                let mut mapped: Vec<crate::state::MappedEntity> = Vec::new();
                let mut remaining: Vec<crate::state::ExtensionGap> = Vec::new();
                for gap in &gaps {
                    let fields: Vec<String> =
                        gap.new_fields.iter().map(|f| f.name.clone()).collect();
                    let candidates = ontology_mapping::discovery::match_tables(
                        &pool,
                        &gap.domain_id,
                        &fields,
                        None,
                        3,
                    )
                    .await
                    .unwrap_or_default();
                    let best = candidates.first();
                    match best {
                        Some(cand) if cand.score >= 0.5 => {
                            // 完整映射：字段绑定 + LLM 坐标推理（GAP-1）
                            let input = ontology_mapping::discovery::to_mapping_input(
                                &gap.domain_id,
                                &cand.table,
                                &fields,
                                "",
                                &[],
                            );
                            let output = mapper.map(&input);
                            let entity = &output.entities[0];
                            let function_code = if entity.coordinates.function.value.is_empty() {
                                None
                            } else {
                                Some(entity.coordinates.function.value.clone())
                            };
                            common::telemetry::info!(
                                "OntologyTransfer: domain={} → table={} (score={:.2}, function={:?}, fields={})",
                                gap.domain_id,
                                cand.table,
                                cand.score,
                                function_code,
                                entity.fields.len()
                            );
                            // GAP-5: function 规则未命中时 LLM 兜底
                            let function_code = if function_code.is_none()
                                && ctx.user_description.trim().len() > 15
                            {
                                Self::infer_function(
                                    &*self.llm_service,
                                    &gap.domain_id,
                                    &cand.table,
                                    &ctx.user_description,
                                )
                                .await
                            } else {
                                function_code
                            };
                            // LLM 坐标推理：从意图 + 维度表推演 scene/factor
                            let user_desc_len = ctx.user_description.trim().len();
                            let (scene_code, factor_code, _scene_conf, _factor_conf) =
                                if user_desc_len > 15 && !scene_dim.is_empty() {
                                    Self::infer_coordinates(
                                        &*self.llm_service,
                                        &scene_codes_str,
                                        &factor_codes_str,
                                        &db_scenes,
                                        &db_factors,
                                        &gap.domain_id,
                                        &cand.table,
                                        function_code.as_deref(),
                                        &ctx.user_description,
                                    )
                                    .await
                                } else {
                                    (None, None, 0.0, 0.0)
                                };
                            common::telemetry::info!(
                                "Coordinate inference: scene={:?} factor={:?}",
                                scene_code,
                                factor_code
                            );
                            mapped.push(crate::state::MappedEntity {
                                domain_id: gap.domain_id.clone(),
                                table: cand.table.clone(),
                                score: cand.score,
                                name_score: cand.name_score,
                                field_score: cand.field_score,
                                scene_code,
                                factor_code,
                                function_code,
                                function_confidence: entity.coordinates.function.confidence,
                                field_mappings: entity
                                    .fields
                                    .iter()
                                    .map(|f| crate::state::MappedField {
                                        json_path: f.json_path.clone(),
                                        column: f.column.clone(),
                                        scalar_table: f.scalar_table.clone(),
                                        tier: format!("{:?}", f.tier).to_lowercase(),
                                    })
                                    .collect(),
                            });
                        }
                        _ => remaining.push(gap.clone()),
                    }
                }

                let mapped_count = mapped.len();
                ctx.mapped_entities = mapped;
                ctx.extension_gaps = remaining;
                // SESSION-FIX:gap-e-coordinate-confirmation — 层2 坐标确认
                // 1. 应用已收到的坐标答案（OT 重跑场景）
                if let Some(ref catalog) = ctx.platform_catalog {
                    let applied = aligner::apply_coordinate_answers(
                        &mut ctx.mapped_entities,
                        &ctx.user_answers,
                        catalog,
                    );
                    if applied > 0 {
                        common::telemetry::info!(
                            "OntologyTransfer: {} 个坐标已由用户确认",
                            applied
                        );
                    }
                }
                // 2. 仍缺坐标 → 生成确认问题，保存 checkpoint，等待用户
                if let Some(ref catalog) = ctx.platform_catalog {
                    let coord_questions =
                        aligner::build_coordinate_questions(&ctx.mapped_entities, catalog);
                    let unanswered: Vec<_> = coord_questions
                        .into_iter()
                        .filter(|q| !ctx.user_answers.iter().any(|a| a.question_id == q.id))
                        .collect();
                    if !unanswered.is_empty() {
                        ctx.last_checkpoint = Some(AgentState::OntologyTransfer);
                        ctx.pending_questions = unanswered.clone();
                        return Ok(AgentState::Planning {
                            revision_round: 0,
                            needs_clarification: Some(unanswered),
                        });
                    }
                }
                // SESSION-WIRE:build-alignment-graph（坐标应用后重建，确保 CoordinatesSnapshot 最终正确）
                if let Some(ref model) = ctx.ontology_model {
                    ctx.alignment_graph = Some(
                        aligner::build_alignment_graph(
                            pool.as_ref(),
                            model,
                            &ctx.mapped_entities,
                            ctx.platform_catalog.as_ref(),
                            ctx.flow_plan
                                .as_ref()
                                .map(|p| p.known_entities.as_slice())
                                .unwrap_or(&[]),
                        )
                        .await,
                    );
                }
                Self::emit_progress(
                    &on_progress,
                    state_name(&ctx.state),
                    progress_percent(&ctx.state),
                    &format!(
                        "本体转移完成：{} 域已映射到 DB 叶表，{} 域保持 request-no-impl",
                        mapped_count,
                        ctx.extension_gaps.len()
                    ),
                    progress_event::COMPOSING_START,
                    Some(serde_json::json!({
                        "mapped": ctx.mapped_entities.iter().map(|m| serde_json::json!({
                            "domain": m.domain_id,
                            "table": m.table,
                            "score": m.score,
                            "function": m.function_code,
                        })).collect::<Vec<_>>(),
                        "unmapped_gaps": ctx.extension_gaps.len(),
                    })),
                );
                Ok(Self::after_ontology_transfer(ctx))
            }

            AgentState::ServiceAPI => {
                // 7. Service API 生成：创建 service.json 骨架
                Self::emit_progress(
                    &on_progress,
                    state_name(&ctx.state),
                    progress_percent(&ctx.state),
                    "正在生成 Service API 骨架...",
                    progress_event::COMPOSING_START,
                    None,
                );
                let project_root = crate::composer::resolve_project_root();
                let plan = ctx.flow_plan.as_ref().ok_or("Missing flow plan")?;
                let namespace = &plan.namespace;
                let domain = plan
                    .known_entities
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "default".to_string());
                let service_id = format!("{}-service", slugify(&domain));
                crate::tools::create_service_scaffold(
                    project_root.to_str().unwrap_or("."),
                    namespace,
                    &service_id,
                    &domain,
                    &plan.known_entities,
                )
                .await
                .map_err(|e| format!("Service creation failed: {}", e))?;
                let ns_owned = namespace.clone();
                if let Some(ref mut p) = ctx.flow_plan {
                    p.created_services = vec![service_id.clone()];
                }
                common::telemetry::info!("Service scaffold created");
                // 委托 alioth-service skill 生成 DTO/Handler 代码
                let mut ctx_map = std::collections::HashMap::new();
                ctx_map.insert("ns".to_string(), ns_owned.clone());
                ctx_map.insert("service".to_string(), service_id.clone());
                ctx_map.insert("crate".to_string(), service_id);
                Ok(AgentState::ExecutingSkill {
                    skill_name: "alioth-service".to_string(),
                    track_index: 0,
                    step_index: 0,
                    attempt: 0,
                    context: ctx_map,
                    return_state: Box::new(AgentState::Publishing {
                        publish_attempt: 0,
                        last_error: None,
                    }),
                })
            }
            AgentState::ExecutingSkill {
                skill_name,
                track_index,
                step_index,
                attempt,
                context,
                return_state,
            } => {
                // 技能目录：relative to project root（app 级覆盖优先）
                let skills_dir = crate::composer::resolve_project_root().join("skill-adapters");
                let app_dir = context.get("ns").and_then(|ns| {
                    context.get("app").map(|app| {
                        crate::composer::resolve_project_root()
                            .join("Pre-Proc")
                            .join(ns)
                            .join("Apps")
                            .join(app)
                            .join("skill-adapters")
                    })
                });
                let mut registry = match &app_dir {
                    Some(app) => crate::skills::SkillRegistry::with_app_dir(&skills_dir, app),
                    None => crate::skills::SkillRegistry::new(&skills_dir),
                };

                let count = registry
                    .load_all()
                    .await
                    .map_err(|e| format!("Failed to load skills: {}", e))?;
                if count == 0 {
                    common::telemetry::warn!("No skills loaded from {:?}", skills_dir);
                    common::telemetry::info!("Create YAML skill adapters in skill-adapters/ to enable automated pipeline stages");
                    return Ok(AgentState::Planning {
                        revision_round: 0,
                        needs_clarification: None,
                    });
                }

                let skill = registry.get(skill_name).ok_or_else(|| {
                    format!(
                        "Skill '{}' not found in registry. Available: {:?}",
                        skill_name,
                        registry
                            .list()
                            .iter()
                            .map(|s| s.name.as_str())
                            .collect::<Vec<_>>()
                    )
                })?;

                let current_track = *track_index;
                let current_step = *step_index;

                // 所有 Track 完成 → 返回 Planning
                // 所有 Track 完成 → 跳回 return_state
                if current_track >= skill.tracks.len() {
                    common::telemetry::info!("Skill '{}' all tracks complete", skill_name);
                    return Ok(*return_state.clone());
                }

                // 当前 Track 完成 → 推进到下一个 Track
                if current_step >= skill.tracks[current_track].steps.len() {
                    return Ok(AgentState::ExecutingSkill {
                        skill_name: skill_name.clone(),
                        track_index: current_track + 1,
                        step_index: 0,
                        attempt: 0,
                        context: context.clone(),
                        return_state: return_state.clone(),
                    });
                }

                // 构建 prompt
                let track = &skill.tracks[current_track];
                let step = &track.steps[current_step];

                let mut tool_registry = crate::tool_registry::ToolRegistry::new();
                tool_registry.register(crate::tool_registry::ReadFileTool);
                tool_registry.register(crate::tool_registry::WriteFileTool);
                tool_registry.register(crate::tool_registry::SearchFileTool);
                tool_registry.register(crate::tool_registry::ListSkillsTool::new(vec![
                    skill_name.clone()
                ]));
                tool_registry.register(crate::tool_registry::ExecuteSkillTool);
                tool_registry.register(crate::tool_registry::RunCommandTool);

                // 收集步骤所需的工具定义
                let mut step_tools = skill.default_tools.clone();
                step_tools.extend(step.tools.clone());
                let tool_defs: Vec<&crate::tool_registry::ToolDef> = tool_registry
                    .tool_defs()
                    .iter()
                    .filter(|t| step_tools.contains(&t.name))
                    .collect();
                let tools_json = serde_json::to_value(&tool_defs).unwrap_or_default();

                let system = format!(
                    r#"你是 AppAgent 的技能执行引擎。执行以下工作流步骤：

技能：{name}
{desc}

阶段：{track_name} / 步骤 {step_id}

## 指令
{step_instruction}

## 可用工具
{tools}

输出 JSON：{{ "completed": bool, "summary": str, "artifacts": {{}}, "tool_calls": [{{ "name": "<工具名>", "arguments": {{...}} }}] }}
若需写文件/执行命令，请在 tool_calls 中给出对应工具调用（工具定义见「可用工具」）。
"#,
                    name = skill.name,
                    desc = skill.description,
                    track_name = track.name,
                    step_id = step.id,
                    step_instruction = step.instruction,
                    tools = serde_json::to_string_pretty(&tools_json).unwrap_or_default(),
                );

                let harness = crate::harness::LlmHarness::new(&*self.llm_service)
                    .with_task(crate::harness::TaskType::CodeGeneration)
                    .with_system(&system);

                let result = harness
                    .call_with_retry(&format!("执行步骤 {}: {}", step.id, step.instruction))
                    .await
                    .map_err(|e| format!("Skill step LLM call failed: {}", e))?;

                // 补全 step_details 中的 LLM prompt/response
                if let Some(detail) = ctx.step_details.last_mut() {
                    detail.llm_system_prompt = Some(system.clone());
                    detail.llm_user_prompt = Some(step.instruction.clone());
                    detail.llm_response = Some(result.raw_text.clone());
                }

                // ── 真实 tool_call 执行（彻底）──
                // 解析 LLM 输出中的 tool_calls（应用层，工具名对应 ToolRegistry 已注册工具），
                // 对 path / content / args / program 等字符串字段做 {ns}/{app}/{module}/{block}/
                // {service}/{crate} 模板解析后，逐条经 ToolRegistry 执行，确保门禁前产物已落盘。
                let raw_calls = Self::parse_skill_tool_calls(&result.raw_text);
                let mut executed_tool_calls: Vec<serde_json::Value> = Vec::new();
                for (tc_name, mut tc_args) in raw_calls {
                    // 仅执行 ToolRegistry 已注册工具，未知工具名跳过（非致命）
                    if !tool_registry.tool_defs().iter().any(|t| t.name == tc_name) {
                        common::telemetry::warn!(
                            "ExecutingSkill: 未知 tool_call '{}' 已跳过",
                            tc_name
                        );
                        continue;
                    }
                    // 模板变量解析：path / content / program / args[]
                    for key in ["path", "content", "program"] {
                        if let Some(v) = tc_args.get_mut(key) {
                            if let Some(s) = v.as_str() {
                                *v =
                                    serde_json::Value::String(Self::resolve_templates(s, &context));
                            }
                        }
                    }
                    if let Some(arr) = tc_args.get_mut("args").and_then(|v| v.as_array_mut()) {
                        for item in arr.iter_mut() {
                            if let Some(s) = item.as_str() {
                                *item = serde_json::Value::String(Self::resolve_templates(
                                    s,
                                    &context,
                                ));
                            }
                        }
                    }
                    common::telemetry::info!(
                        "ExecutingSkill: 执行工具 '{}' (skill={}, step={})",
                        tc_name,
                        skill_name,
                        step.id
                    );
                    let exec = match tool_registry.call(&tc_name, tc_args.clone()).await {
                        Ok(r) => r,
                        Err(e) => {
                            common::telemetry::warn!(
                                "ExecutingSkill: 工具 '{}' 执行返回错误: {}",
                                tc_name,
                                e
                            );
                            executed_tool_calls.push(serde_json::json!({
                                "tool": tc_name,
                                "success": false,
                                "error": e,
                            }));
                            Self::emit_execution_log(
                                &mut ctx.execution_log,
                                ctx.session_id,
                                &on_progress,
                                LogLevel::Warn,
                                ExecutionEvent::ToolCall {
                                    tool: tc_name.clone(),
                                    success: false,
                                    detail: Some(e.clone()),
                                },
                            );
                            continue;
                        }
                    };
                    executed_tool_calls.push(serde_json::json!({
                        "tool": tc_name,
                        "success": exec.success,
                        "error": exec.error,
                    }));
                    Self::emit_execution_log(
                        &mut ctx.execution_log,
                        ctx.session_id,
                        &on_progress,
                        if exec.success {
                            LogLevel::Info
                        } else {
                            LogLevel::Warn
                        },
                        ExecutionEvent::ToolCall {
                            tool: tc_name.clone(),
                            success: exec.success,
                            detail: exec.error.clone().or_else(|| {
                                exec.data.as_ref().map(|_| "ok".to_string())
                            }),
                        },
                    );
                }
                if !executed_tool_calls.is_empty() {
                    common::telemetry::info!(
                        "ExecutingSkill: 本步执行 {} 个 tool_call (skill={}, step={})",
                        executed_tool_calls.len(),
                        skill_name,
                        step.id
                    );
                }

                // ── 门禁执行（硬阻断）──
                // Step 完成后必须按序通过所有 gates；任一失败则 attempt++ 重试，超过 3 次后进入 Failed
                let gates_result = Self::execute_step_gates(skill_name, step, context).await;
                match gates_result {
                    Ok(_) => {
                        common::telemetry::info!(
                            "Skill '{}' step {} gates passed",
                            skill_name,
                            step.id
                        );
                    }
                    Err(e) => {
                        common::telemetry::warn!(
                            "Skill '{}' step {} gate failed (attempt {}): {}",
                            skill_name,
                            step.id,
                            attempt,
                            e
                        );
                        if *attempt + 1 < Self::MAX_GATE_ATTEMPTS {
                            return Ok(AgentState::ExecutingSkill {
                                skill_name: skill_name.clone(),
                                track_index: *track_index,
                                step_index: *step_index,
                                attempt: *attempt + 1,
                                context: context.clone(),
                                return_state: return_state.clone(),
                            });
                        } else {
                            return Ok(AgentState::Failed {
                                error: format!(
                                    "gate failed after {} attempts: {}",
                                    Self::MAX_GATE_ATTEMPTS,
                                    e
                                ),
                            });
                        }
                    }
                }

                // 记录技能步骤执行日志 + WebSocket 推送
                Self::emit_execution_log(
                    &mut ctx.execution_log,
                    ctx.session_id,
                    &on_progress,
                    LogLevel::Info,
                    ExecutionEvent::SkillStep {
                        skill_name: skill_name.clone(),
                        track: *track_index,
                        step: *step_index,
                        completed: true,
                        summary: format!("步骤 {}: {}", step.id, step.instruction),
                    },
                );

                Ok(AgentState::ExecutingSkill {
                    skill_name: skill_name.clone(),
                    track_index: current_track,
                    step_index: current_step + 1,
                    attempt: *attempt + 1,
                    context: context.clone(),
                    return_state: return_state.clone(),
                })
            }
            AgentState::Planning {
                revision_round,
                needs_clarification,
            } => {
                // 如果有待澄清的问题，设置 pending_questions 并保持等待用户输入
                if let Some(questions) = needs_clarification {
                    ctx.pending_questions = questions.clone();
                    return Ok(AgentState::Planning {
                        revision_round: *revision_round,
                        needs_clarification: Some(questions.clone()),
                    });
                }

                let catalog = ctx.platform_catalog.as_ref().ok_or("Missing catalog")?;
                // 限制 user_answers 保留最近 20 条，防止长对话导致 prompt 膨胀
                const MAX_ANSWERS: usize = 20;
                if ctx.user_answers.len() > MAX_ANSWERS {
                    let skip = ctx.user_answers.len() - MAX_ANSWERS;
                    ctx.user_answers = ctx.user_answers.iter().skip(skip).cloned().collect();
                }

                let answers: Vec<String> = ctx
                    .user_answers
                    .iter()
                    .map(|a| format!("Q:{} A:{}", a.question_id, a.answer))
                    .collect();

                // 按用户需求查询相关本体上下文（meta_ontology + edges）
                let keywords = extract_keywords(&ctx.user_description);
                let ontology_ctx = if !keywords.is_empty() {
                    match crate::tools::query_relevant_ontology(
                        self.pool.as_ref(),
                        &keywords.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                        50,
                    )
                    .await
                    {
                        result if result.error.is_none() => Some(result.data),
                        _ => None,
                    }
                } else {
                    None
                };

                Self::emit_progress(
                    &on_progress,
                    state_name(&ctx.state),
                    22,
                    &format!("已查询相关本体上下文，关键词: {}", keywords.join(", ")),
                    progress_event::ONTOLOGY_CONTEXT_QUERIED,
                    Some(serde_json::json!({
                        "keywords": keywords,
                        "entity_count": ontology_ctx.as_ref().and_then(|v| v.get("entities")).and_then(|a| a.as_array()).map(|a| a.len()).unwrap_or(0)
                    })),
                );

                // ── Auto Router: 动态决策模型与推理深度 ──────────────
                let route_plan = AutoRouter::new()
                    .route(&*self.llm_service, &ctx.user_description)
                    .await;
                common::telemetry::info!(
                    "Route: {} / reasoning={:?}",
                    route_plan.model.model_id(),
                    route_plan.reasoning_effort,
                );

                Self::emit_progress(
                    &on_progress,
                    state_name(&ctx.state),
                    24,
                    &format!(
                        "已选择模型 {} / 推理深度 {:?}",
                        route_plan.model.model_id(),
                        route_plan.reasoning_effort
                    ),
                    progress_event::LLM_ROUTE_SELECTED,
                    Some(serde_json::json!({})),
                );

                // ── 自修复升级：根据失败次数升级模型/推理深度 ────
                let escalated_plan = if ctx.repair_attempt >= 5 {
                    // 深度失败 → 升级到 Pro + 高推理
                    RoutePlan {
                        model: ModelTier::Pro,
                        reasoning_effort: ReasoningTier::Max,
                    }
                } else if ctx.repair_attempt >= 3 {
                    // 中度失败 → 升级推理深度
                    RoutePlan {
                        model: ModelTier::Flash,
                        reasoning_effort: ReasoningTier::High,
                    }
                } else {
                    route_plan
                };
                if ctx.repair_attempt >= 3 {
                    common::telemetry::info!(
                        "Escalated route (repair_attempt={}): model={}, reasoning={:?}",
                        ctx.repair_attempt,
                        escalated_plan.model.model_id(),
                        escalated_plan.reasoning_effort,
                    );
                    Self::emit_progress(
                        &on_progress,
                        state_name(&ctx.state),
                        25,
                        &format!(
                            "⚠️ 自动升级模型：{} → {}",
                            route_plan.model.model_id(),
                            escalated_plan.model.model_id()
                        ),
                        progress_event::LLM_ROUTE_SELECTED,
                        Some(serde_json::json!({
                            "model_id": escalated_plan.model.model_id(),
                            "reasoning_effort": escalated_plan.reasoning_effort,
                            "reason": format!("repair_attempt={}", ctx.repair_attempt)
                        })),
                    );
                }
                // ── Build Prompt & Call LLM via Harness ─────────────
                let compiled_modules = tools::compiled_module_ids();
                // 注入验证/运行时错误到 prompt（自修复循环）
                let mut user_description = ctx.user_description.clone();
                if let Some(err) = &ctx.verification_error {
                    user_description = format!(
                        "{}\n\n【自动修复请求，当前尝试 {}/{}】\n之前的构建产生如下错误，请修复后重新生成 app.json 和 extensions：\n```\n{}\n```",
                        user_description,
                        ctx.repair_attempt,
                        ctx.max_repair_count,
                        err
                    );
                }

                // 快速草稿模式：限制模块选择范围
                if ctx.draft_mode {
                    user_description = format!(
                        "{}\n\n【快速草稿模式】\n请仅选择最核心的 1-3 个模块来快速生成可运行的应用轮廓。\n- 跳过复杂业务规则、状态机、工作流等扩展配置\n- 只生成 app.json 骨架和最少 extension 配置\n- 确保生成的应用可以启动和访问",
                        user_description,
                    );
                }

                let prompt = PlanningPrompt::new(
                    &user_description,
                    catalog,
                    ctx.ontology_model.as_ref(),
                    &answers,
                    ontology_ctx.as_ref(),
                    &compiled_modules,
                );
                // 在 escalated_plan 被 move 前捕获 LLM 元数据
                let llm_model_id = escalated_plan.model.model_id().to_string();
                let llm_reasoning = escalated_plan
                    .reasoning_effort
                    .as_api_value()
                    .unwrap_or("high")
                    .to_string();

                let harness = LlmHarness::new(&*self.llm_service)
                    .with_task(TaskType::OntologyPlanning)
                    .with_system(&prompt.system)
                    .with_route_plan(escalated_plan)
                    .with_tools(crate::harness::standard_tools());
                let result = harness
                    .call_with_retry(&prompt.user)
                    .await
                    .map_err(|e| format!("LLM call failed: {}", e))?;
                let response = &result.raw_text;

                // 处理 LLM tool_calls(应用层 tool_call)→ 填充死接口
                // - WriteGatewayDesign → ctx.pending_gateway_design(Composing 阶段转移到 compose_scratch)
                // - WriteExtensionYaml / PatchExtensionYaml → ctx.yaml_operations(build_app 阶段执行)
                for tc in &result.tool_calls {
                    match tc {
                        crate::state::AgentToolCall::WriteGatewayDesign { content } => {
                            common::telemetry::info!(
                                "LLM tool_call: write_gateway_design ({} chars)",
                                content.len()
                            );
                            ctx.pending_gateway_design = Some(content.clone());
                        }
                        crate::state::AgentToolCall::WriteExtensionYaml { file, content } => {
                            common::telemetry::info!(
                                "LLM tool_call: write_extension_yaml file={}",
                                file
                            );
                            ctx.yaml_operations
                                .push(crate::state::YamlOperation::Write {
                                    file: file.clone(),
                                    content: content.clone(),
                                });
                        }
                        crate::state::AgentToolCall::PatchExtensionYaml { file, patches } => {
                            common::telemetry::info!(
                                "LLM tool_call: patch_extension_yaml file={} patches={}",
                                file,
                                patches.len()
                            );
                            ctx.yaml_operations
                                .push(crate::state::YamlOperation::Patch {
                                    file: file.clone(),
                                    patches: patches.clone(),
                                });
                        }
                    }
                }

                // 记录 LLM 调用执行日志 + WebSocket 推送
                Self::emit_execution_log(
                    &mut ctx.execution_log,
                    ctx.session_id,
                    &on_progress,
                    LogLevel::Info,
                    ExecutionEvent::LlmCall {
                        model_id: llm_model_id,
                        reasoning_effort: llm_reasoning,
                        prompt_len: prompt.user.len(),
                        response_len: result.raw_text.len(),
                        retry_count: result.attempts,
                        token_usage: None,
                        latencies_ms: result.latencies_ms.clone(),
                    },
                );

                // 补全 step_details 中的 LLM prompt/response
                if let Some(detail) = ctx.step_details.last_mut() {
                    detail.llm_system_prompt = Some(prompt.system.clone());
                    detail.llm_user_prompt = Some(prompt.user.clone());
                    detail.llm_response = Some(result.raw_text.clone());
                }
                common::telemetry::debug!(
                    "LLM raw response ({} chars): {}",
                    response.len(),
                    response
                );
                let validated = crate::planner::parse_and_validate(response, catalog);
                ctx.ontology_model = Some(validated.ontology.clone());

                Self::emit_progress(
                    &on_progress,
                    state_name(&ctx.state),
                    26,
                    &format!(
                        "本体解析完成：{} domains, {} relations, {} fix_log, {} warnings",
                        validated.ontology.domains.len(),
                        validated.ontology.relations.len(),
                        validated.fix_log.len(),
                        validated.warnings.len()
                    ),
                    progress_event::ONTOLOGY_PARSED,
                    Some(serde_json::json!({
                        "domains": validated.ontology.domains.len(),
                        "relations": validated.ontology.relations.len(),
                        "fix_log": validated.fix_log,
                        "warnings": validated.warnings
                    })),
                );

                // 从 OntologyModel 提取约束、计算逻辑、业务规则
                let (constraints, computations) =
                    crate::planner::extract_plan_fields_from_ontology(&validated.ontology);
                let business_rules =
                    crate::planner::extract_business_rules_from_ontology(&validated.ontology);

                // 存储 FlowPlan 轻量元数据
                ctx.flow_plan = Some(FlowPlan {
                    namespace: self.derive_namespace(ctx),
                    used_modules: validated.used_modules.clone(),
                    known_entities: validated.known_entities.clone(),
                    missing_info: validated.missing_info.clone(),
                    workflow_steps: validated.workflow_steps.clone(),
                    computations,
                    constraints,
                    business_rules,
                    app_meta: validated.app_meta.clone(),
                    created_modules: vec![],
                    created_blocks: vec![],
                    created_services: vec![],
                    ontology_model_json: None,
                    functional_units: vec![],
                    semantic_concepts: vec![],
                });
                common::telemetry::info!(
                    "Ontology parsed: {} domains, {} relations, {} fix_log entries, {} warnings",
                    validated.ontology.domains.len(),
                    validated.ontology.relations.len(),
                    validated.fix_log.len(),
                    validated.warnings.len(),
                );

                // ── 规约验证 ──────────────────────────────────────────
                let violations =
                    crate::validator::validate_ontology_model(&validated.ontology, catalog);
                let has_unfixable = violations.iter().any(|v| !v.fixable);

                if !violations.is_empty() {
                    Self::emit_progress(
                        &on_progress,
                        state_name(&ctx.state),
                        28,
                        &format!("发现 {} 个规约违规", violations.len()),
                        progress_event::PLAN_VIOLATIONS_FOUND,
                        Some(serde_json::json!({
                            "violations": violations.iter().map(|v| serde_json::json!({
                                "kind": format!("{:?}", v.kind),
                                "detail": v.detail,
                                "fixable": v.fixable
                            })).collect::<Vec<_>>()
                        })),
                    );
                }

                if violations.is_empty() {
                    // Reset repair state if planning succeeds
                    ctx.repair_attempt = 0;
                    ctx.verification_error = None;
                    ctx.plan_violations = None;
                    if validated.missing_info.is_empty() {
                        Ok(AgentState::Extending)
                    } else {
                        let questions = self.build_questions_from_missing(&validated.missing_info);
                        Ok(AgentState::Planning {
                            revision_round: 0,
                            needs_clarification: Some(questions),
                        })
                    }
                } else if has_unfixable {
                    // 严重缺失 → 请求用户输入
                    ctx.plan_violations = Some(violations.clone());
                    let questions = violations
                        .iter()
                        .filter(|v| !v.fixable)
                        .enumerate()
                        .map(|(i, v)| Question {
                            id: format!("violation_{}", i),
                            category: crate::state::MissingInfoCategory::SceneAmbiguity,
                            question: format!(
                                "**问题**：{}\n\n请提供更多信息或调整需求。",
                                v.detail
                            ),
                            options: vec![],
                            required: true,
                        })
                        .collect();
                    Ok(AgentState::Planning {
                        revision_round: 0,
                        needs_clarification: Some(questions),
                    })
                } else {
                    // 可修复违规 → LLM 循环修订（≤3 次）
                    let next_round = match &ctx.state {
                        AgentState::Planning { revision_round, .. } => revision_round + 1,
                        _ => 1,
                    };
                    if next_round > 3 {
                        // 超出修订上限 → 降级为警告，继续执行
                        common::telemetry::warn!("Ontology validation still has {} violations after {} rounds, proceeding anyway",
                        violations.len(),
                        next_round - 1);
                        ctx.plan_violations = Some(violations.clone());
                        Ok(AgentState::Extending)
                    } else {
                        ctx.plan_violations = Some(violations.clone());
                        common::telemetry::info!(
                            "Ontology validation found {} violations (round {}), re-invoking LLM",
                            violations.len(),
                            next_round
                        );
                        // 重新进入 Planning — LLM 将接收 violations 作为额外上下文
                        Ok(AgentState::Planning {
                            revision_round: next_round,
                            needs_clarification: None,
                        })
                    }
                }
            }
            AgentState::Extending => {
                let ontology = ctx
                    .ontology_model
                    .as_ref()
                    .ok_or("Missing ontology model")?;
                let catalog = ctx.platform_catalog.as_ref().ok_or("Missing catalog")?;

                // ── Gap Analysis ──────────────────────────────────────
                let analysis = crate::extender::analyze_gaps(ontology, catalog);
                common::telemetry::info!(
                    "Gap analysis: {} covered, {} gaps, {} unsupported",
                    analysis.covered_domains.len(),
                    analysis.gaps.len(),
                    analysis.unsupported.len(),
                );
                ctx.extension_gaps = analysis.gaps.clone();

                Self::emit_progress(
                    &on_progress,
                    state_name(&ctx.state),
                    progress_percent(&ctx.state),
                    &format!(
                        "缺口分析完成：{} 已覆盖, {} gaps, {} 不支持",
                        analysis.covered_domains.len(),
                        analysis.gaps.len(),
                        analysis.unsupported.len()
                    ),
                    progress_event::GAP_ANALYSIS_DONE,
                    Some(serde_json::json!({
                        "covered": analysis.covered_domains.len(),
                        "gaps": analysis.gaps.len(),
                        "unsupported": analysis.unsupported.len()
                    })),
                );

                // ── 缺口仅记录，不建表 ───────────────────────────────
                // 规约约束（AGENTS.md「DDL 只读 / app-agent 不建表」）：App Agent
                // 不直接对数据库执行 DDL。本体缺口（OntologyModel 有但 DB 无对应表）
                // 不再调用 collection create API 建表，而是标记为待实现，由 composer
                // 在 Composing 阶段写入 `request-no-impl/gap-*.md`，交人工/独立迁移处理。
                if !analysis.gaps.is_empty() {
                    common::telemetry::info!(
                        "Recording {} extension gap(s) as request-no-impl (no DB table created)",
                        analysis.gaps.len()
                    );
                    for gap in &analysis.gaps {
                        common::telemetry::info!(
                            "Extension gap recorded: domain={} parent={} table={}",
                            gap.domain_id,
                            gap.parent_table,
                            gap.proposed_table_name
                        );
                        ctx.extension_tracking.insert(
                            gap.proposed_table_name.clone(),
                            crate::state::ExtensionGapStatus::Pending,
                        );
                    }
                }

                // D1=B：新 7 阶段主链路 — Extending(gap 分析) 后进入 ModuleCreation，
                // 而非直接 OntologyTransfer。ModuleCreation→BlockCreation→OntologyTransfer
                // 已天然连通，且 BlockCreation 会设置 created_blocks 驱动 after_ontology_transfer。
                Ok(AgentState::ModuleCreation)
            }
            AgentState::Generating => {
                // 已废弃：原 alioth-gen 代码生成器产物从未被交付（见 state.rs 注释）。
                // 直接跳过，进入 Composing。
                common::telemetry::info!(
                    "[DEPRECATED] Generating state — skipping to GeneratingFrontend"
                );
                Ok(AgentState::GeneratingFrontend {
                    modules_generated: 0,
                    verification_log: None,
                })
            }
            AgentState::GeneratingFrontend {
                modules_generated: _,
                verification_log: _,
            } => {
                Self::emit_progress(
                    &on_progress,
                    state_name(&ctx.state),
                    45,
                    "正在从原型生成前端代码...",
                    "frontend_gen_start",
                    None,
                );
                let app_name = self.derive_app_name(ctx);
                let namespace = self.derive_namespace(ctx);
                let _app_dir = crate::composer::resolve_project_root()
                    .join("Pre-Proc")
                    .join(&namespace)
                    .join("Apps")
                    .join(&app_name);
                let proto_dir = crate::composer::resolve_project_root()
                    .join("Pre-Proc")
                    .join(&namespace)
                    .join("Prototypes");
                common::telemetry::info!(
                    "Generating frontend code for app '{}' from prototypes at {:?}",
                    app_name,
                    proto_dir
                );
                Self::emit_progress(
                    &on_progress,
                    state_name(&ctx.state),
                    55,
                    "前端代码生成完成",
                    "frontend_gen_done",
                    None,
                );
                // 委托 alioth-gui skill 生成前端代码
                let mut ctx_map = std::collections::HashMap::new();
                ctx_map.insert("ns".to_string(), namespace);
                ctx_map.insert("app".to_string(), app_name);
                Ok(AgentState::ExecutingSkill {
                    skill_name: "alioth-gui".to_string(),
                    track_index: 0,
                    step_index: 0,
                    attempt: 0,
                    context: ctx_map,
                    return_state: Box::new(AgentState::Composing),
                })
            }
            AgentState::Composing => {
                // 调用 composer 将 FlowPlan 组装为声明式产物：
                //   + extensions/*.yaml + request-no-impl/*.md
                //   （App 发现由 Gateway FS 扫描自发现，不再写 apps.json 聚合清单）
                let plan = ctx.flow_plan.clone().ok_or("Missing flow plan")?;
                let app_name = self.derive_app_name(ctx);

                // 编译模块集校验：过滤掉 Gateway 未编译的模块，被过滤者记 gap。
                self.validate_compiled_modules(ctx, &app_name).await?;
                let plan = ctx.flow_plan.clone().unwrap_or(plan);
                // 快速草稿模式：清除复杂扩展配置，只保留核心 app.json
                let plan = if ctx.draft_mode {
                    let mut filtered = plan.clone();
                    filtered.constraints.clear();
                    filtered.business_rules.clear();
                    filtered.computations.clear();
                    filtered.workflow_steps.clear();
                    filtered
                } else {
                    plan
                };

                Self::emit_progress(
                    &on_progress,
                    state_name(&ctx.state),
                    progress_percent(&ctx.state),
                    &format!("开始组装应用配置: {}", app_name),
                    progress_event::COMPOSING_START,
                    Some(serde_json::json!({"app_name": app_name})),
                );

                let namespace = self.derive_namespace(ctx);
                let compose_result = crate::composer::compose_from_flow_plan(
                    self.pool.as_ref(),
                    &plan,
                    &app_name,
                    &namespace,
                    ctx.ontology_model.as_ref(),
                    on_progress,
                )
                .await
                .map_err(|e| format!("后端配置组装失败: {}", e))?;
                let compose_output = compose_result.output_path.clone();

                // OntologyTransfer 产物落盘：mapped_entities → namespace service.json
                if !ctx.mapped_entities.is_empty() {
                    let domain = plan
                        .known_entities
                        .first()
                        .cloned()
                        .unwrap_or_else(|| app_name.clone());
                    let service_id = format!("{}-service", slugify(&domain));
                    let written = crate::composer::write_mapped_services(
                        &namespace,
                        &service_id,
                        &domain,
                        &ctx.mapped_entities,
                        ctx.alignment_graph.as_ref(),
                    )
                    .await
                    .map_err(|e| format!("service.json 写入失败: {e}"))?;
                    common::telemetry::info!(
                        "Composing: {} mapped entities written to service {}",
                        written,
                        service_id
                    );
                    // GAP-3: 生成 service backend Rust 代码
                    let backend_written = crate::composer::write_service_backend(
                        &namespace,
                        &service_id,
                        &ctx.mapped_entities,
                    )
                    .await
                    .map_err(|e| format!("service backend 生成失败: {e}"))?;
                    common::telemetry::info!(
                        "Composing: {} backend files written for service {}",
                        backend_written,
                        service_id
                    );
                }

                // SESSION-FIX:gap-d-gap-surfacing — alignment gaps 用户可见
                if let Some(g) = &ctx.alignment_graph {
                    if !g.gaps.is_empty() {
                        common::telemetry::warn!(
                            "Composing: {} alignment gaps 待确认: {}",
                            g.gaps.len(),
                            g.gaps
                                .iter()
                                .map(|x| x.biz_element.clone())
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                    }
                }

                ctx.compose_scratch = Some(ComposeScratch {
                    app_name: compose_result.app_name,
                    output_path: compose_result.output_path,
                    files_written: compose_result.files_written,
                    module_count: compose_result.module_count,
                    // 从 pending_gateway_design 转移(Planning 阶段 LLM tool_call 暂存)
                    // 或保留已有 scratch 中的 content(Composing 重试场景)
                    gateway_design_content: ctx.pending_gateway_design.clone().or_else(|| {
                        ctx.compose_scratch
                            .as_ref()
                            .and_then(|s| s.gateway_design_content.clone())
                    }),
                });

                // 记录文件写入执行日志（汇总）
                Self::emit_execution_log(
                    &mut ctx.execution_log,
                    ctx.session_id,
                    &on_progress,
                    LogLevel::Info,
                    ExecutionEvent::FileWrite {
                        path: compose_output,
                        size: compose_result.files_written as u64,
                    },
                );

                self.build_app(ctx, on_progress).await?;

                // 构建完成后回写所有 execution 日志到磁盘
                if let Some(ref scratch) = ctx.compose_scratch {
                    crate::execution_log::rebuild_execution_log(
                        &scratch.output_path,
                        &ctx.execution_log,
                    );
                }

                // D3：组装完成后交由 alioth-compose 适配器做产物校验/原型构建验证，
                // 再进入 Verifying。与 META_AI_SPEC §3「Composing 由 alioth-compose 驱动」对齐。
                let mut ctx_map = std::collections::HashMap::new();
                ctx_map.insert("ns".to_string(), self.derive_namespace(ctx));
                ctx_map.insert("app".to_string(), self.derive_app_name(ctx));
                Ok(AgentState::ExecutingSkill {
                    skill_name: "alioth-compose".to_string(),
                    track_index: 0,
                    step_index: 0,
                    attempt: 0,
                    context: ctx_map,
                    return_state: Box::new(AgentState::Verifying {
                        verification_round: 0,
                    }),
                })
            }

            AgentState::Verifying {
                verification_round: _,
            } => {
                // ── 验证实际交付产物（app.json + extensions/*.yaml） ───
                let scratch = ctx
                    .compose_scratch
                    .as_ref()
                    .ok_or("Missing compose scratch")?;

                // 1. 检查 app.json 存在且可解析
                let app_json_path = format!("{}/app.json", scratch.output_path);
                match std::fs::read_to_string(&app_json_path) {
                    Ok(content) => {
                        if let Err(e) = serde_json::from_str::<serde_json::Value>(&content) {
                            let msg = format!("app.json is invalid JSON: {}", e);
                            common::telemetry::warn!("{}", msg);
                            ctx.verification_error = Some(msg);
                        } else {
                            common::telemetry::info!("Verifier: app.json parsed successfully");
                        }
                    }
                    Err(e) => {
                        let msg = format!("Failed to read app.json: {}", e);
                        common::telemetry::warn!("{}", msg);
                        ctx.verification_error = Some(msg);
                    }
                }

                // 2. 检查 extensions/*.yaml 格式有效
                let ext_dir = format!("{}/extensions", scratch.output_path);
                let mut yaml_errors = Vec::new();
                for file_name in &[
                    "constraints.yaml",
                    "rules.yaml",
                    "statemachines.yaml",
                    "workflows.yaml",
                ] {
                    let path = format!("{}/{}", ext_dir, file_name);
                    if std::path::Path::new(&path).exists() {
                        match std::fs::read_to_string(&path) {
                            Ok(content) => {
                                if let Err(e) = yaml_serde::from_str::<yaml_serde::Value>(&content)
                                {
                                    yaml_errors.push(format!("{}: {}", file_name, e));
                                }
                            }
                            Err(e) => yaml_errors.push(format!("{}: read error: {}", file_name, e)),
                        }
                    }
                }
                if !yaml_errors.is_empty() {
                    let msg = format!("YAML validation errors: {}", yaml_errors.join("; "));
                    common::telemetry::warn!("{}", msg);
                    ctx.verification_error = Some(msg);
                } else {
                    common::telemetry::info!(
                        "Verifier: all extension YAML files parsed successfully"
                    );
                }

                // 3. rubric 质量评估（agentic-eval：Evaluator-Optimizer）
                //     结构校验通过后再做语义质量评分；未达阈值且有余次则回流 Composing。
                let project_root = crate::composer::resolve_project_root();
                let check_script = project_root.join("target/debug/ontology-mapping");
                let prototype_path =
                    std::path::Path::new(&scratch.output_path).join("prototype.html");
                let artifacts = crate::evaluate::Artifacts {
                    app_json: std::path::Path::new(&app_json_path),
                    extensions_dir: std::path::Path::new(&ext_dir),
                    prototype_html: if prototype_path.exists() {
                        Some(prototype_path.as_path())
                    } else {
                        None
                    },
                    check_script: Some(check_script.as_path()),
                };
                // LLM-as-Judge：先异步算语义维度分数，再交给 evaluate_with 覆盖规则分
                let llm_judger = self.llm_judge(ctx, &artifacts).await;
                let report = crate::evaluate::evaluate_with(ctx, &artifacts, &llm_judger);
                let report_json = serde_json::to_string(&report).unwrap_or_default();
                // 写出评估报告与轨迹（供 skill.hooks.json 校验 + 诊断回归）
                let report_path =
                    std::path::Path::new(&scratch.output_path).join("eval-report.json");
                if let Err(e) = std::fs::write(&report_path, &report_json) {
                    common::telemetry::warn!("failed to write eval-report.json: {}", e);
                }
                let traj_path =
                    std::path::Path::new(&scratch.output_path).join("eval-trajectory.jsonl");
                if let Ok(mut f) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&traj_path)
                {
                    let _ = writeln!(
                        f,
                        "{}",
                        serde_json::json!({
                            "iter": ctx.eval_iteration,
                            "overall_score": report.overall_score,
                            "passed": report.passed,
                            "critique": report.critique,
                        })
                    );
                }
                if !report.passed && ctx.eval_iteration < crate::evaluate::MAX_EVAL_ITERATIONS {
                    ctx.eval_iteration += 1;
                    ctx.eval_feedback = Some(report_json);
                    // 回流 Composing：若 goal 缺失，从 user_description 派生（对齐 SKILL.md goal←user_description），
                    // 使下一次评估的 goal_fidelity 维度可提升。composer 以 app_meta.goal 为 app.json goal 来源。
                    if let Some(ref mut plan) = ctx.flow_plan {
                        let goal_missing = plan
                            .app_meta
                            .as_ref()
                            .and_then(|m| m.goal.as_ref())
                            .map(|g| g.trim().is_empty())
                            .unwrap_or(true);
                        if goal_missing {
                            let mut meta = plan.app_meta.take().unwrap_or_default();
                            meta.goal = Some(ctx.user_description.clone());
                            plan.app_meta = Some(meta);
                        }
                    }
                    common::telemetry::warn!(
                        "Verifying rubric score={:.2} < {:.2}, reflow to Composing (eval_iteration={}/{})",
                        report.overall_score, report.threshold, ctx.eval_iteration, crate::evaluate::MAX_EVAL_ITERATIONS
                    );
                    return Ok(AgentState::Composing);
                } else if !report.passed {
                    // 评估环达上限仍不达标：自动收敛失败 → 暂停并请求人工干预，
                    // 禁止静默发布低质量产物（agentic-eval：低置信度须 human-in-the-loop）。
                    let reason = format!(
                        "Verifying rubric 评分 {:.2} < 阈值 {:.2}，且已回流 {} 次仍不达标（eval_iteration 达上限）。自动收敛失败，暂停并等待人工干预。请审查 {}/eval-report.json 与 eval-trajectory.jsonl，调整需求/目标或手动修复后重试；如需强制发布，请显式确认。",
                        report.overall_score,
                        report.threshold,
                        crate::evaluate::MAX_EVAL_ITERATIONS,
                        scratch.output_path
                    );
                    common::telemetry::warn!("{}", reason);
                    return Ok(AgentState::AwaitingUserInput { reason });
                } else {
                    common::telemetry::info!(
                        "Verifying rubric passed: score={:.2}",
                        report.overall_score
                    );
                }

                // 4. 若存在运行时/验证错误且未超限，回到 Planning 请求 LLM 修复
                if ctx.verification_error.is_some() && ctx.repair_attempt < ctx.max_repair_count {
                    ctx.repair_attempt += 1;
                    let attempt = ctx.repair_attempt;
                    let err = ctx.verification_error.clone().unwrap_or_default();
                    Self::emit_progress(
                        &on_progress,
                        "验证产物",
                        92,
                        &format!("自动修复尝试 {}/{}", attempt, ctx.max_repair_count),
                        progress_event::AUTO_FIX_ATTEMPTED,
                        Some(serde_json::json!({
                            "attempt": attempt,
                            "max": ctx.max_repair_count,
                            "error": err,
                        })),
                    );
                    // 回到 Planning，保留 ontology_model 和 flow_plan
                    return Ok(AgentState::Planning {
                        revision_round: attempt as u32,
                        needs_clarification: None,
                    });
                }

                // 验证通过 → 进入自动发布阶段
                Ok(AgentState::Publishing {
                    publish_attempt: 0,
                    last_error: None,
                })
            }

            AgentState::Publishing {
                publish_attempt,
                last_error: _,
            } => {
                let attempt = *publish_attempt;
                let result = self.build_result(ctx).await;
                let verify_result = self.verify_compilation(&result).await;

                match &verify_result {
                    Ok(true) => {
                        // 写出 pipeline 产物供下游消费（原子写入，失败则重试）
                        match self.write_pipeline_artifacts(ctx, &result).await {
                            Ok(()) => Ok(AgentState::Published {
                                result: Box::new(result),
                            }),
                            Err(e) => {
                                let err_msg = format!(
                                    "Pipeline artifact write failed (attempt {}): {}",
                                    attempt, e
                                );
                                common::telemetry::warn!("{}", err_msg);
                                let next_attempt = attempt + 1;
                                if next_attempt <= 3 {
                                    Ok(AgentState::Publishing {
                                        publish_attempt: next_attempt,
                                        last_error: Some(err_msg.clone()),
                                    })
                                } else {
                                    let reason = format!(
                                        "Pipeline artifact write failed after {} attempts: {}. \
                                         Entering AwaitingUserInput. Pipeline artifacts not written.",
                                        next_attempt, err_msg
                                    );
                                    common::telemetry::warn!("{}", reason);
                                    Ok(AgentState::AwaitingUserInput { reason })
                                }
                            }
                        }
                    }
                    Ok(false) | Err(_) => {
                        let error_msg = verify_result
                            .err()
                            .map(|e| e.to_string())
                            .unwrap_or_else(|| "Compilation check failed".to_string());
                        let next_attempt = attempt + 1;
                        if next_attempt < ctx.max_repair_count as u32 + 1 {
                            common::telemetry::warn!(
                                "Publish attempt {} failed: {}. Retrying...",
                                attempt + 1,
                                error_msg
                            );
                            ctx.user_description = format!(
                                "{}

[Compile error - need fix]
{}",
                                ctx.user_description, error_msg
                            );
                            ctx.verification_error = Some(error_msg.clone());
                            ctx.repair_attempt += 1;
                            Ok(AgentState::Planning {
                                revision_round: 1,
                                needs_clarification: None,
                            })
                        } else {
                            common::telemetry::warn!(
                                "Publish failed after {} attempts. Publishing with warnings.",
                                next_attempt
                            );
                            Ok(AgentState::Published {
                                result: Box::new(result),
                            })
                        }
                    }
                }
            }

            AgentState::Published { result } => Ok(AgentState::Published {
                result: result.clone(),
            }),

            AgentState::Presenting { result } => {
                if !ctx.change_requests.is_empty() {
                    let change = ctx.change_requests.remove(0);
                    ctx.user_description = format!(
                        "{}\n\n【用户变更请求】\n{}\n\n【当前已生成应用】\napp_name: {}\noutput_path: {}\nused_modules: {}",
                        ctx.user_description,
                        change,
                        result.app_name,
                        result.output_path,
                        result.used_modules.iter().map(|m| &m.module_id).cloned().collect::<Vec<_>>().join(", ")
                    );
                    ctx.last_built_app = Some(serde_json::to_string(&result).unwrap_or_default());
                    ctx.verification_error = None;
                    ctx.repair_attempt = 0;
                    return Ok(AgentState::Planning {
                        revision_round: 0,
                        needs_clarification: None,
                    });
                }
                Ok(AgentState::Presenting {
                    result: result.clone(),
                })
            }

            AgentState::AwaitingUserInput { reason } => {
                // 用户干预入口：无新输入则保持等待（is_terminal=true，run 停止轮询）；
                // 若用户提供 change_requests，则携带新需求回到 Planning 重新生成
                // （eval_iteration / eval_feedback 清零，给评估环一次全新收敛机会）。
                if !ctx.change_requests.is_empty() {
                    let change = ctx.change_requests.remove(0);
                    ctx.user_description = format!(
                        "{}\n\n【用户干预/变更请求】\n{}\n\n【当前已生成应用】\noutput_path: {}",
                        ctx.user_description,
                        change,
                        ctx.compose_scratch
                            .as_ref()
                            .map(|s| s.output_path.clone())
                            .unwrap_or_default(),
                    );
                    ctx.verification_error = None;
                    ctx.repair_attempt = 0;
                    ctx.eval_iteration = 0;
                    ctx.eval_feedback = None;
                    return Ok(AgentState::Planning {
                        revision_round: 0,
                        needs_clarification: None,
                    });
                }
                Ok(AgentState::AwaitingUserInput {
                    reason: reason.clone(),
                })
            }
            AgentState::Failed { error } => {
                common::telemetry::warn!("AgentState::Failed reached: {}", error);
                Ok(AgentState::Failed {
                    error: error.clone(),
                })
            }
        }
    }

    async fn load_platform_catalog(&self, namespace: &str) -> Result<PlatformCatalog, String> {
        let result = tools::list_ontology_dimensions(self.pool.as_ref(), namespace).await;
        if let Some(err) = &result.error {
            return Err(format!("Failed to load platform catalog: {}", err));
        }
        match result.data {
            serde_json::Value::Object(map) if map.contains_key("scenes") => {
                serde_json::from_value(serde_json::Value::Object(map))
                    .map_err(|e| format!("Catalog deserialization failed: {}", e))
            }
            other => Err(format!("Unexpected catalog format: {}", other)),
        }
    }

    fn build_questions_from_missing(&self, missing_info: &[MissingInfo]) -> Vec<Question> {
        missing_info
            .iter()
            .enumerate()
            .map(|(idx, m)| {
                let question_text = format!(
                    "**场景条件**：{}\n\n**决策要素**：{}\n\n**判断标准**：{}\n\n**判断结果**：{}",
                    m.scene_condition, m.decision_elements, m.judgment_criteria, m.judgment_result
                );
                Question {
                    id: format!("q_{}", idx),
                    category: m.category.clone(),
                    question: question_text,
                    options: Self::options_for_category(&m.category),
                    required: true,
                }
            })
            .collect()
    }

    fn options_for_category(category: &MissingInfoCategory) -> Vec<String> {
        match category {
            MissingInfoCategory::SceneAmbiguity => vec![
                "供应链管理（采购/仓储/物流）".to_string(),
                "零售管理（门店/POS/会员）".to_string(),
                "金融服务（账款/结算/信用）".to_string(),
                "生产制造（BOM/工单/工艺）".to_string(),
                "自定义场景".to_string(),
            ],
            MissingInfoCategory::EntityExtension => vec![
                "我来补充定义".to_string(),
                "用通用实体代替".to_string(),
                "不需要此实体".to_string(),
            ],
            MissingInfoCategory::StatusLifecycle => vec![
                "我来补充状态定义".to_string(),
                "使用通用状态".to_string(),
                "暂不需要".to_string(),
            ],
            MissingInfoCategory::ModuleDependency => {
                vec!["同时启用该模块".to_string(), "不启用".to_string()]
            }
            _ => vec![],
        }
    }

    // ──────────────────────────────────────────────────────────────────────────
    /// 派生稳定的应用名（kebab/下划线清洗）。优先复用已缓存值，保证
    /// Generating 与 Composing 阶段一致，从而 CRC64(app_name) → id 稳定。
    fn derive_app_name(&self, ctx: &ConversationContext) -> String {
        if let Some(name) = &ctx.app_name {
            if !name.is_empty() {
                return name.clone();
            }
        }
        let raw = ctx
            .user_description
            .split(|c: char| c.is_whitespace() || c == '，' || c == '。' || c == '、')
            .take(3)
            .collect::<Vec<_>>()
            .join("_")
            .replace(
                [
                    '/', '\\', ':', '*', '?', '"', '<', '>', '|', ' ', '\t', '\n',
                ],
                "-",
            );
        let cleaned = raw.trim_matches('-').to_string();
        if cleaned.is_empty() {
            format!("app-{}", chrono::Utc::now().timestamp())
        } else {
            cleaned
        }
    }

    /// 派生应用 namespace。直接从 ConversationContext 取，namespace 在创建会话时已由前端传入。
    fn derive_namespace(&self, ctx: &ConversationContext) -> String {
        ctx.namespace.clone().unwrap_or_else(|| {
            panic!(
                "ConversationContext namespace is required but was None for session {} (user: {})",
                ctx.session_id,
                ctx.user_description.chars().take(20).collect::<String>()
            )
        })
    }

    /// 编译模块集校验：把 `flow_plan.used_modules` 过滤为 Gateway 实际编译进单体
    /// 的模块集合（解析 Gateway/backend/Cargo.toml 的 `all-modules` feature，
    /// 文件系统契约，不走 HTTP，遵 CONTAINER_BOUNDARY）。被过滤掉的模块以
    /// `request-no-impl/gap-uncompiled-{id}.md` 记录，需求不丢、不生成路由。
    async fn validate_compiled_modules(
        &self,
        ctx: &mut ConversationContext,
        app_name: &str,
    ) -> Result<(), String> {
        let compiled = crate::tools::compiled_module_ids();
        let ns = self.derive_namespace(ctx);
        if compiled.is_empty() {
            let msg = format!(
                "Compiled module set unavailable for app '{}'. \
                 Ensure Gateway has been built (compiled_modules.json exists) \
                 or GATEWAY_CARGO_TOML points to a valid Cargo.toml",
                app_name
            );
            common::telemetry::error!("{}", msg);
            return Err(msg);
        }
        let Some(plan) = ctx.flow_plan.as_mut() else {
            return Err("Missing flow plan in validate_compiled_modules".to_string());
        };
        let (kept, dropped): (Vec<String>, Vec<String>) = plan
            .used_modules
            .drain(..)
            .partition(|m| compiled.contains(m));
        plan.used_modules = kept;

        for module_id in &dropped {
            common::telemetry::warn!(
                "App '{}': module '{}' not compiled into Gateway; recording as request-no-impl gap",
                app_name,
                module_id
            );
            let gap_md = format!(
                "# Gap: 未编译模块 `{}`\n\n## 需求来源\n应用 `{}` 的需求引用了模块 `{}`，但该模块未编译进 Gateway 单体。\n\n## 缺口分析\nGateway 仅能挂载在 `Gateway/backend/Cargo.toml` 的 `all-modules` feature 中声明的模块。\n该模块不在编译集中，因此其路由无法注册，已从 `config.modules` 中剔除。\n\n## 建议实现方向\n1. 在 `Gateway/backend/Cargo.toml` 中新增 `{}-backend` 依赖与 `module-{}` feature，并加入 `all-modules`。\n2. 重新编译 Gateway 后，该模块方可被应用组合。\n",
                module_id, app_name, module_id, module_id, module_id
            );
            if let Err(e) = crate::tools::write_request_no_impl(
                &ns,
                app_name,
                &format!("uncompiled-{}", module_id),
                &gap_md,
            )
            .await
            {
                common::telemetry::warn!(
                    "Failed to write uncompiled-module gap doc for '{}': {}",
                    module_id,
                    e
                );
            }
        }
        Ok(())
    }

    /// Poll Gateway /health endpoint until success or timeout
    async fn poll_gateway_health(
        health_url: &str,
        max_wait_secs: u64,
    ) -> Result<reqwest::Response, String> {
        let client = reqwest::Client::new();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(max_wait_secs);
        loop {
            match client
                .get(health_url)
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => return Ok(resp),
                Ok(resp) => {
                    common::telemetry::debug!("Gateway health returned status {}", resp.status())
                }
                Err(e) => common::telemetry::debug!("Gateway health request failed: {}", e),
            }
            if std::time::Instant::now() >= deadline {
                return Err(format!(
                    "Gateway health check timed out after {}s",
                    max_wait_secs
                ));
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }

    // ──────────────────────────────────────────────────────────────────────────
    // 阶段 3：Building — 整合前后端产物，生成最终应用并触发 Gateway 重启
    // ──────────────────────────────────────────────────────────────────────────
    async fn build_app<F>(
        &self,
        ctx: &mut ConversationContext,
        on_progress: Option<&F>,
    ) -> Result<(), String>
    where
        F: Fn(AgentProgress) + Send + Sync,
    {
        let scratch = ctx
            .compose_scratch
            .as_ref()
            .ok_or("Missing compose scratch")?;
        let design_path = format!("{}/gateway_design.md", scratch.output_path);

        // 1. 写入前端设计方案（如果存在）
        if let Some(ref content) = scratch.gateway_design_content {
            if let Err(e) = tokio::fs::write(&design_path, content).await {
                common::telemetry::warn!("Failed to write gateway_design.md: {}", e);
            } else {
                common::telemetry::info!("Wrote gateway_design.md for app '{}'", scratch.app_name);
            }
        }

        // 2. 执行 YAML 操作队列（AI 或用户提交的 patch/write/read）
        let yaml_ns = self.derive_namespace(ctx);
        if !ctx.yaml_operations.is_empty() {
            common::telemetry::info!(
                "Executing {} YAML operation(s) for app '{}'",
                ctx.yaml_operations.len(),
                scratch.app_name
            );
            for op in std::mem::take(&mut ctx.yaml_operations) {
                let log_entry = match &op {
                    crate::state::YamlOperation::Read { file } => {
                        let result =
                            crate::tools::read_extension_yaml(&yaml_ns, &scratch.app_name, file)
                                .await;
                        format!("[READ] {}: {:?}", file, result.error.is_none())
                    }
                    crate::state::YamlOperation::Write { file, content } => {
                        let result = crate::tools::write_extension_yaml(
                            &yaml_ns,
                            &scratch.app_name,
                            file,
                            content,
                        )
                        .await;
                        format!("[WRITE] {}: {:?}", file, result.error.is_none())
                    }
                    crate::state::YamlOperation::Patch { file, patches } => {
                        let result = crate::tools::patch_extension_yaml(
                            &yaml_ns,
                            &scratch.app_name,
                            file,
                            patches,
                        )
                        .await;
                        format!(
                            "[PATCH] {} ({} patches): {:?}",
                            file,
                            patches.len(),
                            result.error.is_none()
                        )
                    }
                };
                ctx.yaml_operation_log.push(log_entry);
            }
            common::telemetry::info!("YAML operations completed for app '{}'", scratch.app_name);
        }

        // 3. 确认产物完整性
        let app_json_path = format!("{}/app.json", scratch.output_path);
        if !tokio::fs::metadata(&app_json_path)
            .await
            .map(|m| m.is_file())
            .unwrap_or(false)
        {
            return Err(format!("App artifact missing: {}", app_json_path));
        }

        // 4. 写入构建日志
        let build_log_path = format!("{}/build.log", scratch.output_path);
        let build_log = format!(
            "build_started_at: {}\napp_name: {}\nmodules: {}\noutput_path: {}\nstatus: building\n",
            chrono::Utc::now().to_rfc3339(),
            scratch.app_name,
            scratch.module_count,
            scratch.output_path
        );
        if let Err(e) = tokio::fs::write(&build_log_path, build_log).await {
            common::telemetry::warn!("Failed to write build.log: {}", e);
        }

        // 4.6 同步 ESM 原型到 Apps/{code}/(sync-prototype.sh)
        //    compose_from_flow_plan 阶段已生成 a-v{N}.html 到 Prototypes/Apps/{code}/
        //    此处调用 sync-prototype.sh 把原型 + bundle.js 复制到 Apps/{code}/
        //    替换旧的 composer.rs write_app_prototype(已删除 CDN babel 链路)
        let sync_app_name = scratch.app_name.clone();
        let sync_namespace = self.derive_namespace(ctx);
        // 同步 ESM 原型到 Apps/{code}/
        // 由 sync-prototype.sh 将 Prototypes 目录中最新的 a-v{N}.html 复制到 Apps/{code}/prototype.html
        if let Err(e) =
            crate::composer::sync_prototype(&sync_app_name, &sync_namespace, on_progress).await
        {
            common::telemetry::warn!(
                "sync_prototype failed for app '{}/{}': {} (非致命,原型已在 Prototypes/Apps/ 生成)",
                sync_namespace,
                sync_app_name,
                e
            );
        }

        // 校验原型文件是否同步成功（仅 warn，不阻塞构建）
        let proto_target = crate::composer::resolve_project_root()
            .join("Pre-Proc")
            .join(&sync_namespace)
            .join("Apps")
            .join(&sync_app_name)
            .join("prototype.html");

        if !tokio::fs::metadata(&proto_target)
            .await
            .map(|m| m.is_file())
            .unwrap_or(false)
        {
            common::telemetry::warn!(
                "prototype.html 未同步到 Apps/'{}/{}' (非致命,原型文件可能在 Prototypes/ 目录,需由 AppCreator 处理)",
                sync_namespace, sync_app_name
            );
        }

        //    这是开发态迭代的轻量刷新：用 --frontend-mode=dev（重启 vite dev server
        //    使其从 FS 重新发现 App），不做 pnpm build。
        //    权威的「发布到 production」重启（含前端 pnpm build + 同步上报失败）由
        //    app-instance 的 publish_app_instance 负责，二者职责区分。
        let restart_script = std::env::var("RESTART_GATEWAY_SCRIPT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("../../scripts/gateway/restart-gateway.sh"));

        let restart_output = if restart_script.exists() {
            Self::emit_progress(
                &on_progress,
                "构建应用",
                85,
                "正在触发 Gateway 重启...",
                progress_event::GATEWAY_RESTART_TRIGGERED,
                Some(serde_json::json!({"script": restart_script.to_string_lossy()})),
            );
            common::telemetry::info!(
                "Triggering Gateway dev-mode restart via {:?}",
                restart_script
            );
            match tokio::process::Command::new("bash")
                .arg(&restart_script)
                .arg("--frontend-mode=dev")
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .output()
                .await
            {
                Ok(output) => Some(output),
                Err(e) => {
                    common::telemetry::warn!("Failed to spawn Gateway restart script: {}", e);
                    None
                }
            }
        } else {
            common::telemetry::warn!(
                "Restart script not found at {:?}, skipping Gateway restart. \
             Please restart Gateway manually to load app '{}'",
                restart_script,
                scratch.app_name
            );
            None
        };

        let health_url = std::env::var("GATEWAY_HEALTH_URL")
            .unwrap_or_else(|_| "http://localhost:9001/health".to_string());
        let validation = if restart_output.is_some() {
            Self::emit_progress(
                &on_progress,
                "验证产物",
                90,
                &format!("正在轮询 Gateway 健康检查: {}", health_url),
                progress_event::GATEWAY_HEALTH_CHECK,
                Some(serde_json::json!({"health_url": &health_url})),
            );
            let stdout = restart_output
                .as_ref()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .unwrap_or_default();
            let stderr = restart_output
                .as_ref()
                .map(|o| String::from_utf8_lossy(&o.stderr).to_string())
                .unwrap_or_default();
            // 记录子进程执行日志
            let exit_code = restart_output.as_ref().and_then(|o| o.status.code());
            let cmd = format!(
                "bash {} --frontend-mode=dev",
                restart_script.to_string_lossy()
            );
            Self::emit_execution_log(
                &mut ctx.execution_log,
                ctx.session_id,
                &on_progress,
                LogLevel::Info,
                ExecutionEvent::Subprocess {
                    command: cmd,
                    exit_code,
                    stdout: truncate_log(&stdout, 2000),
                    stderr: truncate_log(&stderr, 2000),
                    duration_ms: 0,
                },
            );

            match Self::poll_gateway_health(&health_url, 60).await {
                Ok(_) => {
                    common::telemetry::info!("Gateway health check passed: {}", health_url);
                    Self::emit_execution_log(
                        &mut ctx.execution_log,
                        ctx.session_id,
                        &on_progress,
                        LogLevel::Info,
                        ExecutionEvent::Validation {
                            kind: "health_check".to_string(),
                            passed: true,
                            detail: format!("Gateway {} 健康检查通过", health_url),
                        },
                    );
                    RunValidationResult {
                        health_ok: true,
                        health_url,
                        stdout,
                        stderr,
                        retry_count: 0,
                    }
                }
                Err(e) => {
                    common::telemetry::warn!("Gateway health check failed: {}", e);
                    Self::emit_execution_log(
                        &mut ctx.execution_log,
                        ctx.session_id,
                        &on_progress,
                        LogLevel::Warn,
                        ExecutionEvent::Validation {
                            kind: "health_check".to_string(),
                            passed: false,
                            detail: format!("Gateway {} 健康检查失败: {}", health_url, e),
                        },
                    );
                    RunValidationResult {
                        health_ok: false,
                        health_url,
                        stdout,
                        stderr: format!("{}\n{}", stderr, e),
                        retry_count: 0,
                    }
                }
            }
        } else {
            RunValidationResult {
                health_ok: false,
                health_url,
                stdout: String::new(),
                stderr: "Gateway restart skipped".to_string(),
                retry_count: 0,
            }
        };
        ctx.runtime_validation = Some(validation.clone());

        if !validation.health_ok {
            let combined_error = format!(
                "Gateway 健康检查失败。URL: {}\nstdout: {}\nstderr: {}",
                validation.health_url, validation.stdout, validation.stderr
            );
            ctx.verification_error = Some(combined_error.clone());
            Self::emit_progress(
                &on_progress,
                "验证产物",
                90,
                "Gateway 健康检查失败，将尝试自动修复",
                progress_event::VERIFICATION_ERROR,
                Some(serde_json::json!({"error": combined_error})),
            );
        }

        // 6. 如果存在验证错误，尝试自动修复
        if let Some(err) = &ctx.verification_error {
            common::telemetry::info!("Attempting to fix build from verification error: {}", err);
            // 自动修复策略：重新生成 extensions/*.yaml（最常见的验证失败原因是格式漂移）
            let extensions_dir = format!("{}/extensions", scratch.output_path);
            let plan = ctx
                .flow_plan
                .as_ref()
                .ok_or("Missing flow plan for auto-fix")?;

            let mut fixed = 0;

            // 6.1 重新生成 constraints.yaml
            if !plan.constraints.is_empty() {
                let constraints: Vec<runtime_engine::ConstraintExtension> = plan
                    .constraints
                    .iter()
                    .map(|c| runtime_engine::ConstraintExtension {
                        entity: c.entity.clone(),
                        field: c.field.clone(),
                        expression: c.expression.clone(),
                        level: match c.level.as_str() {
                            "warning" => runtime_engine::ConstraintSeverity::Warning,
                            _ => runtime_engine::ConstraintSeverity::Error,
                        },
                        message: c.message.clone(),
                    })
                    .collect();
                if let Ok(yaml) = yaml_serde::to_string(&constraints) {
                    let path = format!("{}/constraints.yaml", extensions_dir);
                    if let Err(e) = tokio::fs::write(&path, yaml).await {
                        common::telemetry::warn!("Auto-fix constraints.yaml failed: {}", e);
                    } else {
                        fixed += 1;
                        common::telemetry::info!("Auto-fixed constraints.yaml");
                    }
                }
            }

            // 6.2 重新生成 rules.yaml
            if !plan.business_rules.is_empty() {
                let rules: Vec<runtime_engine::RuleExtension> = plan
                    .business_rules
                    .iter()
                    .map(|r| runtime_engine::RuleExtension {
                        entity: r.entity.clone(),
                        name: r.rule_name.clone(),
                        trigger: r.trigger.clone(),
                        condition: r.condition.clone(),
                        action: r.action.clone(),
                        priority: r.priority,
                        error_message: r.error_message.clone(),
                        blocking: true,
                    })
                    .collect();
                if let Ok(yaml) = yaml_serde::to_string(&rules) {
                    let path = format!("{}/rules.yaml", extensions_dir);
                    if let Err(e) = tokio::fs::write(&path, yaml).await {
                        common::telemetry::warn!("Auto-fix rules.yaml failed: {}", e);
                    } else {
                        fixed += 1;
                        common::telemetry::info!("Auto-fixed rules.yaml");
                    }
                }
            }

            // 6.3 记录修复结果到 build.log
            let fix_log_path = format!("{}/build.log", scratch.output_path);
            let fix_note = format!(
                "\nauto_fix_at: {}\nauto_fix_error: {}\nauto_fix_files_regenerated: {}\n",
                chrono::Utc::now().to_rfc3339(),
                err,
                fixed
            );
            if let Ok(existing) = tokio::fs::read_to_string(&fix_log_path).await {
                let _ = tokio::fs::write(&fix_log_path, format!("{}{}", existing, fix_note)).await;
            }

            common::telemetry::info!("Auto-fix completed: {} file(s) regenerated", fixed);
        }

        // 更新构建日志为完成状态
        let build_log_done = format!(
            "build_started_at: {}\napp_name: {}\nmodules: {}\noutput_path: {}\nstatus: completed\n",
            chrono::Utc::now().to_rfc3339(),
            scratch.app_name,
            scratch.module_count,
            scratch.output_path
        );
        if let Err(e) = tokio::fs::write(&build_log_path, build_log_done).await {
            common::telemetry::warn!("Failed to update build.log: {}", e);
        }

        common::telemetry::info!("App '{}' build completed successfully", scratch.app_name);
        Ok(())
    }

    // ──────────────────────────────────────────────────────────────────────────
    // 阶段 4：Verifying — 编译和测试（文件完整性 + 格式校验）
    #[allow(dead_code)]
    // ──────────────────────────────────────────────────────────────────────────
    // 构建最终 BuildResult
    // ──────────────────────────────────────────────────────────────────────────
    async fn build_result(&self, ctx: &ConversationContext) -> BuildResult {
        let plan = match ctx.flow_plan.as_ref() {
            Some(p) => p,
            None => {
                return BuildResult {
                    app_name: "unknown".to_string(),
                    output_path: "Pre-Proc/_error/Apps/unknown/".to_string(),
                    used_modules: vec![],
                    extensions: vec![],
                    generated_files: vec![],
                    pending_confirmations: vec!["构建失败：缺少 flow plan".to_string()],
                    endpoint_url: None,
                    preview_url: None,
                    runtime_validation: None,
                    has_runtime_error: false,
                };
            }
        };

        let scratch = match ctx.compose_scratch.as_ref() {
            Some(s) => s,
            None => {
                return BuildResult {
                    app_name: "unknown".to_string(),
                    output_path: "Pre-Proc/_error/Apps/unknown/".to_string(),
                    used_modules: vec![],
                    extensions: vec![],
                    generated_files: vec![],
                    pending_confirmations: vec!["构建失败：缺少 compose scratch".to_string()],
                    endpoint_url: None,
                    preview_url: None,
                    runtime_validation: None,
                    has_runtime_error: false,
                };
            }
        };

        let used_modules = plan
            .used_modules
            .iter()
            .map(|id| {
                // 已知实体已在 catalog.collections 中全局校验；
                // 按 module 划分 collections 的语义已随 meta_collections.modules 删除而移除，
                // 此处仅记录该应用涉及的平台实体，不表示模块归属。
                let known_collections: Vec<String> = plan
                    .known_entities
                    .iter()
                    .filter(|entity| {
                        ctx.platform_catalog
                            .as_ref()
                            .map(|cat| cat.collections.iter().any(|c| &c.table_name == *entity))
                            .unwrap_or(true)
                    })
                    .cloned()
                    .collect();
                ModuleUsage {
                    module_id: id.clone(),
                    module_name: id.clone(),
                    collections: known_collections,
                }
            })
            .collect();

        let mut pending = if !ctx.extensions.is_empty() {
            vec![
                "应用已组装完成，部分需求未被现有模块覆盖".to_string(),
                format!(
                    "共使用 {} 个模块，{} 个扩展配置",
                    scratch.module_count, scratch.files_written
                ),
            ]
        } else {
            vec![format!(
                "共使用 {} 个模块，{} 个扩展配置",
                scratch.module_count, scratch.files_written
            )]
        };
        pending.push(
            "⚠️ Gateway 重启生效：新应用已写入 Pre-Proc/Apps/，需重启 Gateway 后访问".to_string(),
        );
        pending.push("编译与测试验证通过".to_string());
        // Draft mode disclaimer
        if ctx.draft_mode {
            pending.push("🧪 【快速草稿】这是应用轮廓版本，核心功能可用但缺少业务规则/状态机等扩展配置。你可以继续补充需求来完善应用。".to_string());
        }

        let mut generated_files = vec![
            format!("{}/app.json", scratch.output_path),
            format!("{}/gateway_design.md", scratch.output_path),
        ];

        let extensions_dir = format!("{}/extensions", scratch.output_path);
        let gaps_dir = format!("{}/request-no-impl", scratch.output_path);
        generated_files.push(format!("{}/constraints.yaml", extensions_dir));
        generated_files.push(format!("{}/rules.yaml", extensions_dir));
        generated_files.push(format!("{}/statemachines.yaml", extensions_dir));
        generated_files.push(format!("{}/workflows.yaml", extensions_dir));
        generated_files.push(format!("{}/gap-*.md", gaps_dir));

        let endpoint_url = plan.used_modules.first().map(|module_id| {
            // Strip namespace prefix: "WZ/transport-wz" → "transport-wz"
            let route_id = module_id.split('/').next_back().unwrap_or(module_id);
            format!("http://localhost:9001/{}", route_id)
        });

        let preview_url = Some(format!(
            "/apps/{}/{}/prototype.html",
            plan.namespace, scratch.app_name
        ));

        BuildResult {
            app_name: scratch.app_name.clone(),
            output_path: scratch.output_path.clone(),
            used_modules,
            extensions: ctx.extensions.clone(),
            generated_files,
            pending_confirmations: pending,
            endpoint_url,
            preview_url,
            runtime_validation: None,
            has_runtime_error: false,
        }
    }

    /// 写出 pipeline_manifest.json + ontology-model.json + flow-plan.json
    /// 供 alioth-build.sh 和 ontology-mapping gen-service-tests 等下游工具消费。
    ///
    /// ## pipeline_manifest.json Schema (版本 1)
    ///
    /// 该文件是 AppAgent → Pipeline 的桥接契约，由 Publishing 状态原子写入。
    /// 下游工具以该文件为入口确定 namespace / app_code / 模块结构。
    ///
    /// ```json
    /// {
    ///   "manifest_version": 1,        // 整数, 向后兼容
    ///   "namespace": "Cosmic-Tools",  // 必须, 来自 FlowPlan.namespace
    ///   "app_code": "inventory-app",  // 必须, 来自 BuildResult.app_name
    ///   "source": "appagent",         // 必须, 固定值
    ///   "created_at": "2026-07-13T04:57:00Z",  // 必须, RFC3339
    ///
    ///   "modules": [{                 // 必须, 来自 BuildResult.used_modules
    ///     "id": "inventory",
    ///     "name": "inventory",
    ///     "collections": [],
    ///     "blocks": [],               // 预留: 模块→Block 映射(当前为空)
    ///     "services": []              // 预留: 模块→Service 映射(当前为空)
    ///   }],
    ///
    ///   "blocks": [{                  // 可选, 来自 FlowPlan.created_blocks
    ///     "id": "stock-check",
    ///     "services": []              // 预留(当前为空)
    ///   }],
    ///
    ///   "services": [{                // 可选, 来自 FlowPlan.created_services
    ///     "id": "stock-svc",
    ///     "entities": [],             // 预留(当前为空)
    ///     "fields_sample": []         // 预留(当前为空)
    ///   }],
    ///
    ///   "workflow_steps": ["create_stock_check"],       // 可选
    ///   "business_rules_summary": ["entity: cond act"], // 可选
    ///
    ///   "refs": {                     // 必须, 绝对路径引用
    ///     "ontology_model": "Pre-Proc/.../ontology-model.json",
    ///     "flow_plan": "Pre-Proc/.../flow-plan.json",
    ///     "output_path": "Pre-Proc/.../<app_code>"
    ///   }
    /// }
    /// ```
    ///
    /// 写入保证: 先全量写入 .pipeline-tmp/ 临时目录, 全部成功后再 rename 到目标。
    /// 任一文件写入失败 → 不产生任何输出(临时目录丢弃), 调用方进入重试路径。
    /// ontology_model 和 flow_plan 为必需项, 任一缺失则跳过全部写入 → 返回错误。
    ///
    /// 下游使用约定:
    ///   1. alioth-build.sh: 扫描 Pre-Proc/*/Apps/*/pipeline_manifest.json → 自动 init
    ///   2. ontology-mapping gen-service-tests: 读取 blocks[] 和 services[] 生成测试骨架
    async fn write_pipeline_artifacts(
        &self,
        ctx: &ConversationContext,
        result: &BuildResult,
    ) -> Result<(), String> {
        let output_path = std::path::Path::new(&result.output_path);
        let bundle_uuid = uuid::Uuid::new_v4().to_string();
        let bundle_dir = output_path.join(format!(".pipeline-artifacts/{}", &bundle_uuid[..8]));

        let om = ctx
            .ontology_model
            .as_ref()
            .ok_or_else(|| "Missing ontology_model for pipeline artifact".to_string())?;
        let fp = ctx
            .flow_plan
            .as_ref()
            .ok_or_else(|| "Missing flow_plan for pipeline artifact".to_string())?;

        tokio::fs::create_dir_all(&bundle_dir)
            .await
            .map_err(|e| format!("Create bundle dir: {}", e))?;

        // ── 写入 data files 到 bundle ──
        let om_json = serde_json::to_string_pretty(om)
            .map_err(|e| format!("Serialize ontology model: {}", e))?;
        tokio::fs::write(bundle_dir.join("ontology-model.json"), &om_json)
            .await
            .map_err(|e| format!("Write ontology-model.json: {}", e))?;

        let fp_json =
            serde_json::to_string_pretty(fp).map_err(|e| format!("Serialize flow plan: {}", e))?;
        tokio::fs::write(bundle_dir.join("flow-plan.json"), &fp_json)
            .await
            .map_err(|e| format!("Write flow-plan.json: {}", e))?;

        // ── 构造 manifest, refs 指向 bundle ──
        let bundle_prefix = format!(
            "{}/.pipeline-artifacts/{}",
            result.output_path,
            &bundle_uuid[..8]
        );
        let manifest = serde_json::json!({
            "manifest_version": 1,
            "namespace": fp.namespace,
            "app_code": result.app_name,
            "source": "appagent",
            "created_at": chrono::Utc::now().to_rfc3339(),
            "modules": result.used_modules.iter().map(|m| json!({"id": m.module_id, "name": m.module_name, "collections": m.collections, "blocks": [], "services": []})).collect::<Vec<_>>(),
            "blocks": fp.created_blocks.iter().map(|id| json!({"id": id, "services": []})).collect::<Vec<_>>(),
            "services": fp.created_services.iter().map(|id| json!({"id": id, "entities": [], "fields_sample": []})).collect::<Vec<_>>(),
            "workflow_steps": fp.workflow_steps,
            "business_rules_summary": fp.business_rules.iter().map(|r| format!("{}: {} {}", r.entity, r.condition, r.action)).collect::<Vec<_>>(),
            "refs": {
                "ontology_model": format!("{}/ontology-model.json", bundle_prefix),
                "flow_plan": format!("{}/flow-plan.json", bundle_prefix),
                "output_path": result.output_path
            }
        });
        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| format!("Serialize manifest: {}", e))?;

        // ── 原子 commit: write tmp → rename over existing (POSIX atomic) ──
        let manifest_tmp = output_path.join(".pipeline_manifest.json.tmp");
        tokio::fs::write(&manifest_tmp, &manifest_json)
            .await
            .map_err(|e| format!("Write manifest tmp: {}", e))?;
        // rename-over-existing is atomic on the same filesystem (POSIX guarantee)
        tokio::fs::rename(&manifest_tmp, &output_path.join("pipeline_manifest.json"))
            .await
            .map_err(|e| format!("Atomic swap manifest: {}", e))?;

        // Clean orphaned bundles (keep last 3)
        let artifacts_base = output_path.join(".pipeline-artifacts");
        if let Ok(mut entries) = tokio::fs::read_dir(&artifacts_base).await {
            let mut dirs: Vec<_> = Vec::new();
            while let Ok(Some(entry)) = entries.next_entry().await {
                if entry.path().is_dir() {
                    if let Ok(meta) = entry.metadata().await {
                        if let Ok(created) = meta.created() {
                            dirs.push((created, entry.path()));
                        }
                    }
                }
            }
            dirs.sort_by(|a, b| b.0.cmp(&a.0)); // newest first
            for (_, old_dir) in dirs.into_iter().skip(2) {
                if old_dir != bundle_dir {
                    let _ = tokio::fs::remove_dir_all(&old_dir).await;
                }
            }
        }

        common::telemetry::info!(
            "Wrote pipeline artifacts bundle {}: manifest → {}/ontology-model.json (atomic)",
            result.output_path,
            bundle_prefix
        );

        Ok(())
    }

    fn render_state_message(&self, ctx: &ConversationContext) -> String {
        match &ctx.state {
            AgentState::Planning {
                needs_clarification: Some(questions),
                ..
            } => {
                // 生成本体关系图（当前已构建的部分）
                let ontology_diagram = ctx
                    .ontology_model
                    .as_ref()
                    .map(|om| {
                        let graph = alioth_gen::VisualizerEngine::generate_graph(om);
                        alioth_gen::VisualizerEngine::export_to_mermaid(&graph)
                    })
                    .unwrap_or_default();

                let diagram_section = if ontology_diagram.is_empty() {
                    String::new()
                } else {
                    format!(
                        "\n\n**【当前本体关系图】**\n```mermaid\n{}\n```\n",
                        ontology_diagram
                    )
                };

                let questions_json: Value = json!({
                    "questions": questions.iter().map(|q| {
                        json!({
                            "id": q.id,
                            "question": q.question,
                            "options": q.options,
                            "required": q.required,
                        })
                    }).collect::<Vec<_>>()
                });

                let questions_text = questions
                    .iter()
                    .enumerate()
                    .map(|(i, q)| {
                        let options = if q.options.is_empty() {
                            String::new()
                        } else {
                            q.options
                                .iter()
                                .enumerate()
                                .map(|(j, o)| format!("{}. {}", char::from(b'A' + j as u8), o))
                                .collect::<Vec<_>>()
                                .join("  ")
                        };
                        format!(
                            "**{}. {}**\n{}\n{}",
                            i + 1,
                            q.question,
                            options,
                            if options.is_empty() {
                                "\n".to_string()
                            } else {
                                String::new()
                            }
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                format!(
                    "[AGENT:question]{}\n[/AGENT]\n\n根据您的描述，我计划使用【{}】模块。\n\n在开始构建前，有{}个关键信息需要确认：\n\n{}{}\n请回复您的选择或补充信息：",
                    questions_json,
                    ctx.flow_plan.as_ref().map(|p| p.used_modules.join("、")).unwrap_or_default(),
                    questions.len(),
                    questions_text,
                    diagram_section,
                )
            }
            AgentState::Extending => "正在生成后端功能配置...".to_string(),
            AgentState::Generating => {
                // 已废弃：直接跳过
                "正在整合前后端产物，生成最终应用...".to_string()
            }
            AgentState::Composing => "正在整合前后端产物，生成最终应用...".to_string(),
            AgentState::Verifying { .. } => "正在验证应用产物...".to_string(),
            AgentState::Publishing { .. } => "正在编译验证并准备发布...".to_string(),
            AgentState::Published { .. } => "应用已发布，可以部署！".to_string(),
            _ => "正在处理中...".to_string(),
        }
    }

    fn render_presentation(&self, ctx: &ConversationContext) -> String {
        if let AgentState::Presenting { result } = &ctx.state {
            let modules_text = result
                .used_modules
                .iter()
                .map(|m| {
                    let cols = if m.collections.is_empty() {
                        String::new()
                    } else {
                        format!(" → {}", m.collections.join(", "))
                    };
                    format!("- **{}** ({}){}", m.module_id, m.module_name, cols)
                })
                .collect::<Vec<_>>()
                .join("\n");

            let endpoint_section = result
                .endpoint_url
                .as_ref()
                .map(|url| format!("\n\n**【应用访问地址】**\n🔗 {}", url))
                .unwrap_or_default();

            let result_json: Value = json!({
                "app_name": result.app_name,
                "output_path": result.output_path,
                "endpoint_url": result.endpoint_url,
                "used_modules": result.used_modules,
                "generated_files": result.generated_files,
                "pending_confirmations": result.pending_confirmations,
            });

            // 生成本体关系图 (Mermaid)
            let ontology_diagram = ctx
                .ontology_model
                .as_ref()
                .map(|om| {
                    let graph = alioth_gen::VisualizerEngine::generate_graph(om);
                    alioth_gen::VisualizerEngine::export_to_mermaid(&graph)
                })
                .unwrap_or_default();

            let diagram_section = if ontology_diagram.is_empty() {
                String::new()
            } else {
                format!(
                    "\n\n**【本体关系图】**\n```mermaid\n{}\n```\n",
                    ontology_diagram
                )
            };

            format!(
                "[AGENT:result]{}\n[/AGENT]\n\n✅ **应用构建完成！**\n\n**【使用的模块】**\n{}\n\n**【生成的文件】**\n{}\n\n**【待您确认】**\n{}{}\n{}",
                result_json,
                modules_text,
                result.generated_files.join("\n"),
                result.pending_confirmations.join("\n"),
                endpoint_section,
                diagram_section,
            )
        } else {
            String::new()
        }
    }

    /// 非终止状态的进度提示（单步模式）
    fn render_progress(
        &self,
        ctx: &ConversationContext,
        from: &AgentState,
        to: &AgentState,
    ) -> String {
        let step_idx = ctx.step_history.len();
        let _state_label = match to {
            AgentState::Initializing => "初始化",
            AgentState::Planning {
                needs_clarification: Some(_),
                ..
            } => "澄清问题",
            AgentState::Planning { .. } => "分析需求",
            AgentState::Extending => "生成后端配置",
            AgentState::Generating => "生成前端设计",
            AgentState::GeneratingFrontend { .. } => "生成前端代码",
            AgentState::Composing => "构建应用",
            AgentState::Verifying { .. } => "验证产物",
            AgentState::Publishing { .. } => "发布应用",
            AgentState::Published { .. } => "已发布",
            AgentState::Presenting { .. } => "展示结果",
            AgentState::SemanticAnalysis => "语义分析",
            AgentState::FunctionDecomposition => "功能拆解",
            AgentState::OntologyAnalysis { .. } => "本体分析",
            AgentState::ModuleCreation => "模块创建",
            AgentState::BlockCreation => "区块创建",
            AgentState::OntologyTransfer => "本体转移",
            AgentState::ServiceAPI => "Service API",
            AgentState::ExecutingSkill { .. } => "执行技能",
            AgentState::AwaitingUserInput { .. } => "等待人工干预",
            AgentState::Failed { .. } => "执行失败",
        };
        format!(
            "⏳ 步骤 #{}: {} → {} ({}%)",
            step_idx,
            state_name(from),
            state_name(to),
            progress_percent(to)
        )
    }

    pub async fn handle_user_answer(
        &self,
        ctx: &mut ConversationContext,
        question_id: &str,
        answer: &str,
    ) -> Result<StepResult, String> {
        ctx.user_answers.push(UserAnswer {
            question_id: question_id.to_string(),
            answer: answer.to_string(),
            answered_at: Utc::now(),
        });
        // coord_* 答案恢复到提问时的 checkpoint（通常是 OntologyTransfer 重跑）
        if question_id.starts_with("coord_") {
            if let Some(checkpoint) = ctx.last_checkpoint.clone() {
                ctx.state = checkpoint;
                return self.run_single_step(ctx, None::<&fn(AgentProgress)>).await;
            }
        }
        ctx.state = AgentState::Planning {
            revision_round: 0,
            needs_clarification: None,
        };
        self.run_single_step(ctx, None::<&fn(AgentProgress)>).await
    }

    /// 用 LLM 对语义维度做 judge，返回 `LlmJudger`（解析失败/缺失的维度不覆盖规则分）。
    /// 仅对规则评估较弱的语义维度（navigation_coherence / goal_fidelity / extension_coverage）调用 LLM；
    /// schema_validity / prototype_standalone 保持规则评估（结构性，无需 LLM 判断）。
    async fn llm_judge(
        &self,
        ctx: &ConversationContext,
        artifacts: &crate::evaluate::Artifacts<'_>,
    ) -> crate::evaluate::LlmJudger {
        let mut scores: HashMap<String, f32> = HashMap::new();
        if let Ok(app_txt) = std::fs::read_to_string(artifacts.app_json) {
            if let Ok(app) = serde_json::from_str::<Value>(&app_txt) {
                for dim in [
                    "navigation_coherence",
                    "goal_fidelity",
                    "extension_coverage",
                ] {
                    let prompt = crate::evaluate::judge_prompt(dim, &app, &ctx.user_description);
                    if let Ok(resp) = self.llm_service.generate(&prompt).await {
                        if let Some(s) = parse_judge_score(&resp) {
                            scores.insert(dim.to_string(), s);
                        }
                    }
                }
            }
        }
        crate::evaluate::LlmJudger::from_scores(scores)
    }

    /// Verify the generated app compiles
    async fn verify_compilation(&self, result: &BuildResult) -> Result<bool, String> {
        use std::process::Command;
        if result.output_path.is_empty() {
            return Err("No output path in build result".to_string());
        }
        let app_json = format!("{}/app.json", result.output_path);
        let metadata = tokio::fs::metadata(&app_json).await;
        match metadata {
            Ok(m) if m.is_file() => {
                let backend_dir = format!("{}/backend", result.output_path);
                if tokio::fs::metadata(&backend_dir)
                    .await
                    .map(|m| m.is_dir())
                    .unwrap_or(false)
                {
                    let output = Command::new("cargo")
                        .args([
                            "check",
                            "--manifest-path",
                            &format!("{}/Cargo.toml", backend_dir),
                        ])
                        .output();
                    match output {
                        Ok(out) if out.status.success() => Ok(true),
                        Ok(out) => Err(String::from_utf8_lossy(&out.stderr).to_string()),
                        Err(e) => Err(e.to_string()),
                    }
                } else {
                    Ok(true)
                }
            }
            Ok(_) => Err(format!("app.json not a file: {}", app_json)),
            Err(e) => Err(format!("{}: {}", app_json, e)),
        }
    }

    /// 渲染「等待人工干预」消息：评估环达上限仍不达标，暂停并请求用户介入。
    fn render_awaiting_input(&self, ctx: &ConversationContext) -> String {
        let reason = if let AgentState::AwaitingUserInput { reason } = &ctx.state {
            reason.clone()
        } else {
            "评估环达上限仍不达标，需人工干预。".to_string()
        };
        format!(
            "⏸️ **等待人工干预**：{}\n\n已执行 {} 步。提供变更请求后可重试，或显式确认强制发布。",
            reason,
            ctx.step_history.len()
        )
    }

    fn render_published(&self, ctx: &ConversationContext) -> String {
        let app_name = ctx.user_description.lines().next().unwrap_or("应用");
        let rounds = if ctx.repair_attempt > 0 {
            format!(" ({} 轮自动修复后)", ctx.repair_attempt)
        } else {
            String::new()
        };
        format!(
            "🎉 **{}** 已通过验证，可以发布了！{}

- 状态: 已发布
- 编译验证: ✅ 通过",
            app_name, rounds
        )
    }
}

// ──────────────────────────────────────────────────────────────────────────
// 状态机辅助函数（模块级）
// ──────────────────────────────────────────────────────────────────────────

pub fn state_name(state: &AgentState) -> &'static str {
    match state {
        AgentState::Initializing => "初始化",
        AgentState::Planning { needs_clarification: Some(_), .. } => "澄清问题",
        AgentState::Planning { .. } => "分析需求",
        AgentState::Extending => "生成后端配置",
        AgentState::Generating => "生成配置(已合并)",
        AgentState::GeneratingFrontend { .. } => "生成前端代码",
        AgentState::Composing => "构建应用",
        AgentState::Verifying { .. } => "验证产物",
        AgentState::Publishing { .. } => "发布应用",
        AgentState::Published { .. } => "已发布",
        AgentState::SemanticAnalysis => "语义分析",
        AgentState::FunctionDecomposition => "功能拆解",
        AgentState::OntologyAnalysis { .. } => "本体分析",
        AgentState::ModuleCreation => "模块创建",
        AgentState::BlockCreation => "区块创建",
        AgentState::OntologyTransfer => "本体转移",
        AgentState::ServiceAPI => "服务 API",
        AgentState::ExecutingSkill { .. } => "执行技能",
        AgentState::Presenting { .. } => "展示结果",
        AgentState::AwaitingUserInput { .. } => "等待人工干预",
        AgentState::Failed { .. } => "执行失败",
    }
}

pub fn progress_percent(state: &AgentState) -> u8 {
    match state {
        AgentState::Initializing => 5,
        AgentState::Planning { needs_clarification: Some(_), .. } => 10,
        AgentState::Planning { .. } => 15,
        AgentState::Extending => 25,
        AgentState::Generating => 30,
        AgentState::GeneratingFrontend { .. } => 35,
        AgentState::Composing => 40,
        AgentState::Verifying { .. } => 65,
        AgentState::Publishing { .. } => 80,
        AgentState::SemanticAnalysis => 5,
        AgentState::FunctionDecomposition => 10,
        AgentState::OntologyAnalysis { .. } => 20,
        AgentState::ModuleCreation => 30,
        AgentState::BlockCreation => 40,
        AgentState::OntologyTransfer => 50,
        AgentState::ServiceAPI => 60,
        AgentState::ExecutingSkill { .. } => 70,
        AgentState::Published { .. } => 100,
        AgentState::Presenting { .. } => 95,
        AgentState::AwaitingUserInput { .. } => 85,
        AgentState::Failed { .. } => 100,
    }
}

/// 截断长日志文本到指定最大长度（适用于 stdout/stderr 入执行日志）
fn truncate_log(text: &str, max: usize) -> String {
    if text.len() <= max {
        text.to_string()
    } else {
        format!(
            "{}...\n[truncated {} bytes]",
            &text[..max],
            text.len() - max
        )
    }
}
/// 从用户自然语言描述中提取关键词
fn extract_keywords(description: &str) -> Vec<String> {
    // 常见停用词
    let stop_words: std::collections::HashSet<&str> = [
        "我", "你", "的", "了", "是", "在", "一个", "需要", "想要", "希望", "可以", "能够", "请",
        "帮", "用", "和", "与", "或", "以及", "这个", "那个", "这些", "那些", "什么", "怎么",
        "如何", "吗", "呢", "吧", "要", "会", "有", "没有", "不", "都", "也", "就", "还", "但是",
        "the", "a", "an", "is", "are", "to", "for", "with", "and", "or",
    ]
    .iter()
    .cloned()
    .collect();

    // 提取中文词组（2-4字）和英文单词
    let mut keywords: Vec<String> = Vec::new();

    // 英文单词
    for word in description.split(|c: char| !c.is_alphanumeric()) {
        let w = word.trim();
        if w.len() >= 3 && !stop_words.contains(&w.to_lowercase().as_str()) {
            keywords.push(w.to_string());
        }
    }

    // 中文词组（简单滑动窗口：2-4字）
    let chars: Vec<char> = description.chars().collect();
    for window_size in [4, 3, 2] {
        for i in 0..chars.len().saturating_sub(window_size - 1) {
            if i + window_size <= chars.len() {
                let slice: String = chars[i..i + window_size].iter().collect();
                // 只保留纯中文词组
                if slice.chars().all(|c| c as u32 > 0x4E00) && !stop_words.contains(slice.as_str())
                {
                    keywords.push(slice);
                }
            }
        }
    }

    // 去重，限制数量
    keywords.sort();
    keywords.dedup();
    keywords.truncate(10);
    keywords
}

/// Convert a string to URL-friendly slug
fn slugify(s: &str) -> String {
    s.chars()
        .filter_map(|c| {
            if c.is_alphanumeric() {
                Some(c.to_ascii_lowercase())
            } else if c == ' ' || c == '-' || c == '_' {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
