//! AppCreator shared library
//!
//! 独立服务，与 Meta 仅共享 DB。
//! API 无交互，认证通过 SSO JWT (ES256)。

pub mod app_repository;
pub mod auth_config;
pub mod chat;
pub mod handlers;
pub mod meta_reader;
pub mod middleware;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Alioth 模型版本默认值
pub const DEFAULT_ALIOTH_MODEL_VERSION: &str = "10.0.0";

/// 构建配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub namespace: String,
    pub project_root: String,
    pub version: String,
    pub alioth_model_version: String,
    pub port: u16,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            namespace: "Alioth".to_string(),
            project_root: ".".to_string(),
            version: "0.1.0".to_string(),
            alioth_model_version: DEFAULT_ALIOTH_MODEL_VERSION.to_string(),
            port: 8080,
        }
    }
}

/// 错误
#[derive(Error, Debug)]
pub enum AppCreatorError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// 构建结果
#[derive(Debug, Serialize)]
pub struct BuildOutput {
    pub lock_content: String,
    pub compose_content: String,
    pub artifacts: Vec<String>,
}
