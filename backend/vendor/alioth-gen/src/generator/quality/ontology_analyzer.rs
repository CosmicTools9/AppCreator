//! Ontology Quality Analyzer
//!
//! 分析本体模型的质量指标：
//! - 文档覆盖率
//! - 约束覆盖率
//! - 继承层次分析
//! - 孤立实体检测
//! - 命名规范检查

use crate::generator::ir::quality::OntologyQualityMetrics;
use crate::generator::ir::GeneratorModel;

/// Analyze model quality and return metrics
pub fn analyze_model_quality(model: &GeneratorModel) -> OntologyQualityMetrics {
    OntologyQualityMetrics {
        max_hierarchy_depth: calculate_max_hierarchy_depth(model),
        avg_hierarchy_depth: calculate_avg_hierarchy_depth(model),
        classes_without_properties: count_classes_without_properties(model),
        properties_without_constraints: count_properties_without_constraints(model),
        documentation_coverage: calculate_documentation_coverage(model),
        constraint_coverage: calculate_constraint_coverage(model),
        quality_rule_coverage: calculate_quality_rule_coverage(model),
    }
}

/// Calculate maximum class hierarchy depth
fn calculate_max_hierarchy_depth(model: &GeneratorModel) -> u32 {
    model
        .entities
        .iter()
        .map(|e| e.inheritance_depth)
        .max()
        .unwrap_or(0)
}

/// Calculate average class hierarchy depth
fn calculate_avg_hierarchy_depth(model: &GeneratorModel) -> f64 {
    if model.entities.is_empty() {
        return 0.0;
    }

    let total: u32 = model.entities.iter().map(|e| e.inheritance_depth).sum();

    total as f64 / model.entities.len() as f64
}

/// Count classes without any properties
fn count_classes_without_properties(model: &GeneratorModel) -> usize {
    model
        .entities
        .iter()
        .filter(|e| e.fields.is_empty() && e.relations.is_empty())
        .count()
}

/// Count properties without constraints
fn count_properties_without_constraints(model: &GeneratorModel) -> usize {
    model
        .entities
        .iter()
        .flat_map(|e| &e.fields)
        .filter(|f| f.constraints.is_empty() && f.validations.is_empty())
        .count()
}

/// Calculate documentation coverage percentage
fn calculate_documentation_coverage(model: &GeneratorModel) -> f64 {
    let mut total_items = 0;
    let mut documented_items = 0;

    // Count entities
    for entity in &model.entities {
        total_items += 1;
        if entity.description.is_some() {
            documented_items += 1;
        }

        // Count fields
        for field in &entity.fields {
            total_items += 1;
            if field.description.is_some() {
                documented_items += 1;
            }
        }
    }

    if total_items == 0 {
        return 1.0;
    }

    documented_items as f64 / total_items as f64
}

/// Calculate constraint coverage percentage
fn calculate_constraint_coverage(model: &GeneratorModel) -> f64 {
    let mut total_fields = 0;
    let mut constrained_fields = 0;

    for entity in &model.entities {
        for field in &entity.fields {
            total_fields += 1;

            // Check if field has any constraints
            if !field.constraints.is_empty()
                || !field.validations.is_empty()
                || field.unique
                || !field.nullable
                || field.is_functional
            {
                constrained_fields += 1;
            }
        }
    }

    if total_fields == 0 {
        return 1.0;
    }

    constrained_fields as f64 / total_fields as f64
}

/// Calculate quality rule coverage percentage
fn calculate_quality_rule_coverage(model: &GeneratorModel) -> f64 {
    let mut total_entities = 0;
    let mut entities_with_quality_rules = 0;

    for entity in &model.entities {
        total_entities += 1;

        let has_entity_rules = !entity.quality_rules.is_empty();
        let has_field_rules = entity.fields.iter().any(|f| !f.quality_rules.is_empty());

        if has_entity_rules || has_field_rules {
            entities_with_quality_rules += 1;
        }
    }

    if total_entities == 0 {
        return 1.0;
    }

    entities_with_quality_rules as f64 / total_entities as f64
}

