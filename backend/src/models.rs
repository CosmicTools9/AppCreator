use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Project ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub namespace: String,
    pub description: String,
    pub status: String,
    pub config: serde_json::Value,
    pub template_id: Option<i64>,
    pub created_by: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub namespace: String,
    pub template_id: Option<i64>,
    #[serde(default = "default_config")]
    pub config: serde_json::Value,
}

fn default_config() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub config: Option<serde_json::Value>,
}

// ── Template ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub category: String,
}

// ── Build ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Build {
    pub id: i64,
    pub project_id: i64,
    pub status: String,
    pub log: String,
    pub created_at: DateTime<Utc>,
}

// ── Deployment ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub id: i64,
    pub project_id: i64,
    pub build_id: i64,
    pub status: String,
    pub target: String,
    pub created_at: DateTime<Utc>,
}

// ── Pagination ────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
}

impl PaginationParams {
    pub fn page(&self) -> i64 { self.page.unwrap_or(1).max(1) }
    pub fn per_page(&self) -> i64 { self.per_page.unwrap_or(20).clamp(1, 100) }
    pub fn offset(&self) -> i64 { (self.page() - 1) * self.per_page() }
}
