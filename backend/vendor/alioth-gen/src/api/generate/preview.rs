//! 代码生成预览 API
//!
//! 提供代码生成预览功能，返回生成的输出与磁盘文件的差异对比，不写入磁盘。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::generator::output::protected::Conflict;
use crate::generator::output::{
    CapturedFile, ConflictReport, DiffEngine, FileChange, MergeEngine, MergeOptions, MergeResult,
};
use crate::generator::{GenerateError, GeneratedOutput};
use similar::{ChangeTag, TextDiff};

/// 预览变更类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PreviewChangeType {
    /// 新创建的文件
    Created,
    /// 更新的文件（内容变化）
    Updated,
    /// 删除的文件
    Deleted,
    /// 未变化的文件
    Unchanged,
}

impl From<&FileChange> for PreviewChangeType {
    fn from(change: &FileChange) -> Self {
        match change {
            FileChange::Created { .. } => PreviewChangeType::Created,
            FileChange::Updated { .. } => PreviewChangeType::Updated,
            FileChange::Deleted { .. } => PreviewChangeType::Deleted,
            FileChange::Unchanged { .. } => PreviewChangeType::Unchanged,
        }
    }
}

/// 预览文件条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewFileEntry {
    /// 文件路径
    pub path: String,
    /// 文件内容
    pub content: String,
    /// 内容校验和
    pub checksum: String,
    /// 变更类型
    pub change_type: PreviewChangeType,
    /// 统一差异（仅当 change_type 为 Updated 时有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<String>,
}

/// 预览响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewResponse {
    /// 文件预览列表
    pub files: Vec<PreviewFileEntry>,
    /// 冲突报告（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflict_report: Option<SerializableConflictReport>,
    /// 统计信息
    pub stats: PreviewStats,
}

/// 预览统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewStats {
    /// 总文件数
    pub total: usize,
    /// 创建数量
    pub created: usize,
    /// 更新数量
    pub updated: usize,
    /// 删除数量
    pub deleted: usize,
    /// 未变化数量
    pub unchanged: usize,
}

/// 可序列化的冲突报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableConflictReport {
    /// 文件路径
    pub file_path: String,
    /// 发现的冲突列表
    pub conflicts: Vec<SerializableConflict>,
    /// 合并建议
    pub suggestion: String,
}

/// 可序列化的单个冲突
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableConflict {
    /// 冲突类型
    pub conflict_type: String,
    /// 冲突发生的行号
    pub line_number: Option<usize>,
    /// 冲突描述
    pub description: String,
    /// 用户代码中的符号（如果有）
    pub user_symbol: Option<String>,
    /// 生成代码中的符号（如果有）
    pub generated_symbol: Option<String>,
}

impl From<&ConflictReport> for SerializableConflictReport {
    fn from(report: &ConflictReport) -> Self {
        Self {
            file_path: report.file_path.to_string_lossy().to_string(),
            conflicts: report.conflicts.iter().map(|c| c.into()).collect(),
            suggestion: report.suggestion.clone(),
        }
    }
}

impl From<&Conflict> for SerializableConflict {
    fn from(conflict: &Conflict) -> Self {
        Self {
            conflict_type: format!("{:?}", conflict.conflict_type),
            line_number: conflict.line_number,
            description: conflict.description.clone(),
            user_symbol: conflict.user_symbol.clone(),
            generated_symbol: conflict.generated_symbol.clone(),
        }
    }
}

/// 预览请求
#[derive(Debug, Deserialize)]
pub struct PreviewRequest {
    /// 输出目录（用于对比现有文件）
    pub output_dir: String,
    /// 是否包含未变化的文件
    #[serde(default = "default_include_unchanged")]
    pub include_unchanged: bool,
    /// 是否检查受保护区域冲突
    #[serde(default = "default_check_conflicts")]
    pub check_conflicts: bool,
}

fn default_include_unchanged() -> bool {
    true
}

fn default_check_conflicts() -> bool {
    true
}

