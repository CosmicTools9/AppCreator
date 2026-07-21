//! LLM Request Harness — 结构化调用封装
//!
//! ## 核心能力
//! 1. **Retry with exponential backoff** — 网络抖动/限流自动重试
//! 2. **Reasoning effort 按任务类型动态调节** — Planning 用 high，修复用 low
//! 3. **Temperature 按阶段分档** — 创造性任务高 temp，修复任务低 temp
//! 4. **Structured JSON 输出验证** — 提前发现 LLM 输出结构缺陷
//!
//! ## 用法
//! ```ignore
//! let harness = LlmHarness::new(llm_service);
//! let result = harness
//!     .with_task(TaskType::OntologyPlanning)
//!     .with_system(system_prompt)
//!     .call(user_prompt)
//!     .await?;
//! ```

use crate::auto_router::{ModelTier, RoutePlan};
use crate::llm::{GenerationOverrides, LlmService};
use crate::state::AgentToolCall;
use std::time::Duration;

/// 工具定义(描述 LLM 可调用的 tool_call)
///
/// 注入到 system prompt 中,让 LLM 在 JSON 输出中包含 `tool_calls` 数组。
/// AppAgent 采用**应用层 tool_call**(不依赖 LLM API 原生 function calling),
/// 避免修改 LlmService trait 与底层 HTTP API 兼容性。
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    /// 工具名(对应 AgentToolCall 变体的 snake_case 名)
    pub name: &'static str,
    /// 工具描述
    pub description: &'static str,
    /// 参数 JSON Schema(简化为字符串描述)
    pub parameters: &'static str,
}

/// 标准 tool 定义(激活 gateway_design + yaml_operations 死接口)
pub fn standard_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "write_gateway_design",
            description:
                "写入 gateway_design.md 前端设计方案(markdown 格式,遵循 GATEWAY_DESIGN 规约)",
            parameters: r#"{"type":"object","properties":{"content":{"type":"string","description":"markdown 内容"}},"required":["content"]}"#,
        },
        ToolDefinition {
            name: "write_extension_yaml",
            description: "覆盖写入 extension YAML 文件(完整内容替换)",
            parameters: r#"{"type":"object","properties":{"file":{"type":"string","enum":["constraints.yaml","rules.yaml","statemachines.yaml","workflows.yaml"]},"content":{"type":"string"}},"required":["file","content"]}"#,
        },
        ToolDefinition {
            name: "patch_extension_yaml",
            description: "结构化 Patch extension YAML 文件(路径表达式 + 新值)",
            parameters: r#"{"type":"object","properties":{"file":{"type":"string"},"patches":{"type":"array","items":{"type":"object","properties":{"path":{"type":"string"},"value":{}}}}},"required":["file","patches"]}"#,
        },
    ]
}

/// 按任务类型预设的参数配置
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaskType {
    /// 本体规划 — 需要深度推理来理解业务语义
    OntologyPlanning,
    /// 语义修复 — 轻量级修正 LLM 输出错误
    SemanticRepair,
    /// 代码生成 — 生成 Rust/TypeScript 代码
    CodeGeneration,
    /// 结构化输出修正 — 将带格式文本转为纯 JSON
    FormatCorrection,
}

impl TaskType {
    /// 适合该任务的 reasoning_effort（DeepSeek v4 专用）
    ///
    /// 参考 oh-my-pi 的 Effort 5 级枚举：
    /// minimal, low, medium, high, xhigh
    pub fn reasoning_effort(&self) -> &'static str {
        match self {
            Self::OntologyPlanning => "high",
            Self::SemanticRepair => "low",
            Self::CodeGeneration => "medium",
            Self::FormatCorrection => "minimal",
        }
    }

    /// 适合该任务的 temperature
    pub fn temperature(&self) -> f64 {
        match self {
            Self::OntologyPlanning => 0.3,  // 低温度，保持一致性
            Self::SemanticRepair => 0.1,    // 极低温度，确定性修正
            Self::CodeGeneration => 0.2,    // 低温度，代码一致性
            Self::FormatCorrection => 0.05, // 极低温度，纯格式转换
        }
    }

    /// 适合该任务的 max_tokens
    pub fn max_tokens(&self) -> u64 {
        match self {
            Self::OntologyPlanning => 16384, // 本体模型可能很大
            Self::SemanticRepair => 4096,    // 小范围修正
            Self::CodeGeneration => 32768,   // 代码生成可能很长
            Self::FormatCorrection => 8192,  // 格式转换
        }
    }

    /// 最大重试次数
    pub fn max_retries(&self) -> u32 {
        match self {
            Self::OntologyPlanning => 3,
            Self::SemanticRepair => 2,
            Self::CodeGeneration => 3,
            Self::FormatCorrection => 2,
        }
    }

    /// 是否开启 reasoning（对某些任务可能不是所有模型都支持）
    pub fn reasoning_enabled(&self) -> bool {
        matches!(self, Self::OntologyPlanning)
    }
}

