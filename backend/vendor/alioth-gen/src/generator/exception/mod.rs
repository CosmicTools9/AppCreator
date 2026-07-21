//! Exception Generator Module (Phase 26)
//!
//! 提供异常层次结构生成、代码生成、国际化支持和 HTTP 映射功能

pub mod code_gen;
pub mod hierarchy;
pub mod http_mapping;
pub mod i18n;

use crate::generator::ir::exception::{GeneratorException, GeneratorExceptionHandler};
use crate::generator::ir::GeneratorModel;
use crate::generator::{GenerateError, GeneratedOutput, Generator};

/// 异常生成器配置
#[derive(Debug, Clone)]
pub struct ExceptionGeneratorConfig {
    /// 是否生成 Rust 代码
    pub generate_rust: bool,
    /// 是否生成 TypeScript 代码
    pub generate_typescript: bool,
    /// 是否生成国际化文件
    pub generate_i18n: bool,
    /// 是否生成 OpenAPI 规范
    pub generate_openapi: bool,
    /// 默认语言环境
    pub default_locale: String,
    /// 支持的语言环境
    pub supported_locales: Vec<String>,
}

impl Default for ExceptionGeneratorConfig {
    fn default() -> Self {
        Self {
            generate_rust: true,
            generate_typescript: true,
            generate_i18n: true,
            generate_openapi: true,
            default_locale: "en".to_string(),
            supported_locales: vec!["en".to_string(), "zh".to_string()],
        }
    }
}

/// 异常生成器
pub struct ExceptionGenerator {
    config: ExceptionGeneratorConfig,
}

impl Default for ExceptionGenerator {
    fn default() -> Self {
        Self::new(ExceptionGeneratorConfig::default())
    }
}

impl ExceptionGenerator {
    /// 创建新的异常生成器
    pub fn new(config: ExceptionGeneratorConfig) -> Self {
        Self { config }
    }

    /// 生成所有异常相关代码
    pub fn generate(
        &self,
        exceptions: &[GeneratorException],
        handlers: &[GeneratorExceptionHandler],
    ) -> Result<Vec<GeneratedOutput>, GenerateError> {
        let mut outputs = Vec::new();

        // Generate exception hierarchy
        if self.config.generate_rust {
            let hierarchy_output =
                hierarchy::ExceptionHierarchyGenerator::generate_rust_hierarchy(exceptions)?;
            outputs.push(hierarchy_output);
        }

        if self.config.generate_typescript {
            let hierarchy_output =
                hierarchy::ExceptionHierarchyGenerator::generate_typescript_hierarchy(exceptions)?;
            outputs.push(hierarchy_output);
        }

        // Generate error code enums
        if self.config.generate_rust {
            let code_output =
                code_gen::ExceptionCodeGenerator::generate_rust_error_enum(exceptions)?;
            outputs.push(code_output);
        }

        if self.config.generate_typescript {
            let code_output =
                code_gen::ExceptionCodeGenerator::generate_typescript_error_module(exceptions)?;
            outputs.push(code_output);
        }

        // Generate i18n resources
        if self.config.generate_i18n {
            let locales: Vec<&str> = self
                .config
                .supported_locales
                .iter()
                .map(|s| s.as_str())
                .collect();
            let i18n_output = i18n::ExceptionI18nGenerator::generate_i18n_resources(
                exceptions,
                &self.config.default_locale,
                &locales,
            )?;
            outputs.push(i18n_output);

            if self.config.generate_rust {
                let rust_i18n =
                    i18n::ExceptionI18nGenerator::generate_rust_i18n_module(exceptions)?;
                outputs.push(rust_i18n);
            }

            if self.config.generate_typescript {
                let ts_i18n =
                    i18n::ExceptionI18nGenerator::generate_typescript_i18n_module(exceptions)?;
                outputs.push(ts_i18n);
            }
        }

        // Generate HTTP handlers
        if self.config.generate_rust {
            let http_output =
                http_mapping::ExceptionHttpMappingGenerator::generate_rust_http_handlers(
                    exceptions, handlers,
                )?;
            outputs.push(http_output);
        }

        if self.config.generate_typescript {
            let http_output =
                http_mapping::ExceptionHttpMappingGenerator::generate_typescript_http_handlers(
                    exceptions, handlers,
                )?;
            outputs.push(http_output);
        }

        // Generate OpenAPI schemas
        if self.config.generate_openapi {
            let openapi_output =
                http_mapping::ExceptionHttpMappingGenerator::generate_openapi_error_schemas(
                    exceptions,
                )?;
            outputs.push(openapi_output);
        }

        Ok(outputs)
    }

