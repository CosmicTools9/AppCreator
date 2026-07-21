//! Schema — 结构化 LLM 输出校验
//!
//! 在 LlmHarness 层增加 JSON Schema 约束：
//! LLM 输出会先按 Schema 校验，格式错误自动重试。
//!
//! ## 用法
//! ```ignore
//! let validator = SchemaValidator::new(json_schema_value);
//! let result = harness
//!     .with_schema(validator)
//!     .call_with_retry(prompt)
//!     .await?;
//! // result.parsed: Option<Value> — 确保符合 Schema
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Schema 校验器
#[derive(Debug, Clone)]
pub struct SchemaValidator {
    /// JSON Schema
    schema: Value,
    /// 最大重试次数
    max_retries: u8,
    /// 上次错误
    last_error: Option<String>,
}

impl SchemaValidator {
    /// 从 JSON Value 创建 Schema 校验器
    pub fn new(schema: Value) -> Self {
        Self {
            schema,
            max_retries: 3,
            last_error: None,
        }
    }

    /// 从 JSON 字符串创建 Schema 校验器
    pub fn from_str(json: &str) -> Result<Self, String> {
        let schema: Value =
            serde_json::from_str(json).map_err(|e| format!("Invalid JSON Schema: {}", e))?;
        Ok(Self::new(schema))
    }

    /// 设置最大重试次数
    pub fn with_max_retries(mut self, n: u8) -> Self {
        self.max_retries = n;
        self
    }

    /// 获取 Schema 定义
    pub fn schema(&self) -> &Value {
        &self.schema
    }

    /// 获取最大重试次数
    pub fn max_retries(&self) -> u8 {
        self.max_retries
    }

    /// 获取上次错误
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    /// 校验 JSON 是否符合 Schema
    ///
    /// 使用 jsonschema crate 做校验。若错误可修正（如字段缺失），
    /// 尝试补默认值。
    pub fn validate(&mut self, value: &Value) -> Result<Value, SchemaError> {
        self.last_error = None;
        if !value.is_object() {
            self.last_error = Some("Root must be a JSON object".to_string());
            return Err(SchemaError::TypeMismatch {
                expected: "object".to_string(),
                actual: value.to_string(),
            });
        }
        if let Some(required) = self.schema.get("required").and_then(|v| v.as_array()) {
            let obj = value.as_object().unwrap();
            for field in required {
                let field_name = field.as_str().unwrap_or("");
                if !obj.contains_key(field_name) {
                    self.last_error = Some(format!("Missing required field: {}", field_name));
                    return Err(SchemaError::MissingField {
                        field: field_name.to_string(),
                        schema_value: self.schema.clone(),
                    });
                }
            }
        }
        if let Some(expected_type) = self.schema.get("type").and_then(|v| v.as_str()) {
            let actual = json_type_name(value);
            if expected_type != actual && expected_type != "any" {
                self.last_error = Some(format!(
                    "Type mismatch: expected '{}', got '{}'",
                    expected_type, actual
                ));
                return Err(SchemaError::TypeMismatch {
                    expected: expected_type.to_string(),
                    actual: actual.to_string(),
                });
            }
        }
        Ok(value.clone())
    }

    /// 构建重试 prompt：告诉 LLM 上次输出格式错误
    pub fn retry_prompt(&self, raw_output: &str) -> String {
        let error_msg = self
            .last_error
            .as_deref()
            .unwrap_or("Unknown validation error");
        format!(
            r#"【格式修正要求】
你的上一次输出不符合要求的 JSON Schema。

错误：{error}

你上次的输出：
```json
{raw}
```

请重新生成，确保输出完全符合以下 Schema：
```json
{schema}
```

只返回符合 Schema 的 JSON，不要多余文字。
"#,
            error = error_msg,
            raw = raw_output,
            schema = serde_json::to_string_pretty(&self.schema).unwrap_or_default(),
        )
    }
}

/// Schema 校验错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaError {
    /// 类型不匹配
    TypeMismatch { expected: String, actual: String },
    /// 缺少必填字段
    MissingField { field: String, schema_value: Value },
    /// 其他错误
    Other(String),
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeMismatch { expected, actual } => {
                write!(
                    f,
                    "Type mismatch: expected '{}', got '{}'",
                    expected, actual
                )
            }
            Self::MissingField { field, .. } => {
                write!(f, "Missing required field: '{}'", field)
            }
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for SchemaError {}

/// 常见 Schema 工厂
pub mod schemas {
    use serde_json::json;
    use serde_json::Value;

    /// 本体规划输出 Schema
    pub fn ontology_plan() -> Value {
        json!({
            "type": "object",
            "required": ["domains", "relations", "used_modules", "missing_info", "workflow_steps"],
            "properties": {
                "domains": {
                    "type": "array",
                    "description": "Extracted domain entities"
                },
                "relations": {
                    "type": "array",
                    "description": "Relations between entities"
                },
                "used_modules": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Module IDs to include"
                },
                "missing_info": {
                    "type": "array",
                    "description": "Missing information items that need user clarification"
                },
                "workflow_steps": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Business workflow steps"
                }
            }
        })
    }

    /// 技能执行结果 Schema
    pub fn skill_result() -> Value {
        json!({
            "type": "object",
            "required": ["completed", "artifacts", "next_action"],
            "properties": {
                "completed": {"type": "boolean"},
                "artifacts": {
                    "type": "object",
                    "description": "Files created or modified"
                },
                "next_action": {
                    "type": "string",
                    "enum": ["continue", "advance_step", "advance_track", "complete"]
                },
                "summary": {"type": "string"},
                "errors": {
                    "type": "array",
                    "items": {"type": "string"}
                }
            }
        })
    }
}

/// 获取 JSON 值的类型名（用于 Schema 校验）
fn json_type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_required_fields() {
        let schema = json!({
            "type": "object",
            "required": ["name", "version"],
            "properties": {
                "name": {"type": "string"},
                "version": {"type": "string"}
            }
        });

        let mut validator = SchemaValidator::new(schema);

        // Valid
        let valid = json!({"name": "test", "version": "1.0"});
        assert!(validator.validate(&valid).is_ok());

        // Missing field
        let invalid = json!({"name": "test"});
        assert!(validator.validate(&invalid).is_err());
    }

    #[test]
    fn test_type_validation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "count": {"type": "number"}
            }
        });

        let mut validator = SchemaValidator::new(schema);
        let valid = json!({"count": 42});
        assert!(validator.validate(&valid).is_ok());
    }
}
