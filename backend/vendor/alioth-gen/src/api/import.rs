//! 导入 API (旧版，基于 DSL，已弃用)
//!
//! 新的导入功能在 Phase 2 中基于 IR 模型重新实现

/// 导入服务
pub struct ImportService;

impl ImportService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ImportService {
    fn default() -> Self {
        Self::new()
    }
}

// 占位类型
pub struct ImportRequest;
pub struct ImportResponse;
pub struct ImportOptions;
pub struct ImportError;
pub struct ValidationResponse;
pub struct ValidationError;
pub fn import_dsl() {}
pub fn validate_dsl_content() {}

// 旧类型占位符
pub struct DslParser;
pub struct ParseOptions;
pub struct DslFormat;
pub struct DslError;
pub struct SourceLocation;
pub fn parse_dsl_from_str() {}
