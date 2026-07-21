use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Tiered confidence model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Safe,
    Suggest,
    Unclear,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputMeta {
    pub tool_version: String,
    pub alioth_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TieredValue {
    pub value: String,
    pub tier: Tier,
    pub confidence: f64,
    pub source: String,
}

// ---------------------------------------------------------------------------
// Mapping output (result)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMapping {
    pub table: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherits: Option<String>,
    pub source: String,
    pub tier: Tier,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub scene: TieredValue,
    pub factor: TieredValue,
    pub function: TieredValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    pub json_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scalar_table: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_table: Option<String>,
    pub tier: Tier,
    pub confidence: f64,
    pub source: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternatives: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipMapping {
    pub target: String,
    #[serde(rename = "type")]
    pub rel_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
    pub tier: Tier,
    pub confidence: f64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappedEntity {
    pub name: String,
    pub mapping: EntityMapping,
    pub coordinates: Coordinates,
    pub fields: Vec<FieldMapping>,
    pub relationships: Vec<RelationshipMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierSummary {
    pub safe: usize,
    pub suggest: usize,
    pub unclear: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingOutput {
    pub meta: OutputMeta,
    pub entities: Vec<MappedEntity>,
    pub summary: TierSummary,
}

// ---------------------------------------------------------------------------
// Mapping input (prototype JSON)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingInput {
    pub scene_code: String,
    pub factor_ids: Vec<String>,
    pub entities: Vec<EntityInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityInput {
    pub name: String,
    #[serde(default)]
    pub fields: Vec<FieldInput>,
    #[serde(default)]
    pub nested: Vec<NestedInput>,
    /// 可选的叶表名（由 discovery 预先确定）。格式: "isahl.zc_id_xxx" 或 "zc_id_xxx"
    #[serde(default)]
    pub table: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInput {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub r#enum: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedInput {
    pub name: String,
    #[serde(rename = "type")]
    pub nested_type: String,
    pub items: NestedEntityInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedEntityInput {
    pub name: String,
    #[serde(default)]
    pub fields: Vec<FieldInput>,
}
