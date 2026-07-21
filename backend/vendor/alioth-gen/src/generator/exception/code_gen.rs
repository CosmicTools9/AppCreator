//! Exception Code Generator
//!
//! 生成具体的异常处理代码，包括 Rust Error 枚举和 TypeScript 错误类

use crate::generator::ir::exception::{GeneratorException, HttpStatusCode};
use crate::generator::{GenerateError, GeneratedFile, GeneratedOutput, GenerationMetadata};

/// 异常代码生成器
pub struct ExceptionCodeGenerator;

impl ExceptionCodeGenerator {
    /// 生成 Rust 错误枚举
    pub fn generate_rust_error_enum(
        exceptions: &[GeneratorException],
    ) -> Result<GeneratedOutput, GenerateError> {
        let content = Self::generate_rust_error_enum_content(exceptions)?;

        let files = vec![GeneratedFile {
            path: "src/errors/error_enum.rs".into(),
            content: content.clone(),
            checksum: format!("{:x}", md5::compute(content.as_bytes())),
        }];

        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "exception_code_gen".to_string(),
                entity_count: exceptions.len(),
                c_file_count: 1,
            },
        })
    }

    /// 生成 Rust 错误枚举内容
    fn generate_rust_error_enum_content(
        exceptions: &[GeneratorException],
    ) -> Result<String, GenerateError> {
        let mut output = String::new();

        output.push_str("//! Error Enum\n");
        output.push_str("//!\n");
        output.push_str("//! Auto-generated error types using thiserror\n\n");

        output.push_str("use thiserror::Error;\n");
        output.push_str("use serde::{Deserialize, Serialize};\n\n");

        // Generate error code constants
        output.push_str("/// Error code constants\n");
        output.push_str("pub mod error_codes {\n");
        for exception in exceptions {
            if let Some(ref code) = exception.error_code {
                output.push_str(&format!(
                    "    pub const {}: &str = \"{}\";\n",
                    exception.error_code_constant, code
                ));
            }
        }
        output.push_str("}\n\n");

        // Generate the main Error enum
        output.push_str("/// Application error types\n");
        output.push_str("#[derive(Error, Debug, Clone, Serialize, Deserialize, PartialEq)]\n");
        output.push_str("pub enum Error {\n");

        for exception in exceptions {
            if exception.is_abstract {
                continue; // Skip abstract exceptions in enum
            }

            let variant_doc = format!(
                "    /// {} - HTTP {}\n",
                exception.name.raw,
                exception.http_status.as_u16()
            );
            output.push_str(&variant_doc);

            // Generate the error variant with fields
            let fields_str = Self::generate_enum_variant_fields(exception);
            output.push_str(&format!(
                "    #[error(\"{}: {{message}}\")]\n",
                exception.name.pascal
            ));
            output.push_str(&format!("    {}{},\n", exception.name.pascal, fields_str));
            output.push('\n');
        }

        // Add a generic variant
        output.push_str("    /// Generic error\n");
        output.push_str("    #[error(\"Generic error: {message}\")]\n");
        output.push_str("    Generic { message: String, code: String },\n");

        output.push_str("}\n\n");

        // Generate impl block for Error
        output.push_str("impl Error {\n");

        // Generate constructor for each error variant
        for exception in exceptions {
            if exception.is_abstract {
                continue;
            }

            output.push_str(&Self::generate_error_constructor(exception)?);
            output.push('\n');
        }

        // Generate http_status method
        output.push_str("    /// Get HTTP status code for this error\n");
        output.push_str("    pub fn http_status(&self) -> u16 {\n");
        output.push_str("        match self {\n");
        for exception in exceptions {
            if exception.is_abstract {
                continue;
            }
            output.push_str(&format!(
                "            Self::{} {{ .. }} => {},\n",
                exception.name.pascal,
                exception.http_status.as_u16()
            ));
        }
        output.push_str("            Self::Generic { .. } => 500,\n");
        output.push_str("        }\n");
        output.push_str("    }\n\n");

        // Generate error_code method
        output.push_str("    /// Get error code for this error\n");
        output.push_str("    pub fn error_code(&self) -> &str {\n");
        output.push_str("        match self {\n");
        for exception in exceptions {
            if exception.is_abstract {
                continue;
            }
            output.push_str(&format!(
                "            Self::{} {{ code, .. }} => code.as_str(),\n",
                exception.name.pascal
            ));
        }
        output.push_str("            Self::Generic { code, .. } => code.as_str(),\n");
        output.push_str("        }\n");
        output.push_str("    }\n\n");

        // Generate is methods for error type checking
        output.push_str("    /// Check if this is a client error (4xx)\n");
        output.push_str("    pub fn is_client_error(&self) -> bool {\n");
        output.push_str("        (400..500).contains(&self.http_status())\n");
        output.push_str("    }\n\n");

        output.push_str("    /// Check if this is a server error (5xx)\n");
        output.push_str("    pub fn is_server_error(&self) -> bool {\n");
        output.push_str("        self.http_status() >= 500\n");
        output.push_str("    }\n");

        output.push_str("}\n\n");

        // Generate From implementations
        output.push_str("// Implementations for common error conversions\n\n");

        // From<&str> and From<String>
        output.push_str(
            r#"impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self::Generic {
            message: s.to_string(),
            code: "GENERIC_ERROR".to_string(),
        }
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Self::Generic {
            message: s,
            code: "GENERIC_ERROR".to_string(),
        }
    }
}

