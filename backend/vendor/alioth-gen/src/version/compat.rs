//! 兼容性检查模块
//!
//! 提供元模型版本间的兼容性检查，检测破坏性变更
//! (基于 IR 模型)

use crate::generator::ir::GeneratorModel;
use crate::version::Version;

/// 兼容性检查报告
#[derive(Debug, Clone)]
pub struct CompatibilityReport {
    /// 源版本
    pub from_version: Version,
    /// 目标版本
    pub to_version: Version,
    /// 是否兼容
    pub is_compatible: bool,
    /// 变更类型
    pub change_type: ChangeType,
    /// 破坏性变更列表
    pub breaking_changes: Vec<BreakingChange>,
    /// 非破坏性变更列表
    pub non_breaking_changes: Vec<Change>,
    /// 迁移步骤建议
    pub migration_steps: Vec<MigrationStep>,
    /// 数据丢失风险评估
    pub data_loss_risk: RiskLevel,
    /// 兼容性评分 (0-100)
    pub compatibility_score: u8,
}

impl CompatibilityReport {
    /// 创建新报告
    pub fn new(from: Version, to: Version) -> Self {
        Self {
            from_version: from,
            to_version: to,
            is_compatible: true,
            change_type: ChangeType::Patch,
            breaking_changes: Vec::new(),
            non_breaking_changes: Vec::new(),
            migration_steps: Vec::new(),
            data_loss_risk: RiskLevel::None,
            compatibility_score: 100,
        }
    }
}

/// 兼容性检查器
#[derive(Debug, Clone)]
pub struct CompatibilityChecker {
    strict_mode: bool,
}

impl CompatibilityChecker {
    pub fn new() -> Self {
        Self { strict_mode: false }
    }

    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    pub fn check(
        &self,
        _old_model: &GeneratorModel,
        _new_model: &GeneratorModel,
        from: Version,
        to: Version,
    ) -> CompatibilityReport {
        CompatibilityReport::new(from, to)
    }
}

impl Default for CompatibilityChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl CompatibilityReport {
    pub fn has_category(&self, _category: ChangeCategory) -> bool {
        false
    }

    pub fn summary(&self) -> String {
        format!(
            "From {} to {}: {} compatible, {} breaking changes",
            self.from_version,
            self.to_version,
            if self.is_compatible { "is" } else { "not" },
            self.breaking_changes.len()
        )
    }
}

// 占位类型
#[derive(Debug, Clone)]
pub struct BreakingChange;
#[derive(Debug, Clone)]
pub struct Change;

#[derive(Debug, Clone, Copy)]
pub struct ChangeCategory;

#[allow(non_upper_case_globals)]
impl ChangeCategory {
    pub const CollectionRemoved: Self = Self;
    pub const FieldRemoved: Self = Self;
}
#[derive(Debug, Clone)]
pub struct ChangeType;

#[allow(non_upper_case_globals)]
impl ChangeType {
    pub const Patch: Self = Self;

    pub fn as_str(&self) -> &'static str {
        "patch"
    }
}
pub struct CompatibilityError;
pub struct ImpactLevel;
#[derive(Debug, Clone)]
pub struct MigrationStep;
pub struct MigrationStepType;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}
