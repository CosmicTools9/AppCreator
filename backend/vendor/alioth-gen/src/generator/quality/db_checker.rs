//! Database Quality Checker
//!
//! 生成 SQL 查询用于数据库质量检查：
//! - 完整性检查（NULL 值统计）
//! - 准确性检查（范围验证）
//! - 唯一性检查（重复值检测）
//! - 时效性检查（数据新鲜度）
//! - 一致性检查（外键完整性）

use crate::generator::ir::quality::{QualityCheckSql, QualityMetric, QualityResultType};
use crate::generator::ir::{GeneratorEntity, GeneratorField, GeneratorFieldType, GeneratorModel};

/// Generate quality check SQL queries for the entire model
pub fn generate_quality_check_sql(model: &GeneratorModel) -> Vec<QualityCheckSql> {
    let mut checks = Vec::new();

    for entity in &model.entities {
        // Generate entity-level checks
        checks.extend(generate_entity_checks(entity));

        // Generate field-level checks
        for field in &entity.fields {
            checks.extend(generate_field_checks(entity, field));
        }
    }

    checks
}

/// Generate SQL checks for entity-level quality rules
fn generate_entity_checks(entity: &GeneratorEntity) -> Vec<QualityCheckSql> {
    let mut checks = Vec::new();

    for rule in &entity.quality_rules {
        if let Some(check) = generate_entity_rule_check(entity, rule) {
            checks.push(check);
        }
    }

    // Add default completeness check if no explicit rules
    if !entity
        .quality_rules
        .iter()
        .any(|r| r.metric == QualityMetric::Completeness)
    {
        checks.push(generate_default_completeness_check(entity));
    }

    checks
}

/// Generate SQL check for a specific entity rule
fn generate_entity_rule_check(
    entity: &GeneratorEntity,
    rule: &crate::generator::ir::quality::GeneratorQualityRule,
) -> Option<QualityCheckSql> {
    let table_name = &entity.name.snake;

    match rule.metric {
        QualityMetric::Completeness => Some(QualityCheckSql {
            name: format!("{}_completeness", table_name),
            metric: QualityMetric::Completeness,
            sql: format!(
                r#"-- Calculate completeness score for {table}
WITH stats AS (
    SELECT 
        COUNT(*) as total_records,
        {columns}
    FROM {table}
)
SELECT 
    CASE 
        WHEN total_records = 0 THEN 1.0
        ELSE ({non_null_checks})::float / (total_records * {column_count})::float
    END as completeness_score
FROM stats"#,
                table = table_name,
                columns = generate_column_null_checks(entity),
                non_null_checks = generate_non_null_count(entity),
                column_count = entity.fields.len().max(1)
            ),
            result_type: QualityResultType::Percentage,
            threshold: rule.threshold,
        }),

        QualityMetric::Timeliness => Some(QualityCheckSql {
            name: format!("{}_timeliness", table_name),
            metric: QualityMetric::Timeliness,
            sql: format!(
                r#"-- Check data freshness for {table}
SELECT 
    CASE 
        WHEN COUNT(*) = 0 THEN 1.0
        ELSE COUNT(CASE WHEN updated_at > NOW() - INTERVAL '7 days' THEN 1 END)::float 
            / COUNT(*)::float
    END as freshness_score
FROM {table}"#,
                table = table_name
            ),
            result_type: QualityResultType::Percentage,
            threshold: rule.threshold,
        }),

        QualityMetric::Consistency => Some(QualityCheckSql {
            name: format!("{}_consistency", table_name),
            metric: QualityMetric::Consistency,
            sql: format!(
                r#"-- Check referential integrity for {table}
SELECT 
    CASE 
        WHEN total_refs = 0 OR total_refs IS NULL THEN 1.0
        ELSE valid_refs::float / total_refs::float
    END as consistency_score
FROM (
    SELECT 
        COUNT(*) as total_refs,
        COUNT(CASE WHEN ref.id IS NOT NULL THEN 1 END) as valid_refs
    FROM {table} t
    {joins}
) subq"#,
                table = table_name,
                joins = generate_referential_joins(entity)
            ),
            result_type: QualityResultType::Percentage,
            threshold: rule.threshold,
        }),

        _ => None,
    }
}

/// Generate SQL checks for field-level quality rules
fn generate_field_checks(entity: &GeneratorEntity, field: &GeneratorField) -> Vec<QualityCheckSql> {
    let mut checks = Vec::new();

    for rule in &field.quality_rules {
        if let Some(check) = generate_field_rule_check(entity, field, rule) {
            checks.push(check);
        }
    }

    // Add default checks based on field properties
    if !field.nullable
        && !field
            .quality_rules
            .iter()
            .any(|r| r.metric == QualityMetric::Completeness)
    {
        checks.push(generate_field_completeness_check(entity, field));
    }

    if field.unique
        && !field
            .quality_rules
            .iter()
            .any(|r| r.metric == QualityMetric::Uniqueness)
    {
        checks.push(generate_field_uniqueness_check(entity, field));
    }

    checks
}