/// 生成统一差异文本
fn generate_unified_diff(old_content: &str, new_content: &str, path: &str) -> String {
    let diff = TextDiff::from_lines(old_content, new_content);

    let mut output = String::new();
    output.push_str(&format!("--- a/{path}\n"));
    output.push_str(&format!("+++ b/{path}\n"));

    for group in diff.grouped_ops(3) {
        // 计算 hunk 的范围
        let mut old_start = None::<usize>;
        let mut old_lines = 0usize;
        let mut new_start = None::<usize>;
        let mut new_lines = 0usize;

        // 先遍历一遍计算范围
        for op in &group {
            for change in diff.iter_changes(op) {
                match change.tag() {
                    ChangeTag::Delete | ChangeTag::Equal => {
                        if old_start.is_none() {
                            old_start = Some(change.old_index().unwrap_or(0));
                        }
                        old_lines += 1;
                    }
                    _ => {}
                }
                match change.tag() {
                    ChangeTag::Insert | ChangeTag::Equal => {
                        if new_start.is_none() {
                            new_start = Some(change.new_index().unwrap_or(0));
                        }
                        new_lines += 1;
                    }
                    _ => {}
                }
            }
        }

        let old_start = old_start.unwrap_or(0);
        let new_start = new_start.unwrap_or(0);

        // 输出 hunk 头
        output.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            old_start + 1,
            old_lines,
            new_start + 1,
            new_lines
        ));

        // 输出变更行
        for op in group {
            for change in diff.iter_changes(&op) {
                let line = change.value();
                let line_without_newline = line.strip_suffix('\n').unwrap_or(line);

                match change.tag() {
                    ChangeTag::Delete => {
                        output.push_str(&format!("-{line_without_newline}\n"));
                    }
                    ChangeTag::Insert => {
                        output.push_str(&format!("+{line_without_newline}\n"));
                    }
                    ChangeTag::Equal => {
                        output.push_str(&format!(" {line_without_newline}\n"));
                    }
                }
            }
        }
    }

    output
}

/// 应用变更请求
#[derive(Debug, Deserialize)]
pub struct ApplyChangesRequest {
    /// 要应用的文件列表
    pub files: Vec<ApplyFileEntry>,
    /// 输出目录
    pub output_dir: String,
}

/// 单个文件条目
#[derive(Debug, Deserialize)]
pub struct ApplyFileEntry {
    /// 文件路径
    pub path: String,
    /// 文件内容
    pub content: String,
}

/// 应用变更响应
#[derive(Debug, Serialize)]
pub struct ApplyChangesResponse {
    /// 是否成功
    pub success: bool,
    /// 已应用的文件数量
    pub applied_count: usize,
    /// 失败的文件列表
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub failed_files: Vec<FailedFileEntry>,
}

/// 失败的文件条目
#[derive(Debug, Serialize)]
pub struct FailedFileEntry {
    /// 文件路径
    pub path: String,
    /// 错误信息
    pub error: String,
}

/// 回滚请求
#[derive(Debug, Deserialize)]
pub struct RollbackRequest {
    /// 要回滚的文件路径
    pub path: String,
    /// 输出目录
    pub output_dir: String,
}

/// 回滚响应
#[derive(Debug, Serialize)]
pub struct RollbackResponse {
    /// 是否成功
    pub success: bool,
    /// 文件路径
    pub path: String,
    /// 回滚后的内容（如果成功）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 预览服务
pub struct PreviewService;

impl PreviewService {
    /// 创建新的预览服务
    pub fn new() -> Self {
        Self
    }

