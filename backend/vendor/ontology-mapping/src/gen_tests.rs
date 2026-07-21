//! gen_tests.rs — 测试骨架生成器（1:1 移植 gen-service-tests.py + gen-flow-tests.py）
//!
//! - `gen_service_tests`：pipeline_manifest + service.json → CRUD 测试骨架（模板渲染，确定性）
//! - `gen_flow_tests`：manifest refs + flow-plan + extensions/*.yaml → 流式场景测试
//!   （--llm 模式经 reqwest 调 deepseek 生成测试体，含断言/DB 设置/测试函数三重校验）

use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};

// ── CRUD 模板（与 Python 逐字一致）─────────────────────

const TEMPLATE_HEADER: &str = r#"//! {svc_id} factor — generated CRUD 测试骨架
//!
//! 由 ontology-mapping gen-service-tests 自动生成 (dry-run / --write)。
//! 手工填充字段值后，并入 CI 套件。
//!
//! 模板来源: {manifest_path}

use test_utils::prelude::*;
use sqlx::PgPool;

"#;

fn render_entity(entity: &serde_json::Value) -> Option<String> {
    let name = entity.get("name")?.as_str()?;
    let table = entity
        .get("table")
        .and_then(|t| t.as_str())
        .map(String::from)
        .unwrap_or_else(|| format!("isahl.zc_id_{}", name.to_lowercase()));
    let ent_lower = {
        let mut c = name.chars();
        c.next().unwrap().to_lowercase().collect::<String>() + c.as_str()
    };

    Some(format!(
        r#"// ══════════════════════════════════════════════════
// {name} — {table}
// ══════════════════════════════════════════════════

async fn insert_{ent_lower}(pool: &PgPool, notice: &str) -> i64 {{
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO {table} (notice) VALUES ($1) RETURNING id"
    )
    .bind(notice)
    .fetch_one(pool)
    .await
    .expect("insert_{ent_lower} failed")
}}

async fn insert_{ent_lower}_with_code(pool: &PgPool, notice: &str, code: &str) -> i64 {{
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO {table} (notice, code) VALUES ($1, $2) RETURNING id"
    )
    .bind(notice)
    .bind(code)
    .fetch_one(pool)
    .await
    .expect("insert_{ent_lower}_with_code failed")
}}

async fn count_{ent_lower}(pool: &PgPool) -> i64 {{
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM {table} WHERE deleted_at IS NULL"
    )
    .fetch_one(pool)
    .await
    .expect("count_{ent_lower} failed")
}}

async fn get_{ent_lower}_notice(pool: &PgPool, id: i64) -> String {{
    sqlx::query_scalar::<_, String>(
        "SELECT notice FROM {table} WHERE id = $1"
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .expect("get_{ent_lower}_notice failed")
}}

async fn update_{ent_lower}_notice(pool: &PgPool, id: i64, new_notice: &str) {{
    sqlx::query("UPDATE {table} SET notice = $1, updated_at = NOW() WHERE id = $2")
        .bind(new_notice)
        .bind(id)
        .execute(pool)
        .await
        .expect("update_{ent_lower}_notice failed");
}}

async fn soft_delete_{ent_lower}(pool: &PgPool, id: i64) {{
    sqlx::query("UPDATE {table} SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .expect("soft_delete_{ent_lower} failed");
}}

// Tests — {name} CRUD

#[tokio::test]
async fn {ent_lower}_create_read() {{
    let pool = connect_test_db().await;
    setup_test_schema(&pool).await.expect("setup failed");

    let id = insert_{ent_lower}(&pool, "test-{ent_lower}").await;
    assert!(id > 0, "insert {ent_lower} should return a valid ID");

    let count = count_{ent_lower}(&pool).await;
    assert!(count > 0, "count should be > 0 after insert");

    let notice = get_{ent_lower}_notice(&pool, id).await;
    assert_eq!(notice, "test-{ent_lower}", "notice should match inserted value");
}}

#[tokio::test]
async fn {ent_lower}_update() {{
    let pool = connect_test_db().await;
    setup_test_schema(&pool).await.expect("setup failed");

    let id = insert_{ent_lower}(&pool, "original-{ent_lower}").await;
    update_{ent_lower}_notice(&pool, id, "updated-{ent_lower}").await;

    let notice = get_{ent_lower}_notice(&pool, id).await;
    assert_eq!(notice, "updated-{ent_lower}", "notice should reflect update");
}}

#[tokio::test]
async fn {ent_lower}_soft_delete() {{
    let pool = connect_test_db().await;
    setup_test_schema(&pool).await.expect("setup failed");

    let id = insert_{ent_lower}(&pool, "delete-{ent_lower}").await;
    soft_delete_{ent_lower}(&pool, id).await;

    // After soft delete, count should be 0
    let remaining = count_{ent_lower}(&pool).await;
    assert_eq!(remaining, 0, "count should be 0 after soft delete");
}}
"#
    ))
}

