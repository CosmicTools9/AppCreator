//! LLM 规划器 — 生成 OntologyModel prompt 并解析 LLM 响应
//!
//! 重构 (方案 A1+P1+V3):
//! - Planner 直接产出 OntologyModel（而非扁平 FlowPlan）
//! - System prompt 内嵌完整 JSON Schema（P1）
//! - 多层容错：json5 解析 → 枚举 case-insensitive 修正 → 语义自动修复（V3）

use crate::state::{FlowPlan, MissingInfo, MissingInfoCategory, PlatformCatalog};
use alioth_gen::generator::ir::ontology::{
    Cardinality, DomainKind, DomainOntology, OntologyMetadata, OntologyModel, PropertyType,
    RelationOntology, RelationType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── 输出类型 ──────────────────────────────────────────────────────────────

/// LLM 原始输出（反序列化目标）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OntologyOutput {
    pub ontology: OntologyModel,
    #[serde(default)]
    pub used_modules: Vec<String>,
    #[serde(default)]
    pub known_entities: Vec<String>,
    #[serde(default)]
    pub missing_info: Vec<MissingInfo>,
    #[serde(default)]
    pub workflow_steps: Vec<String>,
    /// LLM 输出的 App 级元数据(brand/navigation/routing/permissions 等)
    /// 驱动 composer.rs 生成符合 app.schema.json 的 17 字段 app.json
    #[serde(default)]
    pub app_meta: Option<crate::state::AppMeta>,
}

/// 经过验证和自动修复的本体模型
#[derive(Debug, Clone)]
pub struct ValidatedOntology {
    pub ontology: OntologyModel,
    pub used_modules: Vec<String>,
    pub known_entities: Vec<String>,
    pub missing_info: Vec<MissingInfo>,
    pub workflow_steps: Vec<String>,
    /// App 级元数据(从 LLM 输出透传)
    pub app_meta: Option<crate::state::AppMeta>,
    /// 自动修复记录
    pub fix_log: Vec<String>,
    /// 非阻塞警告
    pub warnings: Vec<String>,
}

// ─── PlanningPrompt ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningPrompt {
    pub system: String,
    pub user: String,
}

impl PlanningPrompt {
    /// Static prefix that rarely changes — cache-friendly order:
    /// most static first so DeepSeek prefix cache hits the longest possible span.
    fn static_system_prefix() -> &'static str {
        SYSTEM_PROMPT
    }
    /// Semi-static appendix — kept minimal to avoid overloading the LLM context.
    fn static_appendix() -> String {
        // Intentionally empty: the JSON schema + Few-shot example in SYSTEM_PROMPT
        // already provide sufficient structural guidance. The full ontology spec
        // and dev guide add thousands of tokens without improving JSON validity.
        String::new()
    }

    pub fn new(
        user_description: &str,
        catalog: &PlatformCatalog,
        existing_ontology: Option<&OntologyModel>,
        user_answers: &[String],
        ontology_context: Option<&serde_json::Value>,
        compiled_modules: &std::collections::HashSet<String>,
    ) -> Self {
        // 第二部分（APP_DEVELOPER_GUIDE + ALIOTH_ONTOLOGY_SPEC）相对较大，
        // 但变化频率低，下一次调用时若未改 APP_DEVELOPER_GUIDE 内容，
        // 字节级前缀只需从第二个部分开始首次 miss。
        // 第二部分（APP_DEVELOPER_GUIDE + ALIOTH_ONTOLOGY_SPEC）相对较大，
        // 但变化频率低，下一次调用时若未改 APP_DEVELOPER_GUIDE 内容，
        // 字节级前缀只需从第二个部分开始首次 miss。
        let system = format!(
            "{}\n{}",
            Self::static_system_prefix(),
            Self::static_appendix(),
        );

        // 注入已编译模块白名单：约束 LLM 只能选择 Gateway 已编译的模块
        let system = if !compiled_modules.is_empty() {
            let mut sorted: Vec<&String> = compiled_modules.iter().collect();
            sorted.sort();
            let mod_list = sorted
                .iter()
                .map(|s| format!("  - {}", s))
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "{}\n\n【已编译模块白名单】以下模块已在 Gateway 编译，请仅从这些模块中选择 used_modules：\n{}",
                system, mod_list
            )
        } else {
            system
        };

        let user = if let Some(ontology) = existing_ontology {
            if !ontology.domains.is_empty() {
                build_clarification_prompt(user_description, ontology, user_answers, catalog)
            } else {
                build_initial_prompt(user_description, catalog)
            }
        } else {
            build_initial_prompt(user_description, catalog)
        };

        // 注入本体上下文（仅实体摘要，不注入物理边 —— 符合 APP_LOGIC_EXTENSION.md §6.3）
        let user = if let Some(ctx) = ontology_context {
            let entities = &ctx["entities"];
            let entity_count = entities.as_array().map(|a| a.len()).unwrap_or(0);

            if entity_count > 0 {
                let filtered_ctx = serde_json::json!({
                    "entities": entities,
                });
                let ctx_text = format!(
                    "\n\n【平台本体上下文 — 与您的需求相关的已有实体摘要，共{}个实体。注意：以下仅列出实体列表，不包含实体间的物理边关系，请基于语义自行推断合理关联】\n{}\n【本体上下文结束】",
                    entity_count,
                    serde_json::to_string_pretty(&filtered_ctx).unwrap_or_default()
                );
                format!("{}{}", user, ctx_text)
            } else {
                user
            }
        } else {
            user
        };

        Self { system, user }
    }
}

// ─── Prompt 构建 ───────────────────────────────────────────────────────────

