//! IR Quality Types - Phase 27: Quality Validation
//!
//! Provides data structures for quality rules and validation:
//! - Quality metrics: completeness, accuracy, consistency, timeliness, validity
//! - Quality rules with thresholds
//! - Quality reports and scoring

// use crate::ontology::{OntologyInferenceResult, OntologyModel}; // kept for reference; currently unused after removing OntologyQualityReporter
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Quality metric types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum QualityMetric {
    /// Completeness - percentage of non-null values
    #[default]
    Completeness,
    /// Accuracy - percentage of values matching expected patterns/ranges
    Accuracy,
    /// Consistency - percentage of values consistent across related entities
    Consistency,
    /// Timeliness - percentage of data updated within acceptable timeframe
    Timeliness,
    /// Validity - percentage of values conforming to business rules
    Validity,
    /// Uniqueness - percentage of unique values (no duplicates)
    Uniqueness,
}

impl QualityMetric {
    /// Get the default threshold for this metric (0.0 - 1.0)
    pub fn default_threshold(&self) -> f64 {
        match self {
            QualityMetric::Completeness => 0.95,
            QualityMetric::Accuracy => 0.99,
            QualityMetric::Consistency => 0.90,
            QualityMetric::Timeliness => 0.80,
            QualityMetric::Validity => 0.95,
            QualityMetric::Uniqueness => 1.0,
        }
    }

    /// Get a human-readable name for this metric
    pub fn display_name(&self) -> &'static str {
        match self {
            QualityMetric::Completeness => "完整性",
            QualityMetric::Accuracy => "准确性",
            QualityMetric::Consistency => "一致性",
            QualityMetric::Timeliness => "时效性",
            QualityMetric::Validity => "有效性",
            QualityMetric::Uniqueness => "唯一性",
        }
    }

    /// Get description for this metric
    pub fn description(&self) -> &'static str {
        match self {
            QualityMetric::Completeness => "必填字段的填充率，检查空值和缺失数据",
            QualityMetric::Accuracy => "数据值的准确性，检查范围、格式和业务规则",
            QualityMetric::Consistency => "跨实体数据的一致性，检查关联完整性",
            QualityMetric::Timeliness => "数据更新的及时性，检查数据新鲜度",
            QualityMetric::Validity => "数据格式的有效性，检查约束和模式匹配",
            QualityMetric::Uniqueness => "数据的唯一性，检查重复记录",
        }
    }
}

/// Quality rule definition (IR-1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaQualityRule {
    /// Quality metric type
    pub metric: QualityMetric,
    /// Threshold value (0.0 - 1.0, where 1.0 = 100%)
    pub threshold: f64,
    /// Optional field name (for field-level rules)
    #[serde(default)]
    pub field_name: Option<String>,
    /// Custom error message
    #[serde(default)]
    pub error_message: Option<String>,
    /// Whether this rule is active
    #[serde(default)]
    pub active: bool,
    /// Rule priority (higher = more important)
    #[serde(default)]
    pub priority: i32,
}

impl MetaQualityRule {
    /// Create a new quality rule
    pub fn new(metric: QualityMetric, threshold: f64) -> Self {
        Self {
            metric,
            threshold: threshold.clamp(0.0, 1.0),
            field_name: None,
            error_message: None,
            active: true,
            priority: 0,
        }
    }

    /// Create a field-level quality rule
    pub fn for_field(metric: QualityMetric, threshold: f64, field_name: impl Into<String>) -> Self {
        Self {
            metric,
            threshold: threshold.clamp(0.0, 1.0),
            field_name: Some(field_name.into()),
            error_message: None,
            active: true,
            priority: 0,
        }
    }

    /// Set custom error message
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.error_message = Some(message.into());
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Check if this rule is for a specific field
    pub fn is_field_rule(&self) -> bool {
        self.field_name.is_some()
    }

    /// Get the default error message
    pub fn default_error_message(&self) -> String {
        let field_part = self
            .field_name
            .as_ref()
            .map(|f| format!("字段 '{}' 的", f))
            .unwrap_or_else(|| "实体的".to_string());
        format!(
            "{}{}未达到要求阈值 {:.1}%",
            field_part,
            self.metric.display_name(),
            self.threshold * 100.0
        )
    }
}