"#,
        );

        // From<std::io::Error>
        output.push_str(
            r#"impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => Self::NotFoundError {
                message: e.to_string(),
                code: error_codes::NOTFOUND_ERROR.to_string(),
                resource: "unknown".to_string(),
            },
            _ => Self::Generic {
                message: e.to_string(),
                code: "IO_ERROR".to_string(),
            },
        }
    }
}

"#,
        );

        Ok(output)
    }

    /// 生成枚举变体字段
    fn generate_enum_variant_fields(exception: &GeneratorException) -> String {
        let mut fields = vec!["message: String".to_string(), "code: String".to_string()];

        for field in &exception.fields {
            let field_type = Self::rust_field_type(&field.field_type);
            fields.push(format!("{}: {}", field.name.snake, field_type));
        }

        format!("{{ {} }}", fields.join(", "))
    }

    /// 生成错误构造函数
    fn generate_error_constructor(exception: &GeneratorException) -> Result<String, GenerateError> {
        let mut output = String::new();

        output.push_str(&format!(
            "    /// Create a new {} error\n",
            exception.name.pascal
        ));
        output.push_str(&format!("    pub fn {}(\n", exception.name.snake));
        output.push_str("        _message: impl Into<String>,\n");

        // Add parameters for each field
        for field in &exception.fields {
            output.push_str(&format!(
                "        {}: {},\n",
                field.name.snake,
                Self::rust_field_type(&field.field_type)
            ));
        }

        output.push_str("    ) -> Self {\n");
        output.push_str(&format!("        Self::{} {{\n", exception.name.pascal));
        output.push_str("            message: _message.into(),\n");
        output.push_str(&format!(
            "            code: error_codes::{}.to_string(),\n",
            exception.error_code_constant
        ));

        for field in &exception.fields {
            output.push_str(&format!("            {},\n", field.name.snake));
        }

        output.push_str("        }\n");
        output.push_str("    }\n");

        Ok(output)
    }

    /// 获取 Rust 字段类型
    fn rust_field_type(
        field_type: &crate::generator::ir::exception::GeneratorExceptionFieldType,
    ) -> String {
        use crate::generator::ir::exception::GeneratorExceptionFieldType;

        match field_type {
            GeneratorExceptionFieldType::String => "String".to_string(),
            GeneratorExceptionFieldType::Integer => "i64".to_string(),
            GeneratorExceptionFieldType::Boolean => "bool".to_string(),
            GeneratorExceptionFieldType::Reference(name) => name.clone(),
            GeneratorExceptionFieldType::Array(inner) => {
                format!("Vec<{}>", Self::rust_field_type(inner))
            }
        }
    }

    /// 生成 TypeScript 错误类
    pub fn generate_typescript_error_module(
        exceptions: &[GeneratorException],
    ) -> Result<GeneratedOutput, GenerateError> {
        let content = Self::generate_typescript_error_content(exceptions)?;

        let files = vec![GeneratedFile {
            path: "src/errors/errors.ts".into(),
            content: content.clone(),
            checksum: format!("{:x}", md5::compute(content.as_bytes())),
        }];

        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "exception_code_gen".to_string(),
                entity_count: exceptions.len(),
                c_file_count: 1,
            },
        })
    }

    /// 生成 TypeScript 错误内容
    fn generate_typescript_error_content(
        exceptions: &[GeneratorException],
    ) -> Result<String, GenerateError> {
        let mut output = String::new();

        output.push_str("// Error Types\n");
        output.push_str("// Auto-generated error classes\n\n");

        // Generate error code enum
        output.push_str("export enum ErrorCode {\n");
        for exception in exceptions {
            if let Some(ref code) = exception.error_code {
                output.push_str(&format!(
                    "    {} = '{}',\n",
                    exception.name.screaming_snake, code
                ));
            }
        }
        output.push_str("    GENERIC_ERROR = 'GENERIC_ERROR',\n");
        output.push_str("}\n\n");

        // Generate HTTP status enum
        output.push_str("export enum HttpStatus {\n");
        let mut statuses = std::collections::HashSet::new();
        for exception in exceptions {
            statuses.insert(exception.http_status.as_u16());
        }
        let mut statuses: Vec<_> = statuses.into_iter().collect();
        statuses.sort();
        for status in statuses {
            let name = match status {
                400 => "BAD_REQUEST",
                401 => "UNAUTHORIZED",
                403 => "FORBIDDEN",
                404 => "NOT_FOUND",
                409 => "CONFLICT",
                422 => "UNPROCESSABLE_ENTITY",
                429 => "TOO_MANY_REQUESTS",
                500 => "INTERNAL_SERVER_ERROR",
                503 => "SERVICE_UNAVAILABLE",
                _ => &format!("HTTP_{}", status),
            };
            output.push_str(&format!("    {} = {},\n", name, status));
        }
        output.push_str("}\n\n");

        // Generate base AppError interface
        output.push_str(
            r#"export interface AppError {
    readonly name: string;
    readonly message: string;
    readonly code: ErrorCode;
    readonly httpStatus: HttpStatus;
    readonly timestamp: Date;
    toJSON(): Record<string, unknown>;
}

"#,
        );

        // Generate error type guard functions
        output.push_str("// Error type guards\n");
        for exception in exceptions {
            output.push_str(&format!(
                r#"export function is{}Error(error: unknown): error is {}Error {{
    return error instanceof {}Error;
}}

"#,
                exception.name.pascal, exception.name.pascal, exception.name.pascal
            ));
        }

        // Generate error classes
        for exception in exceptions {
            output.push_str(&Self::generate_typescript_error_class(exception)?);
            output.push('\n');
        }

        // Generate error factory
        output.push_str("// Error Factory\n");
        output.push_str("export function createError(\n");
        output.push_str("    code: ErrorCode,\n");
        output.push_str("    message: string,\n");
        output.push_str("    details?: Record<string, unknown>\n");
        output.push_str("): AppError {\n");
        output.push_str("    switch (code) {\n");

        for exception in exceptions {
            if exception.is_abstract {
                continue;
            }
            if let Some(ref _code) = exception.error_code {
                output.push_str(&format!(
                    "        case ErrorCode.{}:\n            return new {}Error(message, details);\n",
                    exception.name.screaming_snake, exception.name.pascal
                ));
            }
        }

        output.push_str(
            r#"        default:
            return new GenericError(message);
    }
}

