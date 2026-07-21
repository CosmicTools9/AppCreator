//! Diff 引擎 - 比较内存缓冲区输出与磁盘文件
//!
//! 提供文件变更检测、分类和报告功能

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::memory::CapturedFile;
use crate::generator::output::writer::WriteError;

/// 文件变更类型
#[derive(Debug, Clone, PartialEq)]
pub enum FileChange {
    /// 新创建的文件
    Created { path: PathBuf, checksum: String },
    /// 更新的文件（内容变化）
    Updated {
        path: PathBuf,
        old_checksum: String,
        new_checksum: String,
    },
    /// 删除的文件
    Deleted { path: PathBuf, checksum: String },
    /// 未变化的文件
    Unchanged { path: PathBuf, checksum: String },
}

impl FileChange {
    /// 获取变更文件的路径
    pub fn path(&self) -> &PathBuf {
        match self {
            FileChange::Created { path, .. } => path,
            FileChange::Updated { path, .. } => path,
            FileChange::Deleted { path, .. } => path,
            FileChange::Unchanged { path, .. } => path,
        }
    }

    /// 检查是否是创建操作
    pub fn is_created(&self) -> bool {
        matches!(self, FileChange::Created { .. })
    }

    /// 检查是否是更新操作
    pub fn is_updated(&self) -> bool {
        matches!(self, FileChange::Updated { .. })
    }

    /// 检查是否是删除操作
    pub fn is_deleted(&self) -> bool {
        matches!(self, FileChange::Deleted { .. })
    }

    /// 检查是否未变化
    pub fn is_unchanged(&self) -> bool {
        matches!(self, FileChange::Unchanged { .. })
    }

    /// 获取变更的校验和（如果是 Created/Unchanged/Deleted）
    pub fn checksum(&self) -> Option<&str> {
        match self {
            FileChange::Created { checksum, .. } => Some(checksum),
            FileChange::Updated { .. } => None,
            FileChange::Deleted { checksum, .. } => Some(checksum),
            FileChange::Unchanged { checksum, .. } => Some(checksum),
        }
    }

    /// 获取旧校验和（仅 Updated）
    pub fn old_checksum(&self) -> Option<&str> {
        match self {
            FileChange::Updated { old_checksum, .. } => Some(old_checksum),
            _ => None,
        }
    }

    /// 获取新校验和（仅 Updated）
    pub fn new_checksum(&self) -> Option<&str> {
        match self {
            FileChange::Updated { new_checksum, .. } => Some(new_checksum),
            _ => None,
        }
    }
}

/// Diff 结果
#[derive(Debug, Clone, Default)]
pub struct DiffResult {
    /// 路径到变更的映射
    pub changes: HashMap<PathBuf, FileChange>,
}

impl DiffResult {
    /// 创建新的空 DiffResult
    pub fn new() -> Self {
        Self {
            changes: HashMap::new(),
        }
    }

    /// 添加变更
    pub fn add_change(&mut self, change: FileChange) {
        self.changes.insert(change.path().clone(), change);
    }

    /// 获取所有变更
    pub fn get_changes(&self) -> &HashMap<PathBuf, FileChange> {
        &self.changes
    }

    /// 获取创建的文件
    pub fn created(&self) -> Vec<&FileChange> {
        self.changes.values().filter(|c| c.is_created()).collect()
    }

    /// 获取更新的文件
    pub fn updated(&self) -> Vec<&FileChange> {
        self.changes.values().filter(|c| c.is_updated()).collect()
    }

    /// 获取删除的文件
    pub fn deleted(&self) -> Vec<&FileChange> {
        self.changes.values().filter(|c| c.is_deleted()).collect()
    }

    /// 获取未变化的文件
    pub fn unchanged(&self) -> Vec<&FileChange> {
        self.changes.values().filter(|c| c.is_unchanged()).collect()
    }

    /// 统计创建数量
    pub fn created_count(&self) -> usize {
        self.created().len()
    }

    /// 统计更新数量
    pub fn updated_count(&self) -> usize {
        self.updated().len()
    }

