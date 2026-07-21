//! AppAgent 产物质量评估器（agentic-eval 启发：Evaluator-Optimizer + Rubric-Based）
//!
//! 与 `validator.rs`（schema/格式硬校验）互补：本模块只做**语义质量 rubric**
//! （navigation_coherence / goal_fidelity / extension_coverage），不重复 schema 校验。
//!
//! 设计约束（来自规约审计 `.planning/compose-eval-loop-audit.md`）：
//! - 日志统一用 `common::telemetry`，禁止 `tracing` crate
//! - 解析 `app.json`/`extensions` 用 `serde_json`/`std::fs`，禁止正则（NO_REGEX_FOR_PARSING）
//! - 不加 `handlers/models/repositories/services` 标准骨架（app-agent 是引擎 crate）
//! - `eval_iteration` 持久化由 orchestrator 负责（存 `ConversationContext`），
//!   本模块只产出 `EvalReport`，自身无状态

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::state::{ConversationContext, FlowPlan};
use common::telemetry;

/// rubric 通过阈值（overall_score ≥ 此值才放行）
pub const THRESHOLD: f32 = 0.8;

/// Verifying → Composing 评估回流的最大次数（达上限强制放行）
pub const MAX_EVAL_ITERATIONS: u32 = 3;

/// 评估维度权重（与 agentic-eval Rubric-Based 公式一致：overall = Σ wᵢ·sᵢ）
pub const WEIGHTS: &[(&str, f32)] = &[
    ("schema_validity", 0.20),
    ("navigation_coherence", 0.20),
    ("goal_fidelity", 0.20),
    ("extension_coverage", 0.15),
    ("prototype_standalone", 0.25),
];

/// 单个评估维度结果
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvalDimension {
    pub name: String,
    pub weight: f32,
    pub score: f32,
    pub note: String,
}

/// 结构化 critique 项（对齐项目 tool_call JSON 习惯：status + feedback）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CritiqueItem {
    pub dimension: String,
    /// "PASS" | "FAIL"
    pub status: String,
    pub feedback: String,
}

/// 完整评估报告（JSON 序列化后可供 orchestrator 持久化 / 回流 Composing）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvalReport {
    pub overall_score: f32,
    pub threshold: f32,
    pub passed: bool,
    pub dimensions: Vec<EvalDimension>,
    pub critique: Vec<CritiqueItem>,
}

/// 待评估产物路径集合（由 orchestrator 在 Verifying 阶段传入）
pub struct Artifacts<'a> {
    pub app_json: &'a Path,
    pub extensions_dir: &'a Path,
    pub prototype_html: Option<&'a Path>,
    /// `ontology-mapping` Rust 二进制绝对路径（orchestrator 已知 repo root）
    pub check_script: Option<&'a Path>,
}

/// 评估器抽象：规则评估器（默认，无 LLM 依赖）或 LLM-as-Judge（orchestrator 注入）
///
/// `judge` 返回 `Some(score)` 时覆盖对应维度的规则评分；返回 `None` 时保留规则评分。
pub trait Judger {
    fn judge(
        &self,
        dimension: &str,
        ctx: &ConversationContext,
        artifacts: &Artifacts,
    ) -> Option<f32>;
}

/// 默认规则评估器：所有维度返回 `None`，即完全使用内置确定性规则评估
pub struct RuleBasedJudger;

impl Judger for RuleBasedJudger {
    fn judge(
        &self,
        _dimension: &str,
        _ctx: &ConversationContext,
        _artifacts: &Artifacts,
    ) -> Option<f32> {
        None
    }
}

/// LLM-as-Judge 预计算结果：持有各维度的 LLM 评分（0-1），供 `evaluate_with` 覆盖规则分。
///
/// 由 orchestrator 在调用前用 `LlmService` 异步算出分数后构造。本模块不依赖 LLM，
/// 仅做分数容器 + `Judger` 实现，保持与 LLM 基础设施解耦（符合设计：LLM 接线在 orchestrator）。
pub struct LlmJudger {
    pub scores: std::collections::HashMap<String, f32>,
}

impl LlmJudger {
    pub fn from_scores(scores: std::collections::HashMap<String, f32>) -> Self {
        Self { scores }
    }
}

impl Judger for LlmJudger {
    fn judge(
        &self,
        dimension: &str,
        _ctx: &ConversationContext,
        _artifacts: &Artifacts,
    ) -> Option<f32> {
        self.scores.get(dimension).copied()
    }
}