fn svc_dir(ns: &str, svc_id: &str) -> PathBuf {
    if ns.is_empty() {
        PathBuf::from(format!("Sources/Services/{svc_id}"))
    } else {
        PathBuf::from(format!("Pre-Proc/{ns}/Sources/Services/{svc_id}"))
    }
}

#[derive(Debug, Serialize)]
pub struct GenResult {
    pub service: String,
    pub output_file: String,
    pub bytes: usize,
    pub written: bool,
    pub skipped: Option<String>,
}

/// gen-service-tests：manifest → 每个 service 的 CRUD 测试骨架
pub fn gen_service_tests(manifest_path: &Path, write: bool) -> Result<Vec<GenResult>> {
    let content = std::fs::read_to_string(manifest_path)?;
    let manifest: serde_json::Value = serde_json::from_str(&content)?;
    let ns = manifest
        .get("namespace")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let services = manifest
        .get("services")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    anyhow::ensure!(
        !services.is_empty(),
        "manifest 中无 services 列表（AppAgent 可能尚未创建 service）"
    );

    let mut results = Vec::new();
    for svc in &services {
        let Some(svc_id) = svc.get("id").and_then(|v| v.as_str()) else {
            continue;
        };
        let svc_json_path = svc_dir(ns, svc_id).join("service.json");
        if !svc_json_path.exists() {
            results.push(GenResult {
                service: svc_id.into(),
                output_file: String::new(),
                bytes: 0,
                written: false,
                skipped: Some(format!(
                    "service.json not found at {}",
                    svc_json_path.display()
                )),
            });
            continue;
        }
        let svc_json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&svc_json_path)?)?;
        let entities = svc_json
            .get("ontology")
            .and_then(|o| o.get("entities"))
            .and_then(|e| e.as_array())
            .cloned()
            .unwrap_or_default();
        if entities.is_empty() {
            results.push(GenResult {
                service: svc_id.into(),
                output_file: String::new(),
                bytes: 0,
                written: false,
                skipped: Some("no ontology.entities defined".into()),
            });
            continue;
        }

        let mut sections = vec![TEMPLATE_HEADER
            .replace("{svc_id}", svc_id)
            .replace("{manifest_path}", &manifest_path.display().to_string())];
        for entity in &entities {
            if let Some(code) = render_entity(entity) {
                sections.push(code);
            }
        }
        let content = sections.join("\n") + "\n";

        let out_file = svc_dir(ns, svc_id).join("backend/tests/generated_crud_test.rs");
        if write {
            if let Some(parent) = out_file.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&out_file, &content)?;
        }
        results.push(GenResult {
            service: svc_id.into(),
            output_file: out_file.display().to_string(),
            bytes: content.len(),
            written: write,
            skipped: None,
        });
    }
    Ok(results)
}

// ── Flow 测试场景推导 ──────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct FlowScenario {
    pub name: String,
    pub description: String,
    pub steps: Vec<String>,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transitions: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phases: Option<serde_json::Value>,
}