/// Detect orphan entities (entities with no relations to/from other entities)
pub fn detect_orphan_entities(model: &GeneratorModel) -> Vec<String> {
    let mut orphan_entities = Vec::new();

    // Collect all entities that are referenced by relations
    let referenced_entities: std::collections::HashSet<_> = model
        .entities
        .iter()
        .flat_map(|e| &e.relations)
        .map(|r| &r.target_entity)
        .cloned()
        .collect();

    // Collect all entities that have relations
    let entities_with_relations: std::collections::HashSet<_> = model
        .entities
        .iter()
        .filter(|e| !e.relations.is_empty())
        .map(|e| &e.name.raw)
        .cloned()
        .collect();

    for entity in &model.entities {
        let is_referenced = referenced_entities.contains(&entity.name.raw);
        let has_relations = entities_with_relations.contains(&entity.name.raw);

        // An entity is an orphan if it's not referenced and has no outgoing relations
        // (except for root/parent entities which may be intentionally isolated)
        if !is_referenced && !has_relations && entity.parent_classes.is_empty() {
            orphan_entities.push(entity.name.raw.clone());
        }
    }

    orphan_entities
}

/// Detect deep inheritance hierarchies (warning if > 3 levels)
pub fn detect_deep_hierarchies(model: &GeneratorModel, threshold: u32) -> Vec<(String, u32)> {
    model
        .entities
        .iter()
        .filter(|e| e.inheritance_depth > threshold)
        .map(|e| (e.name.raw.clone(), e.inheritance_depth))
        .collect()
}

/// Check naming conventions
pub fn check_naming_conventions(model: &GeneratorModel) -> NamingConventionReport {
    NamingConventionReport {
        entity_violations: check_entity_naming(model),
        field_violations: check_field_naming(model),
    }
}

/// Check entity naming conventions (PascalCase)
fn check_entity_naming(model: &GeneratorModel) -> Vec<NamingViolation> {
    let mut violations = Vec::new();

    for entity in &model.entities {
        if !is_pascal_case(&entity.name.raw) {
            violations.push(NamingViolation {
                item_type: "entity".to_string(),
                name: entity.name.raw.clone(),
                expected_format: "PascalCase".to_string(),
                suggestion: to_pascal_case(&entity.name.raw),
            });
        }
    }

    violations
}

/// Check field naming conventions (camelCase)
fn check_field_naming(model: &GeneratorModel) -> Vec<NamingViolation> {
    let mut violations = Vec::new();

    for entity in &model.entities {
        for field in &entity.fields {
            if !is_camel_case(&field.name.raw) && !is_snake_case(&field.name.raw) {
                violations.push(NamingViolation {
                    item_type: "field".to_string(),
                    name: format!("{}.{}", entity.name.raw, field.name.raw),
                    expected_format: "camelCase or snake_case".to_string(),
                    suggestion: to_camel_case(&field.name.raw),
                });
            }
        }
    }

    violations
}

/// Check if string is PascalCase
fn is_pascal_case(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }

    let first_char = s.chars().next().unwrap();
    first_char.is_uppercase() && !s.contains('_') && !s.contains('-')
}

/// Check if string is camelCase
fn is_camel_case(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }

    let first_char = s.chars().next().unwrap();
    first_char.is_lowercase() && !s.contains('_') && !s.contains('-')
}

/// Check if string is snake_case
fn is_snake_case(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }

    s.chars()
        .all(|c| c.is_lowercase() || c == '_' || c.is_numeric())
        && !s.starts_with('_')
        && !s.ends_with('_')
}

/// Convert string to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect()
}

/// Convert string to camelCase
fn to_camel_case(s: &str) -> String {
    let pascal = to_pascal_case(s);
    if pascal.is_empty() {
        return pascal;
    }

    let mut chars = pascal.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
    }
}