/// 为某维度构造 LLM-as-Judge 提示词（要求模型只返回 0-100 整数分数，便于解析）。
/// 纯函数，无 LLM 依赖；由 orchestrator 调用。
pub fn judge_prompt(dimension: &str, app_json: &JsonValue, user_description: &str) -> String {
    let app = serde_json::to_string_pretty(app_json).unwrap_or_default();
    format!(
        "你是一名 Alioth 应用产物质量评审专家。请评估以下 App 产物在「{dim}」维度的质量，仅返回一个 0-100 的整数分数（不要任何解释）。\n\n用户原始需求:\n{user}\n\n当前 app.json:\n{app}\n\n维度含义:\n- schema_validity: schema 关键约束是否满足\n- navigation_coherence: 导航模块组是否覆盖所用模块(used_modules)\n- goal_fidelity: app.json goal 是否与用户需求一致\n- extension_coverage: FlowPlan 提取的约束/规则/状态机/工作流是否都有对应 extension yaml\n- prototype_standalone: 原型是否可独立加载\n\n只输出分数(0-100):",
        dim = dimension,
        user = user_description,
        app = app
    )
}

/// 评估入口（默认规则评估器）
pub fn evaluate(ctx: &ConversationContext, artifacts: &Artifacts) -> EvalReport {
    evaluate_with(ctx, artifacts, &RuleBasedJudger)
}

/// 评估入口（可注入 LLM-as-Judge 覆盖特定维度）
pub fn evaluate_with(
    ctx: &ConversationContext,
    artifacts: &Artifacts,
    judger: &dyn Judger,
) -> EvalReport {
    let app = load_app_json(artifacts.app_json).unwrap_or(JsonValue::Null);
    let flow = ctx.flow_plan.as_ref();
    let user_desc = &ctx.user_description;

    let (schema_s, schema_n) = assess_schema_validity(&app);
    let (nav_s, nav_n) = assess_navigation_coherence(&app, flow);
    let (goal_s, goal_n) = assess_goal_fidelity(&app, user_desc);
    let (ext_s, ext_n) = assess_extension_coverage(flow, artifacts.extensions_dir);
    let (proto_s, proto_n) =
        assess_prototype_standalone(artifacts.prototype_html, artifacts.check_script);

    let mut dims = vec![
        EvalDimension {
            name: "schema_validity".into(),
            weight: 0.20,
            score: schema_s,
            note: schema_n,
        },
        EvalDimension {
            name: "navigation_coherence".into(),
            weight: 0.20,
            score: nav_s,
            note: nav_n,
        },
        EvalDimension {
            name: "goal_fidelity".into(),
            weight: 0.20,
            score: goal_s,
            note: goal_n,
        },
        EvalDimension {
            name: "extension_coverage".into(),
            weight: 0.15,
            score: ext_s,
            note: ext_n,
        },
        EvalDimension {
            name: "prototype_standalone".into(),
            weight: 0.25,
            score: proto_s,
            note: proto_n,
        },
    ];

    // LLM-as-Judge 覆盖（仅覆盖返回 Some 的维度）
    for d in &mut dims {
        if let Some(s) = judger.judge(&d.name, ctx, artifacts) {
            d.score = s.clamp(0.0, 1.0);
        }
    }

    let overall = overall_score(&dims);
    let critique: Vec<CritiqueItem> = dims.iter().map(critique_for).collect();

    telemetry::info!(
        "app evaluation overall_score={:.2} passed={} (schema={:.2} nav={:.2} goal={:.2} ext={:.2} proto={:.2})",
        overall, overall >= THRESHOLD, dims[0].score, dims[1].score, dims[2].score, dims[3].score, dims[4].score
    );

    EvalReport {
        overall_score: overall,
        threshold: THRESHOLD,
        passed: overall >= THRESHOLD,
        dimensions: dims,
        critique,
    }
}

/// 加权总分：overall = Σ(wᵢ·sᵢ) / Σwᵢ
pub fn overall_score(dims: &[EvalDimension]) -> f32 {
    let total_w: f32 = dims.iter().map(|d| d.weight).sum();
    if total_w == 0.0 {
        return 0.0;
    }
    let acc: f32 = dims.iter().map(|d| d.weight * d.score).sum();
    (acc / total_w * 100.0).round() / 100.0
}

fn critique_for(d: &EvalDimension) -> CritiqueItem {
    let status = if d.score >= 0.6 { "PASS" } else { "FAIL" }.to_string();
    CritiqueItem {
        dimension: d.name.clone(),
        status,
        feedback: d.note.clone(),
    }
}