fn build_initial_prompt(user_description: &str, catalog: &PlatformCatalog) -> String {
    let modules = format_catalog_modules(catalog);
    let collections = format_catalog_collections(catalog);
    let scenes = format_catalog_scenes(catalog);
    let lifecycle_entities = catalog.lifecycle_entities.join("\n  - ");

    format!(
        r#"用户需求：
"{}"

请分析上述需求，输出一个完整的本体模型 JSON（不要包裹在 markdown 代码块中，直接输出 JSON）。

【平台能力参考】
已知 Module（模块元数据，集合信息已独立列出）：
{}

已知 Collection（平台实体集合，全局元数据，不专属于某一 Module）：
{}

已知 Scene（场景维度）：
{}

已知 Lifecycle 根类型：
{}

【模块复用策略 — 关键原则】
1. **优先复用已有模块**：平台已提供上述 Module，每个 Module 包含完整的 CRUD、状态管理和基础业务逻辑。
   - 若用户需求可被现有模块覆盖，请在 `used_modules` 中列出对应 module_id，**不要**为已有能力新建 domains。
   - 例如：用户需要"订单管理"，平台已有 `orders` 模块，则 `used_modules` 应包含 `"orders"`，而非新建 `Order` domain。
2. **仅对缺口创建新实体**：当现有模块确实无法覆盖用户需求时，才在 `domains` 中定义新实体。
   - 新实体必须继承合适的平台根类型（`zc_id_entity`、`zc_id_lifecycle` 等）。
3. **应用级逻辑扩展**：业务约束、规则、状态机流转等**不在**模块中重复实现，而是通过 `constraints` + `computations` + `transaction_lifecycle` 表达，由平台运行时注入到已复用模块。

【输出要求】
1. domains: 定义所有业务领域本体（**仅缺口部分**，已被模块覆盖的无需重复定义）
   - 新实体继承平台根类型时，parent_ids 填写对应的 zc_id_* 表名
   - kind 取值：Entity（业务实体）、ValueObject（值对象）、AggregateRoot（聚合根）、Enumeration（枚举）
2. relations: 定义领域间的关系
   - Composition: 聚合根与其组成部分（如订单→订单明细）
   - Association: 跨聚合引用（如订单明细→物料）
   - Inheritance: 继承关系（子类型→父类型）
3. constraints: 定义业务不变量（约束表达式，运行时注入）
4. computations: 定义自动计算逻辑（公式、触发时机）
5. transaction_lifecycle: 如果有业务流程，定义阶段和转换
6. used_modules: **必须**列出所有可复用的平台 module_id
7. known_entities: 引用的平台已有实体表名（zc_id_*）
8. missing_info: 需要用户进一步确认的信息，必须按「场景条件→决策要素→判断标准→判断结果」四段式输出
9. workflow_steps: 推断的业务流程步骤

注意事项：
- 命名：领域 id 使用 snake_case，领域 name 使用中文
- used_modules 必须是平台已存在的 module_id
- known_entities 必须是 zc_id_* 表名
- 如果用户需求描述了一个聚合关系（如"订单包含明细"），必须用 Composition 关系表达
- 交易生命周期是可选的，仅当用户描述包含审批流/状态流转时才定义"#,
        user_description, modules, collections, scenes, lifecycle_entities
    )
}

fn build_clarification_prompt(
    user_description: &str,
    ontology: &OntologyModel,
    user_answers: &[String],
    _catalog: &PlatformCatalog,
) -> String {
    let domain_summary: Vec<String> = ontology
        .domains
        .iter()
        .map(|d| {
            format!(
                "  - {} ({}) [{}]",
                d.name,
                d.id,
                format!("{:?}", d.kind).to_lowercase()
            )
        })
        .collect();

    let relation_summary: Vec<String> = ontology
        .relations
        .iter()
        .map(|r| {
            format!(
                "  - {} → {} ({})",
                r.source_ontology,
                r.target_ontology,
                format!("{:?}", r.relation_type).to_lowercase()
            )
        })
        .collect();

    let answers = user_answers.join("; ");

    format!(
        r#"用户原始需求：
"{}"

当前本体模型已有以下定义：
领域：
{}

关系：
{}

用户对之前问题的回复：
"{}"

请基于用户回复，更新本体模型。只输出更新后的完整 JSON（不要包裹在 markdown 代码块中）。

【必须保留的字段】
更新时请务必保留以下字段（若用户回复未涉及相关部分，直接复制原值）：
- used_modules: 已选中的平台模块列表
- known_entities: 已识别的平台实体表名
- constraints: 业务约束配置
- computations: 自动计算逻辑
- transaction_lifecycle: 交易生命周期定义
- missing_info: 待澄清信息，必须按四段式（scene_condition, decision_elements, judgment_criteria, judgment_result）输出
- workflow_steps: 业务流程步骤

注意：保留已有的领域定义，仅修改用户回复涉及的部分。"#,
        user_description,
        domain_summary.join("\n"),
        relation_summary.join("\n"),
        answers
    )
}

