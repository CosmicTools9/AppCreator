//! Quality Validation Engine
//!
//! 提供数据质量验证的核心逻辑：
//! - 完整性检查 (Completeness)
//! - 准确性检查 (Accuracy)
//! - 一致性检查 (Consistency)
//! - 时效性检查 (Timeliness)
//! - 有效性检查 (Validity)
//! - 唯一性检查 (Uniqueness)

use crate::generator::ir::quality::{
    GeneratorQualityRule, QualityDimension, QualityMetric, QualityRecommendation, QualityReport,
    QualityScore, QualitySummary, QualityViolation, ViolationSeverity,
};
use crate::generator::ir::{GeneratorEntity, GeneratorField, GeneratorFieldType};
use crate::generator::GenerateError;

/// Validate entity quality and generate report
pub fn validate_entity_quality(entity: &GeneratorEntity) -> QualityReport {
    let mut violations = Vec::new();

    // Validate entity-level quality rules
    for rule in &entity.quality_rules {
        if rule.active {
            let violation = validate_entity_rule(entity, rule);
            if let Some(v) = violation {
                violations.push(v);
            }
        }
    }

    // Validate field-level quality rules
    for field in &entity.fields {
        for rule in &field.quality_rules {
            if rule.active {
                let violation = validate_field_rule(entity, field, rule);
                if let Some(v) = violation {
                    violations.push(v);
                }
            }
        }
    }

    // Calculate dimension scores
    let dimension_scores = calculate_dimension_scores(entity, &violations);

    // Calculate overall score
    let overall_score = calculate_overall_score(&dimension_scores);

    // Generate summary
    let summary = generate_summary(&violations);

    // Generate recommendations
    let recommendations = generate_recommendations(entity, &violations, &dimension_scores);

    QualityReport {
        id: format!(
            "quality_{}_{}",
            entity.name.snake,
            chrono::Utc::now().timestamp()
        ),
        entity: entity.name.raw.clone(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        overall_score,
        dimension_scores,
        violations,
        summary,
        recommendations,
        risk_items: vec![],
    }
}

/// Validate an entity-level rule
fn validate_entity_rule(
    entity: &GeneratorEntity,
    rule: &GeneratorQualityRule,
) -> Option<QualityViolation> {
    // Simulate quality check (in real implementation, this would query the database)
    let actual_score = simulate_quality_check(entity, None, rule);

    if actual_score < rule.threshold {
        Some(QualityViolation {
            id: format!("vuln_{}_{}", entity.name.snake, rule.metric_fn_name()),
            metric: rule.metric,
            entity: entity.name.raw.clone(),
            field: None,
            message: rule.error_message.clone().unwrap_or_else(|| {
                format!(
                    "Entity {} {} is {:.1}%, below threshold {:.1}%",
                    entity.name.raw,
                    rule.metric.display_name(),
                    actual_score * 100.0,
                    rule.threshold * 100.0
                )
            }),
            severity: determine_severity(actual_score, rule.threshold),
            actual_value: actual_score,
            threshold: rule.threshold,
            samples: vec![],
            detected_at: chrono::Utc::now().to_rfc3339(),
        })
    } else {
        None
    }
}

/// Validate a field-level rule
fn validate_field_rule(
    entity: &GeneratorEntity,
    field: &GeneratorField,
    rule: &GeneratorQualityRule,
) -> Option<QualityViolation> {
    let actual_score = simulate_quality_check(entity, Some(field), rule);

    if actual_score < rule.threshold {
        Some(QualityViolation {
            id: format!(
                "vuln_{}_{}_{}",
                entity.name.snake,
                field.name.snake,
                rule.metric_fn_name()
            ),
            metric: rule.metric,
            entity: entity.name.raw.clone(),
            field: Some(field.name.raw.clone()),
            message: rule.error_message.clone().unwrap_or_else(|| {
                format!(
                    "Field {}.{} {} is {:.1}%, below threshold {:.1}%",
                    entity.name.raw,
                    field.name.raw,
                    rule.metric.display_name(),
                    actual_score * 100.0,
                    rule.threshold * 100.0
                )
            }),
            severity: determine_severity(actual_score, rule.threshold),
            actual_value: actual_score,
            threshold: rule.threshold,
            samples: generate_violation_samples(field, rule),
            detected_at: chrono::Utc::now().to_rfc3339(),
        })
    } else {
        None
    }
}

/// Simulate quality check (placeholder for actual database queries)
fn simulate_quality_check(
    _entity: &GeneratorEntity,
    field: Option<&GeneratorField>,
    rule: &GeneratorQualityRule,
) -> f64 {
    // In a real implementation, this would execute SQL queries to check data quality
    // For now, we return a simulated score based on the rule and field properties

    match rule.metric {
        QualityMetric::Completeness => {
            if let Some(f) = field {
                if f.nullable {
                    0.92 // Nullable fields typically have lower completeness
                } else {
                    0.98 // Required fields should have high completeness
                }
            } else {
                0.95
            }
        }
        QualityMetric::Accuracy => {
            if let Some(f) = field {
                match f.field_type {
                    GeneratorFieldType::Integer
                    | GeneratorFieldType::BigInt
                    | GeneratorFieldType::Decimal => 0.97,
                    _ => 0.94,
                }
            } else {
                0.95
            }
        }
        QualityMetric::Consistency => 0.93,
        QualityMetric::Timeliness => 0.88,
        QualityMetric::Validity => 0.96,
        QualityMetric::Uniqueness => {
            if let Some(f) = field {
                if f.unique {
                    1.0 // Unique fields should have 100% uniqueness
                } else {
                    0.85
                }
            } else {
                0.90
            }
        }
    }
}

/// Determine severity based on how much the score is below threshold
fn determine_severity(actual: f64, threshold: f64) -> ViolationSeverity {
    let gap = threshold - actual;

    if gap > 0.20 {
        ViolationSeverity::Critical
    } else if gap > 0.10 {
        ViolationSeverity::High
    } else if gap > 0.05 {
        ViolationSeverity::Medium
    } else if gap > 0.0 {
        ViolationSeverity::Low
    } else {
        ViolationSeverity::Info
    }
}

/// Generate sample violation data
fn generate_violation_samples(field: &GeneratorField, rule: &GeneratorQualityRule) -> Vec<String> {
    match rule.metric {
        QualityMetric::Completeness => {
            if field.nullable {
                vec!["null".to_string(), "".to_string()]
            } else {
                vec!["null".to_string()]
            }
        }
        QualityMetric::Uniqueness => {
            vec![
                "duplicate_value_1".to_string(),
                "duplicate_value_2".to_string(),
            ]
        }
        QualityMetric::Validity => {
            vec!["invalid_format".to_string()]
        }
        _ => vec![],
    }
}

/// Calculate scores for each quality dimension
fn calculate_dimension_scores(
    _entity: &GeneratorEntity,
    violations: &[QualityViolation],
) -> Vec<QualityScore> {
    let mut scores = Vec::new();

    for metric in [
        QualityMetric::Completeness,
        QualityMetric::Accuracy,
        QualityMetric::Consistency,
        QualityMetric::Timeliness,
        QualityMetric::Validity,
        QualityMetric::Uniqueness,
    ] {
        let dimension = QualityDimension::from_metric(metric);

        // Count violations for this metric (single pass, no allocation)
        let failed = violations.iter().filter(|v| v.metric == metric).count();
        let passed = if failed == 0 { 1 } else { 0 };

        // Calculate score based on rules and violations
        let score = if failed == 0 {
            1.0
        } else {
            let total_checks = passed + failed;
            let passed_checks = passed;
            passed_checks as f64 / total_checks as f64
        };

        scores.push(QualityScore::with_counts(dimension, score, passed, failed));
    }

    // Add overall score
    let overall = calculate_overall_score(&scores);
    scores.push(QualityScore::new(QualityDimension::Overall, overall));

    scores
}

/// Calculate overall quality score
fn calculate_overall_score(dimension_scores: &[QualityScore]) -> f64 {
    if dimension_scores.is_empty() {
        return 1.0;
    }

    let sum: f64 = dimension_scores.iter().map(|s| s.score).sum();
    sum / dimension_scores.len() as f64
}

/// Generate summary statistics
fn generate_summary(violations: &[QualityViolation]) -> QualitySummary {
    QualitySummary {
        total_checks: violations.len() + 1, // +1 for passed checks
        passed_checks: if violations.is_empty() { 1 } else { 0 },
        failed_checks: violations.len(),
        critical_count: violations
            .iter()
            .filter(|v| v.severity == ViolationSeverity::Critical)
            .count(),
        high_count: violations
            .iter()
            .filter(|v| v.severity == ViolationSeverity::High)
            .count(),
        medium_count: violations
            .iter()
            .filter(|v| v.severity == ViolationSeverity::Medium)
            .count(),
        low_count: violations
            .iter()
            .filter(|v| v.severity == ViolationSeverity::Low)
            .count(),
        info_count: violations
            .iter()
            .filter(|v| v.severity == ViolationSeverity::Info)
            .count(),
    }
}

/// Generate improvement recommendations
fn generate_recommendations(
    entity: &GeneratorEntity,
    violations: &[QualityViolation],
    _scores: &[QualityScore],
) -> Vec<QualityRecommendation> {
    let mut recommendations = Vec::new();

    for violation in violations {
        let rec = match violation.metric {
            QualityMetric::Completeness => QualityRecommendation {
                id: format!("rec_{}_completeness", entity.name.snake),
                priority: violation.severity.weight() * 10,
                title: format!("Improve {} completeness", entity.name.raw),
                description: format!(
                    "The {} field has {:.1}% completeness, below the required {:.1}%",
                    violation.field.as_deref().unwrap_or("entity"),
                    violation.actual_value * 100.0,
                    violation.threshold * 100.0
                ),
                metric: QualityMetric::Completeness,
                estimated_impact: violation.threshold - violation.actual_value,
                actions: vec![
                    "Review data entry processes".to_string(),
                    "Add required field validation".to_string(),
                    "Implement data quality monitoring".to_string(),
                ],
            },
            QualityMetric::Accuracy => QualityRecommendation {
                id: format!("rec_{}_accuracy", entity.name.snake),
                priority: violation.severity.weight() * 10,
                title: format!("Improve {} accuracy", entity.name.raw),
                description: format!(
                    "The {} field has {:.1}% accuracy, below the required {:.1}%",
                    violation.field.as_deref().unwrap_or("entity"),
                    violation.actual_value * 100.0,
                    violation.threshold * 100.0
                ),
                metric: QualityMetric::Accuracy,
                estimated_impact: violation.threshold - violation.actual_value,
                actions: vec![
                    "Add range validation".to_string(),
                    "Implement format checking".to_string(),
                    "Review data source quality".to_string(),
                ],
            },
            QualityMetric::Consistency => QualityRecommendation {
                id: format!("rec_{}_consistency", entity.name.snake),
                priority: violation.severity.weight() * 10,
                title: format!("Improve {} consistency", entity.name.raw),
                description: "Cross-entity data consistency issues detected".to_string(),
                metric: QualityMetric::Consistency,
                estimated_impact: 0.1,
                actions: vec![
                    "Review referential integrity".to_string(),
                    "Implement foreign key constraints".to_string(),
                ],
            },
            QualityMetric::Timeliness => QualityRecommendation {
                id: format!("rec_{}_timeliness", entity.name.snake),
                priority: violation.severity.weight() * 5,
                title: format!("Improve {} data freshness", entity.name.raw),
                description: "Data is not being updated within acceptable timeframes".to_string(),
                metric: QualityMetric::Timeliness,
                estimated_impact: 0.05,
                actions: vec![
                    "Review data update schedules".to_string(),
                    "Implement real-time data feeds".to_string(),
                ],
            },
            QualityMetric::Validity => QualityRecommendation {
                id: format!("rec_{}_validity", entity.name.snake),
                priority: violation.severity.weight() * 10,
                title: format!("Improve {} validity", entity.name.raw),
                description: "Data does not conform to expected formats or business rules"
                    .to_string(),
                metric: QualityMetric::Validity,
                estimated_impact: violation.threshold - violation.actual_value,
                actions: vec![
                    "Add input validation".to_string(),
                    "Implement regex pattern matching".to_string(),
                    "Review business rules".to_string(),
                ],
            },
            QualityMetric::Uniqueness => QualityRecommendation {
                id: format!("rec_{}_uniqueness", entity.name.snake),
                priority: violation.severity.weight() * 10,
                title: format!("Fix {} duplicates", entity.name.raw),
                description: "Duplicate records detected in the dataset".to_string(),
                metric: QualityMetric::Uniqueness,
                estimated_impact: violation.threshold - violation.actual_value,
                actions: vec![
                    "Add unique constraints".to_string(),
                    "Implement deduplication process".to_string(),
                    "Review data merge strategy".to_string(),
                ],
            },
        };

        recommendations.push(rec);
    }

    // Sort by priority (highest first)
    recommendations.sort_by_key(|b| std::cmp::Reverse(b.priority));

    recommendations
}

/// Generate Rust validation functions for entity
pub fn generate_rust_validation_functions(
    entity: &GeneratorEntity,
) -> Result<String, GenerateError> {
    let mut code = String::new();

    // Add imports
    code.push_str("//! Quality Validation for ");
    code.push_str(&entity.name.pascal);
    code.push_str("\n\n");
    code.push_str("use anyhow::Result;\n");
    code.push_str("use serde::{Deserialize, Serialize};\n");
    code.push_str("use common::telemetry::{info, warn, error};\n\n");

    // Generate validation functions for each rule
    for rule in &entity.quality_rules {
        if rule.active {
            code.push_str(&generate_entity_validation_fn(entity, rule)?);
            code.push('\n');
        }
    }

    // Generate field-level validation functions
    for field in &entity.fields {
        for rule in &field.quality_rules {
            if rule.active {
                code.push_str(&generate_field_validation_fn(entity, field, rule)?);
                code.push('\n');
            }
        }
    }

    // Generate comprehensive validation function
    code.push_str(&generate_comprehensive_validation_fn(entity)?);

    Ok(code)
}

/// Generate entity-level validation function
fn generate_entity_validation_fn(
    entity: &GeneratorEntity,
    rule: &GeneratorQualityRule,
) -> Result<String, GenerateError> {
    let fn_name = &rule.validation_fn_name;
    let metric_name = rule.metric.display_name();

    let code = format!(
        r##"/// Check {metric_name} for {entity_name}
/// Threshold: {threshold:.1}%
pub async fn {fn_name}(pool: &sqlx::PgPool) -> Result<QualityCheckResult> {{
    let threshold = {threshold};
    
    // Execute quality check query
    let result: (i64, i64) = sqlx::query_as(
        r#"
        SELECT 
            COUNT(*) as total,
            COUNT(CASE WHEN {condition} THEN 1 END) as passed
        FROM {table_name}
        "#
    )
    .fetch_one(pool)
    .await?;
    
    let total = result.0;
    let passed = result.1;
    let score = if total > 0 {{ passed as f64 / total as f64 }} else {{ 1.0 }};
    
    info!("{entity_name} {metric_name}: {{:.2}}%", score * 100.0);
    
    Ok(QualityCheckResult {{
        metric: QualityMetric::{metric_variant:?},
        score,
        passed: score >= threshold,
        total: total as usize,
        passed_count: passed as usize,
    }})
}}
"##,
        metric_name = metric_name,
        entity_name = entity.name.pascal,
        fn_name = fn_name,
        threshold = rule.threshold,
        condition = generate_quality_condition(rule),
        table_name = entity.name.snake,
        metric_variant = rule.metric,
    );

    Ok(code)
}

/// Generate field-level validation function
fn generate_field_validation_fn(
    entity: &GeneratorEntity,
    field: &GeneratorField,
    rule: &GeneratorQualityRule,
) -> Result<String, GenerateError> {
    let fn_name = &rule.validation_fn_name;
    let metric_name = rule.metric.display_name();

    let code = format!(
        r##"/// Check {metric_name} for {entity_name}.{field_name}
/// Threshold: {threshold:.1}%
pub async fn {fn_name}(pool: &sqlx::PgPool) -> Result<QualityCheckResult> {{
    let threshold = {threshold};
    
    let result: (i64, i64) = sqlx::query_as(
        r#"
        SELECT 
            COUNT(*) as total,
            COUNT(CASE WHEN {condition} THEN 1 END) as passed
        FROM {table_name}
        "#
    )
    .fetch_one(pool)
    .await?;
    
    let total = result.0;
    let passed = result.1;
    let score = if total > 0 {{ passed as f64 / total as f64 }} else {{ 1.0 }};
    
    if score < threshold {{
        warn!(
            "{entity_name}.{field_name} {metric_name} below threshold: {{:.2}}% < {{:.2}}%",
            score * 100.0,
            threshold * 100.0
        );
    }}
    
    Ok(QualityCheckResult {{
        metric: QualityMetric::{metric_variant:?},
        score,
        passed: score >= threshold,
        total: total as usize,
        passed_count: passed as usize,
    }})
}}
"##,
        metric_name = metric_name,
        entity_name = entity.name.pascal,
        field_name = field.name.raw,
        fn_name = fn_name,
        threshold = rule.threshold,
        condition = generate_field_quality_condition(field, rule),
        table_name = entity.name.snake,
        metric_variant = rule.metric,
    );

    Ok(code)
}

/// Generate comprehensive validation function
fn generate_comprehensive_validation_fn(entity: &GeneratorEntity) -> Result<String, GenerateError> {
    let entity_snake = &entity.name.snake;
    let entity_pascal = &entity.name.pascal;

    let mut rule_calls = String::new();

    // Add calls for entity-level rules
    for rule in &entity.quality_rules {
        if rule.active {
            rule_calls.push_str(&format!(
                "    results.push({}(pool).await?);\n",
                rule.validation_fn_name
            ));
        }
    }

    // Add calls for field-level rules
    for field in &entity.fields {
        for rule in &field.quality_rules {
            if rule.active {
                rule_calls.push_str(&format!(
                    "    results.push({}(pool).await?);\n",
                    rule.validation_fn_name
                ));
            }
        }
    }

    let code = format!(
        r##"/// Run all quality validations for {entity_pascal}
pub async fn validate_{entity_snake}_quality(pool: &sqlx::PgPool) -> Result<Vec<QualityCheckResult>> {{
    let mut results = Vec::new();
    
{rule_calls}
    
    Ok(results)
}}

/// Quality check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityCheckResult {{
    pub metric: QualityMetric,
    pub score: f64,
    pub passed: bool,
    pub total: usize,
    pub passed_count: usize,
}}
"##,
        entity_pascal = entity_pascal,
        entity_snake = entity_snake,
        rule_calls = rule_calls,
    );

    Ok(code)
}