    /// 从 GeneratorModel 生成异常代码
    pub fn generate_from_model(
        &self,
        model: &GeneratorModel,
    ) -> Result<Vec<GeneratedOutput>, GenerateError> {
        self.generate(&model.exceptions, &model.exception_handlers)
    }
}

impl Generator for ExceptionGenerator {
    fn name(&self) -> &'static str {
        "exception"
    }

    fn generate(&self, model: &GeneratorModel) -> Result<GeneratedOutput, GenerateError> {
        // Combine all outputs into a single output
        let outputs = self.generate_from_model(model)?;

        // Flatten outputs
        let mut all_files = Vec::new();
        let mut entity_count = 0;

        for output in outputs {
            entity_count += output.metadata.entity_count;
            all_files.extend(output.files);
        }

        let c_file_count = all_files.len();

        Ok(GeneratedOutput {
            files: all_files,
            metadata: GenerationMetadata {
                generator_name: self.name().to_string(),
                entity_count,
                c_file_count,
            },
        })
    }

    fn validate(&self, model: &GeneratorModel) -> Result<(), crate::generator::ValidationError> {
        // Validate exceptions don't have circular inheritance
        use crate::generator::ir::exception::ExceptionHierarchyAnalyzer;

        let cycles = ExceptionHierarchyAnalyzer::detect_circular_inheritance(
            &model
                .exceptions
                .iter()
                .map(|e| crate::generator::ir::exception::MetaException {
                    name: e.name.raw.clone(),
                    description: e.description.clone(),
                    parent_exceptions: e.parent_exceptions.iter().map(|p| p.raw.clone()).collect(),
                    fields: vec![],
                    error_code: e.error_code.clone(),
                    http_status: Some(e.http_status),
                    is_abstract: e.is_abstract,
                    i18n_message: None,
                    annotations: vec![],
                })
                .collect::<Vec<_>>(),
        );

        if !cycles.is_empty() {
            return Err(crate::generator::ValidationError::CircularDependency(
                format!("Circular inheritance detected in exceptions: {:?}", cycles),
            ));
        }

        Ok(())
    }

    fn supports_incremental(&self) -> bool {
        true
    }

    fn file_extensions(&self) -> Vec<&'static str> {
        vec!["rs", "ts", "json", "yaml"]
    }
}

use crate::generator::GenerationMetadata;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::exception::{ExceptionName, HttpStatusCode};

    #[allow(dead_code)]
    fn create_test_exception(name: &str) -> GeneratorException {
        GeneratorException {
            name: ExceptionName::from_raw(name),
            description: None,
            parent_exceptions: vec![],
            fields: vec![],
            error_code: Some(format!("ERR_{}", name.to_uppercase())),
            error_code_constant: format!("{}_ERROR", name.to_uppercase()),
            http_status: HttpStatusCode::BadRequest,
            is_abstract: false,
            inheritance_depth: 0,
            i18n_message: None,
        }
    }

    #[test]
    fn test_exception_generator_default() {
        let generator = ExceptionGenerator::default();
        assert!(generator.config.generate_rust);
        assert!(generator.config.generate_typescript);
    }

    #[test]
    fn test_exception_generator_name() {
        let generator = ExceptionGenerator::default();
        assert_eq!(generator.name(), "exception");
    }

    #[test]
    fn test_exception_generator_file_extensions() {
        let generator = ExceptionGenerator::default();
        let extensions = generator.file_extensions();
        assert!(extensions.contains(&"rs"));
        assert!(extensions.contains(&"ts"));
    }
}
