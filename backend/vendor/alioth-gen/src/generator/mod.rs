//! 代码生成框架
//!
//! 提供三层 IR 系统、模板引擎和生成器注册表
pub mod convention_checker;

pub mod ast;
pub mod ir;
pub mod output;
pub mod registry;

/// Zod Validation Generator (Phase 4)
pub mod zod;

/// Exception Generator (Phase 26)
pub mod exception;

/// Quality Validation Generator (Phase 27)
pub mod quality;

/// API Generator (Phase 5)
pub mod api;

/// Frontend Component Generator (Phase 6)
pub mod frontend;

/// Module Backend Generator (Phase 47)
pub mod module;

/// Repository Generator with audit support
pub mod repository;

/// Rules module (migrated from dsl::rules)
pub mod rules;

/// LLM-based Code Generator
pub mod llm_generator;

/// OntologyModel → Prompt 结构化转换器 (LLM MVP)
pub mod prompt_builder;

use ir::GeneratorModel;

// 重新导出 Zod 生成器
pub use zod::{
    FullZodGenerator, HookGenerator, ReactHookFormGenerator, TypeScriptGenerator, ZodGenerator,
    ZodSchemaGenerator, ZodTypeMapper,
};

// 重新导出 API 生成器
pub use api::{
    ActixHandlerGenerator, ApiContract, ApiGenerator, ClientGeneratorOptions, ClientType,
    FrontendClientGenerator, GenerationContext, OpenApiGenerator, OpenApiSpec,
};

// 重新导出 Frontend 生成器
pub use frontend::{
    ChartComponentGenerator, ChartGeneratorOptions, ChartType, DataTableGenerator,
    FormComponentGenerator, FrontendComponentGenerator,
};

// 重新导出 Quality 生成器
pub use quality::{QualityGenerator, QualityGeneratorConfig, ReportFormat};

// 重新导出 Module 生成器 (Phase 47)
pub use module::{ModuleApiGenOptions, ModuleApiGenerator};

// 重新导出 Repository 生成器 (with audit support)
pub use repository::SqlxRepositoryGenerator;

use std::path::PathBuf;
use thiserror::Error;

/// 核心 trait，所有代码生成器必须实现
pub trait Generator: Send + Sync {
    /// 生成器唯一标识符
    fn name(&self) -> &'static str;

    /// 从模型生成代码
    fn generate(&self, model: &GeneratorModel) -> Result<GeneratedOutput, GenerateError>;

    /// 验证模型是否可以被此生成器处理
    fn validate(&self, model: &GeneratorModel) -> Result<(), ValidationError>;

    /// 此生成器是否支持增量更新
    fn supports_incremental(&self) -> bool;

    /// 此生成器生成的文件扩展名
    fn file_extensions(&self) -> Vec<&'static str>;
}

/// 代码生成的输出
#[derive(Debug, Clone)]
pub struct GeneratedOutput {
    pub files: Vec<GeneratedFile>,
    pub metadata: GenerationMetadata,
}

/// 单个生成的文件
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    pub path: PathBuf,
    pub content: String,
    pub checksum: String,
}

/// 生成过程的元数据
#[derive(Debug, Clone)]
pub struct GenerationMetadata {
    pub generator_name: String,
    pub entity_count: usize,
    pub c_file_count: usize,
}

/// 代码生成过程中的错误
#[derive(Error, Debug)]
pub enum GenerateError {
    #[error("模板错误: {0}")]
    Template(String),

    #[error("验证失败: {0}")]
    Validation(String),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("转换错误: {0}")]
    Transform(String),
}

/// 验证过程中的错误
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("不支持的字段类型: 实体 {entity} 中的字段 {field}")]
    UnsupportedFieldType { entity: String, field: String },

    #[error("缺少必需字段: 实体 {entity} 中的字段 {field}")]
    MissingRequiredField { entity: String, field: String },

    #[error("无效的字段: 实体 {entity} 中的字段 {field}: {reason}")]
    InvalidField {
        entity: String,
        field: String,
        reason: String,
    },

    #[error("无效的名称: {name}: {reason}")]
    InvalidName { name: String, reason: String },

    #[error("检测到循环依赖: {0}")]
    CircularDependency(String),

    #[error("验证失败: {0}")]
    Other(String),
}

/// 支持增量更新的生成器 trait
pub trait IncrementalGenerator: Generator {
    /// 生成当前代码与生成代码之间的差异报告
    fn generate_diff(
        &self,
        model: &GeneratorModel,
        existing_files: &[ExistingFile],
    ) -> Result<DiffReport, GenerateError>;

    /// 检测合并冲突
    fn detect_conflicts(
        &self,
        generated: &GeneratedOutput,
        existing: &[ExistingFile],
    ) -> Vec<MergeConflict>;
}

/// 磁盘上的现有文件
#[derive(Debug, Clone)]
pub struct ExistingFile {
    pub path: PathBuf,
    pub content: String,
    pub checksum: String,
}

/// 增量生成的差异报告
#[derive(Debug, Clone)]
pub struct DiffReport {
    pub files_changed: Vec<FileChange>,
    pub files_added: Vec<GeneratedFile>,
    pub files_removed: Vec<PathBuf>,
}

/// 单个文件变更
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: PathBuf,
    pub old_content: String,
    pub new_content: String,
    pub diff: String,
}

/// 合并冲突
#[derive(Debug, Clone)]
pub struct MergeConflict {
    pub path: PathBuf,
    pub description: String,
}