/// Naming convention violation
#[derive(Debug, Clone)]
pub struct NamingViolation {
    pub item_type: String,
    pub name: String,
    pub expected_format: String,
    pub suggestion: String,
}

/// Naming convention report
#[derive(Debug, Clone)]
pub struct NamingConventionReport {
    pub entity_violations: Vec<NamingViolation>,
    pub field_violations: Vec<NamingViolation>,
}

/// Format ontology metrics for display
pub fn format_metrics(metrics: &OntologyQualityMetrics) -> String {
    let mut output = String::new();

    output.push_str("# 本体质量分析报告\n\n");
    output.push_str(&format!(
        "**生成时间**: {}\n\n",
        chrono::Utc::now().to_rfc3339()
    ));

    // Overall score
    output.push_str("## 总体评分\n\n");
    output.push_str(&format!(
        "**质量评分**: {:.1}%\n\n",
        metrics.overall_score() * 100.0
    ));
    output.push_str(&format!(
        "**等级**: {}\n\n",
        calculate_grade(metrics.overall_score())
    ));

    // Metrics table
    output.push_str("## 详细指标\n\n");
    output.push_str("| 指标 | 值 | 状态 |\n");
    output.push_str("|------|-----|------|\n");

    output.push_str(&format!(
        "| 文档覆盖率 | {:.1}% | {} |\n",
        metrics.documentation_coverage * 100.0,
        get_status_indicator(metrics.documentation_coverage, 0.8)
    ));

    output.push_str(&format!(
        "| 约束覆盖率 | {:.1}% | {} |\n",
        metrics.constraint_coverage * 100.0,
        get_status_indicator(metrics.constraint_coverage, 0.8)
    ));

    output.push_str(&format!(
        "| 质量规则覆盖率 | {:.1}% | {} |\n",
        metrics.quality_rule_coverage * 100.0,
        get_status_indicator(metrics.quality_rule_coverage, 0.5)
    ));

    output.push_str(&format!(
        "| 最大继承深度 | {} | {} |\n",
        metrics.max_hierarchy_depth,
        if metrics.max_hierarchy_depth > 3 {
            "⚠️"
        } else {
            "✅"
        }
    ));

    output.push_str(&format!(
        "| 平均继承深度 | {:.1} | {} |\n",
        metrics.avg_hierarchy_depth,
        if metrics.avg_hierarchy_depth > 2.0 {
            "⚠️"
        } else {
            "✅"
        }
    ));

    output.push_str(&format!(
        "| 无属性类数量 | {} | {} |\n",
        metrics.classes_without_properties,
        if metrics.classes_without_properties > 0 {
            "⚠️"
        } else {
            "✅"
        }
    ));

    output.push_str(&format!(
        "| 无约束属性数量 | {} | {} |\n\n",
        metrics.properties_without_constraints,
        if metrics.properties_without_constraints > 0 {
            "⚠️"
        } else {
            "✅"
        }
    ));

    // Recommendations
    output.push_str("## 改进建议\n\n");

    if metrics.documentation_coverage < 0.8 {
        output.push_str("- **提高文档覆盖率**: 为实体和字段添加描述信息\n");
    }
    if metrics.constraint_coverage < 0.8 {
        output.push_str("- **增加约束定义**: 为字段添加验证规则\n");
    }
    if metrics.quality_rule_coverage < 0.5 {
        output.push_str("- **配置质量规则**: 为关键实体添加数据质量验证\n");
    }
    if metrics.max_hierarchy_depth > 3 {
        output.push_str("- **简化继承层次**: 考虑减少继承深度以提高可维护性\n");
    }
    if metrics.classes_without_properties > 0 {
        output.push_str("- **完善类定义**: 为无属性的类添加适当的字段\n");
    }

    output.push_str("\n---\n\n*由 AliothStudio 自动生成*\n");

    output
}