impl Default for MetaQualityRule {
    fn default() -> Self {
        Self {
            metric: QualityMetric::Completeness,
            threshold: 0.95,
            field_name: None,
            error_message: None,
            active: true,
            priority: 0,
        }
    }
}

/// Quality dimension for aggregated scoring
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum QualityDimension {
    /// Completeness dimension
    Completeness,
    /// Accuracy dimension
    Accuracy,
    /// Consistency dimension
    Consistency,
    /// Timeliness dimension
    Timeliness,
    /// Validity dimension
    Validity,
    /// Uniqueness dimension
    Uniqueness,
    /// Overall quality score
    Overall,
}

impl QualityDimension {
    /// Convert from QualityMetric
    pub fn from_metric(metric: QualityMetric) -> Self {
        match metric {
            QualityMetric::Completeness => QualityDimension::Completeness,
            QualityMetric::Accuracy => QualityDimension::Accuracy,
            QualityMetric::Consistency => QualityDimension::Consistency,
            QualityMetric::Timeliness => QualityDimension::Timeliness,
            QualityMetric::Validity => QualityDimension::Validity,
            QualityMetric::Uniqueness => QualityDimension::Uniqueness,
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            QualityDimension::Completeness => "完整性",
            QualityDimension::Accuracy => "准确性",
            QualityDimension::Consistency => "一致性",
            QualityDimension::Timeliness => "时效性",
            QualityDimension::Validity => "有效性",
            QualityDimension::Uniqueness => "唯一性",
            QualityDimension::Overall => "总体质量",
        }
    }
}

/// Quality score for a specific dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScore {
    /// Quality dimension
    pub dimension: QualityDimension,
    /// Score value (0.0 - 1.0)
    pub score: f64,
    /// Number of checks passed
    pub passed: usize,
    /// Number of checks failed
    pub failed: usize,
    /// Total number of checks
    pub total: usize,
    /// Weight for overall scoring (0.0 - 1.0)
    #[serde(default = "default_weight")]
    pub weight: f64,
}

fn default_weight() -> f64 {
    1.0
}

impl QualityScore {
    /// Create a new quality score
    pub fn new(dimension: QualityDimension, score: f64) -> Self {
        Self {
            dimension,
            score: score.clamp(0.0, 1.0),
            passed: 0,
            failed: 0,
            total: 0,
            weight: 1.0,
        }
    }

    /// Create a score with counts
    pub fn with_counts(
        dimension: QualityDimension,
        score: f64,
        passed: usize,
        failed: usize,
    ) -> Self {
        Self {
            dimension,
            score: score.clamp(0.0, 1.0),
            passed,
            failed,
            total: passed + failed,
            weight: 1.0,
        }
    }

    /// Get score percentage
    pub fn percentage(&self) -> f64 {
        self.score * 100.0
    }

    /// Get grade (A-F) based on score
    pub fn grade(&self) -> &'static str {
        match self.score {
            s if s >= 0.95 => "A",
            s if s >= 0.90 => "B",
            s if s >= 0.80 => "C",
            s if s >= 0.70 => "D",
            _ => "F",
        }
    }

    /// Check if score meets threshold
    pub fn meets_threshold(&self, threshold: f64) -> bool {
        self.score >= threshold
    }
}

/// Quality violation/finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityViolation {
    /// Violation ID
    pub id: String,
    /// Quality metric that failed
    pub metric: QualityMetric,
    /// Entity name
    pub entity: String,
    /// Field name (if field-level)
    #[serde(default)]
    pub field: Option<String>,
    /// Violation message
    pub message: String,
    /// Severity level
    pub severity: ViolationSeverity,
    /// Actual value/score
    pub actual_value: f64,
    /// Expected threshold
    pub threshold: f64,
    /// Sample of problematic values
    #[serde(default)]
    pub samples: Vec<String>,
    /// Timestamp of detection
    pub detected_at: String,
}

/// Violation severity
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ViolationSeverity {
    /// Critical - must be fixed
    Critical,
    /// High - should be fixed soon
    High,
    /// Medium - should be addressed
    Medium,
    /// Low - informational
    Low,
    /// Info - for awareness only
    Info,
}