/// Generate SQL check for a specific field rule
fn generate_field_rule_check(
    entity: &GeneratorEntity,
    field: &GeneratorField,
    rule: &crate::generator::ir::quality::GeneratorQualityRule,
) -> Option<QualityCheckSql> {
    let table_name = &entity.name.snake;
    let column_name = &field.name.snake;

    match rule.metric {
        QualityMetric::Completeness => Some(QualityCheckSql {
            name: format!("{}_{}_completeness", table_name, column_name),
            metric: QualityMetric::Completeness,
            sql: format!(
                r#"-- Check completeness for {table}.{column}
SELECT 
    CASE 
        WHEN COUNT(*) = 0 THEN 1.0
        ELSE COUNT(CASE WHEN {column} IS NOT NULL AND {column}::text != '' THEN 1 END)::float 
            / COUNT(*)::float
    END as completeness_score
FROM {table}"#,
                table = table_name,
                column = column_name
            ),
            result_type: QualityResultType::Percentage,
            threshold: rule.threshold,
        }),

        QualityMetric::Uniqueness => Some(QualityCheckSql {
            name: format!("{}_{}_uniqueness", table_name, column_name),
            metric: QualityMetric::Uniqueness,
            sql: format!(
                r#"-- Check uniqueness for {table}.{column}
SELECT 
    CASE 
        WHEN COUNT(*) = 0 THEN 1.0
        ELSE COUNT(DISTINCT {column})::float / COUNT(*)::float
    END as uniqueness_score
FROM {table}
WHERE {column} IS NOT NULL"#,
                table = table_name,
                column = column_name
            ),
            result_type: QualityResultType::Percentage,
            threshold: rule.threshold,
        }),

        QualityMetric::Accuracy => {
            let condition = generate_accuracy_condition(field);
            Some(QualityCheckSql {
                name: format!("{}_{}_accuracy", table_name, column_name),
                metric: QualityMetric::Accuracy,
                sql: format!(
                    r#"-- Check accuracy for {table}.{column}
SELECT 
    CASE 
        WHEN COUNT(*) = 0 THEN 1.0
        ELSE COUNT(CASE WHEN {condition} THEN 1 END)::float / COUNT(*)::float
    END as accuracy_score
FROM {table}
WHERE {column} IS NOT NULL"#,
                    table = table_name,
                    column = column_name,
                    condition = condition
                ),
                result_type: QualityResultType::Percentage,
                threshold: rule.threshold,
            })
        }

        QualityMetric::Validity => {
            let condition = generate_validity_condition(field);
            Some(QualityCheckSql {
                name: format!("{}_{}_validity", table_name, column_name),
                metric: QualityMetric::Validity,
                sql: format!(
                    r#"-- Check validity for {table}.{column}
SELECT 
    CASE 
        WHEN COUNT(*) = 0 THEN 1.0
        ELSE COUNT(CASE WHEN {condition} THEN 1 END)::float / COUNT(*)::float
    END as validity_score
FROM {table}
WHERE {column} IS NOT NULL"#,
                    table = table_name,
                    column = column_name,
                    condition = condition
                ),
                result_type: QualityResultType::Percentage,
                threshold: rule.threshold,
            })
        }

        _ => None,
    }
}

/// Generate default completeness check for entity
fn generate_default_completeness_check(entity: &GeneratorEntity) -> QualityCheckSql {
    let table_name = &entity.name.snake;

    QualityCheckSql {
        name: format!("{}_default_completeness", table_name),
        metric: QualityMetric::Completeness,
        sql: format!(
            r#"-- Default completeness check for {table}
SELECT 
    CASE 
        WHEN COUNT(*) = 0 THEN 1.0
        ELSE COUNT(CASE WHEN {conditions} THEN 1 END)::float / COUNT(*)::float
    END as completeness_score
FROM {table}"#,
            table = table_name,
            conditions = entity
                .fields
                .iter()
                .filter(|f| !f.nullable)
                .map(|f| format!("{} IS NOT NULL", f.name.snake))
                .collect::<Vec<_>>()
                .join(" AND ")
        ),
        result_type: QualityResultType::Percentage,
        threshold: 0.95,
    }
}