    /// 生成预览
    ///
    /// 流程：
    /// 1. 运行生成到内存缓冲区
    /// 2. 使用 DiffEngine 对比磁盘文件
    /// 3. 使用 MergeEngine 检查受保护区域冲突
    /// 4. 为 Updated 文件生成统一差异
    /// 5. 返回 PreviewResponse（不写入磁盘）
    pub fn preview(
        &self,
        output: GeneratedOutput,
        request: PreviewRequest,
    ) -> Result<PreviewResponse, GenerateError> {
        let start_time = std::time::Instant::now();

        // 1. 获取内存中的生成文件
        let captured_files: Vec<CapturedFile> = output
            .files
            .into_iter()
            .map(|f| CapturedFile {
                path: f.path,
                content: f.content,
                checksum: f.checksum,
            })
            .collect();

        // 2. 使用 DiffEngine 对比磁盘
        let diff_engine = DiffEngine::new(&request.output_dir);
        let diff_result = diff_engine
            .compare(&captured_files)
            .map_err(|e| GenerateError::Io(std::io::Error::other(e)))?;

        // 3. 检查受保护区域冲突（仅针对 Updated 文件）
        let mut conflict_reports: Vec<SerializableConflictReport> = vec![];
        if request.check_conflicts {
            let merge_engine = MergeEngine::with_options(MergeOptions::default());
            let base_path = std::path::Path::new(&request.output_dir);

            for change in diff_result.changes.values() {
                if let FileChange::Updated { path, .. } = change {
                    if let Some(mem_file) = captured_files.iter().find(|f| &f.path == path) {
                        let full_path = base_path.join(path);
                        if let Ok(existing_content) = std::fs::read_to_string(&full_path) {
                            let merge_result =
                                merge_engine.merge(&existing_content, &mem_file.content, path);

                            if let MergeResult::Conflict(report) = merge_result {
                                conflict_reports.push((&report).into());
                            }
                        }
                    }
                }
            }
        }

        // 4. 构建预览文件条目
        let mut files: Vec<PreviewFileEntry> = vec![];
        let mut path_to_content: HashMap<std::path::PathBuf, &CapturedFile> = HashMap::new();
        for file in &captured_files {
            path_to_content.insert(file.path.clone(), file);
        }

        for change in diff_result.changes.values() {
            let change_type = PreviewChangeType::from(change);

            // 如果不需要包含未变化的文件，跳过
            if change_type == PreviewChangeType::Unchanged && !request.include_unchanged {
                continue;
            }

            let path = change.path();
            let path_str = path.to_string_lossy().to_string();

            // 获取内容和校验和
            let (content, checksum) = match change {
                FileChange::Created { checksum, .. } => {
                    if let Some(mem_file) = path_to_content.get(path) {
                        (mem_file.content.clone(), checksum.clone())
                    } else {
                        (String::new(), checksum.clone())
                    }
                }
                FileChange::Updated { new_checksum, .. } => {
                    if let Some(mem_file) = path_to_content.get(path) {
                        (mem_file.content.clone(), new_checksum.clone())
                    } else {
                        (String::new(), new_checksum.clone())
                    }
                }
                FileChange::Deleted { checksum, .. } => {
                    // 删除的文件，内容为空，尝试读取磁盘内容
                    let disk_path = std::path::Path::new(&request.output_dir).join(path);
                    let content = std::fs::read_to_string(&disk_path).unwrap_or_default();
                    (content, checksum.clone())
                }
                FileChange::Unchanged { checksum, .. } => {
                    if let Some(mem_file) = path_to_content.get(path) {
                        (mem_file.content.clone(), checksum.clone())
                    } else {
                        (String::new(), checksum.clone())
                    }
                }
            };

            // 生成统一差异（仅 Updated 类型）
            let diff = if change_type == PreviewChangeType::Updated {
                let disk_path = std::path::Path::new(&request.output_dir).join(path);
                if let Ok(old_content) = std::fs::read_to_string(&disk_path) {
                    Some(generate_unified_diff(&old_content, &content, &path_str))
                } else {
                    None
                }
            } else {
                None
            };

            files.push(PreviewFileEntry {
                path: path_str,
                content,
                checksum,
                change_type,
                diff,
            });
        }

        // 按路径排序，确保结果稳定
        files.sort_by(|a, b| a.path.cmp(&b.path));

        let stats = PreviewStats {
            total: files.len(),
            created: diff_result.created_count(),
            updated: diff_result.updated_count(),
            deleted: diff_result.deleted_count(),
            unchanged: diff_result.unchanged_count(),
        };

        let elapsed = start_time.elapsed();
        common::telemetry::info!("Preview generated in {:?}: {} files ({} created, {} updated, {} deleted, {} unchanged)",
        elapsed,
        stats.total,
        stats.created,
        stats.updated,
        stats.deleted,
        stats.unchanged);

        Ok(PreviewResponse {
            files,
            conflict_report: conflict_reports.first().cloned(),
            stats,
        })
    }

    /// 应用变更到文件系统
    pub fn apply_changes(
        &self,
        request: ApplyChangesRequest,
    ) -> Result<ApplyChangesResponse, GenerateError> {
        let base_path = std::path::Path::new(&request.output_dir);
        let mut applied_count = 0;
        let mut failed_files = Vec::new();

        for file in request.files {
            let full_path = base_path.join(&file.path);

            // 确保父目录存在
            if let Some(parent) = full_path.parent() {
                if !parent.exists() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        failed_files.push(FailedFileEntry {
                            path: file.path.clone(),
                            error: format!("Failed to create directory: {}", e),
                        });
                        continue;
                    }
                }
            }

