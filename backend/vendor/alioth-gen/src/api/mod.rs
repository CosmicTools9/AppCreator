/// API 模块
///
/// 提供代码生成相关的 API 类型（预览、生成请求/响应等）。
/// 其他运行时 API（发布、版本、审计、市场、注册表）已迁移到 meta-services。
pub mod generate;
pub mod import;

// 重新导出导入 API (简化版，DSL 功能已移除)
pub use import::{
    ImportError, ImportOptions, ImportRequest, ImportResponse, ImportService, ValidationError,
    ValidationResponse,
};

// 占位类型
pub struct ApiError;
pub struct AppState;
pub fn format_extension() {}
pub fn format_mime_type() {}