#[derive(Debug, Default)]
pub struct FlowContext {
    pub namespace: String,
    pub workflow_steps: Vec<String>,
    pub business_rules: Vec<serde_json::Value>,
    pub transaction_lifecycle: Option<serde_json::Value>,
    pub statemachines: Vec<serde_json::Value>,
    pub services: Vec<serde_json::Value>,
}

pub fn extract_flow_context(manifest: &serde_json::Value) -> FlowContext {
    let mut ctx = FlowContext {
        namespace: manifest
            .get("namespace")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string(),
        services: manifest
            .get("services")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default(),
        ..Default::default()
    };
    let refs = manifest.get("refs").cloned().unwrap_or_default();

    if let Some(fp_path) = refs.get("flow_plan").and_then(|v| v.as_str()) {
        if let Ok(content) = std::fs::read_to_string(fp_path) {
            if let Ok(fp) = serde_json::from_str::<serde_json::Value>(&content) {
                ctx.workflow_steps = fp
                    .get("workflow_steps")
                    .and_then(|v| v.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|s| s.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                ctx.business_rules = fp
                    .get("business_rules")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
            }
        }
    }
    if let Some(om_path) = refs.get("ontology_model").and_then(|v| v.as_str()) {
        if let Ok(content) = std::fs::read_to_string(om_path) {
            if let Ok(om) = serde_json::from_str::<serde_json::Value>(&content) {
                ctx.transaction_lifecycle = om.get("transaction_lifecycle").cloned();
            }
        }
    }
    if let Some(out_path) = refs.get("output_path").and_then(|v| v.as_str()) {
        let ext_dir = PathBuf::from(out_path).join("extensions");
        if ext_dir.is_dir() {
            if let Ok(rd) = std::fs::read_dir(&ext_dir) {
                for entry in rd.flatten() {
                    let p = entry.path();
                    if p.extension().is_some_and(|e| e == "yaml" || e == "yml") {
                        let fname = p
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        if fname.contains("statemachines") {
                            if let Ok(content) = std::fs::read_to_string(&p) {
                                if let Ok(data) =
                                    yaml_serde::from_str::<serde_json::Value>(&content)
                                {
                                    let sms = data.get("statemachines").cloned().unwrap_or(data);
                                    match sms {
                                        serde_json::Value::Array(a) => ctx.statemachines.extend(a),
                                        v => ctx.statemachines.push(v),
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    ctx
}

pub fn build_flow_scenarios(ctx: &FlowContext) -> Vec<FlowScenario> {
    let mut scenarios = Vec::new();
    let steps = &ctx.workflow_steps;
    let tlc = &ctx.transaction_lifecycle;
    let rules = &ctx.business_rules;
    let sms = &ctx.statemachines;

    if steps.is_empty() && tlc.is_none() {
        return scenarios;
    }

    if !steps.is_empty() {
        scenarios.push(FlowScenario {
            name: "full_flow".into(),
            description: format!("正向流程: {}", steps.join(" → ")),
            steps: steps.clone(),
            r#type: "happy_path".into(),
            rules: None,
            transitions: None,
            phases: None,
        });
        if !rules.is_empty() {
            let descs: Vec<String> = rules
                .iter()
                .take(3)
                .map(|r| {
                    format!(
                        "rule: {} {}",
                        r.get("condition").and_then(|v| v.as_str()).unwrap_or(""),
                        r.get("action").and_then(|v| v.as_str()).unwrap_or("")
                    )
                })
                .collect();
            scenarios.push(FlowScenario {
                name: "rule_validation".into(),
                description: format!("业务规则验证: {}", descs.join("; ")),
                steps: steps[..1].to_vec(),
                r#type: "rule_check".into(),
                rules: Some(serde_json::Value::Array(rules.clone())),
                transitions: None,
                phases: None,
            });
        }
    }

    if let Some(tlc) = tlc {
        if let Some(phases) = tlc.get("phases").and_then(|v| v.as_array()) {
            if !phases.is_empty() {
                let names: Vec<String> = phases
                    .iter()
                    .map(|p| {
                        p.get("name")
                            .or_else(|| p.get("id"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("?")
                            .to_string()
                    })
                    .collect();
                scenarios.push(FlowScenario {
                    name: "lifecycle_flow".into(),
                    description: format!("生命周期: {}", names.join(" → ")),
                    steps: names,
                    r#type: "lifecycle".into(),
                    rules: None,
                    transitions: None,
                    phases: Some(serde_json::Value::Array(phases.clone())),
                });
            }
        }
    }

    for sm in sms {
        let name = sm
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("state_machine");
        if let Some(transitions) = sm.get("transitions").and_then(|v| v.as_array()) {
            if !transitions.is_empty() {
                let tnames: Vec<String> = transitions
                    .iter()
                    .take(5)
                    .map(|t| {
                        format!(
                            "{}→{}",
                            t.get("from").and_then(|v| v.as_str()).unwrap_or("?"),
                            t.get("to").and_then(|v| v.as_str()).unwrap_or("?")
                        )
                    })
                    .collect();
                scenarios.push(FlowScenario {
                    name: format!("sm_{name}"),
                    description: format!("状态机: {name} ({} 转换)", transitions.len()),
                    steps: tnames,
                    r#type: "state_machine".into(),
                    rules: None,
                    transitions: Some(serde_json::Value::Array(transitions.clone())),
                    phases: None,
                });
            }
        }
    }
    scenarios
}

// ── LLM 模式 ──────────────────────────────────────────

fn build_llm_prompt(
    ctx: &FlowContext,
    scenarios: &[FlowScenario],
    svc_json: Option<&serde_json::Value>,
) -> String {
    let svc_ids: Vec<&str> = ctx
        .services
        .iter()
        .filter_map(|s| s.get("id").and_then(|v| v.as_str()))
        .collect();
    let mut prompt = format!(
        "你是一位 Rust 测试工程师，需要为 AliothStudio 项目的业务流生成端到端测试。\n\n项目 namespace: {}\n涉及的 Service: {}\n\n## 业务上下文\n\nworkflow_steps: {}\n\nbusiness_rules: {}\n\n",
        ctx.namespace,
        if svc_ids.is_empty() { "?".into() } else { svc_ids.join(", ") },
        serde_json::to_string(&ctx.workflow_steps).unwrap_or_default(),
        serde_json::to_string(&ctx.business_rules).unwrap_or_default(),
    );

    if let Some(svc) = svc_json {
        if let Some(entities) = svc
            .get("ontology")
            .and_then(|o| o.get("entities"))
            .and_then(|e| e.as_array())
        {
            if !entities.is_empty() {
                prompt.push_str("## 数据库实体定义\n\n");
                for ent in entities {
                    let name = ent.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                    let table = ent.get("table").and_then(|v| v.as_str()).unwrap_or("?");
                    prompt.push_str(&format!("### {name}\n- 表: `{table}`\n"));
                    if let Some(coords) = ent.get("coordinates") {
                        prompt.push_str(&format!(
                            "- 坐标: scene={} factor={} function={}\n",
                            coords.get("scene").and_then(|v| v.as_str()).unwrap_or("?"),
                            coords.get("factor").and_then(|v| v.as_str()).unwrap_or("?"),
                            coords
                                .get("function")
                                .and_then(|v| v.as_str())
                                .unwrap_or("?"),
                        ));
                    }
                    if let Some(fms) = ent.get("field_mappings").and_then(|v| v.as_array()) {
                        let cols: Vec<&str> = fms
                            .iter()
                            .take(10)
                            .filter_map(|f| f.get("column").and_then(|c| c.as_str()))
                            .collect();
                        prompt.push_str(&format!("- 字段: {}\n", cols.join(", ")));
                    }
                    prompt.push('\n');
                }
            }
        }
    }

    prompt.push_str("## 测试场景\n\n");
    for sc in scenarios {
        prompt.push_str(&format!(
            "### 场景: {}\n描述: {}\n类型: {}\n步骤: {}\n",
            sc.name,
            sc.description,
            sc.r#type,
            serde_json::to_string(&sc.steps).unwrap_or_default(),
        ));
        if let Some(rules) = &sc.rules {
            prompt.push_str(&format!(
                "规则: {}\n",
                serde_json::to_string(rules).unwrap_or_default()
            ));
        }
        if let Some(tr) = &sc.transitions {
            prompt.push_str(&format!(
                "状态转换: {}\n",
                serde_json::to_string(tr).unwrap_or_default()
            ));
        }
    }

    prompt.push_str(
        r#"
## 输出要求

请为每个场景生成一个 `#[tokio::test]` 异步测试函数。使用以下模式和约束：

```rust
use test_utils::prelude::*;
use sqlx::PgPool;

#[tokio::test]
async fn {scenario_name}() {
    let pool = connect_test_db().await;
    setup_test_schema(&pool).await.expect("setup failed");

    // 1. 准备测试数据 (INSERT INTO 上面定义的表)
    // 2. 执行流程步骤
    // 3. 在每个步骤后验证数据状态
    // 4. 验证最终结果符合预期
}
```

**必须引用上面定义的数据库表名和字段，禁止凭空创造表名。**

返回格式: 只返回 Rust 代码块，不要额外解释。
"#,
    );
    prompt
}

async fn call_llm(prompt: &str) -> Option<String> {
    let api_key = std::env::var("LLM_API_KEY").ok()?;
    let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "deepseek/deepseek-v4-flash".into());
    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.deepseek.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&serde_json::json!({
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.3,
        }))
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body: serde_json::Value = resp.json().await.ok()?;
    body.get("choices")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .map(String::from)
}

/// 从 LLM 响应提取 Rust 代码块 + 三重校验（断言/DB 设置/测试函数）
fn extract_and_validate(llm_result: &str) -> Result<String, String> {
    let code = if let Some(rest) = llm_result.split("```rust").nth(1) {
        rest.split("```").next().unwrap_or(rest)
    } else if let Some(rest) = llm_result.split("```").nth(1) {
        rest.split("```").next().unwrap_or(rest)
    } else {
        llm_result
    };
    let code = code.trim();
    if !code.contains("assert!") && !code.contains("assert_eq!") && !code.contains("assert_ne!") {
        return Err("LLM validation FAIL: no assertion (assert!/assert_eq!/assert_ne!)".into());
    }
    if !code.contains("connect_test_db") && !code.contains("setup_test_schema") {
        return Err("LLM validation FAIL: no DB setup (connect_test_db/setup_test_schema)".into());
    }
    if !code.contains("#[tokio::test]") {
        return Err("LLM validation FAIL: no #[tokio::test] function".into());
    }
    Ok(code.to_string())
}

/// gen-flow-tests：manifest → 流式场景测试（--llm 生成；否则仅输出场景计划）
pub async fn gen_flow_tests(
    manifest_path: &Path,
    write: bool,
    use_llm: bool,
    target_svc: Option<&str>,
) -> Result<(Vec<GenResult>, Vec<FlowScenario>)> {
    let content = std::fs::read_to_string(manifest_path)?;
    let manifest: serde_json::Value = serde_json::from_str(&content)?;
    let ns = manifest
        .get("namespace")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let ctx = extract_flow_context(&manifest);
    let scenarios = build_flow_scenarios(&ctx);
    anyhow::ensure!(
        !scenarios.is_empty(),
        "无法从上下文推导业务流场景（需要 workflow_steps 或 transaction_lifecycle）"
    );

    let mut results = Vec::new();
    for svc in &ctx.services {
        let Some(svc_id) = svc.get("id").and_then(|v| v.as_str()) else {
            continue;
        };
        if let Some(t) = target_svc {
            if svc_id != t {
                continue;
            }
        }
        let svc_json_path = svc_dir(ns, svc_id).join("service.json");
        if !svc_json_path.exists() {
            results.push(GenResult {
                service: svc_id.into(),
                output_file: String::new(),
                bytes: 0,
                written: false,
                skipped: Some("no service.json".into()),
            });
            continue;
        }
        let svc_json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&svc_json_path)?)?;

        let out_file = svc_dir(ns, svc_id).join("backend/tests/generated_flow_test.rs");
        if !use_llm {
            results.push(GenResult {
                service: svc_id.into(),
                output_file: out_file.display().to_string(),
                bytes: 0,
                written: false,
                skipped: Some("需要 --llm 生成测试体".into()),
            });
            continue;
        }

        let prompt = build_llm_prompt(&ctx, &scenarios, Some(&svc_json));
        match call_llm(&prompt).await {
            Some(llm_result) => match extract_and_validate(&llm_result) {
                Ok(code) => {
                    let full = format!(
                        "//! {ns} — generated flow tests\n//! 由 ontology-mapping gen-flow-tests --llm 自动生成。\n\nuse test_utils::prelude::*;\nuse sqlx::PgPool;\n\n{code}\n"
                    );
                    if write {
                        if let Some(parent) = out_file.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        std::fs::write(&out_file, &full)?;
                    }
                    results.push(GenResult {
                        service: svc_id.into(),
                        output_file: out_file.display().to_string(),
                        bytes: full.len(),
                        written: write,
                        skipped: None,
                    });
                }
                Err(e) => {
                    results.push(GenResult {
                        service: svc_id.into(),
                        output_file: String::new(),
                        bytes: 0,
                        written: false,
                        skipped: Some(e),
                    });
                }
            },
            None => {
                results.push(GenResult {
                    service: svc_id.into(),
                    output_file: String::new(),
                    bytes: 0,
                    written: false,
                    skipped: Some("LLM 调用失败或无 LLM_API_KEY".into()),
                });
            }
        }
    }
    Ok((results, scenarios))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_entity() {
        let entity = serde_json::json!({"name": "Inventory", "table": "isahl.zc_id_inventory"});
        let code = render_entity(&entity).unwrap();
        assert!(code.contains("async fn insert_inventory("));
        assert!(code.contains("#[tokio::test]"));
        assert!(code.contains("async fn inventory_create_read()"));
        assert!(code.contains("async fn inventory_soft_delete()"));
        assert!(code.contains("isahl.zc_id_inventory"));
    }

    #[test]
    fn test_build_scenarios_workflow() {
        let ctx = FlowContext {
            namespace: "WZ".into(),
            workflow_steps: vec!["下单".into(), "派车".into(), "签收".into()],
            ..Default::default()
        };
        let scenarios = build_flow_scenarios(&ctx);
        assert_eq!(scenarios.len(), 1);
        assert_eq!(scenarios[0].name, "full_flow");
        assert!(scenarios[0].description.contains("下单 → 派车 → 签收"));
    }

    #[test]
    fn test_build_scenarios_lifecycle() {
        let ctx = FlowContext {
            namespace: "Alioth".into(),
            transaction_lifecycle: Some(serde_json::json!({
                "phases": [{"id": "draft"}, {"id": "approved"}]
            })),
            ..Default::default()
        };
        let scenarios = build_flow_scenarios(&ctx);
        assert_eq!(scenarios.len(), 1);
        assert_eq!(scenarios[0].name, "lifecycle_flow");
    }

    #[test]
    fn test_extract_and_validate() {
        let good = "```rust\n#[tokio::test]\nasync fn t() {\n let pool = connect_test_db().await;\n assert!(true);\n}\n```";
        assert!(extract_and_validate(good).is_ok());
        let no_assert = "#[tokio::test]\nasync fn t() { connect_test_db().await; }";
        assert!(extract_and_validate(no_assert).is_err());
        let no_test_fn = "fn t() { assert!(true); }";
        assert!(extract_and_validate(no_test_fn).is_err());
    }

    #[test]
    fn test_svc_dir() {
        assert_eq!(
            svc_dir("WZ", "identity").display().to_string(),
            "Pre-Proc/WZ/Sources/Services/identity"
        );
        assert_eq!(svc_dir("", "x").display().to_string(), "Sources/Services/x");
    }
}