fn format_catalog_modules(catalog: &PlatformCatalog) -> String {
    catalog
        .modules
        .iter()
        .map(|m| {
            let parts = [format!("  - {} ({})", m.id, m.name)];
            parts.join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_catalog_collections(catalog: &PlatformCatalog) -> String {
    if catalog.collections.is_empty() {
        return String::from("  (无)");
    }
    catalog
        .collections
        .iter()
        .map(|c| format!("  - {} ({})", c.name, c.table_name))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_catalog_scenes(catalog: &PlatformCatalog) -> String {
    catalog
        .scenes
        .iter()
        .map(|s| format!("  - {} ({})", s.code, s.notice))
        .collect::<Vec<_>>()
        .join("\n")
}

// ─── 解析与验证 ────────────────────────────────────────────────────────────

/// 解析 LLM 响应并执行多层容错验证
pub fn parse_and_validate(response: &str, catalog: &PlatformCatalog) -> ValidatedOntology {
    let mut fix_log: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // ── 第1层：JSON 提取 ──
    let json_str = extract_json(response);
    common::telemetry::info!(
        "parse_and_validate: extracted JSON length={}, content={}",
        json_str.len(),
        json_str
    );
    let mut output: OntologyOutput = match parse_json_tolerant(&json_str) {
        Ok(o) => o,
        Err(e) => {
            common::telemetry::warn!("parse_and_validate: JSON parse failed, error={}", e);
            return ValidatedOntology {
                ontology: OntologyModel::default(),
                used_modules: vec![],
                known_entities: vec![],
                missing_info: vec![MissingInfo {
                    category: MissingInfoCategory::EntityExtension,
                    scene_condition: "AI 在解析用户自然语言需求时返回了非预期的响应格式"
                        .to_string(),
                    decision_elements: "请确认是否需要重新描述需求，或检查输入内容是否包含特殊字符"
                        .to_string(),
                    judgment_criteria:
                        "若用户输入为正常自然语言描述，则系统应能正确解析；若持续失败，建议简化描述"
                            .to_string(),
                    judgment_result: "当前解析失败，建议用户重新描述需求，避免使用代码块或特殊格式"
                        .to_string(),
                }],
                workflow_steps: vec![],
                app_meta: None,
                fix_log: vec![format!("JSON parse error: {}", e)],
                warnings: vec![],
            };
        }
    };

    // ── 第2层：枚举 case-insensitive 修正（规则2） ──
    fix_domain_kinds(&mut output, &mut fix_log);
    fix_property_types(&mut output, &mut fix_log);
    fix_relation_types(&mut output, &mut fix_log);
    fix_phase_types(&mut output, &mut fix_log);

    // ── 第3层：结构自动修复 ──
    fix_property_domains(&mut output, &mut fix_log);
    fix_missing_cardinalities(&mut output, &mut fix_log);
    fix_required_cardinality_mismatch(&mut output, &mut fix_log, &mut warnings);
    fix_self_referencing_parents(&mut output, &mut fix_log);
    fix_orphan_constraints(&mut output, &mut fix_log, &mut warnings);
    fix_unknown_domain_ids_in_relations(&mut output, &mut fix_log, &mut warnings);

    // ── 第4层：语义验证（不可自动修复的升级为 MissingInfo） ──
    let semantic_issues = detect_unfixable_issues(&output, catalog);

    let mut missing_info = output.missing_info;
    for issue in semantic_issues {
        missing_info.push(MissingInfo {
            category: MissingInfoCategory::EntityExtension,
            scene_condition: format!("在构建本体模型过程中发现以下语义问题: {}", issue),
            decision_elements: "请确认该问题的处理方式：是补充缺失信息、调整需求描述，还是接受当前推断".to_string(),
            judgment_criteria: "以业务语义完整性为准：若该问题影响核心业务流程，必须澄清；若为边缘场景，可接受默认推断".to_string(),
            judgment_result: format!("系统检测到: {}。建议用户确认或补充相关信息以确保模型准确。", issue),
        });
    }

    // ── 清理：设置默认元数据 ──
    if output.ontology.metadata.created_at.is_empty() {
        output.ontology.metadata = OntologyMetadata::default();
    }

    ValidatedOntology {
        ontology: output.ontology,
        used_modules: output.used_modules,
        known_entities: output.known_entities,
        missing_info,
        workflow_steps: output.workflow_steps,
        app_meta: output.app_meta,
        fix_log,
        warnings,
    }
}

// ─── 第1层：JSON 提取 ─────────────────────────────────────────────────────

fn extract_json(raw: &str) -> String {
    let trimmed = raw.trim();

    // 剥离 markdown 代码块
    if let Some(rest) = trimmed.strip_prefix("```json") {
        if let Some(end) = rest.rfind("```") {
            return rest[..end].trim().to_string();
        }
        return rest.trim().to_string();
    }
    if let Some(rest) = trimmed.strip_prefix("```") {
        if let Some(end) = rest.rfind("```") {
            return rest[..end].trim().to_string();
        }
        return rest.trim().to_string();
    }

    // 尝试定位第一个 { 到最后一个 }
    if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        trimmed[start..=end].to_string()
    } else {
        trimmed.to_string()
    }
}

fn parse_json_tolerant(json_str: &str) -> Result<OntologyOutput, String> {
    // 预处理：将 computations 中的字符串数组 inputs/outputs 转换为对象数组
    let preprocessed = preprocess_computation_arrays(json_str);

    // 首选：标准 JSON 解析
    match serde_json::from_str::<OntologyOutput>(&preprocessed) {
        Ok(output) => {
            common::telemetry::info!(
                "parse_json_tolerant: serde_json parsed successfully, domains={}, relations={}",
                output.ontology.domains.len(),
                output.ontology.relations.len()
            );
            return Ok(output);
        }
        Err(serde_err) => {
            common::telemetry::warn!("parse_json_tolerant: serde_json failed: {}", serde_err);
        }
    }

    // 回退：json5 容错解析（容忍尾部逗号、注释、无引号 key）
    json5::from_str::<OntologyOutput>(&preprocessed).map_err(|e| {
        // 最后尝试：serde_json 给出更清晰的错误信息
        match serde_json::from_str::<OntologyOutput>(&preprocessed) {
            Ok(_o) => String::new(), // unreachable
            Err(serde_err) => format!("json5: {}, serde_json: {}", e, serde_err),
        }
    })
}

/// 预处理：将 computations.inputs/outputs 中的字符串数组转换为对象数组
fn preprocess_computation_arrays(json_str: &str) -> String {
    let mut value: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return json_str.to_string(),
    };

    if let Some(ontology) = value.get_mut("ontology") {
        if let Some(computations) = ontology
            .get_mut("computations")
            .and_then(|v| v.as_array_mut())
        {
            for comp in computations {
                if let Some(obj) = comp.as_object_mut() {
                    // inputs
                    if let Some(inputs) = obj.get("inputs").and_then(|v| v.as_array()) {
                        if inputs.iter().all(|v| v.is_string()) {
                            let converted: Vec<serde_json::Value> = inputs
                                .iter()
                                .filter_map(|v| {
                                    let s = v.as_str()?;
                                    let parts: Vec<&str> = s.splitn(2, '.').collect();
                                    let (source_ontology, source_property) = if parts.len() == 2 {
                                        (parts[0], Some(parts[1]))
                                    } else {
                                        (s, None)
                                    };
                                    Some(serde_json::json!({
                                        "id": s.replace('.', "_"),
                                        "name": s,
                                        "source_ontology": source_ontology,
                                        "source_property": source_property,
                                        "input_type": "string"
                                    }))
                                })
                                .collect();
                            obj.insert("inputs".to_string(), serde_json::Value::Array(converted));
                        }
                    }
                    // outputs
                    if let Some(outputs) = obj.get("outputs").and_then(|v| v.as_array()) {
                        if outputs.iter().all(|v| v.is_string()) {
                            let converted: Vec<serde_json::Value> = outputs
                                .iter()
                                .filter_map(|v| {
                                    let s = v.as_str()?;
                                    let parts: Vec<&str> = s.splitn(2, '.').collect();
                                    let (target_ontology, target_property) = if parts.len() == 2 {
                                        (parts[0], Some(parts[1]))
                                    } else {
                                        (s, None)
                                    };
                                    Some(serde_json::json!({
                                        "id": s.replace('.', "_"),
                                        "name": s,
                                        "target_ontology": target_ontology,
                                        "target_property": target_property,
                                        "output_type": "string"
                                    }))
                                })
                                .collect();
                            obj.insert("outputs".to_string(), serde_json::Value::Array(converted));
                        }
                    }
                    // refresh_strategy 默认值
                    if !obj.contains_key("refresh_strategy") {
                        obj.insert(
                            "refresh_strategy".to_string(),
                            serde_json::json!({
                                "refresh_type": "event_driven",
                                "trigger_events": []
                            }),
                        );
                    }
                }
            }
        }
    }

    serde_json::to_string(&value).unwrap_or_else(|_| json_str.to_string())
}

fn fix_domain_kinds(output: &mut OntologyOutput, fix_log: &mut Vec<String>) {
    // 启发式修正：根据领域名称和关系推断 kind
    for domain in &mut output.ontology.domains {
        let inferred = infer_domain_kind(domain, &output.ontology.relations);
        if inferred != domain.kind {
            fix_log.push(format!(
                "修正 domain '{}' 的 kind: {:?} → {:?}",
                domain.id, domain.kind, inferred
            ));
            domain.kind = inferred;
        }
    }
}

fn infer_domain_kind(domain: &DomainOntology, relations: &[RelationOntology]) -> DomainKind {
    let name_lower = domain.id.to_lowercase();
    // 名称启发式
    if name_lower.ends_with("_status")
        || name_lower.ends_with("_type")
        || name_lower.contains("enum")
    {
        return DomainKind::Enumeration;
    }
    if name_lower.contains("event") {
        return DomainKind::DomainEvent;
    }
    if name_lower.contains("service") {
        return DomainKind::DomainService;
    }
    // 关系启发式：如果有 Composition 出边且没有入边，可能是 AggregateRoot
    let has_composition_out = relations
        .iter()
        .any(|r| r.source_ontology == domain.id && r.relation_type == RelationType::Composition);
    let has_composition_in = relations
        .iter()
        .any(|r| r.target_ontology == domain.id && r.relation_type == RelationType::Composition);
    if has_composition_out && !has_composition_in {
        return DomainKind::AggregateRoot;
    }
    // 默认保持原值
    domain.kind.clone()
}

fn fix_property_types(output: &mut OntologyOutput, fix_log: &mut Vec<String>) {
    let primitive_types: &[&str] = &[
        "String", "Integer", "Decimal", "Boolean", "DateTime", "Uuid", "Json",
    ];
    for domain in &mut output.ontology.domains {
        for prop in &mut domain.properties {
            let expected = if primitive_types
                .iter()
                .any(|&p| p.eq_ignore_ascii_case(&prop.range))
            {
                PropertyType::DataProperty
            } else {
                // range 是另一个领域 id → 对象属性
                PropertyType::ObjectProperty
            };
            if prop.property_type != expected {
                fix_log.push(format!(
                    "修正 domain '{}' property '{}' 的 type: {:?} → {:?}",
                    domain.id, prop.id, prop.property_type, expected
                ));
                prop.property_type = expected;
            }
        }
    }
}

fn fix_relation_types(output: &mut OntologyOutput, fix_log: &mut Vec<String>) {
    use alioth_gen::generator::ir::ontology::RelationType;

    for relation in &mut output.ontology.relations {
        let original = format!("{:?}", relation.relation_type);
        let normalized = original.trim().to_lowercase().replace(" ", "_");
        let fixed = match normalized.as_str() {
            "composition" | "compose" | "part_of" => RelationType::Composition,
            "aggregation" | "aggregate" | "has" => RelationType::Aggregation,
            "association" | "associate" | "link" => RelationType::Association,
            "dependency" | "depend" | "requires" => RelationType::Dependency,
            "inheritance" | "inherit" | "is_a" | "generalization" | "generalize" => {
                RelationType::Inheritance
            }
            "realization" | "realize" | "implements" => RelationType::Realization,
            _ => {
                // 若无法识别，默认设为 Association 并记录警告
                fix_log.push(format!(
                    "修正 relation '{}' 的类型: '{}' → 'association' (未知类型)",
                    relation.id, original
                ));
                RelationType::Association
            }
        };
        if std::mem::discriminant(&fixed) != std::mem::discriminant(&relation.relation_type) {
            fix_log.push(format!(
                "修正 relation '{}' 的类型: '{}' → '{:?}'",
                relation.id, original, fixed
            ));
            relation.relation_type = fixed;
        }
    }
}

fn fix_phase_types(output: &mut OntologyOutput, fix_log: &mut Vec<String>) {
    use alioth_gen::generator::ir::ontology::PhaseType;

    if let Some(lifecycle) = &mut output.ontology.transaction_lifecycle {
        let mut seen_ids = std::collections::HashSet::new();
        for phase in &mut lifecycle.phases {
            // 去重检查
            if !seen_ids.insert(phase.id.clone()) {
                fix_log.push(format!("修正 phase 重复 id: '{}', 保留首次出现", phase.id));
            }

            let original = format!("{:?}", phase.phase_type);
            let normalized = original.trim().to_lowercase().replace(" ", "_");
            let fixed = match normalized.as_str() {
                "creation" | "create" | "start" | "begin" | "initial" => PhaseType::Creation,
                "validation" | "validate" | "verify" => PhaseType::Validation,
                "confirmation" | "confirm" | "approve" => PhaseType::Confirmation,
                "execution" | "execute" | "process" | "run" => PhaseType::Execution,
                "settlement" | "settle" | "complete" | "finish" => PhaseType::Settlement,
                "archival" | "archive" | "close" | "closed" => PhaseType::Archival,
                "cancellation" | "cancel" | "abort" => PhaseType::Cancellation,
                _ => {
                    fix_log.push(format!(
                        "修正 phase '{}' 的类型: '{}' → 'execution' (未知类型)",
                        phase.id, original
                    ));
                    PhaseType::Execution
                }
            };
            if std::mem::discriminant(&fixed) != std::mem::discriminant(&phase.phase_type) {
                fix_log.push(format!(
                    "修正 phase '{}' 的类型: '{}' → '{:?}'",
                    phase.id, original, fixed
                ));
                phase.phase_type = fixed;
            }
        }
    }
}

// ─── 第3层：结构自动修复 ──────────────────────────────────────────────────

/// 规则3：property.domain 自动修正为所属 domain 的 id
fn fix_property_domains(output: &mut OntologyOutput, fix_log: &mut Vec<String>) {
    for domain in &mut output.ontology.domains {
        let domain_id = domain.id.clone();
        for prop in &mut domain.properties {
            if prop.domain != domain_id {
                fix_log.push(format!(
                    "修正 property '{}' 的 domain: '{}' → '{}'",
                    prop.id, prop.domain, domain_id
                ));
                prop.domain = domain_id.clone();
            }
        }
    }
}

/// 规则4：缺失 cardinality 默认填充
fn fix_missing_cardinalities(output: &mut OntologyOutput, fix_log: &mut Vec<String>) {
    for domain in &mut output.ontology.domains {
        for prop in &mut domain.properties {
            // Cardinality is already defaulted by serde since it's not Option
            // But serde default might not be what we want
            if prop.cardinality.min.is_none()
                && prop.cardinality.max.is_none()
                && prop.cardinality.exact.is_none()
            {
                prop.cardinality = Cardinality {
                    min: if prop.required { Some(1) } else { Some(0) },
                    max: None,
                    exact: None,
                };
                fix_log.push(format!(
                    "填充默认 cardinality for property '{}': min={}, max=null",
                    prop.id,
                    prop.cardinality.min.unwrap_or(0)
                ));
            }
        }
    }
}

/// 规则7：required=true 但 cardinality.min=0 → 以 required 为准
fn fix_required_cardinality_mismatch(
    output: &mut OntologyOutput,
    fix_log: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    for domain in &mut output.ontology.domains {
        for prop in &mut domain.properties {
            if prop.required && prop.cardinality.min == Some(0) {
                prop.cardinality.min = Some(1);
                fix_log.push(format!(
                    "修正 property '{}': required=true 但 cardinality.min=0，已设为 min=1",
                    prop.id
                ));
            }
            if !prop.required && prop.cardinality.min == Some(1) {
                warnings.push(format!(
                    "property '{}': required=false 但 cardinality.min=1，可能存在矛盾",
                    prop.id
                ));
            }
        }
    }
}

/// 规则8：移除自引用 parent_ids
fn fix_self_referencing_parents(output: &mut OntologyOutput, fix_log: &mut Vec<String>) {
    for domain in &mut output.ontology.domains {
        let domain_id = domain.id.clone();
        let before = domain.parent_ids.len();
        domain.parent_ids.retain(|pid| pid != &domain_id);
        if domain.parent_ids.len() < before {
            fix_log.push(format!("移除 domain '{}' 的自引用 parent_ids", domain_id));
        }
    }
}

/// 规则5+9：孤立约束（引用不存在的实体/字段）→ 移除或降级为 warning
fn fix_orphan_constraints(
    output: &mut OntologyOutput,
    fix_log: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    let domain_ids: Vec<String> = output
        .ontology
        .domains
        .iter()
        .map(|d| d.id.clone())
        .collect();

    // 收集所有 property ids（按 domain 分组）
    let mut domain_props: HashMap<String, Vec<String>> = HashMap::new();
    for domain in &output.ontology.domains {
        let props: Vec<String> = domain.properties.iter().map(|p| p.id.clone()).collect();
        domain_props.insert(domain.id.clone(), props);
    }

    // 修复 constraints 中的引用（通过 scope.target_ontology）
    output.ontology.constraints.retain(|c| {
        if !domain_ids.contains(&c.scope.target_ontology) {
            warnings.push(format!(
                "移除了引用不存在领域 '{}' 的约束 '{}'",
                c.scope.target_ontology, c.id
            ));
            fix_log.push(format!(
                "移除孤立约束 '{}'（引用不存在的领域 '{}'）",
                c.id, c.scope.target_ontology
            ));
            return false;
        }
        true
    });

    // 修复 computations 中的引用（检查 inputs/outputs 中的本体引用）
    output.ontology.computations.retain(|c| {
        let all_input_ids_valid = c.inputs.iter().all(|inp| {
            if !domain_ids.contains(&inp.source_ontology) {
                warnings.push(format!(
                    "计算 '{}' 的输入引用了不存在的领域 '{}'",
                    c.id, inp.source_ontology
                ));
                fix_log.push(format!(
                    "移除孤立计算 '{}'（输入引用不存在的领域 '{}'）",
                    c.id, inp.source_ontology
                ));
                return false;
            }
            true
        });
        if !all_input_ids_valid {
            return false;
        }
        true
    });
}

/// 规则1：relation 引用了不存在的 domain_id → fuzzy match 自动修正
fn fix_unknown_domain_ids_in_relations(
    output: &mut OntologyOutput,
    fix_log: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    let domain_ids: Vec<String> = output
        .ontology
        .domains
        .iter()
        .map(|d| d.id.clone())
        .collect();

    for relation in &mut output.ontology.relations {
        // 检查 source
        if !domain_ids.contains(&relation.source_ontology) {
            if let Some(best) = fuzzy_match(&relation.source_ontology, &domain_ids) {
                fix_log.push(format!(
                    "修正 relation '{}' 的 source: '{}' → '{}'",
                    relation.id, relation.source_ontology, best
                ));
                relation.source_ontology = best;
            } else {
                warnings.push(format!(
                    "relation '{}' 的 source '{}' 无法匹配任何已知领域",
                    relation.id, relation.source_ontology
                ));
            }
        }

        // 检查 target
        if !domain_ids.contains(&relation.target_ontology) {
            if let Some(best) = fuzzy_match(&relation.target_ontology, &domain_ids) {
                fix_log.push(format!(
                    "修正 relation '{}' 的 target: '{}' → '{}'",
                    relation.id, relation.target_ontology, best
                ));
                relation.target_ontology = best;
            } else {
                warnings.push(format!(
                    "relation '{}' 的 target '{}' 无法匹配任何已知领域",
                    relation.id, relation.target_ontology
                ));
            }
        }
    }
}

fn fuzzy_match(target: &str, candidates: &[String]) -> Option<String> {
    let target_lower = target.to_lowercase().replace(['-', '_', ' '], "");

    let mut best: Option<(String, usize)> = None;
    for candidate in candidates {
        let cand_lower = candidate.to_lowercase().replace(['-', '_', ' '], "");
        // 简单的公共子串长度匹配
        let common = longest_common_substring(&target_lower, &cand_lower);
        if common > 0 {
            match &best {
                None => best = Some((candidate.clone(), common)),
                Some((_, prev)) if common > *prev => {
                    best = Some((candidate.clone(), common));
                }
                _ => {}
            }
        }
    }

    best.and_then(|(name, common)| {
        // 公共子串长度至少达到较短名称的 60%，且不低于 3
        let min_len = target.len().min(name.len());
        let threshold = ((min_len as f64) * 0.6).ceil() as usize;
        if common >= threshold.max(3) {
            Some(name)
        } else {
            None
        }
    })
}

fn longest_common_substring(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let mut max_len = 0;

    for i in 0..a_chars.len() {
        for j in 0..b_chars.len() {
            let mut k = 0;
            while i + k < a_chars.len() && j + k < b_chars.len() && a_chars[i + k] == b_chars[j + k]
            {
                k += 1;
            }
            max_len = max_len.max(k);
        }
    }
    max_len
}

// ─── 第4层：不可自动修复的语义问题（E1-E4） ──────────────────────────────

fn detect_unfixable_issues(output: &OntologyOutput, catalog: &PlatformCatalog) -> Vec<String> {
    let mut issues: Vec<String> = Vec::new();

    // E1: 环形继承检测
    if let Some(cycle) = detect_inheritance_cycle(&output.ontology) {
        issues.push(format!("检测到环形继承: {}", cycle));
    }

    // E2: equivalent_ids 与 disjoint_ids 矛盾
    for domain in &output.ontology.domains {
        for eq_id in &domain.equivalent_ids {
            if domain.disjoint_ids.contains(eq_id) {
                issues.push(format!(
                    "领域 '{}' 的 equivalent_ids 和 disjoint_ids 同时包含 '{}'",
                    domain.id, eq_id
                ));
            }
        }
    }

    // E3: 生命周期转换不闭合
    if let Some(lifecycle) = &output.ontology.transaction_lifecycle {
        let phase_ids: Vec<&str> = lifecycle.phases.iter().map(|p| p.id.as_str()).collect();
        for transition in &lifecycle.transitions {
            if !phase_ids.contains(&transition.from_phase.as_str()) {
                issues.push(format!(
                    "生命周期转换引用了不存在的源阶段: '{}'",
                    transition.from_phase
                ));
            }
            if !phase_ids.contains(&transition.to_phase.as_str()) {
                issues.push(format!(
                    "生命周期转换引用了不存在的目标阶段: '{}'",
                    transition.to_phase
                ));
            }
        }

        // 检查是否有阶段无法到达或被孤立
        let mut reachable: Vec<bool> = vec![false; lifecycle.phases.len()];
        if let Some(first) = lifecycle.phases.first() {
            let first_id = &first.id;
            for (i, phase) in lifecycle.phases.iter().enumerate() {
                if &phase.id == first_id {
                    reachable[i] = true;
                }
            }

            // 简单可达性传播
            let mut changed = true;
            while changed {
                changed = false;
                for transition in &lifecycle.transitions {
                    for (i, phase) in lifecycle.phases.iter().enumerate() {
                        if phase.id == transition.from_phase && reachable[i] {
                            for (j, target) in lifecycle.phases.iter().enumerate() {
                                if target.id == transition.to_phase && !reachable[j] {
                                    reachable[j] = true;
                                    changed = true;
                                }
                            }
                        }
                    }
                }
            }

            for (i, phase) in lifecycle.phases.iter().enumerate() {
                if !reachable[i] && !phase.is_terminal {
                    issues.push(format!(
                        "阶段 '{}' 无法从初始阶段到达，生命周期可能不完整",
                        phase.name
                    ));
                }
            }
        }
    }

    // E4: relation 两端都 fuzzy match 失败（已在 fix_unknown_domain_ids_in_relations 中降级为 warning）
    // 此处无需重复处理

    // 检查 used_modules 是否都在 catalog 中
    let known_modules: Vec<String> = catalog.modules.iter().map(|m| m.id.clone()).collect();
    for module in &output.used_modules {
        if !known_modules.contains(module) {
            issues.push(format!(
                "引用了不存在的模块 '{}'，可用模块: {}",
                module,
                known_modules.join(", ")
            ));
        }
    }

    issues
}

fn detect_inheritance_cycle(ontology: &OntologyModel) -> Option<String> {
    // 构建 parent→child 图
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
    for domain in &ontology.domains {
        for parent_id in &domain.parent_ids {
            graph
                .entry(parent_id.as_str())
                .or_default()
                .push(domain.id.as_str());
        }
    }

    // DFS 检测环路
    let mut visited: HashMap<&str, bool> = HashMap::new();
    let mut stack: HashMap<&str, bool> = HashMap::new();

    for domain in &ontology.domains {
        if !visited.contains_key(domain.id.as_str()) {
            if let Some(cycle) = dfs_cycle(domain.id.as_str(), &graph, &mut visited, &mut stack) {
                return Some(cycle);
            }
        }
    }

    None
}

fn dfs_cycle<'a>(
    node: &'a str,
    graph: &HashMap<&'a str, Vec<&'a str>>,
    visited: &mut HashMap<&'a str, bool>,
    stack: &mut HashMap<&'a str, bool>,
) -> Option<String> {
    visited.insert(node, true);
    stack.insert(node, true);

    if let Some(children) = graph.get(node) {
        for &child in children {
            if !visited.contains_key(child) {
                if let Some(cycle) = dfs_cycle(child, graph, visited, stack) {
                    return Some(cycle);
                }
            } else if stack.get(child).copied().unwrap_or(false) {
                return Some(format!("{} → ... → {}", child, node));
            }
        }
    }

    stack.insert(node, false);
    None
}

// ─── System Prompt ─────────────────────────────────────────────────────────

const SYSTEM_PROMPT: &str = r#"你是一个企业应用本体建模专家。根据用户的自然语言需求，构建符合 Alioth 本体模型规范的 JSON 输出。
1. 直接输出纯 JSON（不要 markdown 代码块，不要解释文字）。
2. 优先复用已有模块（used_modules），不要为已有能力新建 domains。
3. 新实体必须继承平台根类型（如 zc_id_entity）。
4. 所有字段名和枚举值使用准确的驼峰命名。
5. JSON 必须合法（无尾部逗号、无注释、无未闭合引号）。
6. scene_code/factor_code MUST 为 null——层2 坐标由用户在 Composing 后填写，不由你猜测。
7. dtoExposes.queries MUST 为 ["list_refs","get_refs"]——禁止输出 entity-specific 查询名，runtime 按 ontology.relations 解析。
8. domain.id 优先匹配 PlatformCatalog 已有集合名（catalog.name），避免为同一业务概念新建实体。
</critical>

【输出 JSON 结构】
{
  "ontology": {
    "id": "kebab-case",
    "name": "中文名",
    "version": "1.0",
    "domains": [{ "id":"snake_case", "name":"中文", "kind":"Entity", "parent_ids":["zc_id_entity"], "properties":[{ "id":"", "name":"", "property_type":"DataProperty", "required":true, "cardinality":{"min":1,"max":null}, "domain":"", "range":"String", "is_functional":true }] }],
    "relations": [{ "id":"", "name":"", "relation_type":"Composition", "source_ontology":"", "target_ontology":"", "is_bidirectional":false }],
    "constraints": [{ "id":"", "name":"", "constraint_type":"Structural", "scope":{"target_ontology":""}, "expression":"", "severity":"Error" }],
    "computations": [{ "id":"", "name":"", "computation_type":"Derivation", "inputs":[], "outputs":[], "formula":"", "trigger_conditions":["OnCreate"] }],
    "transaction_lifecycle": null
  },
  "used_modules": ["inventory", "product"],
  "known_entities": ["zc_id_inventory"],
  "missing_info": [],
  "workflow_steps": ["步骤1"],
  "app_meta": {
    "name": "应用中文展示名",
    "description": "App 功能描述",
    "environment": "development",
    "deploymentMode": null,
    "permissions": { "defaultRoles": ["admin","user"], "publicPaths": ["/login"], "adminRoles": ["admin"] },
    "routing": { "base": "/apps/{app-code}", "defaultRoute": "/{first-module-id}" },
    "brand": { "primary": "262 70% 55%" },
    "navigation": [{ "group": "分组名", "icon": "Settings", "modules": ["inventory"] }],
    "non_scope": ["明确排除的功能"]
  }
}

【kind 取值】Entity / AggregateRoot / ValueObject / Enumeration / DomainService / DomainEvent
【relation_type 取值】Composition（聚合根-子实体）/ Association（跨聚合引用）/ Inheritance（继承）

【app_meta 说明】App 级配置元数据,驱动 app.json 生成。字段对齐 Pre-Proc/Alioth/_schema/app.schema.json:
- name: 应用展示名(中文,缺失时用 app code)
- environment: [development, staging, production](默认 development)
- deploymentMode: [null, "standalone", "embedded"](null=自动发现)
- routing.base: App 基础路径(pattern ^/,如 /apps/warehouse)
- routing.defaultRoute: 默认落地页(pattern ^/,如 /inventory)
- brand.primary: HSL 主色(格式 "H S% L%",如 "262 70% 55%")
- navigation: 菜单分组数组,每项含 group/icon/modules
- non_scope: 明确排除的功能范围

【示例】用户需求："仓库库存管理"
正确输出：
{"ontology":{"id":"warehouse-inventory","name":"仓库库存管理","version":"1.0","domains":[{"id":"inventory","name":"库存","kind":"Entity","parent_ids":["zc_id_entity"],"properties":[{"id":"quantity","name":"数量","property_type":"DataProperty","required":true,"cardinality":{"min":1,"max":null},"domain":"inventory","range":"Integer","is_functional":true}]}],"relations":[],"constraints":[],"computations":[],"transaction_lifecycle":null},"used_modules":["inventory","product"],"known_entities":["zc_id_inventory","zc_id_product"],"missing_info":[],"workflow_steps":["查看库存","入库","出库"],"app_meta":{"name":"仓库库存管理","description":"仓库库存管理系统,提供库存查看与出入库管理","environment":"development","deploymentMode":null,"permissions":{"defaultRoles":["admin","user"],"publicPaths":["/login"],"adminRoles":["admin"]},"routing":{"base":"/apps/warehouse-inventory","defaultRoute":"/inventory"},"brand":{"primary":"262 70% 55%"},"navigation":[{"group":"库存管理","icon":"Package","modules":["inventory"]}],"non_scope":["财务结算","供应商管理"]}}

【missing_info 四段式】当需要用户确认时，每个对象包含：scene_condition（场景条件）、decision_elements（决策要素）、judgment_criteria（判断标准）、judgment_result（判断结果）。
【建模原则】聚合根 kind=AggregateRoot，子实体 kind=Entity，二者之间 relation_type=Composition。跨聚合引用用 Association。继承平台实体时 parent_ids 填 zc_id_* 表名。"#;

// ─── 兼容旧接口（逐步迁移期间保留） ────────────────────────────────────────

/// 从 LLM 响应解析 FlowPlan（兼容旧调用方）
/// 当解析为 OntologyOutput 时，提取其中与 FlowPlan 兼容的字段
pub fn parse_llm_response(response: &str, namespace: &str) -> FlowPlan {
    let validated = parse_and_validate(response, &PlatformCatalog::default());
    let (constraints, computations) = extract_plan_fields_from_ontology(&validated.ontology);
    let business_rules = extract_business_rules_from_ontology(&validated.ontology);
    FlowPlan {
        namespace: namespace.to_string(),
        used_modules: validated.used_modules,
        known_entities: validated.known_entities,
        missing_info: validated.missing_info,
        workflow_steps: validated.workflow_steps,
        computations,
        constraints,
        business_rules,
        app_meta: validated.app_meta,
        created_modules: vec![],
        created_blocks: vec![],
        created_services: vec![],
        ontology_model_json: None,
        functional_units: vec![],
        semantic_concepts: vec![],
    }
}

/// 从 OntologyModel 提取约束和计算逻辑，转换为 FlowPlan 字段
pub fn extract_plan_fields_from_ontology(
    ontology: &OntologyModel,
) -> (
    Vec<crate::state::ConstraintPlan>,
    Vec<crate::state::ComputationPlan>,
) {
    let constraints: Vec<crate::state::ConstraintPlan> = ontology
        .constraints
        .iter()
        .map(|c| crate::state::ConstraintPlan {
            entity: c.scope.target_ontology.clone(),
            field: c.scope.target_property.clone(),
            expression: c.expression.clone(),
            level: match c.severity {
                alioth_gen::generator::ir::ontology::ConstraintSeverity::Warning => {
                    "warning".to_string()
                }
                _ => "error".to_string(),
            },
            message: c
                .error_message_template
                .clone()
                .or_else(|| c.description.clone())
                .unwrap_or_else(|| format!("Constraint {} violated", c.name)),
        })
        .collect();

    let computations: Vec<crate::state::ComputationPlan> = ontology
        .computations
        .iter()
        .map(|c| {
            let depends_on: Vec<String> = c
                .inputs
                .iter()
                .filter_map(|i| i.source_property.clone())
                .collect();
            let target_field = c
                .outputs
                .first()
                .and_then(|o| o.target_property.clone())
                .unwrap_or_default();
            let entity = c
                .outputs
                .first()
                .map(|o| o.target_ontology.clone())
                .unwrap_or_default();
            crate::state::ComputationPlan {
                entity,
                target_field,
                formula: c.formula.clone(),
                depends_on,
                trigger: c
                    .trigger_conditions
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "onCreate".to_string()),
            }
        })
        .collect();

    (constraints, computations)
}

