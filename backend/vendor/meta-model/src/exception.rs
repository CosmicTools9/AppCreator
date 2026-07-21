//! Exception Handling System (Phase 26)
//!
//! 实现基于本体语义的异常定义和处理模型
//! 支持异常层次结构、处理器和国际化

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HTTP 状态码映射
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum HttpStatusCode {
    /// 400 Bad Request
    BadRequest = 400,
    /// 401 Unauthorized
    Unauthorized = 401,
    /// 403 Forbidden
    Forbidden = 403,
    /// 404 Not Found
    NotFound = 404,
    /// 409 Conflict
    Conflict = 409,
    /// 422 Unprocessable Entity
    UnprocessableEntity = 422,
    /// 429 Too Many Requests
    TooManyRequests = 429,
    /// 500 Internal Server Error
    #[default]
    InternalServerError = 500,
    /// 503 Service Unavailable
    ServiceUnavailable = 503,
}

impl HttpStatusCode {
    /// 将整数转换为 HTTP 状态码
    pub fn from_u16(code: u16) -> Option<Self> {
        match code {
            400 => Some(Self::BadRequest),
            401 => Some(Self::Unauthorized),
            403 => Some(Self::Forbidden),
            404 => Some(Self::NotFound),
            409 => Some(Self::Conflict),
            422 => Some(Self::UnprocessableEntity),
            429 => Some(Self::TooManyRequests),
            500 => Some(Self::InternalServerError),
            503 => Some(Self::ServiceUnavailable),
            _ => None,
        }
    }

    /// 获取状态码的整数表示
    pub fn as_u16(&self) -> u16 {
        *self as u16
    }

    /// 获取状态码的标准描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::BadRequest => "Bad Request",
            Self::Unauthorized => "Unauthorized",
            Self::Forbidden => "Forbidden",
            Self::NotFound => "Not Found",
            Self::Conflict => "Conflict",
            Self::UnprocessableEntity => "Unprocessable Entity",
            Self::TooManyRequests => "Too Many Requests",
            Self::InternalServerError => "Internal Server Error",
            Self::ServiceUnavailable => "Service Unavailable",
        }
    }
}

/// IR-1: 异常定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaException {
    /// 异常名称
    pub name: String,
    /// 异常描述
    #[serde(default)]
    pub description: Option<String>,
    /// 父异常列表（支持多重继承）
    #[serde(default)]
    pub parent_exceptions: Vec<String>,
    /// 异常字段
    #[serde(default)]
    pub fields: Vec<MetaExceptionField>,
    /// 错误码
    #[serde(default)]
    pub error_code: Option<String>,
    /// HTTP 状态码
    #[serde(default)]
    pub http_status: Option<HttpStatusCode>,
    /// 是否抽象异常（不能直接抛出）
    #[serde(default)]
    pub is_abstract: bool,
    /// 国际化消息模板
    #[serde(default)]
    pub i18n_message: Option<I18nMessageTemplate>,
    /// 关联的注解
    #[serde(default)]
    pub annotations: Vec<crate::ir1::MetaAnnotation>,
}

/// IR-1: 异常字段定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaExceptionField {
    /// 字段名
    pub name: String,
    /// 字段类型
    pub field_type: MetaExceptionFieldType,
    /// 是否必需
    #[serde(default)]
    pub required: bool,
    /// 描述
    #[serde(default)]
    pub description: Option<String>,
}

/// IR-1: 异常字段类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetaExceptionFieldType {
    String,
    Integer,
    Boolean,
    /// 引用其他实体类型
    Reference(String),
    /// 数组类型
    Array(Box<MetaExceptionFieldType>),
}

/// IR-1: 国际化消息模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nMessageTemplate {
    /// 默认语言消息
    pub default_message: String,
    /// 各语言消息映射
    #[serde(default)]
    pub translations: HashMap<String, String>,
    /// ICU MessageFormat 格式
    #[serde(default)]
    pub icu_format: bool,
}

/// IR-1: @throws 注解数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaThrowsClause {
    /// 异常类型名称
    pub exception_type: String,
    /// 触发条件表达式
    #[serde(default)]
    pub condition: Option<String>,
    /// 字段名（如果是字段级抛出）
    #[serde(default)]
    pub field_name: Option<String>,
    /// 错误消息
    #[serde(default)]
    pub message: Option<String>,
}

