//! 输出写入器 trait

use crate::generator::GeneratedOutput;
use std::path::PathBuf;

/// 写入生成输出的 trait
pub trait OutputWriter: Send + Sync {
    /// 写入所有文件
    fn write(&self, output: &GeneratedOutput) -> Result<(), WriteError>;

    /// 预览将要写入的内容（试运行）
    fn dry_run(&self, output: &GeneratedOutput) -> Result<DryRunResult, WriteError>;
}

/// 试运行结果显示将要执行的操作
#[derive(Debug, Clone)]
pub struct DryRunResult {
    pub files_to_write: Vec<PathBuf>,
    pub files_to_create: Vec<PathBuf>,
    pub files_to_update: Vec<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("遍历目录错误: {0}")]
    WalkDir(String),

    #[error("权限拒绝: {0}")]
    PermissionDenied(PathBuf),

    #[error("无效路径: {0}")]
    InvalidPath(PathBuf),
}

impl From<walkdir::Error> for WriteError {
    fn from(e: walkdir::Error) -> Self {
        WriteError::WalkDir(e.to_string())
    }
}

/// 向生成的内容添加确定性排序标记
pub fn add_ordering_markers(content: &mut String, entity_names: &[Vec<String>]) {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let checksum = hex::encode(hasher.finalize())[..16].to_string();

    // 收集所有实体名称
    let all_names: Vec<String> = entity_names
        .iter()
        .flat_map(|v| v.iter().cloned())
        .collect();

    let marker = format!(
        "// @alioth-generated\n// ORDER: {}\n// CHECKSUM: {}\n// TIMESTAMP: {}\n// DO NOT EDIT MANUALLY\n\n",
        all_names.join(","),
        checksum,
        chrono::Utc::now().to_rfc3339()
    );
    content.insert_str(0, &marker);
}