impl ViolationSeverity {
    /// Get numeric weight for severity
    pub fn weight(&self) -> i32 {
        match self {
            ViolationSeverity::Critical => 5,
            ViolationSeverity::High => 4,
            ViolationSeverity::Medium => 3,
            ViolationSeverity::Low => 2,
            ViolationSeverity::Info => 1,
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            ViolationSeverity::Critical => "严重",
            ViolationSeverity::High => "高",
            ViolationSeverity::Medium => "中",
            ViolationSeverity::Low => "低",
            ViolationSeverity::Info => "信息",
        }
    }
}

/// Quality report (aggregated results)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    /// Report ID
    pub id: String,
    /// Entity name
    pub entity: String,
    /// Report generation timestamp
    pub generated_at: String,
    /// Overall quality score (0.0 - 1.0)
    pub overall_score: f64,
    /// Dimension scores
    pub dimension_scores: Vec<QualityScore>,
    /// All violations found
    pub violations: Vec<QualityViolation>,
    /// Summary statistics
    pub summary: QualitySummary,
    /// Recommendations
    pub recommendations: Vec<QualityRecommendation>,
    /// Risk items list
    #[serde(default)]
    pub risk_items: Vec<String>,
}

/// Quality summary statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualitySummary {
    /// Total number of checks performed
    pub total_checks: usize,
    /// Number of checks passed
    pub passed_checks: usize,
    /// Number of checks failed
    pub failed_checks: usize,
    /// Number of critical violations
    pub critical_count: usize,
    /// Number of high severity violations
    pub high_count: usize,
    /// Number of medium severity violations
    pub medium_count: usize,
    /// Number of low severity violations
    pub low_count: usize,
    /// Number of info items
    pub info_count: usize,
}

/// Quality improvement recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRecommendation {
    /// Recommendation ID
    pub id: String,
    /// Priority (1-10, where 10 is highest)
    pub priority: i32,
    /// Title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Affected metric
    pub metric: QualityMetric,
    /// Estimated impact (score improvement)
    pub estimated_impact: f64,
    /// Suggested actions
    pub actions: Vec<String>,
}

// ============================================================================
// IR-2 Types for Code Generation
// ============================================================================

/// Generator quality rule (IR-2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorQualityRule {
    /// Quality metric type
    pub metric: QualityMetric,
    /// Threshold value (0.0 - 1.0)
    pub threshold: f64,
    /// Threshold percentage for display
    pub threshold_percentage: f64,
    /// Optional field name
    pub field_name: Option<String>,
    /// Snake_case field name for code generation
    pub field_name_snake: Option<String>,
    /// Custom error message
    pub error_message: Option<String>,
    /// Whether this rule is active
    pub active: bool,
    /// Rule priority
    pub priority: i32,
    /// SQL query for data quality check (if applicable)
    #[serde(default)]
    pub check_sql: Option<String>,
    /// Validation function name to generate
    pub validation_fn_name: String,
}

impl GeneratorQualityRule {
    /// Transform from MetaQualityRule
    pub fn from_meta(rule: &MetaQualityRule, entity_name: &str) -> Self {
        let validation_fn_name = if let Some(ref field) = rule.field_name {
            format!(
                "validate_{}_{}_{}",
                entity_name.to_lowercase(),
                field.to_lowercase(),
                rule.metric_fn_suffix()
            )
        } else {
            format!(
                "validate_{}_{}",
                entity_name.to_lowercase(),
                rule.metric_fn_suffix()
            )
        };

        Self {
            metric: rule.metric,
            threshold: rule.threshold,
            threshold_percentage: rule.threshold * 100.0,
            field_name: rule.field_name.clone(),
            field_name_snake: rule
                .field_name
                .as_ref()
                .map(|f| f.to_lowercase().replace(" ", "_")),
            error_message: rule.error_message.clone(),
            active: rule.active,
            priority: rule.priority,
            check_sql: None,
            validation_fn_name,
        }
    }
}