            // 写入文件
            match std::fs::write(&full_path, &file.content) {
                Ok(_) => {
                    common::telemetry::info!("Applied file: {}", file.path);
                    applied_count += 1;
                }
                Err(e) => {
                    common::telemetry::error!("Failed to apply file {}: {}", file.path, e);
                    failed_files.push(FailedFileEntry {
                        path: file.path,
                        error: format!("Failed to write file: {}", e),
                    });
                }
            }
        }

        Ok(ApplyChangesResponse {
            success: failed_files.is_empty(),
            applied_count,
            failed_files,
        })
    }

    /// 回滚文件到备份
    pub fn rollback(&self, request: RollbackRequest) -> Result<RollbackResponse, GenerateError> {
        let backup_path = std::path::Path::new(&request.output_dir)
            .join(".alioth_backup")
            .join(&request.path);

        let target_path = std::path::Path::new(&request.output_dir).join(&request.path);

        // 检查备份文件是否存在
        if !backup_path.exists() {
            return Ok(RollbackResponse {
                success: false,
                path: request.path,
                content: None,
                error: Some("No backup found for this file".to_string()),
            });
        }

        // 读取备份内容
        let backup_content = match std::fs::read_to_string(&backup_path) {
            Ok(c) => c,
            Err(e) => {
                return Ok(RollbackResponse {
                    success: false,
                    path: request.path,
                    content: None,
                    error: Some(format!("Failed to read backup: {}", e)),
                });
            }
        };

        // 恢复到目标文件
        match std::fs::write(&target_path, &backup_content) {
            Ok(_) => {
                common::telemetry::info!("Rolled back file: {}", request.path);
                Ok(RollbackResponse {
                    success: true,
                    path: request.path,
                    content: Some(backup_content),
                    error: None,
                })
            }
            Err(e) => {
                common::telemetry::error!("Failed to rollback file {}: {}", request.path, e);
                Ok(RollbackResponse {
                    success: false,
                    path: request.path,
                    content: None,
                    error: Some(format!("Failed to write file: {}", e)),
                })
            }
        }
    }
}

