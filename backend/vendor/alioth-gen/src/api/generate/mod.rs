//! 代码生成 API
//!
//! 提供代码生成的 REST API 端点，将 CLI 代码生成器功能暴露为 HTTP 服务。
//!
//! # 端点
//!
//! - `POST /api/meta-model/generate/zod` - 生成 Zod Schema
//! - `POST /api/meta-model/generate/react` - 生成 React UI 组件
//!
//! # 认证
//!
//! 所有端点需要有效的认证会话。

pub mod error;
pub mod preview;
pub mod react;
pub mod zod;

// 重新导出常用类型
pub use error::GenerateApiError;

use serde::{Deserialize, Serialize};

/// 代码生成请求
#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    /// IR 模型
    pub model: crate::generator::ir::GeneratorModel,
    /// 生成选项
    #[serde(default)]
    pub options: GenerateOptions,
}

/// 代码生成选项
#[derive(Debug, Deserialize, Default)]
pub struct GenerateOptions {
    /// 输出格式
    pub output_format: Option<String>,
    /// 包含额外文件
    pub include_extra: Option<bool>,
}

/// 代码生成响应
#[derive(Debug, Serialize)]
pub struct GenerateResponse {
    /// 生成的文件列表
    pub files: Vec<GeneratedFileInfo>,
    /// 生成元数据
    pub metadata: GenerationMetadataInfo,
}

/// 生成的文件信息
#[derive(Debug, Serialize)]
pub struct GeneratedFileInfo {
    /// 文件路径
    pub path: String,
    /// 文件内容（Base64 编码）
    pub content: String,
    /// 内容校验和
    pub checksum: String,
}

/// 生成元数据信息
#[derive(Debug, Serialize)]
pub struct GenerationMetadataInfo {
    /// 生成器名称
    pub generator_name: String,
    /// 实体数量
    pub entity_count: usize,
    /// 文件数量
    pub c_file_count: usize,
}

impl From<crate::generator::GeneratedOutput> for GenerateResponse {
    fn from(output: crate::generator::GeneratedOutput) -> Self {
        let files = output
            .files
            .into_iter()
            .map(|f| GeneratedFileInfo {
                path: f.path.to_string_lossy().to_string(),
                content: f.content,
                checksum: f.checksum,
            })
            .collect();

        Self {
            files,
            metadata: GenerationMetadataInfo {
                generator_name: output.metadata.generator_name,
                entity_count: output.metadata.entity_count,
                c_file_count: output.metadata.c_file_count,
            },
        }
    }
}