/// IR-1: 异常处理器定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaExceptionHandler {
    /// 处理器名称
    pub name: String,
    /// 处理的异常类型
    pub exception_type: String,
    /// 处理器函数体（表达式或函数名）
    pub handler_body: String,
    /// HTTP 状态码（覆盖异常默认值）
    #[serde(default)]
    pub http_status: Option<HttpStatusCode>,
    /// 处理器优先级（数字越小优先级越高）
    #[serde(default)]
    pub priority: i32,
    /// 是否异步处理
    #[serde(default)]
    pub is_async: bool,
}

/// IR-2: 异常定义（生成器格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorException {
    /// 异常名称（多格式）
    pub name: ExceptionName,
    /// 异常描述
    pub description: Option<String>,
    /// 父异常列表
    pub parent_exceptions: Vec<ExceptionName>,
    /// 异常字段
    pub fields: Vec<GeneratorExceptionField>,
    /// 错误码
    pub error_code: Option<String>,
    /// 错误码常量名（SCREAMING_SNAKE_CASE）
    pub error_code_constant: String,
    /// HTTP 状态码
    pub http_status: HttpStatusCode,
    /// 是否抽象异常
    pub is_abstract: bool,
    /// 继承深度
    pub inheritance_depth: u32,
    /// 国际化消息
    pub i18n_message: Option<GeneratorI18nMessage>,
}

/// IR-2: 异常名称（多格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionName {
    pub raw: String,
    pub snake: String,
    pub camel: String,
    pub pascal: String,
    pub screaming_snake: String,
}

impl ExceptionName {
    pub fn from_raw(name: &str) -> Self {
        use convert_case::{Case, Casing};
        Self {
            raw: name.to_string(),
            snake: name.to_case(Case::Snake),
            camel: name.to_case(Case::Camel),
            pascal: name.to_case(Case::Pascal),
            screaming_snake: name.to_case(Case::UpperSnake),
        }
    }
}

impl std::fmt::Display for ExceptionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}

/// IR-2: 异常字段（生成器格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorExceptionField {
    /// 字段名
    pub name: ExceptionFieldName,
    /// 字段类型
    pub field_type: GeneratorExceptionFieldType,
    /// 是否必需
    pub required: bool,
    /// 描述
    pub description: Option<String>,
}

/// IR-2: 异常字段名（多格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionFieldName {
    pub raw: String,
    pub snake: String,
    pub camel: String,
}

impl ExceptionFieldName {
    pub fn from_raw(name: &str) -> Self {
        use convert_case::{Case, Casing};
        Self {
            raw: name.to_string(),
            snake: name.to_case(Case::Snake),
            camel: name.to_case(Case::Camel),
        }
    }
}

/// IR-2: 异常字段类型（生成器格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeneratorExceptionFieldType {
    String,
    Integer,
    Boolean,
    Reference(String),
    Array(Box<GeneratorExceptionFieldType>),
}

impl From<&MetaExceptionFieldType> for GeneratorExceptionFieldType {
    fn from(ft: &MetaExceptionFieldType) -> Self {
        match ft {
            MetaExceptionFieldType::String => Self::String,
            MetaExceptionFieldType::Integer => Self::Integer,
            MetaExceptionFieldType::Boolean => Self::Boolean,
            MetaExceptionFieldType::Reference(name) => Self::Reference(name.clone()),
            MetaExceptionFieldType::Array(inner) => {
                Self::Array(Box::new(GeneratorExceptionFieldType::from(inner.as_ref())))
            }
        }
    }
}

/// IR-2: 国际化消息（生成器格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorI18nMessage {
    /// 消息模板键
    pub message_key: String,
    /// 默认消息
    pub default_message: String,
    /// 各语言翻译
    pub translations: HashMap<String, String>,
    /// 是否为 ICU 格式
    pub icu_format: bool,
    /// 模板参数
    pub parameters: Vec<String>,
}

/// IR-2: @throws 子句（生成器格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorThrowsClause {
    /// 异常类型名称
    pub exception_type: ExceptionName,
    /// 触发条件表达式
    pub condition: Option<String>,
    /// 条件函数名（snake_case）
    pub condition_fn_name: Option<String>,
    /// 字段名
    pub field_name: Option<String>,
    /// 错误消息
    pub message: Option<String>,
}

/// IR-2: 异常处理器（生成器格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorExceptionHandler {
    /// 处理器名称
    pub name: ExceptionHandlerName,
    /// 处理的异常类型
    pub exception_type: ExceptionName,
    /// 处理器函数名
    pub handler_fn_name: String,
    /// HTTP 状态码
    pub http_status: HttpStatusCode,
    /// 优先级
    pub priority: i32,
    /// 是否异步
    pub is_async: bool,
}

