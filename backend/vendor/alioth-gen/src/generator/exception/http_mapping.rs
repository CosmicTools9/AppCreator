//! Exception HTTP Mapping Generator
//!
//! 将异常映射到 HTTP 状态码和 API 错误响应

use crate::generator::ir::exception::{GeneratorException, GeneratorExceptionHandler};
use crate::generator::{GenerateError, GeneratedFile, GeneratedOutput, GenerationMetadata};
use std::collections::HashMap;

/// HTTP 映射生成器
pub struct ExceptionHttpMappingGenerator;

impl ExceptionHttpMappingGenerator {
    /// 生成 Rust HTTP 错误响应处理器
    pub fn generate_rust_http_handlers(
        exceptions: &[GeneratorException],
        handlers: &[GeneratorExceptionHandler],
    ) -> Result<GeneratedOutput, GenerateError> {
        let content = Self::generate_rust_http_handlers_content(exceptions, handlers)?;

        let files = vec![GeneratedFile {
            path: "src/errors/http_handlers.rs".into(),
            content: content.clone(),
            checksum: format!("{:x}", md5::compute(content.as_bytes())),
        }];

        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "exception_http_mapping".to_string(),
                entity_count: exceptions.len(),
                c_file_count: 1,
            },
        })
    }

    /// 生成 Rust HTTP 处理器内容
    fn generate_rust_http_handlers_content(
        _exceptions: &[GeneratorException],
        handlers: &[GeneratorExceptionHandler],
    ) -> Result<String, GenerateError> {
        let mut output = String::new();

        output.push_str("//! HTTP Error Response Handlers\n");
        output.push_str("//!\n");
        output.push_str("//! Auto-generated exception to HTTP response mapping\n\n");

        output.push_str("use actix_web::{\n");
        output.push_str("    HttpResponse, ResponseError,\n");
        output.push_str("    http::StatusCode,\n");
        output.push_str("};\n");
        output.push_str("use serde_json::json;\n");
        output.push_str("use super::error_enum::Error;\n\n");

        // Generate ResponseError implementation
        output.push_str("impl ResponseError for Error {\n");
        output.push_str("    fn error_response(&self) -> HttpResponse {\n");
        output.push_str("        let status = self.status_code();\n");
        output.push_str("        let error_body = json!({\n");
        output.push_str("            \"error\": {\n");
        output.push_str("                \"code\": self.error_code(),\n");
        output.push_str("                \"message\": self.to_string(),\n");
        output.push_str("                \"status\": status.as_u16(),\n");
        output.push_str("            }\n");
        output.push_str("        });\n\n");
        output.push_str("        HttpResponse::build(status).json(error_body)\n");
        output.push_str("    }\n\n");
        output.push_str("    fn status_code(&self) -> StatusCode {\n");
        output.push_str("        StatusCode::from_u16(self.http_status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)\n");
        output.push_str("    }\n");
        output.push_str("}\n\n");

        // Generate error response builder
        output.push_str("/// Error response builder\n");
        output.push_str("pub struct ErrorResponseBuilder {\n");
        output.push_str("    error: Error,\n");
        output.push_str("    details: Option<serde_json::Value>,\n");
        output.push_str("}\n\n");
        output.push_str("impl ErrorResponseBuilder {\n");
        output.push_str("    pub fn new(error: Error) -> Self {\n");
        output.push_str("        Self {\n");
        output.push_str("            error,\n");
        output.push_str("            details: None,\n");
        output.push_str("        }\n");
        output.push_str("    }\n\n");
        output.push_str("    pub fn with_details(mut self, details: impl serde::Serialize) -> Result<Self, serde_json::Error> {\n");
        output.push_str("        self.details = Some(serde_json::to_value(details)?);\n");
        output.push_str("        Ok(self)\n");
        output.push_str("    }\n\n");
        output.push_str("    pub fn build(self) -> HttpResponse {\n");
        output.push_str("        let status = self.error.status_code();\n");
        output.push_str("        let mut body = json!({\n");
        output.push_str("            \"error\": {\n");
        output.push_str("                \"code\": self.error.error_code(),\n");
        output.push_str("                \"message\": self.error.to_string(),\n");
        output.push_str("                \"status\": status.as_u16(),\n");
        output.push_str("            }\n");
        output.push_str("        });\n\n");
        output.push_str("        if let Some(details) = self.details {\n");
        output.push_str("            body[\"error\"][\"details\"] = details;\n");
        output.push_str("        }\n\n");
        output.push_str("        HttpResponse::build(status).json(body)\n");
        output.push_str("    }\n");
        output.push_str("}\n\n");

        // Generate exception handler functions
        output.push_str("// Exception Handler Functions\n\n");

        for handler in handlers {
            output.push_str(&Self::generate_handler_function(handler)?);
            output.push('\n');
        }

        // Generate exception dispatch table
        output.push_str("// Exception Handler Dispatch\n\n");
        output.push_str("use std::sync::Arc;\n");
        output.push_str("use dashmap::DashMap;\n\n");

        output.push_str(
            "/// Exception handler registry
type HandlerFn = Arc<dyn Fn(&Error) -> HttpResponse + Send + Sync>;

pub struct ExceptionHandlerRegistry {
    handlers: DashMap<String, HandlerFn>,
}

impl ExceptionHandlerRegistry {
    pub fn new() -> Self {
        let registry = Self {
            handlers: DashMap::new(),
        };
        registry.register_default_handlers();
        registry
    }

    fn register_default_handlers(&self) {
",
        );

        for handler in handlers {
            output.push_str(&format!(
                r#"        self.handlers.insert(
            "{}".to_string(),
            Arc::new(|e| {}(e)),
        );
"#,
                handler.exception_type.snake, handler.handler_fn_name
            ));
        }

        output.push_str(
            r#"    }

    pub fn handle(&self, error: &Error) -> HttpResponse {
        let error_type = std::any::type_name_of_val(error);
        
        // Try specific handler first
        if let Some(handler) = self.handlers.get(error_type) {
            return handler(error);
        }
        
        // Fall back to generic error response
        error.error_response()
    }

    pub fn register(&self, error_type: String, handler: HandlerFn) {
        self.handlers.insert(error_type, handler);
    }
}

impl Default for ExceptionHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
"#,
        );

        Ok(output)
    }

    /// 生成处理器函数
    fn generate_handler_function(
        handler: &GeneratorExceptionHandler,
    ) -> Result<String, GenerateError> {
        let mut output = String::new();

        output.push_str(&format!(
            "/// Handle {} exception\n",
            handler.exception_type.pascal
        ));
        output.push_str(&format!(
            "/// Returns HTTP {}\n",
            handler.http_status.as_u16()
        ));
        output.push_str(&format!(
            "pub fn {}(error: &Error) -> HttpResponse {{\n",
            handler.handler_fn_name
        ));

        output.push_str("    let body = json!({\n");
        output.push_str("        \"error\": {\n");
        output.push_str("            \"code\": error.error_code(),\n");
        output.push_str("            \"message\": error.to_string(),\n");
        output.push_str(&format!(
            "            \"status\": {},\n",
            handler.http_status.as_u16()
        ));
        output.push_str("        }\n");
        output.push_str("    });\n\n");
        output.push_str(&format!(
            "    HttpResponse::build(StatusCode::from_u16({}).unwrap())\n",
            handler.http_status.as_u16()
        ));
        output.push_str("        .json(body)\n");
        output.push_str("}\n");

        Ok(output)
    }

    /// 生成 TypeScript HTTP 处理器
    pub fn generate_typescript_http_handlers(
        exceptions: &[GeneratorException],
        handlers: &[GeneratorExceptionHandler],
    ) -> Result<GeneratedOutput, GenerateError> {
        let content = Self::generate_typescript_http_handlers_content(exceptions, handlers)?;

        let files = vec![GeneratedFile {
            path: "src/errors/httpHandlers.ts".into(),
            content: content.clone(),
            checksum: format!("{:x}", md5::compute(content.as_bytes())),
        }];

        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "exception_http_mapping".to_string(),
                entity_count: exceptions.len(),
                c_file_count: 1,
            },
        })
    }

    /// 生成 TypeScript HTTP 处理器内容
    fn generate_typescript_http_handlers_content(
        _exceptions: &[GeneratorException],
        handlers: &[GeneratorExceptionHandler],
    ) -> Result<String, GenerateError> {
        let mut output = String::new();

        output.push_str("// HTTP Error Response Handlers\n");
        output.push_str("// Auto-generated exception to HTTP response mapping\n\n");

        output.push_str("import { AppError } from './errors';\n\n");

        // Generate error response interface
        output.push_str(
            r#"export interface ErrorResponse {
  error: {
    code: string;
    message: string;
    status: number;
    details?: Record<string, unknown>;
  };
}

export interface ApiErrorResponse {
  data: null;
  error: {
    code: string;
    message: string;
    status: number;
  };
}

"#,
        );

        // Generate error response builder
        output.push_str(
            r#"export class ErrorResponseBuilder {
  private error: AppError;
  private details?: Record<string, unknown>;

  constructor(error: AppError) {
    this.error = error;
  }

  withDetails(details: Record<string, unknown>): this {
    this.details = details;
    return this;
  }

  build(): ErrorResponse {
    const response: ErrorResponse = {
      error: {
        code: this.error.code,
        message: this.error.message,
        status: this.error.httpStatus,
      },
    };

    if (this.details) {
      response.error.details = this.details;
    }

    return response;
  }

  toJSON(): string {
    return JSON.stringify(this.build());
  }
}

