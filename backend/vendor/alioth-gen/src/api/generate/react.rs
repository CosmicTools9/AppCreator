//! React UI 生成 API
//!
//! 提供 React 组件生成的 HTTP API 端点。

use crate::api::generate::{GenerateApiError, GeneratedFileInfo, GenerationMetadataInfo};
use crate::generator::frontend::FrontendComponentGenerator;
use crate::generator::ir::GeneratorModel;
use crate::generator::Generator;
use serde::{Deserialize, Serialize};

/// React 生成请求
#[derive(Debug, Deserialize)]
pub struct ReactGenerateRequest {
    /// IR 模型
    pub model: GeneratorModel,
    /// 要生成的组件类型
    #[serde(default = "default_components")]
    pub components: Vec<ReactComponentType>,
    /// React 生成选项
    #[serde(default)]
    pub options: ReactGenerateOptions,
}

/// React 组件类型
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReactComponentType {
    /// 表单组件
    Form,
    /// 表格组件
    Table,
    /// 详情组件
    Detail,
    /// 图表组件
    Charts,
    /// 所有组件
    All,
}

fn default_components() -> Vec<ReactComponentType> {
    vec![ReactComponentType::All]
}

/// React 生成选项
#[derive(Debug, Deserialize)]
pub struct ReactGenerateOptions {
    /// 样式方案
    #[serde(default = "default_styling")]
    pub styling: String,
    /// 是否使用 Hooks
    #[serde(default = "default_use_hooks")]
    pub use_hooks: bool,
    /// 是否使用 TypeScript
    #[serde(default = "default_typescript")]
    pub typescript: bool,
}

impl Default for ReactGenerateOptions {
    fn default() -> Self {
        Self {
            styling: default_styling(),
            use_hooks: default_use_hooks(),
            typescript: default_typescript(),
        }
    }
}

fn default_styling() -> String {
    "tailwind".to_string()
}

fn default_use_hooks() -> bool {
    true
}

fn default_typescript() -> bool {
    true
}

/// React 生成响应
#[derive(Debug, Serialize)]
pub struct ReactGenerateResponse {
    /// 生成的文件列表
    pub files: Vec<GeneratedFileInfo>,
    /// 生成元数据
    pub metadata: GenerationMetadataInfo,
    /// React 特定元数据
    pub react_metadata: ReactMetadata,
}

/// React 特定元数据
#[derive(Debug, Serialize)]
pub struct ReactMetadata {
    /// 样式方案
    pub styling: String,
    /// 是否使用 TypeScript
    pub typescript: bool,
    /// 组件数量
    pub component_count: usize,
    /// 生成的组件类型
    pub component_types: Vec<String>,
}

/// 生成 React 组件
pub fn generate_react(
    request: ReactGenerateRequest,
) -> Result<ReactGenerateResponse, GenerateApiError> {
    // 创建生成器（选项暂不使用，因为 FrontendComponentGenerator API 有限制）
    let _styling = &request.options.styling;
    let _use_hooks = request.options.use_hooks;
    let _typescript = request.options.typescript;

    // 创建生成器
    let generator = FrontendComponentGenerator::new();

    // 验证模型
    generator
        .validate(&request.model)
        .map_err(GenerateApiError::from)?;

    // 生成代码
    let output = generator.generate(&request.model).map_err(|e| {
        GenerateApiError::new(
            crate::api::generate::error::GenerateApiErrorCode::GenerationFailed,
            format!("React component generation failed: {}", e),
        )
        .with_suggestions(vec![
            "Ensure entities have at least one displayable field.".to_string(),
            "Check that component styling option is valid.".to_string(),
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

    let component_count = files.len();
    let component_types: Vec<String> = request
        .components
        .iter()
        .map(|c| format!("{:?}", c))
        .collect();

    Ok(ReactGenerateResponse {
        metadata: GenerationMetadataInfo {
            generator_name: output.metadata.generator_name,
            entity_count: output.metadata.entity_count,
            c_file_count: output.metadata.c_file_count,
        },
        react_metadata: ReactMetadata {
            styling: request.options.styling,
            typescript: request.options.typescript,
            component_count,
            component_types,
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
    fn test_generate_react_empty_model() {
        let request = ReactGenerateRequest {
            model: create_test_model(),
            components: vec![ReactComponentType::All],
            options: ReactGenerateOptions::default(),
        };

        let result = generate_react(request);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.react_metadata.styling, "tailwind");
        assert!(response.react_metadata.typescript);
    }

    #[test]
    fn test_generate_react_specific_components() {
        let request = ReactGenerateRequest {
            model: create_test_model(),
            components: vec![ReactComponentType::Form, ReactComponentType::Table],
            options: ReactGenerateOptions {
                styling: "css-modules".to_string(),
                use_hooks: true,
                typescript: false,
            },
        };

        let result = generate_react(request);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.react_metadata.styling, "css-modules");
        assert!(!response.react_metadata.typescript);
    }
}