/// Generate SQL condition for quality check
fn generate_quality_condition(rule: &GeneratorQualityRule) -> String {
    match rule.metric {
        QualityMetric::Completeness => "1=1".to_string(), // Entity-level completeness
        QualityMetric::Accuracy => "1=1".to_string(),
        QualityMetric::Consistency => "1=1".to_string(),
        QualityMetric::Timeliness => "updated_at > NOW() - INTERVAL '7 days'".to_string(),
        QualityMetric::Validity => "1=1".to_string(),
        QualityMetric::Uniqueness => "1=1".to_string(),
    }
}

/// Generate SQL condition for field-level quality check
fn generate_field_quality_condition(field: &GeneratorField, rule: &GeneratorQualityRule) -> String {
    let col = &field.name.snake;

    match rule.metric {
        QualityMetric::Completeness => format!("{} IS NOT NULL", col),
        QualityMetric::Accuracy => match field.field_type {
            GeneratorFieldType::Integer | GeneratorFieldType::BigInt => format!("{} >= 0", col),
            GeneratorFieldType::Decimal => format!("{} >= 0.0", col),
            _ => format!("{} IS NOT NULL", col),
        },
        QualityMetric::Consistency => format!("{} IS NOT NULL", col),
        QualityMetric::Timeliness => format!("{} IS NOT NULL", col),
        QualityMetric::Validity => match field.field_type {
            GeneratorFieldType::Text => format!("LENGTH({}) > 0", col),
            _ => format!("{} IS NOT NULL", col),
        },
        QualityMetric::Uniqueness => format!("{} IS NOT NULL", col), // Uniqueness needs subquery
    }
}