/// LLM Harness 结构化调用结果
#[derive(Debug)]
pub struct HarnessResult {
    /// LLM 原始输出文本
    pub raw_text: String,
    /// 实际使用的次数（含重试）
    pub attempts: u32,
    /// 每次实际延迟（毫秒）
    pub latencies_ms: Vec<u64>,
    /// 从 LLM JSON 输出中解析的 tool_calls(应用层 tool_call)
    pub tool_calls: Vec<AgentToolCall>,
}

/// LLM Harness 错误
#[derive(Debug)]
pub enum HarnessError {
    AllRetriesFailed { attempts: u32, last_error: String },
    Config(String),
}

impl HarnessError {
    pub fn attempts(&self) -> u32 {
        match self {
            Self::AllRetriesFailed { attempts, .. } => *attempts,
            Self::Config(_) => 0,
        }
    }
}

impl std::fmt::Display for HarnessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AllRetriesFailed {
                attempts,
                last_error,
            } => {
                write!(
                    f,
                    "All retries failed after {} attempts: {}",
                    attempts, last_error
                )
            }
            Self::Config(msg) => write!(f, "Config error: {}", msg),
        }
    }
}

impl std::error::Error for HarnessError {}

/// 操作上下文（传递给重试逻辑，用于决定是否值得重试）
#[derive(Debug, Clone)]
pub struct OperationContext {
    /// 已重试的次数
    pub attempt: u32,
    /// 是否检测到非空输出但格式有误（值得重试 vs 不值得）
    pub has_partial_output: bool,
    /// 总耗时（毫秒）
    pub elapsed_ms: u64,
}

/// 结构化 LLM 调用封装
pub struct LlmHarness<'a> {
    llm: &'a dyn LlmService,
    task_type: TaskType,
    system_prompt: Option<String>,
    /// 可选的路线规划覆盖（用于模型/推理深度升级）
    route_plan: Option<RoutePlan>,
    /// 工具定义(注入到 system prompt,让 LLM 输出 tool_calls)
    tools: Vec<ToolDefinition>,
}

impl<'a> LlmHarness<'a> {
    pub fn new(llm: &'a dyn LlmService) -> Self {
        Self {
            llm,
            task_type: TaskType::OntologyPlanning,
            system_prompt: None,
            route_plan: None,
            tools: Vec::new(),
        }
    }

    /// 设置任务类型（自动决定重试策略、temperature、reasoning_effort）
    pub fn with_task(mut self, task: TaskType) -> Self {
        self.task_type = task;
        self
    }

    /// 设置路线规划覆盖（用于自修复时升级模型/推理深度）
    pub fn with_route_plan(mut self, plan: RoutePlan) -> Self {
        self.route_plan = Some(plan);
        self
    }

