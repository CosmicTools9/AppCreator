//! 版本管理器模块 (简化版)
//!
//! DSL 支持已移除，完整功能将在后续版本中基于 IR 模型实现

use crate::generator::ir::GeneratorModel;
use crate::version::compat::{CompatibilityReport, RiskLevel};
use crate::version::Version;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

fn generate_id() -> i64 {
    static COUNTER: AtomicU64 = AtomicU64::new(1000);
    COUNTER.fetch_add(1, Ordering::SeqCst) as i64
}

/// 版本记录
#[derive(Debug, Clone)]
pub struct VersionRecord {
    pub id: i64,
    pub model_name: String,
    pub version: Version,
    pub content: String,
    pub checksum: String,
    pub created_by: i64,
    pub created_at: DateTime<Utc>,
    pub change_summary: Option<String>,
    pub is_active: bool,
    pub tags: Vec<String>,
}

impl VersionRecord {
    pub fn new(
        model_name: impl Into<String>,
        version: Version,
        content: impl Into<String>,
        created_by: i64,
    ) -> Self {
        let content = content.into();
        let checksum = calculate_checksum(&content);

        Self {
            id: generate_id(),
            model_name: model_name.into(),
            version,
            content,
            checksum,
            created_by,
            created_at: Utc::now(),
            change_summary: None,
            is_active: false,
            tags: Vec::new(),
        }
    }

    pub fn parse_model(&self) -> Result<GeneratorModel, Box<dyn std::error::Error>> {
        let model: GeneratorModel = serde_json::from_str(&self.content)?;
        Ok(model)
    }
}