/// Generate completeness check for field
fn generate_field_completeness_check(
    entity: &GeneratorEntity,
    field: &GeneratorField,
) -> QualityCheckSql {
    QualityCheckSql {
        name: format!(
            "{}_{}_default_completeness",
            entity.name.snake, field.name.snake
        ),
        metric: QualityMetric::Completeness,
        sql: format!(
            r#"-- Completeness check for {table}.{column}
SELECT 
    CASE 
        WHEN COUNT(*) = 0 THEN 1.0
        ELSE COUNT(CASE WHEN {column} IS NOT NULL THEN 1 END)::float / COUNT(*)::float
    END as completeness_score
FROM {table}"#,
            table = entity.name.snake,
            column = field.name.snake
        ),
        result_type: QualityResultType::Percentage,
        threshold: 1.0, // Required field
    }
}

/// Generate uniqueness check for field
fn generate_field_uniqueness_check(
    entity: &GeneratorEntity,
    field: &GeneratorField,
) -> QualityCheckSql {
    QualityCheckSql {
        name: format!(
            "{}_{}_default_uniqueness",
            entity.name.snake, field.name.snake
        ),
        metric: QualityMetric::Uniqueness,
        sql: format!(
            r#"-- Uniqueness check for {table}.{column}
SELECT 
    CASE 
        WHEN COUNT(*) = 0 THEN 1.0
        ELSE COUNT(DISTINCT {column})::float / COUNT(*)::float
    END as uniqueness_score
FROM {table}
WHERE {column} IS NOT NULL"#,
            table = entity.name.snake,
            column = field.name.snake
        ),
        result_type: QualityResultType::Percentage,
        threshold: 1.0, // Unique field
    }
}

/// Generate SQL condition for accuracy check based on field type
fn generate_accuracy_condition(field: &GeneratorField) -> String {
    match field.field_type {
        GeneratorFieldType::Integer | GeneratorFieldType::BigInt => {
            format!("{} >= 0", field.name.snake)
        }
        GeneratorFieldType::Decimal => {
            format!("{} >= 0.0", field.name.snake)
        }
        GeneratorFieldType::Text => {
            format!(
                "LENGTH({}) > 0 AND LENGTH({}) <= 255",
                field.name.snake, field.name.snake
            )
        }
        _ => format!("{} IS NOT NULL", field.name.snake),
    }
}

/// Generate SQL condition for validity check based on field type
fn generate_validity_condition(field: &GeneratorField) -> String {
    match field.field_type {
        GeneratorFieldType::Text => {
            if field.name.snake.contains("email") {
                format!(
                    "{} ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\\.[A-Za-z]{{2,}}$'",
                    field.name.snake
                )
            } else if field.name.snake.contains("phone") {
                format!("{} ~* '^[\\d\\s\\-+()]+$'", field.name.snake)
            } else {
                format!("LENGTH(TRIM({})) > 0", field.name.snake)
            }
        }
        GeneratorFieldType::Uuid => {
            format!(
                "{} ~* '^[0-9a-f]{{8}}-[0-9a-f]{{4}}-[0-9a-f]{{4}}-[0-9a-f]{{4}}-[0-9a-f]{{12}}$'",
                field.name.snake
            )
        }
        _ => format!("{} IS NOT NULL", field.name.snake),
    }
}

/// Generate column null checks for completeness calculation
fn generate_column_null_checks(entity: &GeneratorEntity) -> String {
    entity
        .fields
        .iter()
        .map(|f| {
            format!(
                "COUNT(CASE WHEN {} IS NOT NULL THEN 1 END) as {}_not_null",
                f.name.snake, f.name.snake
            )
        })
        .collect::<Vec<_>>()
        .join(",\n        ")
}

/// Generate non-null count expression
fn generate_non_null_count(entity: &GeneratorEntity) -> String {
    entity
        .fields
        .iter()
        .map(|f| format!("{}_not_null", f.name.snake))
        .collect::<Vec<_>>()
        .join(" + ")
}

/// Generate referential integrity join conditions
fn generate_referential_joins(entity: &GeneratorEntity) -> String {
    entity
        .relations
        .iter()
        .enumerate()
        .map(|(i, rel)| {
            format!(
                "LEFT JOIN {} ref{} ON t.{}_id = ref{}.id",
                rel.target_entity.to_lowercase(),
                i,
                rel.name.to_lowercase(),
                i
            )
        })
        .collect::<Vec<_>>()
        .join("\n    ")
}

/// Generate sampling query for large tables
pub fn generate_sampling_query(entity: &GeneratorEntity, sample_size: usize) -> String {
    format!(
        r#"-- Sampling query for {table}
SELECT *
FROM {table}
ORDER BY RANDOM()
LIMIT {sample_size}"#,
        table = entity.name.snake,
        sample_size = sample_size
    )
}