    /// 设置 system prompt
    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system_prompt = Some(system.into());
        self
    }

    /// 注册工具定义(激活应用层 tool_call)
    ///
    /// 工具定义会以 JSON Schema 形式追加到 system prompt,
    /// LLM 在 JSON 输出中包含 `tool_calls` 数组,
    /// `call_with_retry` 自动解析并填入 `HarnessResult.tool_calls`。
    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = tools;
        self
    }

    /// 构建含 tool 定义的 system prompt(若注册了 tools)
    fn build_system_prompt(&self) -> Option<String> {
        let base = self.system_prompt.as_deref()?;
        if self.tools.is_empty() {
            return Some(base.to_string());
        }
        let tools_json: Vec<serde_json::Value> = self
            .tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "parameters": serde_json::from_str::<serde_json::Value>(t.parameters)
                        .unwrap_or(serde_json::json!({})),
                })
            })
            .collect();
        Some(format!(
            "{base}\n\n【可用工具(tool_calls)】\n\
             在 JSON 输出的 `tool_calls` 数组中,可包含以下工具调用:\n{}\n\
             格式: {{\"name\": \"<tool_name>\", \"arguments\": {{...}}}}\n\
             示例: {{\"ontology\": {{...}}, \"tool_calls\": [{{\"name\": \"write_gateway_design\", \"arguments\": {{\"content\": \"# 设计方案\\n...\"}}}}]}}",
            serde_json::to_string_pretty(&tools_json).unwrap_or_default()
        ))
    }

    /// 执行一次性 LLM 调用（不重试），用于不需要 retry 的场景
    pub async fn call_once(&self, prompt: &str) -> Result<String, String> {
        match &self.system_prompt {
            Some(sys) => self.llm.generate_with_system(sys, prompt).await,
            None => self.llm.generate(prompt).await,
        }
        .map_err(|e| format!("LLM call failed: {}", e))
    }

    /// 执行带重试的 LLM 调用
    ///
    /// 自动应用：
    /// - Exponential backoff（初始 1s，最大 15s）
    /// - 按 TaskType 分档的 temperature/effort/tokens
    /// - 按 TaskType 分档的最大重试次数
    pub async fn call_with_retry(&self, prompt: &str) -> Result<HarnessResult, HarnessError> {
        let max_retries = self.task_type.max_retries();
        let mut last_error = String::new();
        let mut attempts = 0u32;
        let mut latencies = Vec::new();

        // Use route_plan override if present, otherwise fall back to task_type defaults
        let (model_tier, reasoning_effort) = match &self.route_plan {
            Some(plan) => {
                let re = plan.reasoning_effort.as_api_value().unwrap_or("high");
                let mt = match plan.model {
                    ModelTier::Pro => Some(ModelTier::Pro),
                    ModelTier::Flash => match re {
                        "high" | "xhigh" => Some(ModelTier::Pro),
                        _ => Some(ModelTier::Flash),
                    },
                };
                (mt, re)
            }
            None => {
                let re = self.task_type.reasoning_effort();
                let mt = match re {
                    "high" | "xhigh" => Some(ModelTier::Pro),
                    _ => Some(ModelTier::Flash),
                };
                (mt, re)
            }
        };
        let response_format = if matches!(self.task_type, TaskType::OntologyPlanning) {
            Some("json_object".to_string())
        } else {
            None
        };
        let overrides = GenerationOverrides {
            temperature: Some(self.task_type.temperature()),
            max_tokens: Some(self.task_type.max_tokens()),
            reasoning_effort: Some(reasoning_effort),
            model_tier,
            response_format,
        };

        for attempt in 0..=max_retries {
            let start = std::time::Instant::now();
            attempts += 1;

            // 若注册了 tools,用扩展后的 system prompt
            let effective_system = self.build_system_prompt();
            let result = match &effective_system {
                Some(sys) => {
                    self.llm
                        .generate_with_params(sys, prompt, overrides.clone())
                        .await
                }
                None => self.llm.generate(prompt).await,
            };

            let elapsed = start.elapsed();
            latencies.push(elapsed.as_millis() as u64);

            match result {
                Ok(text) => {
                    // 空输出校验
                    if text.trim().is_empty() {
                        last_error = format!("Empty response (attempt {})", attempt);
                        if attempt < max_retries {
                            backoff_sleep(attempt).await;
                        }
                        continue;
                    }
                    // 解析 tool_calls(应用层 tool_call)
                    let tool_calls = parse_tool_calls_from_json(&text);
                    return Ok(HarnessResult {
                        raw_text: text,
                        attempts,
                        latencies_ms: latencies,
                        tool_calls,
                    });
                }
                Err(e) => {
                    last_error = e.to_string();
                    if attempt < max_retries {
                        common::telemetry::warn!(
                            "LLM call failed (attempt {}/{}): {}. Retrying...",
                            attempt + 1,
                            max_retries + 1,
                            last_error
                        );
                        backoff_sleep(attempt).await;
                    }
                }
            }
        }

        Err(HarnessError::AllRetriesFailed {
            attempts,
            last_error,
        })
    }
}