impl MetaQualityRule {
    fn metric_fn_suffix(&self) -> String {
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

/// Generator quality configuration (IR-2)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneratorQualityConfig {
    /// Whether quality validation is enabled
    pub enabled: bool,
    /// Entity-level quality rules
    pub entity_rules: Vec<GeneratorQualityRule>,
    /// Field-level quality rules
    pub field_rules: HashMap<String, Vec<GeneratorQualityRule>>,
    /// SQL queries for data quality checks
    pub quality_checks_sql: Vec<QualityCheckSql>,
}

/// SQL query for quality check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityCheckSql {
    /// Check name
    pub name: String,
    /// Quality metric
    pub metric: QualityMetric,
    /// SQL query
    pub sql: String,
    /// Expected result type
    pub result_type: QualityResultType,
    /// Threshold for pass/fail
    pub threshold: f64,
}

/// Quality check result type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum QualityResultType {
    /// Count of problematic records
    Count,
    /// Percentage (0.0 - 1.0)
    Percentage,
    /// Average value
    Average,
    /// Boolean (true = pass)
    Boolean,
}

/// Ontology quality metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OntologyQualityMetrics {
    /// Class hierarchy depth (max)
    pub max_hierarchy_depth: u32,
    /// Average class hierarchy depth
    pub avg_hierarchy_depth: f64,
    /// Number of classes without properties
    pub classes_without_properties: usize,
    /// Number of properties without domain/range
    pub properties_without_constraints: usize,
    /// Documentation coverage percentage
    pub documentation_coverage: f64,
    /// Constraint coverage percentage  
    pub constraint_coverage: f64,
    /// Quality rules coverage
    pub quality_rule_coverage: f64,
}

impl OntologyQualityMetrics {
    /// Calculate overall ontology quality score
    pub fn overall_score(&self) -> f64 {
        let weights = [
            (self.documentation_coverage, 0.3),
            (self.constraint_coverage, 0.4),
            (self.quality_rule_coverage, 0.3),
        ];

        weights.iter().map(|(v, w)| v * w).sum()
    }
}

/// Ontology quality reporter

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_metric_defaults() {
        assert_eq!(QualityMetric::Completeness.default_threshold(), 0.95);
        assert_eq!(QualityMetric::Accuracy.default_threshold(), 0.99);
        assert_eq!(QualityMetric::Consistency.default_threshold(), 0.90);
        assert_eq!(QualityMetric::Timeliness.default_threshold(), 0.80);
        assert_eq!(QualityMetric::Validity.default_threshold(), 0.95);
        assert_eq!(QualityMetric::Uniqueness.default_threshold(), 1.0);
    }

    #[test]
    fn test_quality_rule_creation() {
        let rule = MetaQualityRule::new(QualityMetric::Completeness, 0.95);
        assert_eq!(rule.metric, QualityMetric::Completeness);
        assert_eq!(rule.threshold, 0.95);
        assert!(rule.active);
        assert!(rule.field_name.is_none());

        let field_rule = MetaQualityRule::for_field(QualityMetric::Accuracy, 0.99, "email");
        assert_eq!(field_rule.field_name, Some("email".to_string()));
        assert!(field_rule.is_field_rule());
    }

    #[test]
    fn test_quality_score_grade() {
        assert_eq!(
            QualityScore::new(QualityDimension::Overall, 0.96).grade(),
            "A"
        );
        assert_eq!(
            QualityScore::new(QualityDimension::Overall, 0.92).grade(),
            "B"
        );
        assert_eq!(
            QualityScore::new(QualityDimension::Overall, 0.85).grade(),
            "C"
        );
        assert_eq!(
            QualityScore::new(QualityDimension::Overall, 0.75).grade(),
            "D"
        );
        assert_eq!(
            QualityScore::new(QualityDimension::Overall, 0.60).grade(),
            "F"
        );
    }

    #[test]
    fn test_violation_severity() {
        assert_eq!(ViolationSeverity::Critical.weight(), 5);
        assert_eq!(ViolationSeverity::High.weight(), 4);
        assert_eq!(ViolationSeverity::Medium.weight(), 3);
        assert_eq!(ViolationSeverity::Low.weight(), 2);
        assert_eq!(ViolationSeverity::Info.weight(), 1);
    }

    #[test]
    fn test_threshold_clamping() {
        let rule = MetaQualityRule::new(QualityMetric::Completeness, 1.5);
        assert_eq!(rule.threshold, 1.0);

        let rule2 = MetaQualityRule::new(QualityMetric::Completeness, -0.5);
        assert_eq!(rule2.threshold, 0.0);
    }
}
