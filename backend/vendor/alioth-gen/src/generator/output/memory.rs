//! 用于预览的内存缓冲输出写入器

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::diff::{DiffEngine, DiffResult};
use super::writer::{add_ordering_markers, DryRunResult, OutputWriter, WriteError};
use crate::generator::GeneratedOutput;

/// 在内存中捕获生成的输出而不写入磁盘
pub struct MemoryBufferWriter {
    buffer: Arc<Mutex<Vec<CapturedFile>>>,
}

#[derive(Debug, Clone)]
pub struct CapturedFile {
    pub path: PathBuf,
    pub content: String,
    pub checksum: String,
}

impl MemoryBufferWriter {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn get_buffer(&self) -> Arc<Mutex<Vec<CapturedFile>>> {
        Arc::clone(&self.buffer)
    }

    pub fn get_files(&self) -> Vec<CapturedFile> {
        self.buffer.lock().unwrap().clone()
    }

    pub fn get_file(&self, path: &PathBuf) -> Option<CapturedFile> {
        self.buffer
            .lock()
            .unwrap()
            .iter()
            .find(|f| f.path == *path)
            .cloned()
    }

    pub fn clear(&self) {
        self.buffer.lock().unwrap().clear();
    }

    /// 将内存缓冲区与磁盘上的文件进行对比
    ///
    /// # Arguments
    /// * `base_path` - 磁盘文件的基础路径
    ///
    /// # Returns
    /// 返回 DiffResult，包含所有变更的分类
    pub fn diff_with_disk<P: AsRef<std::path::Path>>(
        &self,
        base_path: P,
    ) -> Result<DiffResult, WriteError> {
        let files = self.get_files();
        let engine = DiffEngine::new(base_path);
        engine.compare(&files)
    }

    /// 比较单个文件与磁盘
    ///
    /// # Arguments
    /// * `path` - 文件路径
    /// * `base_path` - 磁盘文件的基础路径
    ///
    /// # Returns
    /// 返回该文件的变更类型
    pub fn diff_single_with_disk<P: AsRef<std::path::Path>>(
        &self,
        path: &PathBuf,
        base_path: P,
    ) -> Result<super::diff::FileChange, WriteError> {
        let file = self
            .get_file(path)
            .ok_or_else(|| WriteError::InvalidPath(path.clone()))?;

        let engine = DiffEngine::new(base_path);
        engine.compare_single(&file)
    }

    /// 获取文件数量
    pub fn c_file_count(&self) -> usize {
        self.buffer.lock().unwrap().len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.c_file_count() == 0
    }
}

impl Default for MemoryBufferWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputWriter for MemoryBufferWriter {
    fn write(&self, output: &GeneratedOutput) -> Result<(), WriteError> {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.clear();

        // 收集所有文件路径作为实体名称
        let entity_names: Vec<Vec<String>> = output
            .files
            .iter()
            .map(|f| vec![f.path.to_string_lossy().to_string()])
            .collect();

        for file in &output.files {
            let mut content = file.content.clone();
            add_ordering_markers(&mut content, &entity_names);

            buffer.push(CapturedFile {
                path: file.path.clone(),
                content,
                checksum: file.checksum.clone(),
            });
        }

        Ok(())
    }