/// Exponential backoff: 1s, 2s, 4s, 8s, capped at 15s
async fn backoff_sleep(attempt: u32) {
    let secs = 1u64 << attempt.min(4); // 1, 2, 4, 8, 15
    let secs = secs.min(15);
    tokio::time::sleep(Duration::from_secs(secs)).await;
}

// ─── 应用层 tool_call 解析 ─────────────────────────────────────────────────

/// 从 LLM JSON 输出中解析 tool_calls 数组。
///
/// LLM 在 OntologyOutput JSON 中可包含 `tool_calls` 字段:
/// ```json
/// {
///   "ontology": {...},
///   "tool_calls": [
///     {"name": "write_gateway_design", "arguments": {"content": "..."}},
///     {"name": "patch_extension_yaml", "arguments": {"file": "constraints.yaml", "patches": [...]}}
///   ]
/// }
/// ```
///
/// 解析失败时返回空 Vec(非致命,tool_calls 是可选的)。
pub fn parse_tool_calls_from_json(raw_text: &str) -> Vec<AgentToolCall> {
    let stripped = strip_code_fence(raw_text);
    let value: serde_json::Value = match serde_json::from_str(&stripped) {
        Ok(v) => v,
        Err(_) => return Vec::new(), // 非 JSON 或解析失败,返回空
    };
    let tool_calls_arr = match value.get("tool_calls").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Vec::new(), // 无 tool_calls 字段
    };
    tool_calls_arr
        .iter()
        .filter_map(|tc| serde_json::from_value::<AgentToolCall>(tc.clone()).ok())
        .collect()
}

// ─── 结构化输出校验工具 ─────────────────────────────────────────────────

/// 校验 LLM 输出是否为有效 JSON，并可选择性地验证包含必填 key
pub fn validate_json_output(
    text: &str,
    required_keys: &[&str],
) -> Result<serde_json::Value, String> {
    let stripped = strip_code_fence(text);
    let value: serde_json::Value =
        serde_json::from_str(&stripped).map_err(|e| format!("Invalid JSON: {}", e))?;

    if let Some(obj) = value.as_object() {
        for key in required_keys {
            if !obj.contains_key(*key) {
                return Err(format!("Missing required key: {}", key));
            }
        }
    }

    Ok(value)
}