// ─── 维度评估器（确定性规则，可单测） ────────────────────────────────────────

fn load_app_json(path: &Path) -> anyhow::Result<JsonValue> {
    let txt =
        std::fs::read_to_string(path).map_err(|e| anyhow::anyhow!("read app.json failed: {e}"))?;
    let v: JsonValue =
        serde_json::from_str(&txt).map_err(|e| anyhow::anyhow!("parse app.json failed: {e}"))?;
    Ok(v)
}

/// schema 关键约束（轻量；完整 jsonschema 校验由 `validator.rs` 负责，二者互补）
fn assess_schema_validity(app: &JsonValue) -> (f32, String) {
    let required = ["id", "code", "namespace", "name", "version", "status"];
    let missing: Vec<&str> = required
        .iter()
        .copied()
        .filter(|f| app.get(f).is_none() || app.get(f).unwrap().is_null())
        .collect();

    let status_ok = matches!(
        app.get("status").and_then(|v| v.as_str()),
        Some("developing") | Some("active") | Some("deprecated") | Some("archived")
    );
    let dm_ok = match app.get("deploymentMode") {
        None | Some(JsonValue::Null) => true,
        Some(JsonValue::String(s)) if s == "standalone" || s == "embedded" => true,
        _ => false,
    };

    if missing.is_empty() && status_ok && dm_ok {
        (1.0, "schema key constraints satisfied".to_string())
    } else {
        let mut note = String::new();
        if !missing.is_empty() {
            note.push_str(&format!("missing required fields: {:?}; ", missing));
        }
        if !status_ok {
            note.push_str("illegal status enum; ");
        }
        if !dm_ok {
            note.push_str("illegal deploymentMode (must be null/standalone/embedded); ");
        }
        (
            0.0,
            note.trim_end_matches(' ').trim_end_matches(';').to_string(),
        )
    }
}

/// 导航一致性：navigation 模块组是否覆盖 `FlowPlan.used_modules`
fn assess_navigation_coherence(app: &JsonValue, flow: Option<&FlowPlan>) -> (f32, String) {
    let expected: std::collections::HashSet<String> = match flow {
        Some(f) => f.used_modules.iter().cloned().collect(),
        None => return (1.0, "no flow_plan, nothing to verify".to_string()),
    };
    if expected.is_empty() {
        return (1.0, "no modules expected".to_string());
    }

    let mut actual = std::collections::HashSet::new();
    if let Some(groups) = app.get("navigation").and_then(|v| v.as_array()) {
        for g in groups {
            if let Some(mods) = g.get("modules").and_then(|m| m.as_array()) {
                for m in mods {
                    if let Some(s) = m.as_str() {
                        actual.insert(s.to_string());
                    }
                }
            }
        }
    }

    let matched = expected.intersection(&actual).count() as f32;
    let score = matched / expected.len() as f32;
    if score >= 1.0 {
        (1.0, "navigation covers all used_modules".to_string())
    } else {
        let missing: Vec<String> = expected.difference(&actual).cloned().collect();
        (score, format!("navigation missing modules: {:?}", missing))
    }
}

/// 目标保真度：app.json `goal` 与 `user_description` 的词元重叠（Jaccard）
fn assess_goal_fidelity(app: &JsonValue, user_desc: &str) -> (f32, String) {
    let goal = match app.get("goal").and_then(|v| v.as_str()) {
        Some(g) if !g.trim().is_empty() => g,
        _ => return (0.0, "goal field empty/missing".to_string()),
    };
    let a = tokenize(goal);
    let b = tokenize(user_desc);
    let inter = a.intersection(&b).count() as f32;
    let union = a.union(&b).count() as f32;
    let score = if union == 0.0 { 0.0 } else { inter / union };
    if score >= 0.3 {
        (
            score,
            format!("goal aligned with user_description (token overlap={score:.2})"),
        )
    } else {
        (
            score,
            format!("goal weakly aligned with user_description (token overlap={score:.2})"),
        )
    }
}