impl Default for PreviewService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::fs;
    use tempfile::TempDir;

    fn create_test_output(files: Vec<(&str, &str)>) -> GeneratedOutput {
        let generated_files: Vec<_> = files
            .into_iter()
            .map(|(path, content)| {
                let mut hasher = Sha256::new();
                hasher.update(content.as_bytes());
                let checksum = hex::encode(hasher.finalize())[..32].to_string();

                crate::generator::GeneratedFile {
                    path: std::path::PathBuf::from(path),
                    content: content.to_string(),
                    checksum,
                }
            })
            .collect();

        let c_file_count = generated_files.len();

        GeneratedOutput {
            files: generated_files,
            metadata: crate::generator::GenerationMetadata {
                generator_name: "test".to_string(),
                entity_count: c_file_count,
                c_file_count,
            },
        }
    }

    #[test]
    fn test_preview_all_created_for_new_model() {
        let temp_dir = TempDir::new().unwrap();
        let service = PreviewService::new();

        let output =
            create_test_output(vec![("file1.txt", "Content 1"), ("file2.txt", "Content 2")]);

        let request = PreviewRequest {
            output_dir: temp_dir.path().to_string_lossy().to_string(),
            include_unchanged: true,
            check_conflicts: true,
        };

        let response = service.preview(output, request).unwrap();

        assert_eq!(response.stats.total, 2);
        assert_eq!(response.stats.created, 2);
        assert_eq!(response.stats.updated, 0);
        assert_eq!(response.stats.deleted, 0);
        assert_eq!(response.stats.unchanged, 0);

        // 所有文件应该是 Created 类型
        for file in &response.files {
            assert_eq!(file.change_type, PreviewChangeType::Created);
            assert!(file.diff.is_none());
        }
    }

    #[test]
    fn test_preview_updated_with_diff() {
        let temp_dir = TempDir::new().unwrap();

        // 创建现有文件
        fs::write(temp_dir.path().join("existing.txt"), "Old content\nLine 2").unwrap();

        let service = PreviewService::new();

        let output = create_test_output(vec![("existing.txt", "New content\nLine 2\nLine 3")]);

        let request = PreviewRequest {
            output_dir: temp_dir.path().to_string_lossy().to_string(),
            include_unchanged: true,
            check_conflicts: false,
        };

        let response = service.preview(output, request).unwrap();

        assert_eq!(response.stats.total, 1);
        assert_eq!(response.stats.created, 0);
        assert_eq!(response.stats.updated, 1);

        let file = &response.files[0];
        assert_eq!(file.change_type, PreviewChangeType::Updated);
        assert!(file.diff.is_some());

        let diff = file.diff.as_ref().unwrap();
        assert!(diff.contains("--- a/existing.txt"));
        assert!(diff.contains("+++ b/existing.txt"));
        assert!(diff.contains("-Old content"));
        assert!(diff.contains("+New content"));
    }

    #[test]
    fn test_preview_unchanged_for_identical() {
        let temp_dir = TempDir::new().unwrap();

        // 创建相同内容的现有文件
        fs::write(temp_dir.path().join("same.txt"), "Same content").unwrap();

        let service = PreviewService::new();

        let output = create_test_output(vec![("same.txt", "Same content")]);

        let request = PreviewRequest {
            output_dir: temp_dir.path().to_string_lossy().to_string(),
            include_unchanged: true,
            check_conflicts: false,
        };

        let response = service.preview(output, request).unwrap();

        assert_eq!(response.stats.total, 1);
        assert_eq!(response.stats.unchanged, 1);

        let file = &response.files[0];
        assert_eq!(file.change_type, PreviewChangeType::Unchanged);
        assert!(file.diff.is_none());
    }

    #[test]
    fn test_preview_deleted_for_removed_files() {
        let temp_dir = TempDir::new().unwrap();

        // 创建将被删除的现有文件
        fs::write(temp_dir.path().join("orphan.txt"), "Orphan content").unwrap();

        let service = PreviewService::new();

        // 空输出 - 所有现有文件应被标记为删除
        let output = create_test_output(vec![]);

        let request = PreviewRequest {
            output_dir: temp_dir.path().to_string_lossy().to_string(),
            include_unchanged: true,
            check_conflicts: false,
        };

        let response = service.preview(output, request).unwrap();

        assert_eq!(response.stats.total, 1);
        assert_eq!(response.stats.deleted, 1);

        let file = &response.files[0];
        assert_eq!(file.change_type, PreviewChangeType::Deleted);
        assert!(file.diff.is_none());
        assert_eq!(file.content, "Orphan content");
    }

    #[test]
    fn test_preview_exclude_unchanged() {
        let temp_dir = TempDir::new().unwrap();

        // 创建一个相同的文件和一个不同的文件
        fs::write(temp_dir.path().join("same.txt"), "Same content").unwrap();
        fs::write(temp_dir.path().join("different.txt"), "Old content").unwrap();

        let service = PreviewService::new();

        let output = create_test_output(vec![
            ("same.txt", "Same content"),
            ("different.txt", "New content"),
        ]);

        let request = PreviewRequest {
            output_dir: temp_dir.path().to_string_lossy().to_string(),
            include_unchanged: false, // 排除未变化
            check_conflicts: false,
        };

        let response = service.preview(output, request).unwrap();

        // 应该只包含 different.txt
        assert_eq!(response.stats.total, 1);
        assert_eq!(response.files[0].path, "different.txt");
    }

    #[test]
    fn test_generate_unified_diff() {
        let old_content = "line 1\nline 2\nline 3\n";
        let new_content = "line 1\nmodified line 2\nline 3\nline 4\n";

        let diff = generate_unified_diff(old_content, new_content, "test.txt");

        assert!(diff.contains("--- a/test.txt"));
        assert!(diff.contains("+++ b/test.txt"));
        assert!(diff.contains("-line 2"));
        assert!(diff.contains("+modified line 2"));
        assert!(diff.contains("+line 4"));
    }

    #[test]
    fn test_preview_response_serialization() {
        let response = PreviewResponse {
            files: vec![PreviewFileEntry {
                path: "test.txt".to_string(),
                content: "content".to_string(),
                checksum: "abc123".to_string(),
                change_type: PreviewChangeType::Created,
                diff: None,
            }],
            conflict_report: None,
            stats: PreviewStats {
                total: 1,
                created: 1,
                updated: 0,
                deleted: 0,
                unchanged: 0,
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"path\":\"test.txt\""));
        assert!(json.contains("\"change_type\":\"CREATED\""));
        assert!(json.contains("\"total\":1"));
    }
}