/// 剥离 markdown code fence（```json ... ```）
fn strip_code_fence(text: &str) -> String {
    let trimmed = text.trim();
    if let Some(rest) = trimmed.strip_prefix("```") {
        // 跳过第一行（```json 或 ```）
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_type_config() {
        assert_eq!(TaskType::OntologyPlanning.reasoning_effort(), "high");
        assert_eq!(TaskType::SemanticRepair.temperature(), 0.1);
        assert_eq!(TaskType::CodeGeneration.max_tokens(), 32768);
        assert_eq!(TaskType::FormatCorrection.max_retries(), 2);
        assert!(TaskType::OntologyPlanning.reasoning_enabled());
        assert!(!TaskType::FormatCorrection.reasoning_enabled());
    }

    #[test]
    fn test_strip_code_fence() {
        let input = "```json\n{\"key\": \"value\"}\n```";
        assert_eq!(strip_code_fence(input), "{\"key\": \"value\"}");
    }

    #[test]
    fn test_strip_code_fence_no_fence() {
        let input = "{\"key\": \"value\"}";
        assert_eq!(strip_code_fence(input), "{\"key\": \"value\"}");
    }

    #[test]
    fn test_validate_json_output_ok() {
        let input = "```json\n{\"ontology\": {}, \"used_modules\": []}\n```";
        let result = validate_json_output(input, &["ontology", "used_modules"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_json_output_missing_key() {
        let input = "{\"ontology\": {}}";
        let result = validate_json_output(input, &["used_modules"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("used_modules"));
    }

    #[test]
    fn test_validate_json_output_invalid() {
        let input = "not json at all";
        let result = validate_json_output(input, &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid JSON"));
    }

    // ─── tool_call 解析测试 ──────────────────────────────────────────────

    #[test]
    fn test_parse_tool_calls_write_gateway_design() {
        let raw = r##"{"ontology": {}, "tool_calls": [
            {"name": "write_gateway_design", "arguments": {"content": "# 设计方案\n## 页面列表"}}
        ]}"##;
        let calls = parse_tool_calls_from_json(raw);
        assert_eq!(calls.len(), 1);
        match &calls[0] {
            AgentToolCall::WriteGatewayDesign { content } => {
                assert!(content.contains("# 设计方案"));
            }
            other => panic!("expected WriteGatewayDesign, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_tool_calls_patch_extension_yaml() {
        let raw = r#"{"ontology": {}, "tool_calls": [
            {"name": "patch_extension_yaml", "arguments": {
                "file": "constraints.yaml",
                "patches": [{"path": "constraints[0].expression", "value": "x > 0"}]
            }}
        ]}"#;
        let calls = parse_tool_calls_from_json(raw);
        assert_eq!(calls.len(), 1);
        match &calls[0] {
            AgentToolCall::PatchExtensionYaml { file, patches } => {
                assert_eq!(file, "constraints.yaml");
                assert_eq!(patches.len(), 1);
                assert_eq!(patches[0].path, "constraints[0].expression");
            }
            other => panic!("expected PatchExtensionYaml, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_tool_calls_write_extension_yaml() {
        let raw = r#"{"ontology": {}, "tool_calls": [
            {"name": "write_extension_yaml", "arguments": {
                "file": "rules.yaml",
                "content": "rules: []"
            }}
        ]}"#;
        let calls = parse_tool_calls_from_json(raw);
        assert_eq!(calls.len(), 1);
        match &calls[0] {
            AgentToolCall::WriteExtensionYaml { file, content } => {
                assert_eq!(file, "rules.yaml");
                assert_eq!(content, "rules: []");
            }
            other => panic!("expected WriteExtensionYaml, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_tool_calls_multiple() {
        let raw = r#"{"ontology": {}, "tool_calls": [
            {"name": "write_gateway_design", "arguments": {"content": "design"}},
            {"name": "patch_extension_yaml", "arguments": {"file": "rules.yaml", "patches": []}}
        ]}"#;
        let calls = parse_tool_calls_from_json(raw);
        assert_eq!(calls.len(), 2);
    }

    #[test]
    fn test_parse_tool_calls_no_field_returns_empty() {
        let raw = r#"{"ontology": {}}"#;
        let calls = parse_tool_calls_from_json(raw);
        assert!(calls.is_empty(), "无 tool_calls 字段应返回空 Vec");
    }

    #[test]
    fn test_parse_tool_calls_non_json_returns_empty() {
        let raw = "not json at all";
        let calls = parse_tool_calls_from_json(raw);
        assert!(calls.is_empty(), "非 JSON 应返回空 Vec(非致命)");
    }

    #[test]
    fn test_parse_tool_calls_with_code_fence() {
        let raw = "```json\n{\"ontology\": {}, \"tool_calls\": [{\"name\": \"write_gateway_design\", \"arguments\": {\"content\": \"x\"}}]}\n```";
        let calls = parse_tool_calls_from_json(raw);
        assert_eq!(calls.len(), 1);
    }

    #[test]
    fn test_parse_tool_calls_invalid_tool_ignored() {
        let raw = r#"{"ontology": {}, "tool_calls": [
            {"name": "unknown_tool", "arguments": {}},
            {"name": "write_gateway_design", "arguments": {"content": "valid"}}
        ]}"#;
        let calls = parse_tool_calls_from_json(raw);
        // 未知 tool 被忽略,只保留有效的
        assert_eq!(calls.len(), 1);
    }

    #[test]
    fn test_standard_tools_returns_three() {
        let tools = standard_tools();
        assert_eq!(tools.len(), 3);
        assert!(tools.iter().any(|t| t.name == "write_gateway_design"));
        assert!(tools.iter().any(|t| t.name == "write_extension_yaml"));
        assert!(tools.iter().any(|t| t.name == "patch_extension_yaml"));
    }
}