/// 从 OntologyModel 提取业务规则（将计算本体映射为条件-动作规则）
pub fn extract_business_rules_from_ontology(
    ontology: &OntologyModel,
) -> Vec<crate::state::BusinessRulePlan> {
    ontology
        .computations
        .iter()
        .filter_map(|c| {
            let output = c.outputs.first()?;
            let target_property = output.target_property.as_ref()?;
            let entity = output.target_ontology.clone();
            let trigger = c
                .trigger_conditions
                .first()
                .cloned()
                .unwrap_or_else(|| "onCreate".to_string());
            // 计算逻辑映射为赋值动作：target = formula
            let action = format!("{} = {}", target_property, c.formula);
            Some(crate::state::BusinessRulePlan {
                entity,
                rule_name: c.name.clone(),
                trigger,
                condition: "true".to_string(),
                action,
                priority: 0,
                error_message: c
                    .description
                    .clone()
                    .unwrap_or_else(|| format!("Rule {} failed", c.name)),
            })
        })
        .collect()
}

// ─── 测试 ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use alioth_gen::generator::ir::ontology::{DomainOntology, OntologyProperty, PropertyType};

    #[test]
    fn test_extract_json_no_fences() {
        let input =
            r#"{"ontology": {"id": "test", "name": "Test", "version": "1.0", "domains": []}}"#;
        let result = extract_json(input);
        assert!(result.starts_with('{'));
        assert!(result.ends_with('}'));
    }

    #[test]
    fn test_extract_json_with_fences() {
        let input = r#"```json
{"ontology": {"id": "test", "name": "Test", "version": "1.0", "domains": []}}
```"#;
        let result = extract_json(input);
        assert!(result.starts_with('{'));
        assert!(!result.contains("```"));
    }

    #[test]
    fn test_extract_json_with_text_prefix() {
        let input = r#"以下是本体模型：
{"ontology": {"id": "test", "name": "Test", "version": "1.0", "domains": []}}"#;
        let result = extract_json(input);
        assert!(result.starts_with('{'));
    }

    #[test]
    fn test_fix_property_domains() {
        let mut output = OntologyOutput {
            ontology: OntologyModel {
                id: "test".into(),
                name: "Test".into(),
                version: "1.0".into(),
                domains: vec![DomainOntology {
                    id: "order".into(),
                    name: "订单".into(),
                    kind: DomainKind::AggregateRoot,
                    properties: vec![OntologyProperty {
                        id: "order_number".into(),
                        name: "订单号".into(),
                        property_type: PropertyType::DataProperty,
                        required: true,
                        cardinality: Cardinality {
                            min: Some(1),
                            max: Some(1),
                            exact: None,
                        },
                        domain: "wrong_id".into(), // 错误
                        range: "String".into(),
                        is_functional: true,
                        is_transitive: false,
                        is_symmetric: false,
                        constraints: vec![],
                        semantic_description: None,
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            },
            ..Default::default()
        };

        let mut fix_log = vec![];
        fix_property_domains(&mut output, &mut fix_log);
        assert_eq!(output.ontology.domains[0].properties[0].domain, "order");
        assert!(!fix_log.is_empty());
    }

    #[test]
    fn test_fix_self_referencing_parents() {
        let mut output = OntologyOutput {
            ontology: OntologyModel {
                id: "test".into(),
                name: "Test".into(),
                version: "1.0".into(),
                domains: vec![DomainOntology {
                    id: "order".into(),
                    name: "订单".into(),
                    kind: DomainKind::Entity,
                    parent_ids: vec!["order".into(), "zc_id_entity".into()],
                    ..Default::default()
                }],
                ..Default::default()
            },
            ..Default::default()
        };

        let mut fix_log = vec![];
        fix_self_referencing_parents(&mut output, &mut fix_log);
        assert_eq!(output.ontology.domains[0].parent_ids, vec!["zc_id_entity"]);
    }

    #[test]
    fn test_detect_inheritance_cycle() {
        let ontology = OntologyModel {
            id: "test".into(),
            name: "Test".into(),
            version: "1.0".into(),
            domains: vec![
                DomainOntology {
                    id: "a".into(),
                    name: "A".into(),
                    kind: DomainKind::Entity,
                    parent_ids: vec!["b".into()],
                    ..Default::default()
                },
                DomainOntology {
                    id: "b".into(),
                    name: "B".into(),
                    kind: DomainKind::Entity,
                    parent_ids: vec!["c".into()],
                    ..Default::default()
                },
                DomainOntology {
                    id: "c".into(),
                    name: "C".into(),
                    kind: DomainKind::Entity,
                    parent_ids: vec!["a".into()],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let cycle = detect_inheritance_cycle(&ontology);
        assert!(cycle.is_some());
    }

    #[test]
    fn test_no_inheritance_cycle() {
        let ontology = OntologyModel {
            id: "test".into(),
            name: "Test".into(),
            version: "1.0".into(),
            domains: vec![
                DomainOntology {
                    id: "a".into(),
                    name: "A".into(),
                    kind: DomainKind::Entity,
                    parent_ids: vec!["b".into()],
                    ..Default::default()
                },
                DomainOntology {
                    id: "b".into(),
                    name: "B".into(),
                    kind: DomainKind::Entity,
                    parent_ids: vec!["c".into()],
                    ..Default::default()
                },
                DomainOntology {
                    id: "c".into(),
                    name: "C".into(),
                    kind: DomainKind::Entity,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let cycle = detect_inheritance_cycle(&ontology);
        assert!(cycle.is_none());
    }

    #[test]
    fn test_fuzzy_match() {
        let candidates: Vec<String> = vec![
            "purchase_order".into(),
            "purchase_line_item".into(),
            "inventory".into(),
        ];

        let result = fuzzy_match("purchase_order", &candidates);
        assert_eq!(result, Some("purchase_order".into()));

        let result = fuzzy_match("PurchaseOrder", &candidates);
        assert_eq!(result, Some("purchase_order".into()));

        let result = fuzzy_match("unknown_entity", &candidates);
        assert_eq!(result, None);
    }

    #[test]
    fn test_fix_missing_cardinalities() {
        let mut output = OntologyOutput {
            ontology: OntologyModel {
                id: "test".into(),
                name: "Test".into(),
                version: "1.0".into(),
                domains: vec![DomainOntology {
                    id: "order".into(),
                    name: "订单".into(),
                    kind: DomainKind::Entity,
                    properties: vec![OntologyProperty {
                        id: "order_number".into(),
                        name: "订单号".into(),
                        property_type: PropertyType::DataProperty,
                        required: true,
                        cardinality: Cardinality::default(), // all None
                        domain: "order".into(),
                        range: "String".into(),
                        is_functional: true,
                        is_transitive: false,
                        is_symmetric: false,
                        constraints: vec![],
                        semantic_description: None,
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            },
            ..Default::default()
        };

        let mut fix_log = vec![];
        fix_missing_cardinalities(&mut output, &mut fix_log);
        assert_eq!(
            output.ontology.domains[0].properties[0].cardinality.min,
            Some(1)
        );
    }
}

/// 从用户自然语言描述中提取语义概念（简化规则，无需 LLM）
pub fn extract_semantic_concepts(description: &str) -> Vec<String> {
    let keywords = [
        "采购",
        "库存",
        "销售",
        "订单",
        "客户",
        "供应商",
        "产品",
        "物流",
        "仓储",
        "运输",
        "财务",
        "审批",
        "合同",
        "报表",
        "工资",
        "管理",
        "入库",
        "出库",
        "盘点",
        "退货",
        "缴费",
        "申请",
        "验收",
        "检验",
        "计量",
        "质量",
        "工单",
        "维修",
        "计划",
        "排产",
        "调度",
        "监控",
    ];
    let desc_lower = description.to_lowercase();
    keywords
        .iter()
        .filter(|kw| desc_lower.contains(&kw.to_lowercase()))
        .map(|kw| kw.to_string())
        .collect()
}