    /// 统计删除数量
    pub fn deleted_count(&self) -> usize {
        self.deleted().len()
    }

    /// 统计未变化数量
    pub fn unchanged_count(&self) -> usize {
        self.unchanged().len()
    }

    /// 总变更数量
    pub fn total_count(&self) -> usize {
        self.changes.len()
    }

    /// 实际变更数量（不包括 Unchanged）
    pub fn actual_changes_count(&self) -> usize {
        self.created_count() + self.updated_count() + self.deleted_count()
    }

    /// 是否有实际变更
    pub fn has_changes(&self) -> bool {
        self.actual_changes_count() > 0
    }
}

/// 磁盘文件快照
#[derive(Debug, Clone)]
pub struct DiskSnapshot {
    /// 根目录
    pub root: PathBuf,
    /// 文件路径到校验和的映射
    pub files: HashMap<PathBuf, FileInfo>,
}

/// 磁盘文件信息
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub checksum: String,
    pub size: u64,
    pub is_binary: bool,
}

impl DiskSnapshot {
    /// 扫描目录创建快照
    pub fn scan<P: AsRef<Path>>(root: P) -> Result<Self, WriteError> {
        let root = root.as_ref().to_path_buf();
        let mut files = HashMap::new();

        if !root.exists() {
            return Ok(Self { root, files });
        }

        for entry in WalkDir::new(&root).follow_links(false) {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path().to_path_buf();
            let relative_path = path.strip_prefix(&root).unwrap_or(&path).to_path_buf();

            match Self::compute_file_info(&path) {
                Ok(file_info) => {
                    files.insert(relative_path, file_info);
                }
                Err(e) => {
                    common::telemetry::warn!("无法读取文件 {}: {}", path.display(), e);
                }
            }
        }

        Ok(Self { root, files })
    }

    /// 计算单个文件的 FileInfo
    fn compute_file_info(path: &Path) -> Result<FileInfo, WriteError> {
        let content = fs::read(path)?;
        let size = content.len() as u64;
        let is_binary = Self::is_binary_content(&content);
        let checksum = Self::compute_checksum(&content);

        Ok(FileInfo {
            path: path.to_path_buf(),
            checksum,
            size,
            is_binary,
        })
    }

    /// 计算内容校验和
    fn compute_checksum(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        hex::encode(hasher.finalize())[..32].to_string()
    }

    /// 检测是否为二进制内容
    fn is_binary_content(content: &[u8]) -> bool {
        // 检查是否包含空字节或高比例的非打印字符
        if content.is_empty() {
            return false;
        }

        // 空字节检测（二进制文件的强指示）
        if content.contains(&0) {
            return true;
        }

        // 采样检查非打印字符比例
        let sample_size = content.len().min(8192);
        let sample = &content[..sample_size];

        let non_printable = sample
            .iter()
            .filter(|&&b| b < 0x20 && b != 0x09 && b != 0x0a && b != 0x0d)
            .count();

        // 如果超过 30% 是非打印字符，认为是二进制
        (non_printable as f64 / sample_size as f64) > 0.3
    }

    /// 获取文件信息
    pub fn get_file(&self, relative_path: &Path) -> Option<&FileInfo> {
        self.files.get(relative_path)
    }

    /// 检查文件是否存在
    pub fn contains(&self, relative_path: &Path) -> bool {
        self.files.contains_key(relative_path)
    }

    /// 获取文件数量
    pub fn c_file_count(&self) -> usize {
        self.files.len()
    }

    /// 获取所有文件路径
    pub fn paths(&self) -> Vec<&PathBuf> {
        self.files.keys().collect()
    }
}

/// Diff 引擎 - 比较内存缓冲区与磁盘状态
pub struct DiffEngine {
    /// 基础路径（用于构建完整路径）
    pub base_path: PathBuf,
}

