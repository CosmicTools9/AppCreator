//! 中间表示层 (Intermediate Representation)
//!
//! 架构分层：
//! - Ontology Layer: 本体语义表达模型 (ontology.rs)
//! - Visualizer Layer: 可视化推演模型 (ontology_visualizer.rs)
//! - LLM Contract Layer: LLM 代码生成接口契约 (llm_contract.rs)
//! - Generator Model: 代码生成核心模型 (ir2.rs)

// 新架构 IR 模块
pub mod llm_contract;
pub mod ontology;
pub mod ontology_reasoner;
pub mod ontology_visualizer;

// 核心生成模型
pub mod ir2;
pub mod meta_transformer;
pub mod module;
pub mod module_convert;
pub mod naming;
pub mod ontology_transformer;
pub mod validator;

// 扩展模块
pub mod exception;
pub mod permission;
pub mod quality;

// 重新导出新架构 IR 模块
#[allow(ambiguous_glob_reexports)]
pub use llm_contract::*;
#[allow(ambiguous_glob_reexports)]
pub use ontology::*;
pub use ontology_reasoner::{OntologyCache, OntologyReasoner};
pub use ontology_visualizer::{VisualGraph, VisualizerEngine};

// 重新导出核心生成模型（ir2）
pub use ir2::*;

// 重新导出 Meta → Generator 转换器
pub use meta_transformer::{MetaCollection, MetaField, MetaModelTransformer};

// 重新导出权限模块 - Phase 25
pub use permission::{
    ConflictResolution, ConflictType as PermissionConflictType, EntityPermissionConfig,
    ExpressionType, FieldOaStrategy, FieldPermission, FieldPermissionConfig, ImpliedPermission,
    InheritanceStrategy, InheritedPermission, InstancePermission, NgacAssociation,
    NgacMappingConfig, NgacMappingResult, NgacObjectAttribute, NgacUserAttribute,
    OntologyPermission, PermissionAction, PermissionConflict, PermissionInferenceResult,
    PermissionReasoner,
};

// 重新导出异常处理 - Phase 26
pub use exception::{
    ErrorCodeGenerator,
    ExceptionFieldName,
    ExceptionHandlerName,
    ExceptionHandlerRegistry,
    ExceptionHierarchyAnalyzer,
    ExceptionName,
    // IR-2 Types
    GeneratorException,
    GeneratorExceptionField,
    GeneratorExceptionFieldType,
    GeneratorExceptionHandler,
    GeneratorI18nMessage,
    GeneratorThrowsClause,
    // Utilities
    HttpStatusCode,
    I18nMessageTemplate,
    // IR-1 Types
    MetaException,
    MetaExceptionField,
    MetaExceptionFieldType,
    MetaExceptionHandler,
    MetaThrowsClause,
};

// 重新导出质量验证 - Phase 27
pub use quality::{
    GeneratorQualityConfig,
    // IR-2 Types
    GeneratorQualityRule,
    MetaQualityRule,
    OntologyQualityMetrics,
    QualityCheckSql,
    QualityDimension,
    // IR-1 Types
    QualityMetric,
    QualityRecommendation,
    QualityReport,
    QualityResultType,
    QualityScore,
    QualitySummary,
    QualityViolation,
    ViolationSeverity,
};