/// 扩展覆盖：FlowPlan 提取项是否都有对应 `extensions/*.yaml`
fn assess_extension_coverage(flow: Option<&FlowPlan>, ext_dir: &Path) -> (f32, String) {
    let mut expected: Vec<&str> = Vec::new();
    if let Some(f) = flow {
        if !f.constraints.is_empty() {
            expected.push("constraints.yaml");
        }
        if !f.business_rules.is_empty() {
            expected.push("rules.yaml");
        }
        let has_lifecycle = f
            .ontology_model_json
            .as_deref()
            .map(|j| j.contains("transaction_lifecycle"))
            .unwrap_or(false);
        if has_lifecycle {
            expected.push("statemachines.yaml");
        }
        if !f.workflow_steps.is_empty() {
            expected.push("workflows.yaml");
        }
    }
    if expected.is_empty() {
        return (1.0, "no extension content expected".to_string());
    }

    let mut missing = Vec::new();
    for f in &expected {
        let p = ext_dir.join(f);
        match std::fs::read_to_string(&p) {
            Ok(c) if !c.trim().is_empty() => {}
            _ => missing.push(*f),
        }
    }
    let satisfied = (expected.len() - missing.len()) as f32;
    let score = satisfied / expected.len() as f32;
    if score >= 1.0 {
        (1.0, "all expected extension yamls present".to_string())
    } else {
        (score, format!("missing extension yamls: {:?}", missing))
    }
}

/// 原型独立可加载：调用 `ontology-mapping prototype-check`（Rust CLI，与 P3 hooks 同一检查实现）
///
/// 中性策略：当原型或校验二进制未提供 / 原型文件不存在时返回 1.0（不罚分），
/// 避免 Verifying 在原型尚未接入的环境下误触发回流；仅当原型存在且校验失败才记 0。
fn assess_prototype_standalone(
    prototype: Option<&Path>,
    check_script: Option<&Path>,
) -> (f32, String) {
    let (proto, bin) = match (prototype, check_script) {
        (Some(p), Some(s)) => (p, s),
        _ => {
            return (
                1.0,
                "prototype standalone check skipped (prototype or check binary not provided)"
                    .to_string(),
            )
        }
    };
    if !proto.exists() {
        return (1.0, "prototype.html not found, skipped".to_string());
    }
    match std::process::Command::new(bin)
        .arg("prototype-check")
        .arg(proto)
        .output()
    {
        Ok(o) if o.status.success() => (1.0, "prototype passes standalone check".to_string()),
        Ok(o) => (
            0.0,
            format!(
                "standalone check failed: {}",
                String::from_utf8_lossy(&o.stderr)
            ),
        ),
        Err(e) => (0.0, format!("failed to run standalone check: {e}")),
    }
}