impl DiffEngine {
    /// 创建新的 DiffEngine
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// 比较内存缓冲区与磁盘快照
    pub fn compare_with_snapshot(
        &self,
        memory_files: &[CapturedFile],
        snapshot: &DiskSnapshot,
    ) -> DiffResult {
        let mut result = DiffResult::new();
        let mut processed_paths = HashMap::new();

        // 1. 处理内存中的文件（Created, Updated, Unchanged）
        for mem_file in memory_files {
            processed_paths.insert(&mem_file.path, true);

            match snapshot.get_file(&mem_file.path) {
                Some(disk_file) => {
                    // 文件在磁盘上存在，比较校验和
                    if disk_file.checksum == mem_file.checksum {
                        result.add_change(FileChange::Unchanged {
                            path: mem_file.path.clone(),
                            checksum: mem_file.checksum.clone(),
                        });
                    } else {
                        result.add_change(FileChange::Updated {
                            path: mem_file.path.clone(),
                            old_checksum: disk_file.checksum.clone(),
                            new_checksum: mem_file.checksum.clone(),
                        });
                    }
                }
                None => {
                    // 文件在磁盘上不存在，标记为创建
                    result.add_change(FileChange::Created {
                        path: mem_file.path.clone(),
                        checksum: mem_file.checksum.clone(),
                    });
                }
            }
        }

        // 2. 处理磁盘上有但内存中没有的文件（Deleted）
        for (path, file_info) in &snapshot.files {
            if !processed_paths.contains_key(path) {
                result.add_change(FileChange::Deleted {
                    path: path.clone(),
                    checksum: file_info.checksum.clone(),
                });
            }
        }

        result
    }

    /// 直接比较内存缓冲区与磁盘（自动创建快照）
    pub fn compare(&self, memory_files: &[CapturedFile]) -> Result<DiffResult, WriteError> {
        let snapshot = DiskSnapshot::scan(&self.base_path)?;
        Ok(self.compare_with_snapshot(memory_files, &snapshot))
    }

