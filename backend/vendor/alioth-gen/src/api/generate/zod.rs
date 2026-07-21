//! Zod Schema 生成 API
//!
//! 提供 Zod 验证 Schema 和 TypeScript 类型生成的 HTTP API 端点。

use crate::api::generate::{GenerateApiError, GeneratedFileInfo, GenerationMetadataInfo};
use crate::generator::ir::GeneratorModel;
use crate::generator::zod::FullZodGenerator;
use crate::generator::Generator;
use serde::{Deserialize, Serialize};

/// Zod 生成请求
#[derive(Debug, Deserialize)]
pub struct ZodGenerateRequest {
    /// IR 模型
    pub model: GeneratorModel,
    /// Zod 生成选项
    #[serde(default)]
    pub options: ZodGenerateOptions,
}

/// Zod 生成选项
#[derive(Debug, Deserialize)]
pub struct ZodGenerateOptions {
    /// 是否包含 React Hook Form 集成
    #[serde(default = "default_include_hooks")]
    pub include_hooks: bool,
    /// 输出格式
    #[serde(default)]
    pub output_format: ZodOutputFormat,
    /// 是否包含 TypeScript 类型定义
    #[serde(default = "default_include_types")]
    pub include_types: bool,
}

impl Default for ZodGenerateOptions {
    fn default() -> Self {
        Self {
            include_hooks: default_include_hooks(),
            output_format: ZodOutputFormat::TypeScript,
            include_types: default_include_types(),
        }
    }
}

fn default_include_hooks() -> bool {
    true
}

fn default_include_types() -> bool {
    true
}

/// Zod 输出格式
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum ZodOutputFormat {
    /// TypeScript 格式
    #[default]
    TypeScript,
    /// JavaScript 格式
    JavaScript,
}

/// Zod 生成响应
#[derive(Debug, Serialize)]
pub struct ZodGenerateResponse {
    /// 生成的文件列表
    pub files: Vec<GeneratedFileInfo>,
    /// 生成元数据
    pub metadata: GenerationMetadataInfo,
    /// Zod 特定元数据
    pub zod_metadata: ZodMetadata,
}

/// Zod 特定元数据
#[derive(Debug, Serialize)]
pub struct ZodMetadata {
    /// 是否包含 hooks
    pub has_hooks: bool,
    /// 输出格式
    pub format: ZodOutputFormat,
    /// Schema 数量
    pub schema_count: usize,
}

/// 生成 Zod Schema
pub fn generate_zod(request: ZodGenerateRequest) -> Result<ZodGenerateResponse, GenerateApiError> {
    // 创建生成器
    let generator = FullZodGenerator;

    // 验证模型
    generator
        .validate(&request.model)
        .map_err(GenerateApiError::from)?;

    // 生成代码
    let output = generator.generate(&request.model).map_err(|e| {
        GenerateApiError::new(
            crate::api::generate::error::GenerateApiErrorCode::GenerationFailed,
            format!("Zod schema generation failed: {}", e),
        )
        .with_suggestions(vec![
            "Verify that all enum values are non-empty strings.".to_string(),
            "Ensure field types are supported by the Zod generator.".to_string(),
        ])
    })?;

    // 转换为响应
    let files: Vec<GeneratedFileInfo> = output
        .files
        .into_iter()
        .map(|f| GeneratedFileInfo {
            path: f.path.to_string_lossy().to_string(),
            content: f.content,
            checksum: f.checksum,
        })
        .collect();

    let schema_count = request.model.entities.len();

    Ok(ZodGenerateResponse {
        metadata: GenerationMetadataInfo {
            generator_name: output.metadata.generator_name,
            entity_count: output.metadata.entity_count,
            c_file_count: output.metadata.c_file_count,
        },
        zod_metadata: ZodMetadata {
            has_hooks: request.options.include_hooks,
            format: request.options.output_format,
            schema_count,
        },
        files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_model() -> GeneratorModel {
        GeneratorModel {
            i18n_config: None,
            entities: vec![],
            enums: vec![],
            metadata: crate::generator::ir::ModelMetadata {
                generated_at: "2024-01-01T00:00:00Z".to_string(),
                generator_version: "0.1.0".to_string(),
            },
            exceptions: vec![],
            exception_handlers: vec![],
            external_dependencies: vec![],
        }
    }

    #[test]
    fn test_generate_zod_empty_model() {
        let request = ZodGenerateRequest {
            model: create_test_model(),
            options: ZodGenerateOptions::default(),
        };

        let result = generate_zod(request);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.zod_metadata.has_hooks);
        assert_eq!(response.zod_metadata.schema_count, 0);
    }

    #[test]
    fn test_generate_zod_without_hooks() {
        let request = ZodGenerateRequest {
            model: create_test_model(),
            options: ZodGenerateOptions {
                include_hooks: false,
                output_format: ZodOutputFormat::TypeScript,
                include_types: true,
            },
        };

        let result = generate_zod(request);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.zod_metadata.has_hooks);
    }
}