"#,
        );

        // Generate GenericError class
        output.push_str(
            r#"export class GenericError implements AppError {
    readonly name = 'GenericError';
    readonly code = ErrorCode.GENERIC_ERROR;
    readonly httpStatus = HttpStatus.INTERNAL_SERVER_ERROR;
    readonly timestamp = new Date();

    constructor(public readonly message: string) {}

    toJSON() {
        return {
            name: this.name,
            message: this.message,
            code: this.code,
            httpStatus: this.httpStatus,
            timestamp: this.timestamp.toISOString(),
        };
    }
}
"#,
        );

        Ok(output)
    }

    /// 生成 TypeScript 错误类
    fn generate_typescript_error_class(
        exception: &GeneratorException,
    ) -> Result<String, GenerateError> {
        let mut output = String::new();

        output.push_str(&format!(
            "export class {}Error implements AppError {{\n",
            exception.name.pascal
        ));
        output.push_str(&format!(
            "    readonly name = '{}Error';\n",
            exception.name.pascal
        ));
        output.push_str(&format!(
            "    readonly code = ErrorCode.{};\n",
            exception.name.screaming_snake
        ));
        output.push_str(&format!(
            "    readonly httpStatus = HttpStatus.{};\n",
            Self::http_status_enum_name(exception.http_status)
        ));
        output.push_str("    readonly timestamp = new Date();\n\n");

        // Constructor
        output.push_str("    constructor(\n");
        output.push_str("        public readonly message: string,\n");

        // Add fields as constructor parameters
        for field in &exception.fields {
            output.push_str(&format!(
                "        public readonly {}: {},\n",
                field.name.camel,
                Self::typescript_field_type(&field.field_type)
            ));
        }

        output.push_str("    ) {}\n\n");

        // toJSON method
        output.push_str("    toJSON(): Record<string, unknown> {\n");
        output.push_str("        return {\n");
        output.push_str("            name: this.name,\n");
        output.push_str("            message: this.message,\n");
        output.push_str("            code: this.code,\n");
        output.push_str("            httpStatus: this.httpStatus,\n");
        output.push_str("            timestamp: this.timestamp.toISOString(),\n");

        for field in &exception.fields {
            output.push_str(&format!(
                "            {}: this.{},\n",
                field.name.camel, field.name.camel
            ));
        }

        output.push_str("        };\n");
        output.push_str("    }\n");
        output.push_str("}\n");

        Ok(output)
    }

    /// 获取 TypeScript 字段类型
    fn typescript_field_type(
        field_type: &crate::generator::ir::exception::GeneratorExceptionFieldType,
    ) -> String {
        use crate::generator::ir::exception::GeneratorExceptionFieldType;

        match field_type {
            GeneratorExceptionFieldType::String => "string".to_string(),
            GeneratorExceptionFieldType::Integer => "number".to_string(),
            GeneratorExceptionFieldType::Boolean => "boolean".to_string(),
            GeneratorExceptionFieldType::Reference(name) => name.clone(),
            GeneratorExceptionFieldType::Array(inner) => {
                format!("{}[]", Self::typescript_field_type(inner))
            }
        }
    }

    /// 获取 HTTP 状态枚举名称
    fn http_status_enum_name(status: HttpStatusCode) -> &'static str {
        match status {
            HttpStatusCode::BadRequest => "BAD_REQUEST",
            HttpStatusCode::Unauthorized => "UNAUTHORIZED",
            HttpStatusCode::Forbidden => "FORBIDDEN",
            HttpStatusCode::NotFound => "NOT_FOUND",
            HttpStatusCode::Conflict => "CONFLICT",
            HttpStatusCode::UnprocessableEntity => "UNPROCESSABLE_ENTITY",
            HttpStatusCode::TooManyRequests => "TOO_MANY_REQUESTS",
            HttpStatusCode::InternalServerError => "INTERNAL_SERVER_ERROR",
            HttpStatusCode::ServiceUnavailable => "SERVICE_UNAVAILABLE",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::exception::{
        ExceptionFieldName, ExceptionName, GeneratorExceptionField, GeneratorExceptionFieldType,
        HttpStatusCode,
    };

    fn create_test_exception(name: &str, http_status: HttpStatusCode) -> GeneratorException {
        GeneratorException {
            name: ExceptionName::from_raw(name),
            description: None,
            parent_exceptions: vec![],
            fields: vec![GeneratorExceptionField {
                name: ExceptionFieldName::from_raw("field"),
                field_type: GeneratorExceptionFieldType::String,
                required: true,
                description: None,
            }],
            error_code: Some(format!("ERR_{}", name.to_uppercase())),
            error_code_constant: format!("{}_ERROR", name.to_uppercase()),
            http_status,
            is_abstract: false,
            inheritance_depth: 0,
            i18n_message: None,
        }
    }

    #[test]
    fn test_generate_rust_error_enum() {
        let exceptions = vec![
            create_test_exception("Validation", HttpStatusCode::BadRequest),
            create_test_exception("NotFound", HttpStatusCode::NotFound),
        ];

        let output = ExceptionCodeGenerator::generate_rust_error_enum_content(&exceptions).unwrap();

        assert!(output.contains("pub enum Error"));
        assert!(output.contains("Validation"));
        assert!(output.contains("NotFound"));
        assert!(output.contains("error_codes"));
        assert!(output.contains("http_status"));
    }

    #[test]
    fn test_generate_typescript_error_content() {
        let exceptions = vec![create_test_exception(
            "Validation",
            HttpStatusCode::BadRequest,
        )];

        let output =
            ExceptionCodeGenerator::generate_typescript_error_content(&exceptions).unwrap();

        assert!(output.contains("export enum ErrorCode"));
        assert!(output.contains("export enum HttpStatus"));
        assert!(output.contains("export class ValidationError"));
    }
}