"#,
        );

        // Generate handler functions
        output.push_str("// Exception Handler Functions\n\n");
        output.push_str("export type ErrorHandler = (error: AppError) => ErrorResponse;\n\n");

        for handler in handlers {
            output.push_str(&format!(
                r#"/**
 * Handle {} exception
 * Returns HTTP {}
 */
export function {}(error: AppError): ErrorResponse {{
  return new ErrorResponseBuilder(error).build();
}}

"#,
                handler.exception_type.pascal,
                handler.http_status.as_u16(),
                handler.handler_fn_name
            ));
        }

        // Generate exception handler registry
        output.push_str(
            r#"// Exception Handler Registry

export class ExceptionHandlerRegistry {
  private handlers: Map<string, ErrorHandler> = new Map();

  constructor() {
    this.registerDefaultHandlers();
  }

  private registerDefaultHandlers(): void {
"#,
        );

        for handler in handlers {
            output.push_str(&format!(
                "    this.handlers.set('{}', {});\n",
                handler.exception_type.camel, handler.handler_fn_name
            ));
        }

        output.push_str(
            r#"  }

  handle(error: AppError): ErrorResponse {
    // Extract error type from error name
    const errorType = error.name.replace('Error', '').toLowerCase();
    
    // Try specific handler first
    const handler = this.handlers.get(errorType);
    if (handler) {
      return handler(error);
    }
    
    // Fall back to generic handler
    return new ErrorResponseBuilder(error).build();
  }

  register(errorType: string, handler: ErrorHandler): void {
    this.handlers.set(errorType, handler);
  }

  unregister(errorType: string): void {
    this.handlers.delete(errorType);
  }
}

// Singleton instance
export const exceptionRegistry = new ExceptionHandlerRegistry();

"#,
        );

        // Generate Axios/Fetch error interceptor
        output.push_str(r#"// HTTP Client Error Interceptor

/**
 * Intercept and transform API errors
 */
export function interceptApiError(error: unknown): AppError | null {
  if (error && typeof error === 'object') {
    const apiError = error as { response?: { data?: { error?: { code: string; message: string; status: number } } } };
    
    if (apiError.response?.data?.error) {
      const { code, message, status } = apiError.response.data.error;
      // Reconstruct the error using the error factory
      return {
        name: 'ApiError',
        message,
        code: code as any,
        httpStatus: status,
        timestamp: new Date(),
        toJSON() {
          return {
            name: this.name,
            message: this.message,
            code: this.code,
            httpStatus: this.httpStatus,
            timestamp: this.timestamp.toISOString(),
          };
        },
      };
    }
  }
  
  return null;
}
"#);

        Ok(output)
    }

    /// 生成 OpenAPI 错误响应规范
    pub fn generate_openapi_error_schemas(
        exceptions: &[GeneratorException],
    ) -> Result<GeneratedOutput, GenerateError> {
        let content = Self::generate_openapi_error_schemas_content(exceptions)?;

        let files = vec![GeneratedFile {
            path: "openapi/error-schemas.yaml".into(),
            content: content.clone(),
            checksum: format!("{:x}", md5::compute(content.as_bytes())),
        }];

        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "exception_http_mapping".to_string(),
                entity_count: exceptions.len(),
                c_file_count: 1,
            },
        })
    }

    /// 生成 OpenAPI 错误模式内容
    fn generate_openapi_error_schemas_content(
        exceptions: &[GeneratorException],
    ) -> Result<String, GenerateError> {
        let mut output = String::new();

        output.push_str("# OpenAPI Error Response Schemas\n");
        output.push_str("# Auto-generated from exception definitions\n\n");

        output.push_str("components:\n");
        output.push_str("  schemas:\n");

        // Base Error schema
        output.push_str(
            r#"    Error:
      type: object
      required:
        - error
      properties:
        error:
          type: object
          required:
            - code
            - message
            - status
          properties:
            code:
              type: string
              description: Machine-readable error code
            message:
              type: string
              description: Human-readable error message
            status:
              type: integer
              description: HTTP status code
            details:
              type: object
              description: Additional error details

"#,
        );

        // Generate specific error schemas
        for exception in exceptions {
            if exception.is_abstract {
                continue;
            }

            output.push_str(&format!("    {}Error:\n", exception.name.pascal));
            output.push_str("      allOf:\n");
            output.push_str("        - $ref: '#/components/schemas/Error'\n");
            output.push_str("        - type: object\n");
            output.push_str("          properties:\n");
            output.push_str("            error:\n");
            output.push_str("              type: object\n");
            output.push_str("              properties:\n");

            // Add field documentation
            for field in &exception.fields {
                output.push_str(&format!(
                    "                {}:\n                  type: {}\n",
                    field.name.camel,
                    Self::openapi_type(&field.field_type)
                ));
            }

            output.push('\n');
        }

        // Generate error responses
        output.push_str("  responses:\n");

        let mut status_codes: HashMap<u16, Vec<&GeneratorException>> = HashMap::new();
        for exception in exceptions {
            if exception.is_abstract {
                continue;
            }
            status_codes
                .entry(exception.http_status.as_u16())
                .or_default()
                .push(exception);
        }

        let mut sorted_statuses: Vec<_> = status_codes.iter().collect();
        sorted_statuses.sort_by_key(|(status, _)| *status);

        for (status, exs) in sorted_statuses {
            output.push_str(&format!(
                "    {}:\n      description: {}\n      content:\n        application/json:\n          schema:\n            oneOf:\n",
                status, exs[0].http_status.description()
            ));

            for ex in exs {
                output.push_str(&format!(
                    "              - $ref: '#/components/schemas/{}Error'\n",
                    ex.name.pascal
                ));
            }
            output.push('\n');
        }

        Ok(output)
    }

    /// 将字段类型转换为 OpenAPI 类型
    fn openapi_type(
        field_type: &crate::generator::ir::exception::GeneratorExceptionFieldType,
    ) -> &'static str {
        use crate::generator::ir::exception::GeneratorExceptionFieldType;

        match field_type {
            GeneratorExceptionFieldType::String => "string",
            GeneratorExceptionFieldType::Integer => "integer",
            GeneratorExceptionFieldType::Boolean => "boolean",
            GeneratorExceptionFieldType::Reference(_) => "object",
            GeneratorExceptionFieldType::Array(_) => "array",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::exception::{ExceptionHandlerName, ExceptionName, HttpStatusCode};

    fn create_test_exception(name: &str, status: HttpStatusCode) -> GeneratorException {
        GeneratorException {
            name: ExceptionName::from_raw(name),
            description: None,
            parent_exceptions: vec![],
            fields: vec![],
            error_code: Some(format!("ERR_{}", name.to_uppercase())),
            error_code_constant: format!("{}_ERROR", name.to_uppercase()),
            http_status: status,
            is_abstract: false,
            inheritance_depth: 0,
            i18n_message: None,
        }
    }

    fn create_test_handler(
        name: &str,
        exception_type: &str,
        status: HttpStatusCode,
    ) -> GeneratorExceptionHandler {
        GeneratorExceptionHandler {
            name: ExceptionHandlerName::from_raw(name),
            exception_type: ExceptionName::from_raw(exception_type),
            handler_fn_name: name.to_string(),
            http_status: status,
            priority: 0,
            is_async: false,
        }
    }

    #[test]
    fn test_generate_rust_http_handlers_content() {
        let exceptions = vec![create_test_exception(
            "Validation",
            HttpStatusCode::BadRequest,
        )];
        let handlers = vec![create_test_handler(
            "handle_validation_error",
            "Validation",
            HttpStatusCode::BadRequest,
        )];

        let output = ExceptionHttpMappingGenerator::generate_rust_http_handlers_content(
            &exceptions,
            &handlers,
        )
        .unwrap();

        assert!(output.contains("impl ResponseError for Error"));
        assert!(output.contains("ExceptionHandlerRegistry"));
    }

    #[test]
    fn test_generate_typescript_http_handlers_content() {
        let exceptions = vec![create_test_exception("NotFound", HttpStatusCode::NotFound)];
        let handlers = vec![create_test_handler(
            "handle_not_found",
            "NotFound",
            HttpStatusCode::NotFound,
        )];

        let output = ExceptionHttpMappingGenerator::generate_typescript_http_handlers_content(
            &exceptions,
            &handlers,
        )
        .unwrap();

        assert!(output.contains("ErrorResponseBuilder"));
        assert!(output.contains("ExceptionHandlerRegistry"));
    }

    #[test]
    fn test_generate_openapi_error_schemas() {
        let exceptions = vec![
            create_test_exception("Validation", HttpStatusCode::BadRequest),
            create_test_exception("NotFound", HttpStatusCode::NotFound),
        ];

        let output =
            ExceptionHttpMappingGenerator::generate_openapi_error_schemas_content(&exceptions)
                .unwrap();

        assert!(output.contains("components:"));
        assert!(output.contains("ValidationError"));
        assert!(output.contains("NotFoundError"));
    }
}
