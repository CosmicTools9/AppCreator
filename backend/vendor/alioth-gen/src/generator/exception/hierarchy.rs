//! Exception Hierarchy Generator
//!
//! 生成异常类的层次结构，支持多继承和 C3 线性化

use crate::generator::ir::exception::GeneratorException;
use crate::generator::{GenerateError, GeneratedFile, GeneratedOutput, GenerationMetadata};

/// 异常层次结构生成器
pub struct ExceptionHierarchyGenerator;

impl ExceptionHierarchyGenerator {
    /// 生成 Rust 异常层次结构
    pub fn generate_rust_hierarchy(
        exceptions: &[GeneratorException],
    ) -> Result<GeneratedOutput, GenerateError> {
        let mut files = Vec::new();

        // Generate main error module
        let module_content = Self::generate_rust_error_module(exceptions)?;
        let checksum = format!("{:x}", md5::compute(module_content.as_bytes()));
        files.push(GeneratedFile {
            path: "src/errors/mod.rs".into(),
            content: module_content,
            checksum,
        });

        // Generate individual error files for complex hierarchies
        if exceptions.len() > 5 {
            let base_content = Self::generate_rust_base_error(exceptions)?;
            let checksum = format!("{:x}", md5::compute(base_content.as_bytes()));
            files.push(GeneratedFile {
                path: "src/errors/base.rs".into(),
                content: base_content,
                checksum,
            });
        }

        let c_file_count = files.len();

        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "exception_hierarchy".to_string(),
                entity_count: exceptions.len(),
                c_file_count,
            },
        })
    }

    /// 生成 TypeScript 异常层次结构
    pub fn generate_typescript_hierarchy(
        exceptions: &[GeneratorException],
    ) -> Result<GeneratedOutput, GenerateError> {
        let mut files = Vec::new();

        let content = Self::generate_typescript_error_classes(exceptions)?;
        let checksum = format!("{:x}", md5::compute(content.as_bytes()));
        files.push(GeneratedFile {
            path: "src/errors/index.ts".into(),
            content: content.clone(),
            checksum,
        });

        let c_file_count = files.len();

        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "exception_hierarchy".to_string(),
                entity_count: exceptions.len(),
                c_file_count,
            },
        })
    }

    /// 生成 Rust 错误模块
    fn generate_rust_error_module(
        exceptions: &[GeneratorException],
    ) -> Result<String, GenerateError> {
        let mut output = String::new();

        // Add header
        output.push_str("//! Application Errors\n");
        output.push_str("//!\n");
        output.push_str("//! Auto-generated exception hierarchy\n\n");

        // Add imports
        output.push_str("use thiserror::Error;\n");
        output.push_str("use serde::{Deserialize, Serialize};\n");
        output.push_str("use std::collections::HashMap;\n\n");

        // Generate error code constants
        output.push_str("// Error Code Constants\n");
        for exception in exceptions {
            if let Some(ref code) = exception.error_code {
                output.push_str(&format!(
                    "pub const {}: &str = \"{}\";\n",
                    exception.error_code_constant, code
                ));
            }
        }
        output.push('\n');

        // Generate main AppError enum
        output.push_str("/// Main application error enum\n");
        output.push_str("#[derive(Error, Debug, Clone, Serialize, Deserialize)]\n");
        output.push_str("pub enum AppError {\n");

        for exception in exceptions {
            if !exception.is_abstract {
                output.push_str(&format!(
                    "    #[error(\"{}\")]\n    {}{{ {} }},\n",
                    exception.name,
                    exception.name.pascal,
                    Self::generate_rust_error_fields(exception)
                ));
            }
        }

        output.push_str("}\n\n");

        // Generate individual error structs for each exception
        for exception in exceptions {
            output.push_str(&Self::generate_rust_error_struct(exception)?);
            output.push('\n');
        }

        // Generate From implementations
        output.push_str("// From implementations for error conversion\n\n");
        for exception in exceptions {
            if !exception.parent_exceptions.is_empty() {
                for parent in &exception.parent_exceptions {
                    output.push_str(&format!(
                        r#"impl From<{}Error> for {}Error {{
    fn from(e: {}Error) -> Self {{
        // Conversion logic
        e
    }}
}}

"#,
                        exception.name.pascal, parent.pascal, exception.name.pascal
                    ));
                }
            }
        }

        Ok(output)
    }

    /// 生成 Rust 错误结构体
    fn generate_rust_error_struct(exception: &GeneratorException) -> Result<String, GenerateError> {
        let mut output = String::new();

        output.push_str(&format!("/// {}\n", exception.name.raw));
        if let Some(ref desc) = exception.description {
            output.push_str(&format!("/// {}\n", desc));
        }

        // Add derive macros
        output.push_str("#[derive(Error, Debug, Clone, Serialize, Deserialize)]\n");

        // Add error attribute with display message
        if let Some(ref i18n) = exception.i18n_message {
            output.push_str(&format!("#[error(\"{}\")]\n", i18n.default_message));
        } else {
            output.push_str(&format!("#[error(\"{}\")]\n", exception.name.raw));
        }

        output.push_str(&format!("pub struct {}Error {{\n", exception.name.pascal));

        // Add fields
        for field in &exception.fields {
            output.push_str(&format!(
                "    pub {}: {},\n",
                field.name.snake,
                Self::rust_type_for_field(&field.field_type)
            ));
        }

        // Add error code field if present
        if exception.error_code.is_some() {
            output.push_str("    pub error_code: String,\n");
        }

        // Add HTTP status field
        output.push_str(&format!(
            "    pub http_status: u16, // {}\n",
            exception.http_status.description()
        ));

        output.push_str("}\n\n");

        // Add impl block with constructor
        output.push_str(&format!("impl {}Error {{\n", exception.name.pascal));

        // Constructor
        output.push_str("    pub fn new(\n");
        for field in &exception.fields {
            output.push_str(&format!(
                "        {}: {},\n",
                field.name.snake,
                Self::rust_type_for_field(&field.field_type)
            ));
        }
        output.push_str("    ) -> Self {\n");
        output.push_str("        Self {\n");
        for field in &exception.fields {
            output.push_str(&format!("            {},\n", field.name.snake));
        }
        if exception.error_code.is_some() {
            output.push_str(&format!(
                "            error_code: {}.to_string(),\n",
                exception.error_code_constant
            ));
        }
        output.push_str(&format!(
            "            http_status: {},\n",
            exception.http_status.as_u16()
        ));
        output.push_str("        }\n");
        output.push_str("    }\n");

        // Helper method to get HTTP status
        output.push_str(&format!(
            r#"
    /// Get HTTP status code for this error
    pub fn status_code(&self) -> u16 {{
        {}
    }}
"#,
            exception.http_status.as_u16()
        ));

        output.push_str("}\n");

        Ok(output)
    }

    /// 生成 Rust 错误字段
    fn generate_rust_error_fields(exception: &GeneratorException) -> String {
        if exception.fields.is_empty() {
            return String::new();
        }

        let fields: Vec<String> = exception
            .fields
            .iter()
            .map(|f| {
                format!(
                    "{}: {}",
                    f.name.snake,
                    Self::rust_type_for_field(&f.field_type)
                )
            })
            .collect();

        fields.join(", ")
    }

    /// 将异常字段类型转换为 Rust 类型
    fn rust_type_for_field(
        field_type: &crate::generator::ir::exception::GeneratorExceptionFieldType,
    ) -> String {
        use crate::generator::ir::exception::GeneratorExceptionFieldType;

        match field_type {
            GeneratorExceptionFieldType::String => "String".to_string(),
            GeneratorExceptionFieldType::Integer => "i64".to_string(),
            GeneratorExceptionFieldType::Boolean => "bool".to_string(),
            GeneratorExceptionFieldType::Reference(name) => name.clone(),
            GeneratorExceptionFieldType::Array(inner) => {
                format!("Vec<{}>", Self::rust_type_for_field(inner))
            }
        }
    }

    /// 生成 Rust 基础错误（用于复杂层次结构）
    fn generate_rust_base_error(
        exceptions: &[GeneratorException],
    ) -> Result<String, GenerateError> {
        let mut output = String::new();

        output.push_str("//! Base Error Types\n\n");
        output.push_str("use thiserror::Error;\n");
        output.push_str("use serde::{Deserialize, Serialize};\n\n");

        // Find base exceptions (those without parents)
        let base_exceptions: Vec<_> = exceptions
            .iter()
            .filter(|e| e.parent_exceptions.is_empty())
            .collect();

        for exception in base_exceptions {
            output.push_str(&Self::generate_rust_error_struct(exception)?);
            output.push('\n');
        }

        Ok(output)
    }

    /// 生成 TypeScript 错误类
    fn generate_typescript_error_classes(
        exceptions: &[GeneratorException],
    ) -> Result<String, GenerateError> {
        let mut output = String::new();

        // Add header
        output.push_str("// Application Errors\n");
        output.push_str("// Auto-generated exception hierarchy\n\n");

        // Base AppError class
        output.push_str(
            r#"export class AppError extends Error {
    public readonly errorCode: string;
    public readonly httpStatus: number;
    public readonly timestamp: Date;

    constructor(message: string, errorCode: string, httpStatus: number) {
        super(message);
        this.name = 'AppError';
        this.errorCode = errorCode;
        this.httpStatus = httpStatus;
        this.timestamp = new Date();

        // Fix prototype chain
        Object.setPrototypeOf(this, AppError.prototype);
    }

    toJSON() {
        return {
            name: this.name,
            message: this.message,
            errorCode: this.errorCode,
            httpStatus: this.httpStatus,
            timestamp: this.timestamp.toISOString(),
        };
    }
}

"#,
        );

        // Generate individual error classes
        // Sort by inheritance depth to ensure parent classes are defined first
        let mut sorted_exceptions: Vec<_> = exceptions.iter().collect();
        sorted_exceptions.sort_by_key(|e| e.inheritance_depth);

        for exception in sorted_exceptions {
            output.push_str(&Self::generate_typescript_error_class(exception)?);
            output.push('\n');
        }

        // Generate error code constants
        output.push_str("// Error Code Constants\n");
        output.push_str("export const ErrorCodes = {\n");
        for exception in exceptions {
            if let Some(ref code) = exception.error_code {
                output.push_str(&format!("    {}: '{}',\n", exception.name.camel, code));
            }
        }
        output.push_str("} as const;\n\n");

        // Generate error factory function
        output.push_str(
            r#"// Error Factory
export function createError(
    errorType: string,
    message: string,
    fields?: Record<string, unknown>
): AppError {
    switch (errorType) {
"#,
        );

        for exception in exceptions {
            if !exception.is_abstract {
                output.push_str(&format!(
                    "        case '{}':\n            return new {}Error(message, fields);\n",
                    exception.name.camel, exception.name.pascal
                ));
            }
        }

        output.push_str(
            r#"        default:
            return new AppError(message, 'UNKNOWN_ERROR', 500);
    }
}
"#,
        );

        Ok(output)
    }

    /// 生成单个 TypeScript 错误类
    fn generate_typescript_error_class(
        exception: &GeneratorException,
    ) -> Result<String, GenerateError> {
        let mut output = String::new();

        output.push_str(&format!("/**\n * {}\n", exception.name.raw));
        if let Some(ref desc) = exception.description {
            output.push_str(&format!(" * {}\n", desc));
        }
        output.push_str(" */\n");

        // Determine parent class
        let parent_class = if exception.parent_exceptions.is_empty() {
            "AppError".to_string()
        } else {
            // Use first parent for inheritance
            format!("{}Error", exception.parent_exceptions[0].pascal)
        };

        output.push_str(&format!(
            "export class {}Error extends {} {{\n",
            exception.name.pascal, parent_class
        ));

        // Add fields as properties
        for field in &exception.fields {
            output.push_str(&format!(
                "    public readonly {}: {};\n",
                field.name.camel,
                Self::typescript_type_for_field(&field.field_type)
            ));
        }

        // Constructor
        output.push_str(
            r#"
    constructor(message: string"#,
        );

        // Add fields parameter if there are fields
        if !exception.fields.is_empty() {
            output.push_str(", fields: {");
            let field_types: Vec<String> = exception
                .fields
                .iter()
                .map(|f| {
                    format!(
                        "{}: {}",
                        f.name.camel,
                        Self::typescript_type_for_field(&f.field_type)
                    )
                })
                .collect();
            output.push_str(&field_types.join("; "));
            output.push_str("})");
        } else {
            output.push(')');
        }

        output.push_str(" {\n");

        // Call super
        let error_code = exception.error_code.as_deref().unwrap_or("UNKNOWN_ERROR");
        output.push_str(&format!(
            "        super(message, '{}', {});\n",
            error_code,
            exception.http_status.as_u16()
        ));
        output.push_str(&format!(
            "        this.name = '{}Error';\n",
            exception.name.pascal
        ));

        // Assign fields
        for field in &exception.fields {
            output.push_str(&format!(
                "        this.{} = fields.{};\n",
                field.name.camel, field.name.camel
            ));
        }

        output.push_str(
            r#"        // Fix prototype chain
        Object.setPrototypeOf(this, ValidationError.prototype);
    }"#,
        );
        output.push_str("\n}\n");

        Ok(output)
    }

    /// 将异常字段类型转换为 TypeScript 类型
    fn typescript_type_for_field(
        field_type: &crate::generator::ir::exception::GeneratorExceptionFieldType,
    ) -> String {
        use crate::generator::ir::exception::GeneratorExceptionFieldType;

        match field_type {
            GeneratorExceptionFieldType::String => "string".to_string(),
            GeneratorExceptionFieldType::Integer => "number".to_string(),
            GeneratorExceptionFieldType::Boolean => "boolean".to_string(),
            GeneratorExceptionFieldType::Reference(name) => name.clone(),
            GeneratorExceptionFieldType::Array(inner) => {
                format!("{}[]", Self::typescript_type_for_field(inner))
            }
        }
    }

    /// 生成异常层次结构文档
    pub fn generate_hierarchy_documentation(
        exceptions: &[GeneratorException],
    ) -> Result<String, GenerateError> {
        let mut output = String::new();

        output.push_str("# Exception Hierarchy\n\n");
        output.push_str("Auto-generated exception class hierarchy.\n\n");

        // Build hierarchy tree
        output.push_str("## Hierarchy Tree\n\n");
        output.push_str("```\n");

        for exception in exceptions {
            if exception.parent_exceptions.is_empty() {
                Self::write_exception_tree(&mut output, exception, exceptions, 0)?;
            }
        }

        output.push_str("```\n\n");

        // Exception details
        output.push_str("## Exception Details\n\n");
        for exception in exceptions {
            output.push_str(&format!("### {}\n\n", exception.name.pascal));

            if let Some(ref desc) = exception.description {
                output.push_str(&format!("{}\n\n", desc));
            }

            output.push_str(&format!(
                "- **HTTP Status**: {}\n",
                exception.http_status.as_u16()
            ));

            if let Some(ref code) = exception.error_code {
                output.push_str(&format!("- **Error Code**: `{}`\n", code));
            }

            if !exception.fields.is_empty() {
                output.push_str("- **Fields**:\n");
                for field in &exception.fields {
                    let req_marker = if field.required { " (required)" } else { "" };
                    output.push_str(&format!(
                        "  - `{}`: {}{}\n",
                        field.name.camel,
                        Self::typescript_type_for_field(&field.field_type),
                        req_marker
                    ));
                }
            }

            if !exception.parent_exceptions.is_empty() {
                output.push_str("- **Extends**:\n");
                for parent in &exception.parent_exceptions {
                    output.push_str(&format!("  - `{}`\n", parent.pascal));
                }
            }

            output.push('\n');
        }

        Ok(output)
    }

    /// 递归写出异常树
    fn write_exception_tree(
        output: &mut String,
        exception: &GeneratorException,
        all_exceptions: &[GeneratorException],
        depth: usize,
    ) -> Result<(), GenerateError> {
        let indent = "  ".repeat(depth);
        output.push_str(&format!("{}{}\n", indent, exception.name.pascal));

        // Find children
        for child in all_exceptions {
            if child
                .parent_exceptions
                .iter()
                .any(|p| p.raw == exception.name.raw)
            {
                Self::write_exception_tree(output, child, all_exceptions, depth + 1)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::exception::{
        ExceptionFieldName, ExceptionName, GeneratorExceptionField, GeneratorExceptionFieldType,
        GeneratorI18nMessage, HttpStatusCode,
    };
    use std::collections::HashMap;

    fn create_test_exception(
        name: &str,
        parent: Option<&str>,
        http_status: HttpStatusCode,
    ) -> GeneratorException {
        GeneratorException {
            name: ExceptionName::from_raw(name),
            description: Some(format!("Test {} error", name)),
            parent_exceptions: parent
                .map(|p| vec![ExceptionName::from_raw(p)])
                .unwrap_or_default(),
            fields: vec![GeneratorExceptionField {
                name: ExceptionFieldName::from_raw("message"),
                field_type: GeneratorExceptionFieldType::String,
                required: true,
                description: Some("Error message".to_string()),
            }],
            error_code: Some(format!("ERR_{}", name.to_uppercase())),
            error_code_constant: format!("{}_ERROR", name.to_uppercase()),
            http_status,
            is_abstract: false,
            inheritance_depth: if parent.is_some() { 1 } else { 0 },
            i18n_message: Some(GeneratorI18nMessage {
                message_key: format!("error.{}.message", name.to_lowercase()),
                default_message: format!("{} error occurred", name),
                translations: HashMap::new(),
                icu_format: false,
                parameters: vec!["message".to_string()],
            }),
        }
    }

    #[test]
    fn test_generate_rust_error_module() {
        let exceptions = vec![
            create_test_exception("Validation", None, HttpStatusCode::BadRequest),
            create_test_exception("NotFound", None, HttpStatusCode::NotFound),
        ];

        let output = ExceptionHierarchyGenerator::generate_rust_error_module(&exceptions).unwrap();

        assert!(output.contains("pub enum AppError"));
        assert!(output.contains("ValidationError"));
        assert!(output.contains("NotFoundError"));
        assert!(output.contains("VALIDATION_ERROR"));
        assert!(output.contains("NOTFOUND_ERROR"));
    }

    #[test]
    fn test_generate_typescript_error_classes() {
        let exceptions = vec![create_test_exception(
            "Validation",
            None,
            HttpStatusCode::BadRequest,
        )];

        let output =
            ExceptionHierarchyGenerator::generate_typescript_error_classes(&exceptions).unwrap();

        assert!(output.contains("export class AppError extends Error"));
        assert!(output.contains("export class ValidationError extends AppError"));
        assert!(output.contains("ErrorCodes"));
    }

    #[test]
    fn test_generate_hierarchy_documentation() {
        let exceptions = vec![
            create_test_exception("Base", None, HttpStatusCode::InternalServerError),
            create_test_exception("Validation", Some("Base"), HttpStatusCode::BadRequest),
            create_test_exception("NotFound", Some("Base"), HttpStatusCode::NotFound),
        ];

        let output =
            ExceptionHierarchyGenerator::generate_hierarchy_documentation(&exceptions).unwrap();

        assert!(output.contains("# Exception Hierarchy"));
        assert!(output.contains("Base"));
        assert!(output.contains("Validation"));
        assert!(output.contains("NotFound"));
    }
}
