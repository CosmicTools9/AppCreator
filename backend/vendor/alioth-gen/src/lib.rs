//! 元模型库
//!
//! 提供元模型的定义、代码生成、导出和 Schema 生成功能
//!
//! # 版本信息
//! - Alioth (模型): 从环境变量 MODEL_VERSION 读取，默认 10.0.0
//! - AliothStudio (平台): 从环境变量 STUDIO_VERSION 读取，默认 0.1.0
//! - 应用程序: 从环境变量 APP_VERSION 读取，默认 0.0.1

use std::env;
use std::sync::LazyLock;

/// Alioth 模型版本 (从环境变量 MODEL_VERSION 读取，默认 10.0.0)
pub static ALIOTH_MODEL_VERSION: LazyLock<String> =
    LazyLock::new(|| env::var("MODEL_VERSION").unwrap_or_else(|_| "10.0.0".to_string()));

/// AliothStudio 平台版本 (从环境变量 STUDIO_VERSION 读取，默认 0.1.1)
pub static ALIOTH_STUDIO_VERSION: LazyLock<String> =
    LazyLock::new(|| env::var("STUDIO_VERSION").unwrap_or_else(|_| "0.1.1".to_string()));

/// 应用程序版本 (从环境变量 APP_VERSION 读取，默认 0.0.1)
pub static APP_VERSION: LazyLock<String> =
    LazyLock::new(|| env::var("APP_VERSION").unwrap_or_else(|_| "0.0.1".to_string()));

pub mod api;
pub mod cli;
pub mod docgen;
pub mod dsl;
pub mod generator;
pub mod metrics;
pub mod templates;
pub mod version;

#[cfg(test)]
pub mod test_utils;

// IR 模型重新导出
pub use generator::ir::{
    ConflictResolution,
    EntityName,
    EntityPermissionConfig,
    ExpressionType,
    FieldName,
    FieldOaStrategy,
    FieldPermission,
    FieldPermissionConfig,
    GeneratorBusinessRule,
    GeneratorEntity,
    GeneratorEnum,
    GeneratorField,
    GeneratorFieldType,
    GeneratorLifecycleHook,
    GeneratorModel,
    GeneratorRelation,
    GeneratorRelationType,
    // Phase 24: 4D Space
    GeneratorState,
    // Phase 22: Behavior & State Machine
    GeneratorStateMachine,
    GeneratorTransition,
    ImpliedPermission,
    InheritanceStrategy,
    InheritedPermission,
    InstancePermission,
    NgacAssociation,
    NgacMappingConfig,
    NgacMappingResult,
    NgacObjectAttribute,
    NgacUserAttribute,
    OntologyPermission,
    // Phase 25: Ontology-based Permissions
    PermissionAction,
    PermissionConflict,
    PermissionConflictType,
    PermissionInferenceResult,
    PermissionReasoner,
    PrimaryKeyType,
};

// 重新导出生成器
pub use generator::{
    GenerateError, GeneratedFile, GeneratedOutput, GenerationMetadata, Generator, ValidationError,
};

// 重新导出可视化引擎
pub use generator::ir::ontology_visualizer::{VisualGraph, VisualizerEngine};

// 重新导出预览模块
pub use api::generate::preview::{
    PreviewChangeType, PreviewFileEntry, PreviewRequest, PreviewResponse, PreviewService,
    PreviewStats, SerializableConflict, SerializableConflictReport,
};

// 重新导出版本管理
pub use version::{Version, VersionConstraint, VersionParseError};

pub use version::compat::{
    Change, ChangeCategory, CompatibilityChecker, CompatibilityError, CompatibilityReport,
    ImpactLevel, MigrationStep, MigrationStepType,
};

pub use version::manager::{UpgradePath, VersionManager, VersionRecord, VersionTimelineEntry};

// 重新导出版本升级迁移管理器
// 重新导出运行时行为类型（已从 runtime_engine 迁移至 runtime_contract）
pub use runtime_contract::behavior::{
    BehaviorMetadata, BusinessRule, BusinessRules, EntityBehavior, LifecycleEvent, LifecycleHook,
    LifecycleHooks, RuleEvaluation, RuleEvaluationSummary, RuleTrigger, State, StateMachine,
    Transition, TransitionTable,
};

// 重新导出规则与约束类型
pub use generator::rules::{
    ComparisonOp, ConflictDetector, ConflictReport, ConflictSeverity,
    ConflictType as DslConflictType, Constraint, ConstraintExpr, ConstraintLevel, Constraints,
    EntityRules, LiteralValue, RuleAtom, RuleContext, RuleOperation, RulesMetadata, SwrlRule,
    SwrlRuleSet, Term,
};

// 重新导出 CLI 模块
pub use cli::{
    BatchArgs, Cli, CliConfig, CliError, CliRunner, Commands, ExportArgs, ExportFormat,
    GenerateArgs, GeneratorType, HistoryArgs, HistoryOutputFormat, InitArgs, PreviewArgs,
    RollbackArgs, ValidateArgs,
};

// 重新导出文档生成器
pub use docgen::{DiagramType, DocGenerator, MarkdownGenerator, MermaidDiagramGenerator};

/// 库版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// 架构原则：AppAgent 不消费文本型规约。正确性由代码化约束保证：
// typed IR（generator::ir）、validators（app-agent/validator.rs）、
// convention_checker、DB 实时查询（schema-info）。LLM prompt 只含蒸馏指令。