/// 词元化：字符级二元组（bigram）
///
/// CJK 字符在 `is_alphanumeric()` 下为真，按非字母数字切分无法分词，
/// 故对全文（去除空白）生成相邻字符二元组——中文文本相似度的标准做法。
/// 英文亦可用（按字母对重叠估算），属非结构化文本的轻量切分，非正则解析结构。
fn tokenize(s: &str) -> std::collections::HashSet<String> {
    let chars: Vec<char> = s.chars().filter(|c| !c.is_whitespace()).collect();
    let mut set = std::collections::HashSet::new();
    if chars.len() >= 2 {
        for w in chars.windows(2) {
            set.insert(w.iter().collect::<String>());
        }
    } else if let Some(&c) = chars.first() {
        set.insert(c.to_string());
    }
    set
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::FlowPlan;
    use serde_json::json;

    fn flow_with(modules: &[&str]) -> FlowPlan {
        FlowPlan {
            used_modules: modules.iter().map(|s| s.to_string()).collect(),
            namespace: "WZ".to_string(),
            known_entities: vec![],
            workflow_steps: vec![],
            missing_info: vec![],
            created_modules: vec![],
            created_blocks: vec![],
            created_services: vec![],
            ontology_model_json: None,
            functional_units: vec![],
            semantic_concepts: vec![],
            computations: vec![],
            constraints: vec![],
            business_rules: vec![],
            app_meta: None,
        }
    }

    #[test]
    fn overall_score_weighted_formula() {
        let dims = vec![
            EvalDimension {
                name: "a".into(),
                weight: 0.20,
                score: 1.0,
                note: String::new(),
            },
            EvalDimension {
                name: "b".into(),
                weight: 0.20,
                score: 0.0,
                note: String::new(),
            },
            EvalDimension {
                name: "c".into(),
                weight: 0.20,
                score: 1.0,
                note: String::new(),
            },
            EvalDimension {
                name: "d".into(),
                weight: 0.15,
                score: 1.0,
                note: String::new(),
            },
            EvalDimension {
                name: "e".into(),
                weight: 0.25,
                score: 1.0,
                note: String::new(),
            },
        ];
        // (0.2*1 + 0.2*0 + 0.2*1 + 0.15*1 + 0.25*1) / 1.0 = 0.8
        assert_eq!(overall_score(&dims), 0.8);
    }

    #[test]
    fn schema_validity_pass_and_fail() {
        let ok = json!({"id":1,"code":"ai-x","namespace":"WZ","name":"X","version":"0.1.0","status":"developing","deploymentMode":null});
        assert_eq!(assess_schema_validity(&ok).0, 1.0);

        let bad_dm = json!({"id":1,"code":"ai-x","namespace":"WZ","name":"X","version":"0.1.0","status":"bogus","deploymentMode":"single_process"});
        let (s, note) = assess_schema_validity(&bad_dm);
        assert_eq!(s, 0.0);
        assert!(note.contains("illegal status"));
        assert!(note.contains("illegal deploymentMode"));

        let missing = json!({"code":"ai-x"});
        assert_eq!(assess_schema_validity(&missing).0, 0.0);
    }

    #[test]
    fn navigation_coherence_covers_all() {
        let app = json!({"navigation":[{"group":"系统管理","modules":["mod-a","mod-b"]}]});
        let (s, _) = assess_navigation_coherence(&app, Some(&flow_with(&["mod-a", "mod-b"])));
        assert_eq!(s, 1.0);

        let (s2, note) =
            assess_navigation_coherence(&app, Some(&flow_with(&["mod-a", "mod-b", "mod-c"])));
        assert_eq!(s2, 2.0 / 3.0);
        assert!(note.contains("mod-c"));
    }

    #[test]
    fn goal_fidelity_overlap() {
        let app = json!({"goal":"管理采购订单与供应商合同"});
        let (s, note) = assess_goal_fidelity(&app, "我需要管理采购订单和供应商合同");
        assert!(s > 0.0);
        assert!(note.contains("aligned"));

        let empty = json!({});
        assert_eq!(assess_goal_fidelity(&empty, "anything").0, 0.0);
    }

    #[test]
    fn extension_coverage_detects_missing() {
        let dir = std::env::temp_dir().join(format!("eval_ext_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        // 写一个 constraints.yaml，不写 rules.yaml
        std::fs::write(dir.join("constraints.yaml"), "constraints: []").unwrap();

        let mut flow = flow_with(&["mod-a"]);
        flow.constraints.push(crate::state::ConstraintPlan {
            entity: "e".into(),
            field: None,
            expression: "x>0".into(),
            level: "error".into(),
            message: "m".into(),
        });
        flow.business_rules.push(crate::state::BusinessRulePlan {
            entity: "e".into(),
            rule_name: "r".into(),
            trigger: "always".into(),
            condition: "c".into(),
            action: "a".into(),
            priority: 1,
            error_message: "e".into(),
        });
        let (s, note) = assess_extension_coverage(Some(&flow), &dir);
        assert_eq!(s, 0.5);
        assert!(note.contains("rules.yaml"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn critique_status_mapping() {
        let pass = EvalDimension {
            name: "x".into(),
            weight: 1.0,
            score: 0.9,
            note: "ok".into(),
        };
        assert_eq!(critique_for(&pass).status, "PASS");
        let fail = EvalDimension {
            name: "x".into(),
            weight: 1.0,
            score: 0.2,
            note: "bad".into(),
        };
        assert_eq!(critique_for(&fail).status, "FAIL");
    }

    #[test]
    fn report_serde_roundtrip() {
        let ctx = ConversationContext::new(1, "管理采购订单".into(), "WZ".into());
        // 构造临时 app.json + 空 extensions 目录
        let base = std::env::temp_dir().join(format!("eval_rt_{}", uuid::Uuid::new_v4()));
        let apps = base.join("Apps");
        let ext = apps.join("extensions");
        std::fs::create_dir_all(&ext).unwrap();
        let app_path = apps.join("app.json");
        std::fs::write(
            &app_path,
            json!({
                "id":1,"code":"ai-x","namespace":"WZ","name":"X","version":"0.1.0",
                "status":"developing","deploymentMode":null,
                "navigation":[{"group":"g","modules":["mod-a"]}],
                "goal":"管理采购订单"
            })
            .to_string(),
        )
        .unwrap();

        let arts = Artifacts {
            app_json: &app_path,
            extensions_dir: &ext,
            prototype_html: None,
            check_script: None,
        };
        let report = evaluate(&ctx, &arts);
        let ser = serde_json::to_string(&report).unwrap();
        let back: EvalReport = serde_json::from_str(&ser).unwrap();
        assert_eq!(report, back);
        assert!(back.overall_score >= 0.0 && back.overall_score <= 1.0);
        let _ = std::fs::remove_dir_all(&base);
    }
}