/// IR-2: 异常处理器名称
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionHandlerName {
    pub raw: String,
    pub snake: String,
    pub camel: String,
}

impl ExceptionHandlerName {
    pub fn from_raw(name: &str) -> Self {
        use convert_case::{Case, Casing};
        Self {
            raw: name.to_string(),
            snake: name.to_case(Case::Snake),
            camel: name.to_case(Case::Camel),
        }
    }
}

/// 异常层次结构分析器
pub struct ExceptionHierarchyAnalyzer;

impl ExceptionHierarchyAnalyzer {
    /// 计算异常的继承深度
    pub fn calculate_inheritance_depth(
        exception_name: &str,
        all_exceptions: &[MetaException],
    ) -> u32 {
        let exception = match all_exceptions.iter().find(|e| e.name == exception_name) {
            Some(e) => e,
            None => return 0,
        };

        if exception.parent_exceptions.is_empty() {
            return 0;
        }

        let mut max_parent_depth = 0;
        for parent in &exception.parent_exceptions {
            let parent_depth = Self::calculate_inheritance_depth(parent, all_exceptions);
            max_parent_depth = max_parent_depth.max(parent_depth);
        }

        max_parent_depth + 1
    }

    /// 获取异常的所有祖先（按从近到远排序）
    pub fn get_ancestors(exception_name: &str, all_exceptions: &[MetaException]) -> Vec<String> {
        let mut ancestors = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = vec![exception_name.to_string()];

        while let Some(current) = queue.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            if let Some(exception) = all_exceptions.iter().find(|e| e.name == current) {
                for parent in &exception.parent_exceptions {
                    if !visited.contains(parent) {
                        ancestors.push(parent.clone());
                        queue.push(parent.clone());
                    }
                }
            }
        }