    fn dry_run(&self, output: &GeneratedOutput) -> Result<DryRunResult, WriteError> {
        let files_to_write: Vec<PathBuf> = output.files.iter().map(|f| f.path.clone()).collect();

        Ok(DryRunResult {
            files_to_write: files_to_write.clone(),
            files_to_create: files_to_write,
            files_to_update: vec![],
        })
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
                    path: PathBuf::from(path),
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
    fn test_memory_buffer_writer_new() {
        let writer = MemoryBufferWriter::new();
        assert!(writer.is_empty());
        assert_eq!(writer.c_file_count(), 0);
    }

    #[test]
    fn test_memory_buffer_writer_write() {
        let writer = MemoryBufferWriter::new();

        let output =
            create_test_output(vec![("file1.txt", "Content 1"), ("file2.txt", "Content 2")]);

        writer.write(&output).unwrap();

        assert_eq!(writer.c_file_count(), 2);
        assert!(!writer.is_empty());

        let files = writer.get_files();
        assert_eq!(files.len(), 2);
        assert!(files[0].content.contains("@alioth-generated"));
    }

    #[test]
    fn test_memory_buffer_writer_get_file() {
        let writer = MemoryBufferWriter::new();

        let output = create_test_output(vec![
            ("specific.txt", "Specific content"),
            ("other.txt", "Other content"),
        ]);

        writer.write(&output).unwrap();

        let file = writer.get_file(&PathBuf::from("specific.txt"));
        assert!(file.is_some());
        assert!(file.unwrap().content.contains("Specific content"));

        let missing = writer.get_file(&PathBuf::from("missing.txt"));
        assert!(missing.is_none());
    }

    #[test]
    fn test_memory_buffer_writer_clear() {
        let writer = MemoryBufferWriter::new();

        let output = create_test_output(vec![("file.txt", "Content")]);
        writer.write(&output).unwrap();

        assert_eq!(writer.c_file_count(), 1);

        writer.clear();

        assert!(writer.is_empty());
        assert_eq!(writer.c_file_count(), 0);
    }

    #[test]
    fn test_memory_buffer_writer_dry_run() {
        let writer = MemoryBufferWriter::new();

        let output = create_test_output(vec![("a.txt", "A"), ("b.txt", "B")]);

        let result = writer.dry_run(&output).unwrap();

        assert_eq!(result.files_to_write.len(), 2);
        assert_eq!(result.files_to_create.len(), 2);
        assert_eq!(result.files_to_update.len(), 0);
    }

    #[test]
    fn test_memory_buffer_writer_default() {
        let writer: MemoryBufferWriter = Default::default();
        assert!(writer.is_empty());
    }

    #[test]
    fn test_diff_with_disk_created() {
        let temp_dir = TempDir::new().unwrap();
        let writer = MemoryBufferWriter::new();

        let output = create_test_output(vec![("new_file.txt", "New content")]);
        writer.write(&output).unwrap();

        let diff_result = writer.diff_with_disk(temp_dir.path()).unwrap();

        assert_eq!(diff_result.created_count(), 1);
        assert!(diff_result.has_changes()); // 创建文件是一种变更
        assert_eq!(diff_result.actual_changes_count(), 1);
    }

    #[test]
    fn test_diff_with_disk_updated() {
        let temp_dir = TempDir::new().unwrap();

        // 创建磁盘文件
        fs::write(temp_dir.path().join("existing.txt"), "Old content").unwrap();

        let writer = MemoryBufferWriter::new();
        let output = create_test_output(vec![("existing.txt", "New content")]);
        writer.write(&output).unwrap();

        let diff_result = writer.diff_with_disk(temp_dir.path()).unwrap();

        assert_eq!(diff_result.updated_count(), 1);
        assert!(diff_result.has_changes());
    }

    #[test]
    fn test_diff_with_disk_unchanged() {
        let temp_dir = TempDir::new().unwrap();

        // 创建相同内容的磁盘文件
        fs::write(temp_dir.path().join("same.txt"), "Same content").unwrap();

        let writer = MemoryBufferWriter::new();
        let output = create_test_output(vec![("same.txt", "Same content")]);
        writer.write(&output).unwrap();

        let diff_result = writer.diff_with_disk(temp_dir.path()).unwrap();

        assert_eq!(diff_result.unchanged_count(), 1);
        assert!(!diff_result.has_changes());
    }

    #[test]
    fn test_diff_with_disk_deleted() {
        let temp_dir = TempDir::new().unwrap();

        // 创建磁盘文件
        fs::write(temp_dir.path().join("orphan.txt"), "Orphan content").unwrap();

        let writer = MemoryBufferWriter::new();
        // 不写入任何文件

        let diff_result = writer.diff_with_disk(temp_dir.path()).unwrap();

        assert_eq!(diff_result.deleted_count(), 1);
        assert!(diff_result.has_changes());
    }

    #[test]
    fn test_diff_single_with_disk() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("test.txt"), "Content").unwrap();

        let writer = MemoryBufferWriter::new();
        let output = create_test_output(vec![("test.txt", "Content"), ("missing.txt", "Missing")]);
        writer.write(&output).unwrap();

        // 测试存在的文件
        let result = writer.diff_single_with_disk(&PathBuf::from("test.txt"), temp_dir.path());
        assert!(result.is_ok());
        let change = result.unwrap();
        assert!(change.is_unchanged());

        // 测试不存在的文件（应该返回 created，因为内存中有但磁盘上没有）
        let result = writer.diff_single_with_disk(&PathBuf::from("missing.txt"), temp_dir.path());
        assert!(result.is_ok());
        let change = result.unwrap();
        assert!(change.is_created());
    }

    #[test]
    fn test_diff_single_invalid_path() {
        let temp_dir = TempDir::new().unwrap();
        let writer = MemoryBufferWriter::new();

        let output = create_test_output(vec![("exists.txt", "Content")]);
        writer.write(&output).unwrap();

        let result =
            writer.diff_single_with_disk(&PathBuf::from("not_in_buffer.txt"), temp_dir.path());
        assert!(result.is_err());
    }
}