/// Get status indicator emoji
fn get_status_indicator(value: f64, threshold: f64) -> &'static str {
    if value >= threshold {
        "✅"
    } else if value >= threshold * 0.8 {
        "⚠️"
    } else {
        "❌"
    }
}

/// Calculate grade from score
fn calculate_grade(score: f64) -> &'static str {
    match score {
        s if s >= 0.95 => "A",
        s if s >= 0.90 => "B",
        s if s >= 0.80 => "C",
        s if s >= 0.70 => "D",
        _ => "F",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::{GeneratorEntity, GeneratorField, GeneratorFieldType};

    fn create_test_model() -> GeneratorModel {
        GeneratorModel {
            i18n_config: None,
            entities: vec![GeneratorEntity {
                name: crate::generator::ir::EntityName {
                    raw: "Customer".to_string(),
                    snake: "customer".to_string(),
                    camel: "customer".to_string(),
                    pascal: "Customer".to_string(),
                    kebab: "customer".to_string(),
                    screaming_snake: "CUSTOMER".to_string(),
                    plural_snake: "customers".to_string(),
                    plural_pascal: "Customers".to_string(),
                    plural_kebab: "customers".to_string(),
                },
                description: Some("A customer".to_string()),
                fields: vec![GeneratorField {
                    name: crate::generator::ir::FieldName {
                        raw: "name".to_string(),
                        snake: "name".to_string(),
                        camel: "name".to_string(),
                        pascal: "Name".to_string(),
                    },
                    field_type: GeneratorFieldType::Text,
                    description: Some("Customer name".to_string()),
                    nullable: false,
                    unique: false,
                    indexed: false,
                    default_value: None,
                    validations: vec![],
                    annotations: vec![],
                    domain: None,
                    range: None,
                    min_cardinality: None,
                    max_cardinality: None,
                    is_functional: false,
                    constraints: vec![],
                    throws_clauses: vec![],
                    quality_rules: vec![],
                }],
                relations: vec![],
                annotations: vec![],
                primary_key_type: crate::generator::ir::PrimaryKeyType::BigInt,
                parent_classes: vec![],
                equivalent_classes: vec![],
                disjoint_classes: vec![],
                is_abstract: false,
                inheritance_depth: 0,
                state_machine: Default::default(),
                transitions: vec![],
                lifecycle_hooks: vec![],
                business_rules: vec![],
                swrl_rules: vec![],
                constraints: vec![],
                quality_rules: vec![],
                quality_config: Default::default(),
                parent_tables: vec![],
            }],
            enums: vec![],
            metadata: Default::default(),
            exceptions: vec![],
            exception_handlers: vec![],
            external_dependencies: vec![],
        }
    }

    #[test]
    fn test_analyze_model_quality() {
        let model = create_test_model();
        let metrics = analyze_model_quality(&model);

        assert_eq!(metrics.max_hierarchy_depth, 0);
        assert_eq!(metrics.classes_without_properties, 0);
        assert!(metrics.documentation_coverage > 0.0);
    }

    #[test]
    fn test_is_pascal_case() {
        assert!(is_pascal_case("Customer"));
        assert!(is_pascal_case("CustomerOrder"));
        assert!(!is_pascal_case("customer"));
        assert!(!is_pascal_case("customer_order"));
    }

    #[test]
    fn test_is_camel_case() {
        assert!(is_camel_case("customer"));
        assert!(is_camel_case("customerName"));
        assert!(!is_camel_case("Customer"));
        assert!(!is_camel_case("customer_name"));
    }

    #[test]
    fn test_is_snake_case() {
        assert!(is_snake_case("customer_name"));
        assert!(is_snake_case("customer"));
        assert!(!is_snake_case("CustomerName"));
        assert!(!is_snake_case("_customer"));
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("customer"), "Customer");
        assert_eq!(to_pascal_case("customer_name"), "CustomerName");
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("customer"), "customer");
        assert_eq!(to_camel_case("customer_name"), "customerName");
    }
}
