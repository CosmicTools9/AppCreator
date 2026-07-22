//! App Composer — 应用组装器（v2：模块组合 + 逻辑扩展）
//!
//! 重构目标：
//! - 不复写模块代码，只生成 `app.json` + `extensions/*.yaml`
//! - 缺口（未覆盖需求）写入 `request-no-impl/*.md`
//! - 移除完整的 backend/frontend 代码生成流水线

mod app_tsx_template;
mod esm_runner;

pub use esm_runner::generate_and_build_app_tsx;
pub use esm_runner::sync_prototype;

use crate::state::progress_event;
use crate::state::{AgentProgress, BusinessRulePlan, ComputationPlan, ConstraintPlan, FlowPlan};
use common::telemetry::info;
use runtime_engine::{
    AppModelConfig, ConstraintExtension, ConstraintSeverity, LifecycleEvent, ModuleModelConfig,
    RuleExtension, State, StateMachineExtension, Transition, WorkflowAction, WorkflowDefinition,
    WorkflowErrorHandling, WorkflowStep, WorkflowTrigger,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::path::{Path, PathBuf};
/// 解析项目根目录（AliothStudio/）
///
/// 策略：从当前目录向上遍历，查找同时满足以下条件的目录：
/// 1. 包含 `pnpm-workspace.yaml`（唯一的项目根标记文件）
/// 2. 包含 `Pre-Proc/` 子目录
///
/// 双重标记（`pnpm-workspace.yaml` + `Pre-Proc/`）确保无论 binary 从
/// `Meta/backend/`、`deploy/meta/bin/` 还是项目根目录启动，
/// 都能可靠定位到正确的 `Pre-Proc/Apps/` 绝对路径。
pub(crate) fn resolve_project_root() -> PathBuf {
    let mut current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for _ in 0..8 {
        // 同时满足两个条件才是真正的项目根
        if current.join("pnpm-workspace.yaml").is_file() && current.join("Pre-Proc").is_dir() {
            return current;
        }
        if !current.pop() {
            break;
        }
    }
    // fallback：返回当前目录（保持向后兼容）
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ComposerError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("YAML serialization error: {0}")]
    YamlSerialization(#[from] yaml_serde::Error),

    #[error("JSON serialization error: {0}")]
    JsonSerialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Validation error: {0}")]
    Validation(String),
}

/// 应用组装结果
#[derive(Debug, Clone)]
pub struct ComposeResult {
    pub app_name: String,
    pub output_path: String,
    pub files_written: usize,
    pub module_count: usize,
}

/// App 状态机：合法转换规则
///
/// - `developing` → `production`（发布）
/// - `production` → `developing`（回滚）
/// - `developing` → `developing`（重新生成覆盖）
///
/// 无效转换返回错误信息。
pub fn validate_status_transition(from: Option<&str>, to: &str) -> Result<(), String> {
    let from = from.unwrap_or("developing");

    // 允许：相同状态（幂等操作）
    if from == to {
        return Ok(());
    }

    match (from, to) {
        ("developing", "production") => Ok(()),
        ("production", "developing") => Ok(()),
        (_, "developing" | "production") => Ok(()), // 未知状态→已知状态：宽容处理
        (_, _) => Err(format!(
            "Invalid status transition: '{}' → '{}'. Allowed: developing↔production.",
            from, to
        )),
    }
}
/// 从 FlowPlan 组装应用配置（声明式，无代码生成）
///
/// 输出结构：
/// ```text
/// Pre-Proc/Apps/{app_name}/
/// ├── app.json                    ← 模块组合配置
/// ├── extensions/
/// │   ├── constraints.yaml        ← 约束验证（可选）
/// │   ├── rules.yaml              ← 业务规则（可选）
/// │   ├── statemachines.yaml      ← 状态机覆盖（可选）
/// │   └── workflows.yaml          ← 流程编排（可选）
/// └── request-no-impl/
///     └── gap-*.md                ← 未覆盖需求文档（可选）
/// ```
/// 事务性应用组装：先写到 staging 目录，全部成功后原子 rename。
///
/// 流程：
/// 1. 创建 staging 目录 `Pre-Proc/Apps/.{app_name}.{uuid}/`
/// 2. 按序写入 app.json / extensions/*.yaml / request-no-impl/*.md
async fn scan_module_registry(
    plan: &FlowPlan,
) -> Result<(AppModelConfig, std::collections::HashMap<String, String>), ComposerError> {
    let mut model_registry = AppModelConfig::default();
    let mut module_versions: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for module_id in &plan.used_modules {
        let mut module_config = ModuleModelConfig::default();
        let module_json_path = format!(
            "../../Pre-Proc/{}/Sources/Modules/{}/module.json",
            plan.namespace, module_id
        );
        if let Ok(content) = tokio::fs::read_to_string(&module_json_path).await {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(version) = json.get("version").and_then(|v| v.as_str()) {
                    module_versions.insert(module_id.clone(), version.to_string());
                }
                if let Some(entities) = json
                    .get("extensionPoints")
                    .and_then(|ep| ep.get("entities"))
                    .and_then(|e| e.as_array())
                {
                    module_config.enabled_entities = entities
                        .iter()
                        .filter_map(|e| {
                            e.get("entity_name")
                                .and_then(|n| n.as_str())
                                .map(|s| s.to_string())
                        })
                        .collect();
                }
            }
        }
        if !module_config.enabled_entities.is_empty() {
            model_registry
                .modules
                .insert(module_id.clone(), module_config);
        }
    }
    Ok((model_registry, module_versions))
}