/// Trait for generating metric function names
trait MetricFnName {
    fn metric_fn_name(&self) -> String;
}

impl MetricFnName for GeneratorQualityRule {
    fn metric_fn_name(&self) -> String {
        match self.metric {
            QualityMetric::Completeness => "completeness".to_string(),
            QualityMetric::Accuracy => "accuracy".to_string(),
            QualityMetric::Consistency => "consistency".to_string(),
            QualityMetric::Timeliness => "timeliness".to_string(),
            QualityMetric::Validity => "validity".to_string(),
            QualityMetric::Uniqueness => "uniqueness".to_string(),
        }
    }
}

#[cfg(test)]
#[allow(dead_code)]
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
            fields: vec![],
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
    fn test_determine_severity() {
        assert_eq!(determine_severity(0.7, 0.95), ViolationSeverity::Critical);
        assert_eq!(determine_severity(0.8, 0.95), ViolationSeverity::High);
        assert_eq!(determine_severity(0.88, 0.95), ViolationSeverity::Medium);
        assert_eq!(determine_severity(0.92, 0.95), ViolationSeverity::Low);
        assert_eq!(determine_severity(0.98, 0.95), ViolationSeverity::Info);
    }

    #[test]
    fn test_calculate_overall_score() {
        let scores = vec![
            QualityScore::new(QualityDimension::Completeness, 0.95),
            QualityScore::new(QualityDimension::Accuracy, 0.90),
        ];
        assert_eq!(calculate_overall_score(&scores), 0.925);
    }
}
