//! 代码生成 API 错误处理
//!
//! 定义代码生成 API 的错误类型和响应格式。

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 代码生成 API 错误码
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenerateApiErrorCode {
    /// 无效的模型数据
    InvalidModel,
    /// 验证失败
    ValidationFailed,
    /// 生成失败
    GenerationFailed,
    /// 内部错误
    InternalError,
}

/// 代码生成 API 错误
#[derive(Debug, Serialize, Clone)]
pub struct GenerateApiError {
    /// 错误码
    pub code: GenerateApiErrorCode,
    /// 错误消息
    pub message: String,
    /// 额外详情
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
    /// 可操作建议
    pub suggestions: Vec<String>,
    /// HTTP 状态码
    #[serde(skip)]
    pub status_code: u16,
}

impl GenerateApiError {
    /// 创建新的错误
    pub fn new(code: GenerateApiErrorCode, message: impl Into<String>) -> Self {
        let status_code = match code {
            GenerateApiErrorCode::InvalidModel => 400,
            GenerateApiErrorCode::ValidationFailed => 400,
            GenerateApiErrorCode::GenerationFailed => 422,
            GenerateApiErrorCode::InternalError => 500,
        };
        Self {
            code,
            message: message.into(),
            details: None,
            suggestions: vec![],
            status_code,
        }
    }

    /// 添加详情
    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    /// 添加可操作建议
    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }

    /// 获取 HTTP 状态码
    pub fn status_code(&self) -> u16 {
        self.status_code
    }
}

impl std::fmt::Display for GenerateApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {}", self.code, self.message)
    }
}

impl std::error::Error for GenerateApiError {}

/// 错误响应结构
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

/// 错误详情
#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
    pub suggestions: Vec<String>,
}

impl From<crate::generator::GenerateError> for GenerateApiError {
    fn from(err: crate::generator::GenerateError) -> Self {
        let suggestions = match &err {
            crate::generator::GenerateError::Template(_) => vec![
                "Check that all entity and field names use valid characters and are not empty."
                    .to_string(),
            ],
            crate::generator::GenerateError::Validation(_) => vec![
                "Review the model for missing required fields or unsupported field types."
                    .to_string(),
            ],
            crate::generator::GenerateError::Io(_) => {
                vec!["Ensure the generation output directory exists and is writable.".to_string()]
            }
            crate::generator::GenerateError::Transform(_) => {
                vec!["Verify the IR model structure matches the expected schema.".to_string()]
            }
        };
        Self::new(GenerateApiErrorCode::GenerationFailed, err.to_string())
            .with_suggestions(suggestions)
    }
}

impl From<crate::generator::ValidationError> for GenerateApiError {
    fn from(err: crate::generator::ValidationError) -> Self {
        let details = match &err {
            crate::generator::ValidationError::UnsupportedFieldType { entity, field } => {
                serde_json::json!({
                    "entity": entity,
                    "field": field
                })
            }
            crate::generator::ValidationError::MissingRequiredField { entity, field } => {
                serde_json::json!({
                    "entity": entity,
                    "field": field
                })
            }
            crate::generator::ValidationError::InvalidField {
                entity,
                field,
                reason,
            } => {
                serde_json::json!({
                    "entity": entity,
                    "field": field,
                    "reason": reason
                })
            }
            crate::generator::ValidationError::InvalidName { name, reason } => {
                serde_json::json!({
                    "name": name,
                    "reason": reason
                })
            }
            crate::generator::ValidationError::CircularDependency(msg) => {
                serde_json::json!({ "dependency": msg })
            }
            crate::generator::ValidationError::Other(msg) => {
                serde_json::json!({ "message": msg })
            }
        };

        let suggestions = match &err {
            crate::generator::ValidationError::UnsupportedFieldType { .. } => vec![
                "Change the field type to a supported type (Text, Integer, BigInt, Decimal, Boolean, DateTime, Json, Enum).".to_string(),
            ],
            crate::generator::ValidationError::MissingRequiredField { .. } => vec![
                "Add the missing field or mark it as nullable.".to_string(),
            ],
            crate::generator::ValidationError::InvalidField { .. } => vec![
                "Review the field configuration for invalid values.".to_string(),
            ],
            crate::generator::ValidationError::InvalidName { .. } => vec![
                "Use only alphanumeric characters and underscores for names.".to_string(),
            ],
            crate::generator::ValidationError::CircularDependency(_) => vec![
                "Break the circular relation by introducing an intermediate entity.".to_string(),
            ],
            crate::generator::ValidationError::Other(_) => vec![
                "Check the model DSL syntax and entity definitions.".to_string(),
            ],
        };

        Self::new(GenerateApiErrorCode::ValidationFailed, err.to_string())
            .with_details(details)
            .with_suggestions(suggestions)
    }
}