/// 对齐 Gateway `runtime-engine` `extension.rs::load_from_dir` 的 `ProfilesWrapper` 契约：
/// `profiles.yaml` 顶层为 `profiles: { <profile_name>: AppModelConfig }`。
/// AppAgent 固定以 `"default"` 作为主档案名写入，保证与 `merge_profiles(["default"])` 命中。
#[derive(Serialize)]
struct ProfilesWrapper {
    profiles: std::collections::HashMap<String, AppModelConfig>,
}
async fn write_extensions_to_staging<F>(
    plan: &FlowPlan,
    app_name: &str,
    stage_dir: &Path,
    ontology: Option<&alioth_gen::generator::ir::ontology::OntologyModel>,
    on_progress: Option<&F>,
) -> Result<usize, ComposerError>
where
    F: Fn(AgentProgress) + Send + Sync,
{
    let extensions_dir = stage_dir.join("extensions");
    tokio::fs::create_dir_all(&extensions_dir).await?;
    let mut count = 0usize;

    // constraints.yaml
    if !plan.constraints.is_empty() {
        let constraints: Vec<ConstraintExtension> = plan
            .constraints
            .iter()
            .map(|c| ConstraintExtension {
                entity: c.entity.clone(),
                field: c.field.clone(),
                expression: c.expression.clone(),
                level: match c.level.as_str() {
                    "warning" => ConstraintSeverity::Warning,
                    _ => ConstraintSeverity::Error,
                },
                message: c.message.clone(),
            })
            .collect();
        let yaml = yaml_serde::to_string(&constraints)?;
        write_file(&extensions_dir.join("constraints.yaml"), &yaml).await?;
        count += 1;
        info!(
            "Generated constraints.yaml: {} constraints",
            constraints.len()
        );
        if let Some(cb) = on_progress {
            let file_name = "constraints.yaml";
            let rel_path = format!("{}/extensions/{}", app_name, file_name);
            cb(AgentProgress::new(
                "构建应用",
                82,
                format!("已写入扩展 {}", file_name),
                progress_event::ARTIFACT_WRITTEN,
                Some(json!({"path": rel_path, "kind": "extension"})),
            ));
        }
    }

    // rules.yaml
    if !plan.business_rules.is_empty() {
        let rules: Vec<RuleExtension> = plan
            .business_rules
            .iter()
            .map(|r| RuleExtension {
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
        let yaml = yaml_serde::to_string(&rules)?;
        write_file(&extensions_dir.join("rules.yaml"), &yaml).await?;
        count += 1;
        info!("Generated rules.yaml: {} rules", rules.len());
        if let Some(cb) = on_progress {
            let file_name = "rules.yaml";
            let rel_path = format!("{}/extensions/{}", app_name, file_name);
            cb(AgentProgress::new(
                "构建应用",
                82,
                format!("已写入扩展 {}", file_name),
                progress_event::ARTIFACT_WRITTEN,
                Some(json!({"path": rel_path, "kind": "extension"})),
            ));
        }
    }

    // statemachines.yaml
    if let Some(ontology) = ontology {
        if let Some(ref lifecycle) = ontology.transaction_lifecycle {
            let sm = state_machine_from_lifecycle(lifecycle);
            let yaml = yaml_serde::to_string(&vec![sm])?;
            write_file(&extensions_dir.join("statemachines.yaml"), &yaml).await?;
            count += 1;
            info!(
                "Generated statemachines.yaml: {} states, {} transitions",
                lifecycle.phases.len(),
                lifecycle.transitions.len()
            );
            if let Some(cb) = on_progress {
                let file_name = "statemachines.yaml";
                let rel_path = format!("{}/extensions/{}", app_name, file_name);
                cb(AgentProgress::new(
                    "构建应用",
                    82,
                    format!("已写入扩展 {}", file_name),
                    progress_event::ARTIFACT_WRITTEN,
                    Some(json!({"path": rel_path, "kind": "extension"})),
                ));
            }
        }
    }

    // workflows.yaml
    if !plan.workflow_steps.is_empty() {
        let workflows = workflow_from_steps(&plan.workflow_steps);
        let yaml = yaml_serde::to_string(&workflows)?;
        write_file(&extensions_dir.join("workflows.yaml"), &yaml).await?;
        count += 1;
        info!("Generated workflows.yaml: {} workflows", workflows.len());

        if let Some(cb) = on_progress {
            let file_name = "workflows.yaml";
            let rel_path = format!("{}/extensions/{}", app_name, file_name);
            cb(AgentProgress::new(
                "构建应用",
                82,
                format!("已写入扩展 {}", file_name),
                progress_event::ARTIFACT_WRITTEN,
                Some(json!({"path": rel_path, "kind": "extension"})),
            ));
        }
    }

    Ok(count)
}

/// 将未实现实体和扩展的缺口文档写入 request-no-impl/*.md
async fn write_gap_docs_to_staging(
    plan: &FlowPlan,
    stage_dir: &Path,
    ontology: Option<&alioth_gen::generator::ir::ontology::OntologyModel>,
) -> Result<usize, ComposerError> {
    let gaps_dir = stage_dir.join("request-no-impl");
    tokio::fs::create_dir_all(&gaps_dir).await?;
    let mut count = 0usize;

    if let Some(ontology) = ontology {
        for domain in &ontology.domains {
            let is_known = plan.known_entities.iter().any(|e| {
                domain.parent_ids.contains(e)
                    || domain.id == e.replace("zc_id_", "")
                    || domain.id == e.replace("zc_ad_", "")
            });
            if !is_known && !plan.used_modules.is_empty() {
                let is_new_entity = matches!(
                    domain.kind,
                    alioth_gen::generator::ir::ontology::DomainKind::Entity
                        | alioth_gen::generator::ir::ontology::DomainKind::AggregateRoot
                );
                if is_new_entity {
                    let gap_md = format_gap_doc(domain);
                    write_file(&gaps_dir.join(format!("gap-{}.md", domain.id)), &gap_md).await?;
                    count += 1;
                }
            }
        }
    }

    // FlowPlan.extensions 已删除(死接口,被 ontology.domains + extension_gaps 取代)
    // gap 文档现在仅从 ontology.domains 生成(见上方循环)

    if count > 0 {
        info!("Generated {} gap documents in request-no-impl/", count);
    }

    Ok(count)
}

/// 将 OntologyTransfer 产出的 mapped_entities 写入 namespace 级 service.json。
///
/// 目标路径：`Pre-Proc/{ns}/Sources/Services/{service_id}/service.json`。
/// Service 是 namespace 级共享产物（非 app 暂存区），直接写入——与
/// `create_service_scaffold` 同一先例。已存在时按实体名合并去重。
///
/// 坐标纪律：scene/factor 待层2 适配确认，实体暂不写 coordinates 块；
/// collector 容忍缺失（Option + WARN），后续 G3.5 确认后回填。
/// SESSION-FIX:gap-a-graph-projection — 从 AlignmentGraph 投影 service.json 实体/关系/gap 报告。
/// 纯函数：不依赖 IO/DB，可单测。
pub struct ServiceProjection {
    pub entities: Vec<serde_json::Value>,
    pub relations: Vec<serde_json::Value>,
    pub gap_report: Vec<String>,
}

pub fn project_from_alignment_graph(
    graph: &crate::state::AlignmentGraph,
    mapped: &[crate::state::MappedEntity],
) -> ServiceProjection {
    let mapped_by_domain: std::collections::HashMap<&str, &crate::state::MappedEntity> =
        mapped.iter().map(|m| (m.domain_id.as_str(), m)).collect();

    let mut entities: Vec<serde_json::Value> = Vec::new();
    for node in &graph.nodes {
        // SESSION-FIX:multi-binding — 展开 [1..] 产生副实体（含约束条件）
        for (idx, binding) in node.alioth_entities.iter().enumerate() {
            let table = if binding.table.starts_with("isahl.") {
                binding.table.clone()
            } else {
                format!("isahl.{}", binding.table)
            };
            if idx == 0 {
                // 优先取 MappedEntity 的 field_mappings（discovery 产物，含列映射）；
                // covered 节点自身 field_mappings 为空时退化为仅 name/table。
                let field_mappings: Vec<serde_json::Value> =
                    match mapped_by_domain.get(node.biz_domain.as_str()) {
                        Some(m) => m
                            .field_mappings
                            .iter()
                            .filter(|f| f.column.is_some())
                            .map(|f| {
                                let mut fm = serde_json::json!({
                                    "json_path": f.json_path,
                                    "column": f.column,
                                });
                                if let Some(s) = &f.scalar_table {
                                    fm["scalar"] = serde_json::json!(s);
                                }
                                fm
                            })
                            .collect(),
                        None => binding
                            .field_mappings
                            .iter()
                            .filter(|f| f.column.is_some())
                            .map(|f| {
                                serde_json::json!({
                                    "json_path": f.json_path,
                                    "column": f.column,
                                })
                            })
                            .collect(),
                    };
                entities.push(serde_json::json!({
                    "name": node.biz_domain,
                    "table": table,
                    "role": binding.role,
                    "confidence": node.confidence,
                    "field_mappings": field_mappings,
                }));
            } else {
                // SESSION-FIX:multi-binding-secure-naming — 副实体用表后缀命名，避免 idx 冲突
                let tbl_suffix = binding
                    .table
                    .strip_prefix("isahl.")
                    .unwrap_or(&binding.table)
                    .strip_prefix("zc_id_")
                    .unwrap_or(&binding.table);
                let sec_name = format!("{}__{}", node.biz_domain, tbl_suffix);
                let mut ent = serde_json::json!({
                    "name": sec_name,
                    "table": table,
                    "role": binding.role,
                    "field_mappings": serde_json::json!([]),
                });
                if !binding.constraints.is_empty() {
                    ent["constraints"] = serde_json::json!(binding.constraints);
                }
                entities.push(ent);
            }
        }
    }

    let relations: Vec<serde_json::Value> = graph
        .edges
        .iter()
        .map(|e| {
            let mechanism = match &e.alioth_mechanism {
                crate::state::AliothRelationMechanism::FK { column, target_table } => {
                    serde_json::json!({"kind": "fk", "column": column, "target_table": target_table})
                }
                crate::state::AliothRelationMechanism::RTable { table } => {
                    serde_json::json!({"kind": "r_table", "table": table})
                }
crate::state::AliothRelationMechanism::RRTable { table } => {
                    serde_json::json!({"kind": "rr_table", "table": table})
}
            };
            let evidence_table = e.evidence.as_ref().map(|ev| ev.relation_table.clone());
            serde_json::json!({
                "biz_rel": e.biz_rel_id,
                "biz_type": e.biz_rel_type,
                "mechanism": mechanism,
                "evidence_table": evidence_table,
            })
        })
        .collect();

    let gap_report: Vec<String> = graph
        .gaps
        .iter()
        .map(|g| format!("{}: {}", g.biz_element, g.description))
        .collect();

    ServiceProjection {
        entities,
        relations,
        gap_report,
    }
}

pub async fn write_mapped_services(
    namespace: &str,
    service_id: &str,
    app_domain: &str,
    mapped: &[crate::state::MappedEntity],
    graph: Option<&crate::state::AlignmentGraph>,
) -> Result<usize, ComposerError> {
    // SESSION-FIX:gap-a-graph-consume — graph 存在时即使有覆盖节点也不应早退
    if mapped.is_empty() && graph.is_none_or(|g| g.nodes.is_empty()) {
        return Ok(0);
    }
    let dir = resolve_project_root()
        .join("Pre-Proc")
        .join(namespace)
        .join("Sources")
        .join("Services")
        .join(service_id);
    tokio::fs::create_dir_all(&dir).await?;
    let path = dir.join("service.json");

    // SESSION-FIX:gap-a-graph-consume — graph 存在时从 AlignmentGraph 投影实体（含覆盖节点）
    let projection = graph.map(|g| project_from_alignment_graph(g, mapped));
    // 构造新实体 JSON
    let new_entities: Vec<serde_json::Value> = match &projection {
        Some(p) => p.entities.clone(),
        None => mapped
            .iter()
            .map(|m| {
                let table = if m.table.starts_with("isahl.") {
                    m.table.clone()
                } else {
                    format!("isahl.{}", m.table)
                };
                let field_mappings: Vec<serde_json::Value> = m
                    .field_mappings
                    .iter()
                    .filter(|f| f.column.is_some())
                    .map(|f| {
                        let mut fm = json!({
                            "json_path": f.json_path,
                            "column": f.column,
                        });
                        if let Some(s) = &f.scalar_table {
                            fm["scalar"] = json!(s);
                        }
                        fm
                    })
                    .collect();
                json!({
                    "name": m.domain_id,
                    "table": table,
                    "field_mappings": field_mappings,
                })
            })
            .collect(),
    };

    // 已存在 → 合并；不存在 → 新建
    let mut doc: serde_json::Value = match tokio::fs::read_to_string(&path).await {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| json!({})),
        Err(_) => json!({}),
    };
    if doc.get("id").is_none() {
        doc = json!({
            "id": service_id,
            "namespace": namespace,
            "domain": app_domain,
            "layer": 0,
            "dtoDependencies": [],
            "dtoExposes": {
                "refs": [],
                "queries": ["list_refs", "get_refs"],
            },
            "backendCrate": format!("{}-service-{}", namespace.to_lowercase(), service_id),
            "hasBackend": true,
            "hasFrontend": false,
            "version": "0.1.0",
            "publishes": [],
            "subscribes": [],
            "services": [],
            "aliothVersion": "10.0.0",
        });
    }

    // 合并实体：按 name 去重（新实体覆盖同名旧实体）
    let ont = doc
        .as_object_mut()
        .unwrap()
        .entry("ontology")
        .or_insert_with(|| json!({"entities": []}));
    let entities_arr = ont
        .as_object_mut()
        .unwrap()
        .entry("entities")
        .or_insert_with(|| json!([]));
    let arr = entities_arr.as_array_mut().unwrap();
    for ne in new_entities {
        let name = ne["name"].as_str().unwrap_or_default().to_string();
        if let Some(pos) = arr.iter().position(|e| e["name"].as_str() == Some(&name)) {
            arr[pos] = ne;
        } else {
            arr.push(ne);
        }
    }
    // GAP-6: 关系映射 — 从 fk_* 字段推断 belongsTo
    let entity_names: Vec<String> = arr
        .iter()
        .map(|e| e["name"].as_str().unwrap_or("").to_string())
        .collect();
    for i in 0..arr.len() {
        let fms: Vec<serde_json::Value> = arr[i]
            .get("field_mappings")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let mut rels: Vec<serde_json::Value> = Vec::new();
        for fm in &fms {
            let Some(col) = fm.get("column").and_then(|v| v.as_str()) else {
                continue;
            };
            if !col.starts_with("fk_") {
                continue;
            }
            let target_suffix = col.strip_prefix("fk_").unwrap_or("").to_lowercase();
            let my_name = arr[i]["name"].as_str().unwrap_or("").to_lowercase();
            for other in &entity_names {
                let other_lower = other.to_lowercase();
                if other_lower.contains(&target_suffix) && other_lower != my_name {
                    rels.push(json!({
                        "target": other.clone(),
                        "type": "belongsTo",
                        "via": col,
                    }));
                }
            }
        }
        if !rels.is_empty() {
            if let Some(obj) = arr[i].as_object_mut() {
                obj.insert("relationships".into(), json!(rels));
            }
        }
    }
    let total = arr.len();

    // 同步 dtoExposes.refs
    let names: Vec<String> = arr
        .iter()
        .filter_map(|e| e["name"].as_str().map(|s| s.to_string()))
        .collect();
    if let Some(exposes) = doc.get_mut("dtoExposes").and_then(|v| v.as_object_mut()) {
        exposes.insert("refs".into(), json!(names));
    }

    // SESSION-FIX:gap-a-graph-consume — relations 与 gap 报告落盘
    if let Some(p) = &projection {
        if !p.relations.is_empty() {
            doc.as_object_mut()
                .unwrap()
                .entry("ontology")
                .or_insert_with(|| json!({}))
                .as_object_mut()
                .unwrap()
                .insert("relations".into(), json!(p.relations));
        }
        if !p.gap_report.is_empty() {
            doc.as_object_mut()
                .unwrap()
                .insert("pendingConfirmations".into(), json!(p.gap_report));
        }
    }

    let content = serde_json::to_string_pretty(&doc)?;
    tokio::fs::write(&path, format!("{}\n", content)).await?;
    info!(
        "write_mapped_services: {} entities written to {}",
        total,
        path.display()
    );
    Ok(total)
}

/// 事务性应用组装：先写到 staging 目录，全部成功后原子 rename。
pub async fn compose_from_flow_plan<F>(
    _pool: &PgPool,
    plan: &FlowPlan,
    app_name: &str,
    namespace: &str,
    ontology: Option<&alioth_gen::generator::ir::ontology::OntologyModel>,
    on_progress: Option<&F>,
) -> Result<ComposeResult, ComposerError>
where
    F: Fn(AgentProgress) + Send + Sync,
{
    info!(
        "Assembling app '{}' (namespace: {}) from flow plan: {} modules, {} constraints, {} rules, {} computations",
        app_name,
        namespace,
        plan.used_modules.len(),
        plan.constraints.len(),
        plan.business_rules.len(),
        plan.computations.len()
    );

    let preproc_root = resolve_project_root().join("Pre-Proc");
    let apps_root = preproc_root.join(namespace).join("Apps");

    // ── 1. 创建 staging 目录 ───────────────────────────────────────────────
    let stage_suffix = uuid::Uuid::new_v4().to_string();
    let stage_dir = apps_root.join(format!(".{}.{}", app_name, &stage_suffix[..8]));
    tokio::fs::create_dir_all(&stage_dir).await?;
    info!("Staging app '{}' in {}", app_name, stage_dir.display());

    // 用于在错误时清理 staging
    let final_dir = apps_root.join(app_name);

    let mut files_written = 0usize;
    let mut cleanup_on_error = true;

    // 闭包：失败时清理 staging 目录
    let result: Result<ComposeResult, ComposerError> = async {
        // ── 2. 扫描模块元数据 → model_registry + versions ──────────────
        // model_registry 写入 extensions/profiles.yaml(对齐 Gateway ProfilesWrapper 契约,
        // 因 app.schema.json 的 config additionalProperties: false 禁止 modelRegistry/moduleVersions)
        let (model_registry, module_versions) = scan_module_registry(plan).await?;

        // ── 2.1 从 FlowPlan.app_meta 读取 LLM 输出的 App 级配置 ────────
        let app_meta = plan.app_meta.as_ref();
        let first_module = plan.used_modules.first().cloned().unwrap_or_default();
        let app_name_display = app_meta
            .and_then(|m| m.name.clone())
            .unwrap_or_else(|| app_name.to_string());
        let description = app_meta
            .and_then(|m| m.description.clone())
            .unwrap_or_else(|| format!("Auto-assembled app: {}", app_name));

        // 默认 navigation: 单组"系统管理"含全部 modules
        let default_navigation = if plan.used_modules.is_empty() {
            None
        } else {
            Some(vec![NavGroupJson {
                group: "系统管理".to_string(),
                icon: Some("Settings".to_string()),
                modules: plan.used_modules.clone(),
            }])
        };

        let app_config = AppJson {
            namespace: namespace.to_string(),
            id: crc64_i64(app_name),
            code: app_name.to_string(),
            name: app_name_display,
            description: Some(description),
            version: "0.1.0".to_string(),
            status: "developing".to_string(),
            environment: Some(
                app_meta
                    .and_then(|m| m.environment.clone())
                    .unwrap_or_else(|| "development".to_string()),
            ),
            deployment_mode: app_meta.and_then(|m| m.deployment_mode.clone()),
            endpoint_url: None,
            app_config: AppConfigJson {
                modules: plan.used_modules.clone(),
                blocks: if plan.created_blocks.is_empty() {
                    None
                } else {
                    Some(plan.created_blocks.clone())
                },
            },
            permissions: Some(
                app_meta
                    .and_then(|m| m.permissions.as_ref())
                    .map(|p| PermissionsJson {
                        default_roles: p.default_roles.clone(),
                        public_paths: p.public_paths.clone(),
                        admin_roles: p.admin_roles.clone(),
                    })
                    .unwrap_or_else(|| PermissionsJson {
                        default_roles: vec!["admin".to_string(), "user".to_string()],
                        public_paths: vec!["/login".to_string()],
                        admin_roles: vec!["admin".to_string()],
                    }),
            ),
            routing: Some(
                app_meta
                    .and_then(|m| m.routing.as_ref())
                    .map(|r| RoutingJson {
                        base: r.base.clone(),
                        default_route: r.default_route.clone(),
                    })
                    .unwrap_or_else(|| RoutingJson {
                        base: format!("/apps/{}", app_name),
                        default_route: if first_module.is_empty() {
                            "/".to_string()
                        } else {
                            format!("/{}", first_module)
                        },
                    }),
            ),
            brand: Some(
                app_meta
                    .and_then(|m| m.brand.as_ref())
                    .map(|b| BrandJson {
                        primary: b.primary.clone(),
                        logo: b.logo.clone(),
                    })
                    .unwrap_or_else(|| BrandJson {
                        primary: Some("262 70% 55%".to_string()),
                        logo: None,
                    }),
            ),
            navigation: app_meta
                .and_then(|m| m.navigation.as_ref())
                .map(|groups| {
                    groups
                        .iter()
                        .map(|g| NavGroupJson {
                            group: g.group.clone(),
                            icon: g.icon.clone(),
                            modules: g.modules.clone(),
                        })
                        .collect()
                })
                .or(default_navigation),
            goal: app_meta.and_then(|m| m.goal.clone()),
            non_scope: app_meta.and_then(|m| m.non_scope.clone()),
            min_alioth_version: Some(alioth_gen::ALIOTH_MODEL_VERSION.to_string()),
        };
        let app_json = serde_json::to_string_pretty(&app_config)?;
        write_file(&stage_dir.join("app.json"), &app_json).await?;
        files_written += 1;

        // ── 2.2 模型档案改为 extensions/profiles.yaml(对齐 Gateway ProfilesWrapper 契约) ──
        // 原独立 model-registry.json 已废弃(见 META_AI_SPEC §9.5 / APP_EXTENSION.md)。
        // profiles.yaml 实际写入在步骤 3 之后(复用已创建的 extensions/ 目录)。
        let _ = module_versions; // 暂不写入(原 app.json.moduleVersions 已移除)
        if let Some(cb) = on_progress {
            let rel_path = format!("{}/Apps/{}/app.json", namespace, app_name);
            cb(AgentProgress::new(
                "构建应用",
                80,
                format!("已写入 {}", rel_path),
                progress_event::ARTIFACT_WRITTEN,
                Some(json!({"path": rel_path, "kind": "app_manifest"})),
            ));
        }

        // ── 3. extensions/*.yaml ──────────────────────────────────────────
        let ext_count =
            write_extensions_to_staging(plan, app_name, &stage_dir, ontology, on_progress).await?;
        files_written += ext_count;

        // ── 3.5 写入 extensions/profiles.yaml(领域模型档案) ────────────────
        // 对齐 Gateway `runtime-engine::extension::load_from_dir` 的 `ProfilesWrapper` 契约：
        //   profiles:
        //     default:
        //       modules:
        //         <module_code>: { enabled_entities: [...], disabled_entities: [...] }
        // 仅当存在启用实体时写入，避免产生空档案。
        if !model_registry.modules.is_empty() {
            let mut profiles = std::collections::HashMap::new();
            profiles.insert("default".to_string(), model_registry);
            let wrapper = ProfilesWrapper { profiles };
            let yaml = yaml_serde::to_string(&wrapper)?;
            write_file(&stage_dir.join("extensions").join("profiles.yaml"), &yaml).await?;
            files_written += 1;
            let module_count = wrapper
                .profiles
                .get("default")
                .map(|p| p.modules.len())
                .unwrap_or(0);
            info!("Generated extensions/profiles.yaml: {} modules", module_count);
        }

        // ── 4. request-no-impl/*.md ───────────────────────────────────────
        let gap_count = write_gap_docs_to_staging(plan, &stage_dir, ontology).await?;
        files_written += gap_count;

        // ── 4.5 ESM 原型(llm-tsx/app.tsx → prototype-tool.js build) ────
        // 替换旧的 CDN babel write_app_prototype(已删除)
        // sync-prototype.sh 在 build_app 阶段调用(需 final_dir 已存在)
        generate_and_build_app_tsx(
            &stage_dir,
            app_name,
            app_name,
            namespace,
            plan,
            plan.app_meta.as_ref(),
            &mut files_written,
            on_progress,
        )
        .await?;
        // ── 5. 原子 rename staging → final ─────────────────────────────────
        // 确保 namespace 父目录存在（rename 要求 parent dir 就绪）
        // 确保 Apps 父目录存在（rename 要求 parent dir 就绪）
        let apps_dir = apps_root.parent(); // will be Some—we just built it as {ns}/Apps
        if let Some(parent) = apps_dir {
            if !parent.exists() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }
        if !apps_root.exists() {
            tokio::fs::create_dir_all(&apps_root).await?;
        }
        // 先提交文件系统，确保目录存在（不再写 apps.json 中央聚合清单）
        if final_dir.exists() {
            let backup_dir = apps_root.join(format!(".{}.bak", app_name));
            // 清理已存在的旧 backup 目录（避免 rename 失败 "Directory not empty"）
            if backup_dir.exists() {
                tokio::fs::remove_dir_all(&backup_dir).await?;
            }
            tokio::fs::rename(&final_dir, &backup_dir).await?;
            info!(
                "Previous version of '{}' backed up to {}",
                app_name,
                backup_dir.display()
            );
        }

        tokio::fs::rename(&stage_dir, &final_dir).await?;
        cleanup_on_error = false;

        info!(
            "App '{}' assembled: {} files written to {}",
            app_name,
            files_written,
            final_dir.display()
        );

        // 不再写 apps.json 中央聚合清单 — App 发现由 Gateway FS 扫描各 namespace 自发现
        // 参考 Gateway/backend/src/preproc/discovery.rs + Gateway/frontend/vite.config.ts

        Ok(ComposeResult {
            app_name: app_name.to_string(),
            output_path: final_dir.to_string_lossy().to_string(),
            files_written,
            module_count: plan.used_modules.len(),
        })
    }
    .await;

    // ── 清理 staging 目录（仅在出错时）─────────────────────────────────────
    if result.is_err() && cleanup_on_error && stage_dir.exists() {
        if let Err(e) = tokio::fs::remove_dir_all(&stage_dir).await {
            common::telemetry::error!(
                "Failed to clean up staging directory {}: {}",
                stage_dir.display(),
                e
            );
        } else {
            common::telemetry::info!("Cleaned up staging directory {}", stage_dir.display());
        }
    }

    result
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal types & helpers
// ─────────────────────────────────────────────────────────────────────────────

/// app.json 结构(对齐 Pre-Proc/Alioth/_schema/app.schema.json 17 字段)
///
/// schema 真相源: `Pre-Proc/Alioth/_schema/app.schema.json`
/// - required: id, code, namespace, name, version, status
/// - config additionalProperties: false(禁止 modelRegistry/moduleVersions,已迁移到 extensions/profiles.yaml)
/// - deploymentMode enum: [null, "standalone", "embedded"](严禁 "single_process")
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppJson {
    /// 应用 id（CRC64(code)）。统一序列化为字符串，避免 JS 大整数精度丢失，
    /// 并与 app-instance / Gateway AppInfo 的 id wire 格式一致。
    #[serde(with = "meta_common::serde_zuid")]
    pub id: i64,
    pub code: String,
    /// namespace 隔离域（如 AVIC-CAASEC、WZ），与目录路径同步
    pub namespace: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 必填(schema required)
    pub version: String,
    /// 必填(schema required): developing/active/deprecated/archived
    pub status: String,
    /// [development, staging, production]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    /// [null, "standalone", "embedded"](严禁 "single_process")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_url: Option<String>,
    #[serde(rename = "config")]
    pub app_config: AppConfigJson,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<PermissionsJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing: Option<RoutingJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand: Option<BrandJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub navigation: Option<Vec<NavGroupJson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    /// schema 字段名 non_scope(snake_case,非 camelCase)
    #[serde(rename = "non_scope", skip_serializing_if = "Option::is_none")]
    pub non_scope: Option<Vec<String>>,
    /// schema 字段名 min_alioth_version(snake_case,非 camelCase)
    #[serde(rename = "min_alioth_version", skip_serializing_if = "Option::is_none")]
    pub min_alioth_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppConfigJson {
    /// 模块组合：Gateway 运行时按此列表挂载模块路由
    pub modules: Vec<String>,
    /// 按需引入的独立 block id(不隶属任何 module)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks: Option<Vec<String>>,
    // ❌ modelRegistry/moduleVersions 已移除(schema additionalProperties: false)
    // model_registry 现写入 extensions/profiles.yaml(对齐 Gateway ProfilesWrapper 契约)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PermissionsJson {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_roles: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub public_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub admin_roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RoutingJson {
    /// pattern ^/
    pub base: String,
    /// pattern ^/
    pub default_route: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BrandJson {
    /// HSL "H S% L%"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NavGroupJson {
    pub group: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub modules: Vec<String>,
}

/// 计算 CRC64-ECMA 并返回 i64（与 AppInstanceRepository 一致，确保 code → id 稳定映射）
fn crc64_i64(data: &str) -> i64 {
    const POLY: u64 = 0x42F0E1EBA9EA3693;
    const INIT: u64 = 0x0000000000000000;
    const XOROUT: u64 = 0x0000000000000000;

    fn build_table() -> [u64; 256] {
        let mut table = [0u64; 256];
        for (i, slot) in table.iter_mut().enumerate() {
            let mut crc = i as u64;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ POLY;
                } else {
                    crc >>= 1;
                }
            }
            *slot = crc;
        }
        table
    }

    static TABLE: std::sync::OnceLock<[u64; 256]> = std::sync::OnceLock::new();
    let table = TABLE.get_or_init(build_table);

    let mut crc = INIT;
    for byte in data.bytes() {
        crc = table[((crc ^ byte as u64) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^= XOROUT;
    crc as i64
}

fn state_machine_from_lifecycle(
    lifecycle: &alioth_gen::generator::ir::ontology::TransactionLifecycle,
) -> StateMachineExtension {
    let states: Vec<State> = lifecycle
        .phases
        .iter()
        .map(|p| State::new(&p.name).with_description(p.id.clone()))
        .collect();

    let transitions: Vec<Transition> = lifecycle
        .transitions
        .iter()
        .map(|t| {
            let from = lifecycle
                .phases
                .iter()
                .find(|p| p.id == t.from_phase)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| t.from_phase.clone());
            let to = lifecycle
                .phases
                .iter()
                .find(|p| p.id == t.to_phase)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| t.to_phase.clone());

            Transition::new(&t.trigger_event, &from, &to)
                .with_guard(t.guard_conditions.join(" AND "))
        })
        .collect();

    let initial_state = lifecycle
        .phases
        .first()
        .map(|p| p.name.clone())
        .unwrap_or_default();

    StateMachineExtension {
        entity: lifecycle.name.clone(),
        state_field: "t_state".to_string(),
        states,
        transitions,
        initial_state,
    }
}

fn workflow_from_steps(steps: &[String]) -> Vec<WorkflowDefinition> {
    let wf_steps: Vec<WorkflowStep> = steps
        .iter()
        .enumerate()
        .map(|(i, step)| WorkflowStep {
            name: format!("step_{}", i + 1),
            action: WorkflowAction::CallProcedure {
                name: step.clone(),
                params: vec![],
            },
            condition: None,
            on_error: WorkflowErrorHandling::Abort,
        })
        .collect();

    vec![WorkflowDefinition {
        name: "auto_workflow".to_string(),
        description: Some("Auto-generated workflow from LLM steps".to_string()),
        trigger: WorkflowTrigger {
            entity: "*".to_string(),
            event: LifecycleEvent::OnCreate,
            condition: None,
        },
        steps: wf_steps,
    }]
}

fn format_gap_doc(domain: &alioth_gen::generator::ir::ontology::DomainOntology) -> String {
    let properties: String = domain
        .properties
        .iter()
        .map(|p| {
            format!(
                "- {} ({}): {} {}{}",
                p.name,
                p.id,
                p.range,
                if p.required { "[required]" } else { "" },
                p.semantic_description
                    .as_ref()
                    .map(|d| format!(" — {}", d))
                    .unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"# Gap: {}

## 需求来源
- 本体领域: `{}` ({:?})
- 描述: {}

## 缺口分析
该实体未被现有模块覆盖，需要新建实现。

## 建议属性
{}

## 建议实现方向
1. 评估是否可归入现有模块（通过扩展点）
2. 如不可归入，需新建实体表并注册到对应模块
3. 更新 module.json 的 extensionPoints 声明新实体
"#,
        domain.name,
        domain.id,
        domain.kind,
        domain.description.as_deref().unwrap_or("无描述"),
        if properties.is_empty() {
            "_暂无属性定义_".to_string()
        } else {
            properties
        }
    )
}

async fn write_file(path: &Path, content: &str) -> Result<(), ComposerError> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(path, content).await?;
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Legacy compatibility: keep old types that orchestrator may reference
// ─────────────────────────────────────────────────────────────────────────────

/// 运行时引擎配置（旧格式，保留供向后兼容）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub app_name: String,
    pub generated_at: String,
    pub computations: Vec<ComputationPlan>,
    pub constraints: Vec<ConstraintPlan>,
    pub business_rules: Vec<BusinessRulePlan>,
}

/// 为 mapped_entities 生成 service backend Rust 代码。
/// 输出到 `Pre-Proc/{ns}/Sources/Services/{service_id}/backend/`。
pub async fn write_service_backend(
    namespace: &str,
    service_id: &str,
    mapped: &[crate::state::MappedEntity],
) -> Result<usize, ComposerError> {
    crate::service_gen::generate_service_backend(namespace, service_id, mapped)
        .await
        .map_err(|e| ComposerError::Validation(e))
}

fn push_model(out: &mut String, name: &str, table: &str) {
    use std::fmt::Write;
    let _ = writeln!(out, "#[derive(Debug, Clone, Serialize, Deserialize)]");
    let _ = writeln!(out, "pub struct {name} {{");
    let _ = writeln!(out, "    pub id: i64,");
    let _ = writeln!(out, "    pub notice: Option<String>,");
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);
    let _ = writeln!(out, "impl Identifiable for {name} {{");
    let _ = writeln!(out, "    fn id(&self) -> i64 {{ self.id }}");
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);
    let _ = writeln!(out, "impl AliothDbEntity for {name} {{");
    let _ = writeln!(
        out,
        "    fn table_name() -> &\'static str {{ r#\"{}\"# }}",
        table
    );
    let _ = writeln!(out, "    const SELECT_FIELDS: &\'static str = r#\"id, notice, created_at, updated_at, deleted_at\"#;");
    let _ = writeln!(out, "    const ENTITY_NAME: &\'static str = \"{name}\";");
    let _ = writeln!(out, "    const SOFT_DELETE: bool = true;");
    let _ = writeln!(out, "    const HAS_AUDIT: bool = false;");
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);
}

#[cfg(test)]
mod tests {
    use super::*;

    // SESSION-FIX:gap-a-projection-tests
    // SESSION-FIX:multi-binding-projection-test
    fn test_multi_binding_graph() -> crate::state::AlignmentGraph {
        crate::state::AlignmentGraph {
            nodes: vec![crate::state::AlignmentNode {
                biz_domain: "in_transit_inv".into(),
                biz_kind: "AggregateRoot".into(),
                alioth_entities: vec![
                    crate::state::AliothBinding {
                        table: "isahl.zc_id_inventory".into(),
                        role: "aggregate_root".into(),
                        coordinates: None,
                        field_mappings: vec![],
                        constraints: vec![],
                    },
                    crate::state::AliothBinding {
                        table: "isahl.zc_id_stus-inventory".into(),
                        role: "filter".into(),
                        coordinates: None,
                        field_mappings: vec![],
                        constraints: vec!["code = IN_TRANSIT".into()],
                    },
                ],
                evidence: "composite".into(),
                confidence: 0.75,
            }],
            edges: vec![],
            gaps: vec![],
        }
    }

    #[test]
    fn test_multi_binding_projections_expands_secondary() {
        let graph = test_multi_binding_graph();
        let p = project_from_alignment_graph(&graph, &[]);
        assert_eq!(p.entities.len(), 2);
        // 主实体
        let primary = &p.entities[0];
        assert_eq!(primary["name"], "in_transit_inv");
        assert_eq!(primary["table"], "isahl.zc_id_inventory");
        assert_eq!(primary["role"], "aggregate_root");
        // 副实体
        let secondary = &p.entities[1];
        assert!(secondary["name"].as_str().unwrap().contains("__"));
        assert!(secondary["name"]
            .as_str()
            .unwrap()
            .contains("stus-inventory"));
        assert_eq!(secondary["table"], "isahl.zc_id_stus-inventory");
        assert_eq!(secondary["constraints"].as_array().unwrap().len(), 1);
        assert_eq!(secondary["constraints"][0], "code = IN_TRANSIT");
    }

    fn test_graph() -> crate::state::AlignmentGraph {
        crate::state::AlignmentGraph {
            nodes: vec![
                crate::state::AlignmentNode {
                    biz_domain: "mapped_dom".into(),
                    biz_kind: "Entity".into(),
                    alioth_entities: vec![crate::state::AliothBinding {
                        table: "isahl.zc_id_mapped".into(),
                        role: "entity".into(),
                        coordinates: None,
                        field_mappings: vec![],
                        constraints: vec![],
                    }],
                    evidence: "discovery".into(),
                    confidence: 0.8,
                },
                crate::state::AlignmentNode {
                    biz_domain: "covered_dom".into(),
                    biz_kind: "AggregateRoot".into(),
                    alioth_entities: vec![crate::state::AliothBinding {
                        table: "isahl.zc_id_covered".into(),
                        role: "aggregate_root".into(),
                        coordinates: None,
                        field_mappings: vec![],
                        constraints: vec![],
                    }],
                    evidence: "covered-binding".into(),
                    confidence: 0.7,
                },
            ],
            edges: vec![crate::state::AlignmentEdge {
                biz_rel_id: "rel1".into(),
                biz_rel_type: "Composition".into(),
                alioth_mechanism: crate::state::AliothRelationMechanism::FK {
                    column: "fk_parent".into(),
                    target_table: "isahl.zc_id_covered".into(),
                },
                evidence: None,
            }],
            gaps: vec![crate::state::AlignmentGap {
                biz_element: "unmapped_dom".into(),
                description: "未匹配".into(),
                suggested_alioth_entities: vec![],
            }],
        }
    }

    #[test]
    fn test_projection_entities_include_covered_nodes() {
        let graph = test_graph();
        let mapped = vec![crate::state::MappedEntity {
            domain_id: "mapped_dom".into(),
            table: "isahl.zc_id_mapped".into(),
            score: 0.8,
            name_score: 0.6,
            field_score: 0.4,
            scene_code: None,
            factor_code: None,
            function_code: None,
            function_confidence: 0.0,
            field_mappings: vec![crate::state::MappedField {
                json_path: "name".into(),
                column: Some("notice".into()),
                scalar_table: None,
                tier: "direct".into(),
            }],
        }];
        let p = project_from_alignment_graph(&graph, &mapped);
        // 2 节点都投影（mapped + covered）
        assert_eq!(p.entities.len(), 2);
        // mapped 节点带 field_mappings
        let mapped_e = p
            .entities
            .iter()
            .find(|e| e["name"] == "mapped_dom")
            .unwrap();
        assert_eq!(mapped_e["field_mappings"].as_array().unwrap().len(), 1);
        // covered 节点也落盘（name/table 完备）
        let covered_e = p
            .entities
            .iter()
            .find(|e| e["name"] == "covered_dom")
            .unwrap();
        assert_eq!(covered_e["table"], "isahl.zc_id_covered");
        assert_eq!(covered_e["role"], "aggregate_root");
    }

    #[test]
    fn test_projection_relations_and_gaps() {
        let graph = test_graph();
        let p = project_from_alignment_graph(&graph, &[]);
        assert_eq!(p.relations.len(), 1);
        assert_eq!(p.relations[0]["mechanism"]["kind"], "fk");
        assert_eq!(p.relations[0]["mechanism"]["column"], "fk_parent");
        assert_eq!(p.gap_report.len(), 1);
        assert!(p.gap_report[0].contains("unmapped_dom"));
    }

    /// 构造测试用 AppJson(含全部 17 字段)
    fn test_app_json() -> AppJson {
        AppJson {
            id: 12345,
            code: "ai-test".to_string(),
            namespace: "Alioth".to_string(),
            name: "TestApp".to_string(),
            description: Some("test".to_string()),
            version: "0.1.0".to_string(),
            status: "developing".to_string(),
            environment: Some("development".to_string()),
            deployment_mode: None,
            endpoint_url: None,
            app_config: AppConfigJson {
                modules: vec!["system-settings".to_string()],
                blocks: None,
            },
            permissions: Some(PermissionsJson {
                default_roles: vec!["admin".to_string()],
                public_paths: vec!["/login".to_string()],
                admin_roles: vec!["admin".to_string()],
            }),
            routing: Some(RoutingJson {
                base: "/apps/ai-test".to_string(),
                default_route: "/system-settings".to_string(),
            }),
            brand: Some(BrandJson {
                primary: Some("262 70% 55%".to_string()),
                logo: None,
            }),
            navigation: Some(vec![NavGroupJson {
                group: "系统管理".to_string(),
                icon: Some("Settings".to_string()),
                modules: vec!["system-settings".to_string()],
            }]),
            goal: Some("test goal".to_string()),
            non_scope: Some(vec!["excluded".to_string()]),
            min_alioth_version: Some("10.0.0".to_string()),
        }
    }

    #[test]
    fn test_app_json_serializes_all_17_fields() {
        let app = test_app_json();
        let json = serde_json::to_string(&app).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        // required 字段
        assert!(v.get("id").is_some(), "id 必须存在");
        assert!(v.get("code").is_some(), "code 必须存在");
        assert!(v.get("namespace").is_some(), "namespace 必须存在");
        assert!(v.get("name").is_some(), "name 必须存在");
        assert!(v.get("version").is_some(), "version 必须存在");
        assert!(v.get("status").is_some(), "status 必须存在");

        // 可选字段
        assert!(v.get("environment").is_some(), "environment 必须存在");
        assert!(v.get("permissions").is_some(), "permissions 必须存在");
        assert!(v.get("routing").is_some(), "routing 必须存在");
        assert!(v.get("brand").is_some(), "brand 必须存在");
        assert!(v.get("navigation").is_some(), "navigation 必须存在");
        assert!(v.get("goal").is_some(), "goal 必须存在");
        assert!(
            v.get("min_alioth_version").is_some(),
            "min_alioth_version 必须存在"
        );

        // non_scope 必须 snake_case(非 camelCase)
        assert!(v.get("non_scope").is_some(), "non_scope 必须 snake_case");
        assert!(v.get("nonScope").is_none(), "nonScope 不应存在");

        // config 不含 modelRegistry/moduleVersions(已迁移)
        let cfg = v.get("config").unwrap();
        assert!(
            cfg.get("modelRegistry").is_none(),
            "config.modelRegistry 已移除"
        );
        assert!(
            cfg.get("moduleVersions").is_none(),
            "config.moduleVersions 已移除"
        );
    }

    #[test]
    fn test_app_json_deployment_mode_not_single_process() {
        let mut app = test_app_json();
        // deploymentMode 为 None(序列化为 null 或省略)
        let json = serde_json::to_string(&app).unwrap();
        assert!(
            !json.contains("single_process"),
            "deploymentMode 严禁 single_process"
        );

        // 设置为合法值
        app.deployment_mode = Some("embedded".to_string());
        let json = serde_json::to_string(&app).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["deploymentMode"], "embedded");
    }

    #[test]
    fn test_app_json_status_enum_valid() {
        let app = test_app_json();
        let json = serde_json::to_string(&app).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let status = v["status"].as_str().unwrap();
        assert!(
            ["developing", "active", "deprecated", "archived"].contains(&status),
            "status 必须在 enum 中,实际: {}",
            status
        );
    }

    #[test]
    fn test_app_json_camel_case_serialization() {
        let app = test_app_json();
        let json = serde_json::to_string(&app).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        // camelCase 字段
        assert_eq!(v["deploymentMode"], serde_json::Value::Null);
        assert_eq!(v["endpointUrl"], serde_json::Value::Null);
        // min_alioth_version 必须 snake_case(schema 要求)
        assert_eq!(v["min_alioth_version"], "10.0.0");

        // config.blocks 存在(可选,skip_serializing_if None)
        assert!(v["config"].get("blocks").is_none() || v["config"]["blocks"].is_null());

        // min_alioth_version 必须 snake_case(非 camelCase)
        assert!(
            v.get("min_alioth_version").is_some(),
            "min_alioth_version 必须 snake_case"
        );
        assert!(
            v.get("minAliothVersion").is_none(),
            "minAliothVersion 不应存在"
        );
    }

    #[tokio::test]
    async fn test_write_mapped_services_creates_and_merges() {
        let ns = "TestOntologyNS";
        let service_id = "inventory-service";
        let mapped = vec![crate::state::MappedEntity {
            domain_id: "inventory".into(),
            table: "zc_id_inventory".into(),
            score: 0.8,
            name_score: 1.0,
            field_score: 0.6,
            scene_code: None,
            factor_code: None,
            function_code: Some("↓_HH".into()),
            function_confidence: 0.75,
            field_mappings: vec![
                crate::state::MappedField {
                    json_path: "name".into(),
                    column: Some("notice".into()),
                    scalar_table: None,
                    tier: "safe".into(),
                },
                crate::state::MappedField {
                    json_path: "qty".into(),
                    column: Some("qk_qty".into()),
                    scalar_table: Some("zc_id_scal-common".into()),
                    tier: "safe".into(),
                },
                crate::state::MappedField {
                    json_path: "mystery".into(),
                    column: None,
                    scalar_table: None,
                    tier: "unclear".into(),
                },
            ],
        }];

        let written = write_mapped_services(ns, service_id, "inventory", &mapped, None)
            .await
            .unwrap();
        assert_eq!(written, 1);

        let path = resolve_project_root()
            .join("Pre-Proc")
            .join(ns)
            .join("Sources/Services")
            .join(service_id)
            .join("service.json");
        let doc: serde_json::Value =
            serde_json::from_str(&tokio::fs::read_to_string(&path).await.unwrap()).unwrap();
        let ents = doc["ontology"]["entities"].as_array().unwrap();
        assert_eq!(ents.len(), 1);
        assert_eq!(ents[0]["table"], "isahl.zc_id_inventory");
        // unclear 字段被过滤，scalar 保留
        let fms = ents[0]["field_mappings"].as_array().unwrap();
        assert_eq!(fms.len(), 2);
        assert_eq!(fms[1]["scalar"], "zc_id_scal-common");
        // 实体不写 coordinates（scene/factor 待层2 确认）
        assert!(ents[0].get("coordinates").is_none());

        // 幂等：再次写入合并去重，不翻倍
        write_mapped_services(ns, service_id, "inventory", &mapped, None)
            .await
            .unwrap();
        let doc2: serde_json::Value =
            serde_json::from_str(&tokio::fs::read_to_string(&path).await.unwrap()).unwrap();
        assert_eq!(doc2["ontology"]["entities"].as_array().unwrap().len(), 1);

        // 清理测试产物
        tokio::fs::remove_dir_all(resolve_project_root().join("Pre-Proc").join(ns))
            .await
            .unwrap();
    }
}