fn calculate_checksum(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// 版本管理器
#[derive(Debug, Clone)]
pub struct VersionManager {
    versions: HashMap<String, Vec<VersionRecord>>,
}

#[derive(Error, Debug)]
pub enum VersionError {
    #[error("版本未找到: {0} {1}")]
    VersionNotFound(String, String),
    #[error("版本已存在: {0} {1}")]
    VersionAlreadyExists(String, String),
    #[error("校验和不匹配")]
    ChecksumMismatch,
    #[error("无效版本: {0}")]
    InvalidVersion(String),
    #[error("不兼容升级: {0}")]
    IncompatibleUpgrade(String),
    #[error("序列化错误: {0}")]
    SerializationError(String),
    #[error("兼容性错误: {0}")]
    CompatibilityError(String),
    #[error("存储错误: {0}")]
    StorageError(String),
    #[error("其他错误: {0}")]
    Other(String),
}

impl VersionManager {
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
        }
    }

    pub fn save_version(&mut self, record: VersionRecord) -> Result<(), VersionError> {
        let model_versions = self.versions.entry(record.model_name.clone()).or_default();

        if model_versions.iter().any(|v| v.version == record.version) {
            return Err(VersionError::VersionAlreadyExists(
                record.model_name,
                record.version.to_string(),
            ));
        }

        model_versions.push(record);
        model_versions.sort_by(|a, b| a.version.cmp(&b.version));

        Ok(())
    }

    pub fn create_version(
        &mut self,
        model: &GeneratorModel,
        model_name: &str,
        version: Version,
        created_by: i64,
        change_summary: Option<impl Into<String>>,
    ) -> Result<VersionRecord, VersionError> {
        let content = serde_json::to_string(model)
            .map_err(|e| VersionError::SerializationError(e.to_string()))?;

        let mut record = VersionRecord::new(model_name, version, content, created_by);

        if let Some(summary) = change_summary {
            record.change_summary = Some(summary.into());
        }

        self.save_version(record.clone())?;

        Ok(record)
    }

    pub fn get_version(&self, model_name: &str, version: &Version) -> Option<&VersionRecord> {
        self.versions
            .get(model_name)
            .and_then(|versions| versions.iter().find(|v| &v.version == version))
    }

    pub fn list_versions(&self, model_name: &str) -> Vec<Version> {
        self.versions
            .get(model_name)
            .map(|versions| versions.iter().map(|v| v.version.clone()).collect())
            .unwrap_or_default()
    }

    pub fn compare_versions(
        &self,
        _model_name: &str,
        from: &Version,
        to: &Version,
    ) -> Result<CompatibilityReport, VersionError> {
        // 简化实现
        Ok(CompatibilityReport::new(from.clone(), to.clone()))
    }

    pub fn plan_upgrade_path(
        &self,
        model_name: &str,
        from: &Version,
        target: &Version,
    ) -> Result<UpgradePath, VersionError> {
        let versions = self.list_versions(model_name);

        if versions.is_empty() {
            return Err(VersionError::VersionNotFound(
                model_name.to_string(),
                "任何版本".to_string(),
            ));
        }

        // 简化实现
        Ok(UpgradePath {
            from: from.clone(),
            target: target.clone(),
            steps: vec![target.clone()],
            risk_level: RiskLevel::Low,
            total_changes: 0,
            breaking_changes: 0,
            estimated_time_minutes: 5,
            reports: vec![],
        })
    }

    pub fn activate_version(
        &mut self,
        model_name: &str,
        version: &Version,
    ) -> Result<(), VersionError> {
        let model_versions = self.versions.get_mut(model_name).ok_or_else(|| {
            VersionError::VersionNotFound(model_name.to_string(), version.to_string())
        })?;

        let mut found = false;
        for record in model_versions.iter_mut() {
            if &record.version == version {
                record.is_active = true;
                found = true;
            } else {
                record.is_active = false;
            }
        }

        if !found {
            return Err(VersionError::VersionNotFound(
                model_name.to_string(),
                version.to_string(),
            ));
        }

        Ok(())
    }

    pub fn get_active_version(&self, model_name: &str) -> Option<&VersionRecord> {
        self.versions
            .get(model_name)
            .and_then(|versions| versions.iter().find(|v| v.is_active))
    }

    /// 获取最新版本 (简化版)
    pub fn get_latest_version(&self, model_name: &str) -> Option<&VersionRecord> {
        self.versions
            .get(model_name)
            .and_then(|versions| versions.iter().max_by(|a, b| a.version.cmp(&b.version)))
    }

    /// 检查版本是否符合约束 (简化版)
    pub fn check_constraint(
        &self,
        model_name: &str,
        constraint: &crate::version::VersionConstraint,
    ) -> bool {
        self.versions
            .get(model_name)
            .map(|versions| versions.iter().any(|v| constraint.matches(&v.version)))
            .unwrap_or(false)
    }

    /// 查找最新匹配的版本 (简化版)
    pub fn find_latest_matching(
        &self,
        model_name: &str,
        constraint: &crate::version::VersionConstraint,
    ) -> Option<&VersionRecord> {
        self.versions.get(model_name).and_then(|versions| {
            versions
                .iter()
                .filter(|v| constraint.matches(&v.version))
                .max_by(|a, b| a.version.cmp(&b.version))
        })
    }

    /// 为版本添加标签 (简化版)
    pub fn tag_version(
        &mut self,
        model_name: &str,
        version: &Version,
        tag: &str,
    ) -> Result<(), VersionError> {
        let model_versions = self.versions.get_mut(model_name).ok_or_else(|| {
            VersionError::VersionNotFound(model_name.to_string(), version.to_string())
        })?;

        for record in model_versions.iter_mut() {
            if &record.version == version {
                if !record.tags.contains(&tag.to_string()) {
                    record.tags.push(tag.to_string());
                }
                return Ok(());
            }
        }

        Err(VersionError::VersionNotFound(
            model_name.to_string(),
            version.to_string(),
        ))
    }

    /// 按标签查找版本 (简化版)
    pub fn find_by_tag(&self, model_name: &str, tag: &str) -> Vec<Version> {
        self.versions
            .get(model_name)
            .map(|versions| {
                versions
                    .iter()
                    .filter(|v| v.tags.contains(&tag.to_string()))
                    .map(|v| v.version.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 获取版本时间线 (简化版)
    pub fn get_version_timeline(&self, model_name: &str) -> Vec<VersionTimelineEntry> {
        self.versions
            .get(model_name)
            .map(|versions| {
                versions
                    .iter()
                    .map(|v| VersionTimelineEntry {
                        version: v.version.clone(),
                        created_at: v.created_at,
                        created_by: v.created_by,
                        change_summary: v.change_summary.clone(),
                        is_active: v.is_active,
                        tags: v.tags.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for VersionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 升级路径
#[derive(Debug, Clone)]
pub struct UpgradePath {
    pub from: Version,
    pub target: Version,
    pub steps: Vec<Version>,
    pub risk_level: RiskLevel,
    pub total_changes: usize,
    pub breaking_changes: usize,
    pub estimated_time_minutes: u32,
    pub reports: Vec<CompatibilityReport>,
}


/// 版本时间线条目
#[derive(Debug, Clone)]
pub struct VersionTimelineEntry {
    pub version: Version,
    pub created_at: DateTime<Utc>,
    pub created_by: i64,
    pub change_summary: Option<String>,
    pub is_active: bool,
    pub tags: Vec<String>,
}