    /// 比较单个文件
    pub fn compare_single(&self, memory_file: &CapturedFile) -> Result<FileChange, WriteError> {
        let full_path = self.base_path.join(&memory_file.path);

        if !full_path.exists() {
            return Ok(FileChange::Created {
                path: memory_file.path.clone(),
                checksum: memory_file.checksum.clone(),
            });
        }

        let file_info = DiskSnapshot::scan(&self.base_path)?
            .get_file(&memory_file.path)
            .cloned();

        match file_info {
            Some(disk_file) => {
                if disk_file.checksum == memory_file.checksum {
                    Ok(FileChange::Unchanged {
                        path: memory_file.path.clone(),
                        checksum: memory_file.checksum.clone(),
                    })
                } else {
                    Ok(FileChange::Updated {
                        path: memory_file.path.clone(),
                        old_checksum: disk_file.checksum,
                        new_checksum: memory_file.checksum.clone(),
                    })
                }
            }
            None => Ok(FileChange::Created {
                path: memory_file.path.clone(),
                checksum: memory_file.checksum.clone(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, relative_path: &str, content: &str) -> PathBuf {
        let full_path = dir.join(relative_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full_path, content).unwrap();
        full_path
    }

    fn create_captured_file(path: &str, content: &str) -> CapturedFile {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let checksum = hex::encode(hasher.finalize())[..32].to_string();

        CapturedFile {
            path: PathBuf::from(path),
            content: content.to_string(),
            checksum,
        }
    }

    #[test]
    fn test_disk_snapshot_scan_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let snapshot = DiskSnapshot::scan(temp_dir.path()).unwrap();

        assert_eq!(snapshot.c_file_count(), 0);
        assert!(snapshot.paths().is_empty());
    }

    #[test]
    fn test_disk_snapshot_scan_with_files() {
        let temp_dir = TempDir::new().unwrap();
        create_test_file(temp_dir.path(), "test.txt", "Hello World");
        create_test_file(temp_dir.path(), "subdir/nested.txt", "Nested content");

        let snapshot = DiskSnapshot::scan(temp_dir.path()).unwrap();

        assert_eq!(snapshot.c_file_count(), 2);
        assert!(snapshot.contains(Path::new("test.txt")));
        assert!(snapshot.contains(Path::new("subdir/nested.txt")));
    }

    #[test]
    fn test_disk_snapshot_checksum() {
        let temp_dir = TempDir::new().unwrap();
        create_test_file(temp_dir.path(), "test.txt", "Hello World");

        let snapshot = DiskSnapshot::scan(temp_dir.path()).unwrap();
        let file_info = snapshot.get_file(Path::new("test.txt")).unwrap();

        assert_eq!(file_info.size, 11); // "Hello World" = 11 bytes
        assert!(!file_info.checksum.is_empty());
    }

    #[test]
    fn test_disk_snapshot_binary_detection() {
        let temp_dir = TempDir::new().unwrap();

        // 文本文件
        create_test_file(temp_dir.path(), "text.txt", "Hello World\nLine 2");

        // 二进制文件（包含空字节）
        let binary_content = vec![0x00, 0x01, 0x02, 0x03, 0x00, 0xFF];
        fs::write(temp_dir.path().join("binary.bin"), &binary_content).unwrap();

        let snapshot = DiskSnapshot::scan(temp_dir.path()).unwrap();

        let text_info = snapshot.get_file(Path::new("text.txt")).unwrap();
        assert!(!text_info.is_binary);

        let binary_info = snapshot.get_file(Path::new("binary.bin")).unwrap();
        assert!(binary_info.is_binary);
    }

    #[test]
    fn test_diff_engine_created() {
        let temp_dir = TempDir::new().unwrap();
        let engine = DiffEngine::new(temp_dir.path());

        // 内存中有，磁盘上没有
        let memory_files = vec![create_captured_file("new_file.txt", "New content")];

        let result = engine.compare(&memory_files).unwrap();

        assert_eq!(result.created_count(), 1);
        assert_eq!(result.updated_count(), 0);
        assert_eq!(result.deleted_count(), 0);
        assert_eq!(result.unchanged_count(), 0);
        assert!(result.has_changes());
    }

    #[test]
    fn test_diff_engine_updated() {
        let temp_dir = TempDir::new().unwrap();

        // 创建磁盘文件
        create_test_file(temp_dir.path(), "existing.txt", "Old content");

        let engine = DiffEngine::new(temp_dir.path());

        // 内存中有相同路径但不同内容
        let memory_files = vec![create_captured_file("existing.txt", "New content")];

        let result = engine.compare(&memory_files).unwrap();

        assert_eq!(result.created_count(), 0);
        assert_eq!(result.updated_count(), 1);
        assert_eq!(result.deleted_count(), 0);
        assert_eq!(result.unchanged_count(), 0);
        assert!(result.has_changes());

        let updated = result.updated();
        assert_eq!(updated.len(), 1);
        assert!(updated[0].old_checksum().is_some());
        assert!(updated[0].new_checksum().is_some());
    }

    #[test]
    fn test_diff_engine_unchanged() {
        let temp_dir = TempDir::new().unwrap();

        // 创建磁盘文件
        create_test_file(temp_dir.path(), "same.txt", "Same content");

        let engine = DiffEngine::new(temp_dir.path());

        // 内存中有相同路径和相同内容
        let memory_files = vec![create_captured_file("same.txt", "Same content")];

        let result = engine.compare(&memory_files).unwrap();

        assert_eq!(result.created_count(), 0);
        assert_eq!(result.updated_count(), 0);
        assert_eq!(result.deleted_count(), 0);
        assert_eq!(result.unchanged_count(), 1);
        assert!(!result.has_changes());
    }

    #[test]
    fn test_diff_engine_deleted() {
        let temp_dir = TempDir::new().unwrap();

        // 创建磁盘文件
        create_test_file(temp_dir.path(), "old.txt", "Old file content");

        let engine = DiffEngine::new(temp_dir.path());

        // 内存中没有这个文件
        let memory_files: Vec<CapturedFile> = vec![];

        let result = engine.compare(&memory_files).unwrap();

        assert_eq!(result.created_count(), 0);
        assert_eq!(result.updated_count(), 0);
        assert_eq!(result.deleted_count(), 1);
        assert_eq!(result.unchanged_count(), 0);
        assert!(result.has_changes());
    }

    #[test]
    fn test_diff_engine_all_change_types() {
        let temp_dir = TempDir::new().unwrap();

        // 创建磁盘文件
        create_test_file(temp_dir.path(), "keep.txt", "Keep content");
        create_test_file(temp_dir.path(), "update.txt", "Old update content");
        create_test_file(temp_dir.path(), "delete.txt", "Will be deleted");

        let engine = DiffEngine::new(temp_dir.path());

        // 内存文件：保留（相同）、更新（不同）、创建（新）、不删除旧文件
        let memory_files = vec![
            create_captured_file("keep.txt", "Keep content"),
            create_captured_file("update.txt", "New update content"),
            create_captured_file("new.txt", "Brand new file"),
        ];

        let result = engine.compare(&memory_files).unwrap();

        assert_eq!(result.created_count(), 1);
        assert_eq!(result.updated_count(), 1);
        assert_eq!(result.deleted_count(), 1);
        assert_eq!(result.unchanged_count(), 1);
        assert_eq!(result.actual_changes_count(), 3);
    }

    #[test]
    fn test_diff_engine_nested_directories() {
        let temp_dir = TempDir::new().unwrap();

        // 创建嵌套目录结构
        create_test_file(temp_dir.path(), "level1/file1.txt", "Content 1");
        create_test_file(temp_dir.path(), "level1/level2/file2.txt", "Content 2");
        create_test_file(
            temp_dir.path(),
            "level1/level2/level3/file3.txt",
            "Content 3",
        );

        let engine = DiffEngine::new(temp_dir.path());

        let memory_files = vec![
            create_captured_file("level1/file1.txt", "Modified 1"),
            create_captured_file("level1/level2/file2.txt", "Content 2"), // 不变
            create_captured_file("level1/level2/new_file.txt", "New nested"),
        ];

        let result = engine.compare(&memory_files).unwrap();

        assert_eq!(result.updated_count(), 1);
        assert_eq!(result.unchanged_count(), 1);
        assert_eq!(result.created_count(), 1);
        assert_eq!(result.deleted_count(), 1); // file3.txt
    }

    #[test]
    fn test_diff_result_methods() {
        let mut result = DiffResult::new();

        result.add_change(FileChange::Created {
            path: PathBuf::from("a.txt"),
            checksum: "abc".to_string(),
        });
        result.add_change(FileChange::Updated {
            path: PathBuf::from("b.txt"),
            old_checksum: "old".to_string(),
            new_checksum: "new".to_string(),
        });
        result.add_change(FileChange::Deleted {
            path: PathBuf::from("c.txt"),
            checksum: "del".to_string(),
        });
        result.add_change(FileChange::Unchanged {
            path: PathBuf::from("d.txt"),
            checksum: "same".to_string(),
        });

        assert_eq!(result.total_count(), 4);
        assert_eq!(result.created_count(), 1);
        assert_eq!(result.updated_count(), 1);
        assert_eq!(result.deleted_count(), 1);
        assert_eq!(result.unchanged_count(), 1);
        assert_eq!(result.actual_changes_count(), 3);
        assert!(result.has_changes());
    }

    #[test]
    fn test_file_change_methods() {
        let created = FileChange::Created {
            path: PathBuf::from("new.txt"),
            checksum: "abc123".to_string(),
        };
        assert!(created.is_created());
        assert!(!created.is_updated());
        assert!(!created.is_deleted());
        assert!(!created.is_unchanged());
        assert_eq!(created.checksum(), Some("abc123"));
        assert_eq!(created.old_checksum(), None);
        assert_eq!(created.new_checksum(), None);

        let updated = FileChange::Updated {
            path: PathBuf::from("mod.txt"),
            old_checksum: "old456".to_string(),
            new_checksum: "new789".to_string(),
        };
        assert!(!updated.is_created());
        assert!(updated.is_updated());
        assert!(!updated.is_deleted());
        assert!(!updated.is_unchanged());
        assert_eq!(updated.checksum(), None);
        assert_eq!(updated.old_checksum(), Some("old456"));
        assert_eq!(updated.new_checksum(), Some("new789"));

        let deleted = FileChange::Deleted {
            path: PathBuf::from("del.txt"),
            checksum: "del000".to_string(),
        };
        assert!(deleted.is_deleted());
        assert_eq!(deleted.path(), &PathBuf::from("del.txt"));

        let unchanged = FileChange::Unchanged {
            path: PathBuf::from("same.txt"),
            checksum: "same999".to_string(),
        };
        assert!(unchanged.is_unchanged());
        assert_eq!(unchanged.checksum(), Some("same999"));
    }

    #[test]
    fn test_compare_single_created() {
        let temp_dir = TempDir::new().unwrap();
        let engine = DiffEngine::new(temp_dir.path());

        let memory_file = create_captured_file("single_new.txt", "New content");
        let result = engine.compare_single(&memory_file).unwrap();

        assert!(matches!(result, FileChange::Created { .. }));
    }

    #[test]
    fn test_compare_single_unchanged() {
        let temp_dir = TempDir::new().unwrap();
        create_test_file(temp_dir.path(), "single_same.txt", "Same content");

        let engine = DiffEngine::new(temp_dir.path());
        let memory_file = create_captured_file("single_same.txt", "Same content");
        let result = engine.compare_single(&memory_file).unwrap();

        assert!(matches!(result, FileChange::Unchanged { .. }));
    }

    #[test]
    fn test_disk_snapshot_nonexistent_dir() {
        let snapshot = DiskSnapshot::scan("/nonexistent/path/12345").unwrap();
        assert_eq!(snapshot.c_file_count(), 0);
    }

    #[test]
    fn test_performance_100_files() {
        use std::time::Instant;

        let temp_dir = TempDir::new().unwrap();
        let mut memory_files = Vec::new();

        // 创建 100 个文件
        for i in 0..100 {
            let filename = format!("file_{:03}.txt", i);
            let content = format!(
                "Content of file {} with some padding to make it realistic",
                i
            );
            create_test_file(temp_dir.path(), &filename, &content);

            // 一半文件相同，一半不同
            if i % 2 == 0 {
                memory_files.push(create_captured_file(&filename, &content));
            } else {
                memory_files.push(create_captured_file(&filename, "Different content"));
            }
        }

        let engine = DiffEngine::new(temp_dir.path());

        let start = Instant::now();
        let result = engine.compare(&memory_files).unwrap();
        let duration = start.elapsed();

        assert_eq!(result.total_count(), 100);
        assert_eq!(result.unchanged_count(), 50);
        assert_eq!(result.updated_count(), 50);

        // 性能要求：100+ 文件 < 500ms
        assert!(
            duration.as_millis() < 500,
            "Diff 100 files took {}ms, expected < 500ms",
            duration.as_millis()
        );
    }

    #[test]
    fn test_empty_directory_handling() {
        let temp_dir = TempDir::new().unwrap();
        let engine = DiffEngine::new(temp_dir.path());

        // 测试空内存文件对比空目录
        let result = engine.compare(&[]).unwrap();
        assert!(!result.has_changes());
        assert_eq!(result.total_count(), 0);
    }

    #[test]
    fn test_binary_file_handling() {
        let temp_dir = TempDir::new().unwrap();

        // 创建二进制文件
        let binary_content: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
        fs::write(temp_dir.path().join("data.bin"), &binary_content).unwrap();

        let engine = DiffEngine::new(temp_dir.path());

        // 读取并计算校验和
        let snapshot = DiskSnapshot::scan(temp_dir.path()).unwrap();
        let file_info = snapshot.get_file(Path::new("data.bin")).unwrap();

        assert!(file_info.is_binary);

        // 创建相同内容的内存文件
        let mut hasher = Sha256::new();
        hasher.update(&binary_content);
        let checksum = hex::encode(hasher.finalize())[..32].to_string();

        let memory_file = CapturedFile {
            path: PathBuf::from("data.bin"),
            content: String::from_utf8_lossy(&binary_content).to_string(),
            checksum,
        };

        let result = engine.compare(&[memory_file]).unwrap();
        assert_eq!(result.unchanged_count(), 1);
    }
}
