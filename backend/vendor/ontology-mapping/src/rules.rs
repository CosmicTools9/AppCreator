use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct RuleSet {
    pub version: String,
    pub alioth_model: String,
    pub field_patterns: FieldPatterns,
    pub scalar_inference: ScalarInference,
    pub nesting_rules: Vec<NestingRule>,
    pub coordinate_inference: CoordinateInference,
}

impl RuleSet {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let rules: Self = yaml_serde::from_str(&content)?;
        Ok(rules)
    }
}

#[derive(Debug, Deserialize)]
pub struct FieldPatterns {
    pub exact: Vec<ExactPattern>,
    pub prefix: Vec<PrefixPattern>,
    pub semantic_groups: Vec<SemanticGroup>,
    pub contextual: Vec<ContextualPattern>,
}

#[derive(Debug, Deserialize)]
pub struct ExactPattern {
    pub pattern: String,
    pub column: String,
    pub confidence: f64,
}

#[derive(Debug, Deserialize)]
pub struct PrefixPattern {
    pub pattern: String,
    pub column_template: String,
    pub confidence: f64,
}

#[derive(Debug, Deserialize)]
pub struct SemanticGroup {
    pub triggers: Vec<String>,
    pub preference: String,
    pub confidence: f64,
}

#[derive(Debug, Deserialize)]
pub struct ContextualPattern {
    pub pattern: String,
    pub candidates: Vec<CandidateMatch>,
    pub default: String,
    pub confidence: f64,
}

#[derive(Debug, Deserialize)]
pub struct CandidateMatch {
    pub column: String,
    #[serde(default)]
    pub when: CandidateWhen,
    pub confidence: f64,
}

#[derive(Debug, Deserialize, Default)]
pub struct CandidateWhen {
    #[serde(default)]
    pub siblings_contain: Vec<String>,
    #[serde(default)]
    pub entity_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ScalarInference {
    pub rules: Vec<ScalarRule>,
}

#[derive(Debug, Deserialize)]
pub struct ScalarRule {
    pub triggers: Vec<String>,
    pub scalar_table: String,
    pub column_prefix: String,
    pub confidence: f64,
}

#[derive(Debug, Deserialize)]
pub struct NestingRule {
    pub when: NestingWhen,
    pub action: String,
    #[serde(default)]
    pub relationship: Option<String>,
    pub confidence: f64,
}

#[derive(Debug, Deserialize)]
pub struct NestingWhen {
    #[serde(default)]
    pub is_array: bool,
    #[serde(default)]
    pub is_object: bool,
    #[serde(default)]
    pub element_has_fields: Option<String>,
    #[serde(default)]
    pub fields_count: Option<String>,
    #[serde(default)]
    pub shared: Option<bool>,
    #[serde(default)]
    pub structure: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CoordinateInference {
    pub scene: SceneCoordinateSource,
    pub factor: FactorCoordinateSource,
    pub function: FunctionCoordinateSource,
}

#[derive(Debug, Deserialize)]
pub struct SceneCoordinateSource {
    pub source: String,
    pub confidence: f64,
}

#[derive(Debug, Deserialize)]
pub struct FactorCoordinateSource {
    pub source: String,
    pub confidence: f64,
}

#[derive(Debug, Deserialize)]
pub struct FunctionCoordinateSource {
    pub rules: Vec<FunctionRule>,
}

#[derive(Debug, Deserialize)]
pub struct FunctionRule {
    pub entity_types: Vec<String>,
    pub default: String,
    pub confidence: f64,
}