        ancestors
    }

    /// 检测循环继承
    pub fn detect_circular_inheritance(all_exceptions: &[MetaException]) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut recursion_stack = std::collections::HashSet::new();

        for exception in all_exceptions {
            if !visited.contains(&exception.name) {
                let mut path = Vec::new();
                Self::dfs_check_cycle(
                    &exception.name,
                    all_exceptions,
                    &mut visited,
                    &mut recursion_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    fn dfs_check_cycle(
        name: &str,
        all_exceptions: &[MetaException],
        visited: &mut std::collections::HashSet<String>,
        recursion_stack: &mut std::collections::HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(name.to_string());
        recursion_stack.insert(name.to_string());
        path.push(name.to_string());

        if let Some(exception) = all_exceptions.iter().find(|e| e.name == name) {
            for parent in &exception.parent_exceptions {
                if !visited.contains(parent) {
                    Self::dfs_check_cycle(
                        parent,
                        all_exceptions,
                        visited,
                        recursion_stack,
                        path,
                        cycles,
                    );
                } else if recursion_stack.contains(parent) {
                    // 发现循环
                    if let Some(pos) = path.iter().position(|p| p == parent) {
                        let cycle = path[pos..].to_vec();
                        cycles.push(cycle);
                    }
                }
            }
        }

        path.pop();
        recursion_stack.remove(name);
    }
}

/// 错误码生成器
pub struct ErrorCodeGenerator;

impl ErrorCodeGenerator {
    /// 生成标准错误码
    pub fn generate_error_code(
        exception_name: &str,
        module_prefix: &str,
        http_status: HttpStatusCode,
    ) -> String {
        let status_part = format!("{:03}", http_status.as_u16());
        let name_part = exception_name.to_uppercase();
        format!("{}_{}_{}", module_prefix, status_part, name_part)
    }

    /// 生成错误码常量名
    pub fn generate_constant_name(exception_name: &str) -> String {
        use convert_case::{Case, Casing};
        exception_name.to_case(Case::UpperSnake)
    }
}

/// 异常处理器注册表
#[derive(Debug, Default)]
pub struct ExceptionHandlerRegistry {
    handlers: HashMap<String, Vec<MetaExceptionHandler>>,
}

impl ExceptionHandlerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册处理器
    pub fn register(&mut self, handler: MetaExceptionHandler) {
        let entry = self
            .handlers
            .entry(handler.exception_type.clone())
            .or_default();
        entry.push(handler);
        // 按优先级排序
        entry.sort_by_key(|h| h.priority);
    }

    /// 获取异常类型的处理器
    pub fn get_handlers(&self, exception_type: &str) -> Vec<&MetaExceptionHandler> {
        self.handlers
            .get(exception_type)
            .map(|h| h.iter().collect())
            .unwrap_or_default()
    }

    /// 获取所有异常类型
    pub fn exception_types(&self) -> Vec<&String> {
        self.handlers.keys().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_status_code() {
        assert_eq!(HttpStatusCode::BadRequest.as_u16(), 400);
        assert_eq!(
            HttpStatusCode::from_u16(404),
            Some(HttpStatusCode::NotFound)
        );
        assert_eq!(HttpStatusCode::from_u16(999), None);
    }

    #[test]
    fn test_exception_name_formats() {
        let name = ExceptionName::from_raw("ValidationError");
        assert_eq!(name.raw, "ValidationError");
        assert_eq!(name.snake, "validation_error");
        assert_eq!(name.camel, "validationError");
        assert_eq!(name.pascal, "ValidationError");
        assert_eq!(name.screaming_snake, "VALIDATION_ERROR");
    }

    #[test]
    fn test_error_code_generator() {
        let code = ErrorCodeGenerator::generate_error_code(
            "NotFoundError",
            "APP",
            HttpStatusCode::NotFound,
        );
        assert_eq!(code, "APP_404_NOTFOUNDERROR");
    }

    #[test]
    fn test_exception_hierarchy_depth() {
        let exceptions = vec![
            MetaException {
                name: "BaseError".to_string(),
                description: None,
                parent_exceptions: vec![],
                fields: vec![],
                error_code: None,
                http_status: None,
                is_abstract: false,
                i18n_message: None,
                annotations: vec![],
            },
            MetaException {
                name: "ValidationError".to_string(),
                description: None,
                parent_exceptions: vec!["BaseError".to_string()],
                fields: vec![],
                error_code: None,
                http_status: None,
                is_abstract: false,
                i18n_message: None,
                annotations: vec![],
            },
            MetaException {
                name: "FieldError".to_string(),
                description: None,
                parent_exceptions: vec!["ValidationError".to_string()],
                fields: vec![],
                error_code: None,
                http_status: None,
                is_abstract: false,
                i18n_message: None,
                annotations: vec![],
            },
        ];

        assert_eq!(
            ExceptionHierarchyAnalyzer::calculate_inheritance_depth("BaseError", &exceptions),
            0
        );
        assert_eq!(
            ExceptionHierarchyAnalyzer::calculate_inheritance_depth("ValidationError", &exceptions),
            1
        );
        assert_eq!(
            ExceptionHierarchyAnalyzer::calculate_inheritance_depth("FieldError", &exceptions),
            2
        );
    }

    #[test]
    fn test_detect_circular_inheritance() {
        let exceptions = vec![
            MetaException {
                name: "A".to_string(),
                description: None,
                parent_exceptions: vec!["B".to_string()],
                fields: vec![],
                error_code: None,
                http_status: None,
                is_abstract: false,
                i18n_message: None,
                annotations: vec![],
            },
            MetaException {
                name: "B".to_string(),
                description: None,
                parent_exceptions: vec!["C".to_string()],
                fields: vec![],
                error_code: None,
                http_status: None,
                is_abstract: false,
                i18n_message: None,
                annotations: vec![],
            },
            MetaException {
                name: "C".to_string(),
                description: None,
                parent_exceptions: vec!["A".to_string()],
                fields: vec![],
                error_code: None,
                http_status: None,
                is_abstract: false,
                i18n_message: None,
                annotations: vec![],
            },
        ];

        let cycles = ExceptionHierarchyAnalyzer::detect_circular_inheritance(&exceptions);
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_handler_registry() {
        let mut registry = ExceptionHandlerRegistry::new();

        registry.register(MetaExceptionHandler {
            name: "handleValidationError".to_string(),
            exception_type: "ValidationError".to_string(),
            handler_body: "return Response.badRequest()".to_string(),
            http_status: Some(HttpStatusCode::BadRequest),
            priority: 0,
            is_async: false,
        });

        registry.register(MetaExceptionHandler {
            name: "handleNotFoundError".to_string(),
            exception_type: "NotFoundError".to_string(),
            handler_body: "return Response.notFound()".to_string(),
            http_status: Some(HttpStatusCode::NotFound),
            priority: 1,
            is_async: false,
        });

        let validation_handlers = registry.get_handlers("ValidationError");
        assert_eq!(validation_handlers.len(), 1);
        assert_eq!(validation_handlers[0].name, "handleValidationError");
    }
}