/// Generate duplicate detection query
pub fn generate_duplicate_detection_query(
    entity: &GeneratorEntity,
    field_names: &[String],
) -> String {
    let columns = field_names.join(", ");
    let group_by = field_names.join(", ");

    format!(
        r#"-- Duplicate detection for {table} on columns: {columns}
SELECT {columns}, COUNT(*) as duplicate_count
FROM {table}
GROUP BY {group_by}
HAVING COUNT(*) > 1
ORDER BY duplicate_count DESC"#,
        table = entity.name.snake,
        columns = columns,
        group_by = group_by
    )
}

/// Generate outlier detection query for numeric fields
pub fn generate_outlier_detection_query(
    entity: &GeneratorEntity,
    field: &GeneratorField,
) -> String {
    format!(
        r#"-- Outlier detection for {table}.{column}
WITH stats AS (
    SELECT 
        AVG({column}) as avg_val,
        STDDEV({column}) as stddev_val
    FROM {table}
    WHERE {column} IS NOT NULL
)
SELECT t.*
FROM {table} t
CROSS JOIN stats s
WHERE t.{column} IS NOT NULL
  AND ABS(t.{column} - s.avg_val) > 3 * s.stddev_val"#,
        table = entity.name.snake,
        column = field.name.snake
    )
}

/// Generate data profiling query for a field
pub fn generate_profiling_query(entity: &GeneratorEntity, field: &GeneratorField) -> String {
    match field.field_type {
        GeneratorFieldType::Integer | GeneratorFieldType::BigInt | GeneratorFieldType::Decimal => {
            format!(
                r#"-- Data profiling for {table}.{column}
SELECT 
    COUNT(*) as total_count,
    COUNT(DISTINCT {column}) as distinct_count,
    COUNT(CASE WHEN {column} IS NULL THEN 1 END) as null_count,
    MIN({column}) as min_value,
    MAX({column}) as max_value,
    AVG({column})::numeric(10,2) as avg_value,
    PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY {column})::numeric(10,2) as median_value
FROM {table}"#,
                table = entity.name.snake,
                column = field.name.snake
            )
        }
        _ => {
            format!(
                r#"-- Data profiling for {table}.{column}
SELECT 
    COUNT(*) as total_count,
    COUNT(DISTINCT {column}) as distinct_count,
    COUNT(CASE WHEN {column} IS NULL THEN 1 END) as null_count,
    MIN(LENGTH({column}::text)) as min_length,
    MAX(LENGTH({column}::text)) as max_length,
    AVG(LENGTH({column}::text))::numeric(10,2) as avg_length
FROM {table}"#,
                table = entity.name.snake,
                column = field.name.snake
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entity() -> GeneratorEntity {
        GeneratorEntity {
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
            description: None,
            fields: vec![GeneratorField {
                name: crate::generator::ir::FieldName {
                    raw: "email".to_string(),
                    snake: "email".to_string(),
                    camel: "email".to_string(),
                    pascal: "Email".to_string(),
                },
                field_type: GeneratorFieldType::Text,
                description: None,
                nullable: false,
                unique: true,
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
        }
    }

    #[test]
    fn test_generate_completeness_check() {
        let entity = create_test_entity();
        let checks = generate_entity_checks(&entity);

        assert!(!checks.is_empty());
        assert!(checks
            .iter()
            .any(|c| c.metric == QualityMetric::Completeness));
    }

    #[test]
    fn test_generate_field_uniqueness_check() {
        let entity = create_test_entity();
        let field = &entity.fields[0];

        let check = generate_field_uniqueness_check(&entity, field);

        assert_eq!(check.metric, QualityMetric::Uniqueness);
        assert_eq!(check.threshold, 1.0);
        assert!(check.sql.contains("COUNT(DISTINCT"));
    }

    #[test]
    fn test_generate_accuracy_condition() {
        let field = GeneratorField {
            name: crate::generator::ir::FieldName {
                raw: "age".to_string(),
                snake: "age".to_string(),
                camel: "age".to_string(),
                pascal: "Age".to_string(),
            },
            field_type: GeneratorFieldType::Integer,
            description: None,
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
        };

        let condition = generate_accuracy_condition(&field);
        assert!(condition.contains(">= 0"));
    }

    #[test]
    fn test_generate_duplicate_detection_query() {
        let entity = create_test_entity();
        let query = generate_duplicate_detection_query(&entity, &["email".to_string()]);

        assert!(query.contains("GROUP BY"));
        assert!(query.contains("HAVING COUNT(*) > 1"));
    }
}
