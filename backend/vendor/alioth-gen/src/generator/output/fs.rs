//! 文件系统输出写入器

use std::fs;
use std::path::PathBuf;

use super::diff::{DiffEngine, DiffResult};
use super::memory::CapturedFile;
use super::writer::{add_ordering_markers, DryRunResult, OutputWriter, WriteError};
use crate::generator::GeneratedOutput;

/// 将生成的文件写入文件系统
pub struct FileSystemWriter {
    base_path: PathBuf,
    create_dirs: bool,
    overwrite_existing: bool,
}

impl FileSystemWriter {
    pub fn new<P: Into<PathBuf>>(base_path: P) -> Self {
        Self {
            base_path: base_path.into(),
            create_dirs: true,
            overwrite_existing: true,
        }
    }

    pub fn with_options(base_path: PathBuf, create_dirs: bool, overwrite_existing: bool) -> Self {
        Self {
            base_path,
            create_dirs,
            overwrite_existing,
        }
    }

    /// 执行差异分析，比较生成输出与磁盘上的现有文件
    pub fn diff(&self, output: &GeneratedOutput) -> Result<DiffResult, WriteError> {
        let memory_files: Vec<CapturedFile> = output
            .files
            .iter()
            .map(|f| CapturedFile {
                path: f.path.clone(),
                content: f.content.clone(),
                checksum: f.checksum.clone(),
            })
            .collect();

        let engine = DiffEngine::new(&self.base_path);
        engine.compare(&memory_files)
    }

    /// 获取将要创建的文件列表
    pub fn get_files_to_create(&self, output: &GeneratedOutput) -> Vec<PathBuf> {
        match self.dry_run(output) {
            Ok(result) => result.files_to_create,
            Err(_) => vec![],
        }
    }

    /// 获取将要更新的文件列表
    pub fn get_files_to_update(&self, output: &GeneratedOutput) -> Vec<PathBuf> {
        match self.dry_run(output) {
            Ok(result) => result.files_to_update,
            Err(_) => vec![],
        }
    }
}

impl OutputWriter for FileSystemWriter {
    fn write(&self, output: &GeneratedOutput) -> Result<(), WriteError> {
        for file in &output.files {
            let full_path = self.base_path.join(&file.path);

            if let Some(parent) = full_path.parent() {
                if self.create_dirs && !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }

            if full_path.exists() && !self.overwrite_existing {
                continue;
            }

            let mut content = file.content.clone();
            // 收集所有文件路径作为实体名称
            let entity_names: Vec<Vec<String>> = output
                .files
                .iter()
                .map(|f| vec![f.path.to_string_lossy().to_string()])
                .collect();
            add_ordering_markers(&mut content, &entity_names);

            fs::write(&full_path, content)?;
        }
        Ok(())
    }

    fn dry_run(&self, output: &GeneratedOutput) -> Result<DryRunResult, WriteError> {
        // 使用 DiffEngine 获取准确的文件变更分析
        let diff_result = self.diff(output)?;

        let mut result = DryRunResult {
            files_to_write: vec![],
            files_to_create: vec![],
            files_to_update: vec![],
        };

        // 构建完整的文件路径列表
        for file in &output.files {
            let full_path = self.base_path.join(&file.path);
            result.files_to_write.push(full_path.clone());

            // 根据 diff 结果分类
            match diff_result.changes.get(&file.path) {
                Some(change) if change.is_created() => {
                    result.files_to_create.push(full_path);
                }
                Some(change) if change.is_updated() => {
                    result.files_to_update.push(full_path);
                }
                Some(change) if change.is_unchanged() => {
                    // 未变化的文件仍然可能需要写入（如果覆盖模式开启）
                    if self.overwrite_existing {
                        result.files_to_update.push(full_path);
                    }
                }
                _ => {
                    // 默认情况：假设是新文件
                    result.files_to_create.push(full_path);
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_output(files: Vec<(&str, &str)>) -> GeneratedOutput {
        use sha2::{Digest, Sha256};

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
    fn test_fs_writer_new() {
        let temp_dir = TempDir::new().unwrap();
        let writer = FileSystemWriter::new(temp_dir.path());

        assert!(writer.create_dirs);
        assert!(writer.overwrite_existing);
    }

    #[test]
    fn test_fs_writer_with_options() {
        let temp_dir = TempDir::new().unwrap();
        let writer = FileSystemWriter::with_options(temp_dir.path().to_path_buf(), false, false);

        assert!(!writer.create_dirs);
        assert!(!writer.overwrite_existing);
    }

    #[test]
    fn test_fs_writer_dry_run_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let writer = FileSystemWriter::new(temp_dir.path());

        let output =
            create_test_output(vec![("file1.txt", "Content 1"), ("file2.txt", "Content 2")]);

        let result = writer.dry_run(&output).unwrap();

        assert_eq!(result.files_to_write.len(), 2);
        assert_eq!(result.files_to_create.len(), 2);
        assert_eq!(result.files_to_update.len(), 0);
    }

    #[test]
    fn test_fs_writer_dry_run_with_existing_files() {
        let temp_dir = TempDir::new().unwrap();

        // 创建现有文件
        fs::write(temp_dir.path().join("existing.txt"), "Old content").unwrap();
        fs::write(temp_dir.path().join("unchanged.txt"), "Same content").unwrap();

        let writer = FileSystemWriter::new(temp_dir.path());

        let output = create_test_output(vec![
            ("existing.txt", "New content"),   // 更新
            ("unchanged.txt", "Same content"), // 不变
            ("new.txt", "Brand new"),          // 创建
        ]);

        let result = writer.dry_run(&output).unwrap();

        assert_eq!(result.files_to_write.len(), 3);
        assert_eq!(result.files_to_create.len(), 1); // new.txt
                                                     // 当 overwrite_existing=true 时，未变化的文件也会被添加到 files_to_update
                                                     // 因为 writer 会重新写入所有现有文件
        assert_eq!(result.files_to_update.len(), 2); // existing.txt + unchanged.txt

        let update_paths: Vec<_> = result
            .files_to_update
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(update_paths.contains(&"existing.txt".to_string()));
        assert!(update_paths.contains(&"unchanged.txt".to_string()));
    }

    #[test]
    fn test_fs_writer_dry_run_no_overwrite() {
        let temp_dir = TempDir::new().unwrap();

        // 创建现有文件
        fs::write(temp_dir.path().join("existing.txt"), "Old content").unwrap();
        fs::write(temp_dir.path().join("unchanged.txt"), "Same content").unwrap();

        // 使用 overwrite_existing=false
        let writer = FileSystemWriter::with_options(temp_dir.path().to_path_buf(), true, false);

        let output = create_test_output(vec![
            ("existing.txt", "New content"),   // 更新
            ("unchanged.txt", "Same content"), // 不变
            ("new.txt", "Brand new"),          // 创建
        ]);

        let result = writer.dry_run(&output).unwrap();

        assert_eq!(result.files_to_write.len(), 3);
        assert_eq!(result.files_to_create.len(), 1); // new.txt
                                                     // 当 overwrite_existing=false 时，只有实际变化的文件会添加到 files_to_update
        assert_eq!(result.files_to_update.len(), 1); // existing.txt

        let update_paths: Vec<_> = result
            .files_to_update
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(update_paths.contains(&"existing.txt".to_string()));
        assert!(!update_paths.contains(&"unchanged.txt".to_string()));
    }

    #[test]
    fn test_fs_writer_diff() {
        let temp_dir = TempDir::new().unwrap();

        // 创建现有文件
        fs::write(temp_dir.path().join("old.txt"), "Old content").unwrap();

        let writer = FileSystemWriter::new(temp_dir.path());

        let output = create_test_output(vec![("old.txt", "New content"), ("new.txt", "New file")]);

        let diff_result = writer.diff(&output).unwrap();

        assert_eq!(diff_result.created_count(), 1);
        assert_eq!(diff_result.updated_count(), 1);
        assert_eq!(diff_result.deleted_count(), 0); // old.txt 被更新了，不是删除
        assert!(diff_result.has_changes());
    }

    #[test]
    fn test_fs_writer_get_files_methods() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("existing.txt"), "Existing").unwrap();

        let writer = FileSystemWriter::new(temp_dir.path());

        let output = create_test_output(vec![
            ("existing.txt", "Modified"),
            ("new.txt", "New content"),
        ]);

        let to_create = writer.get_files_to_create(&output);
        let to_update = writer.get_files_to_update(&output);

        assert_eq!(to_create.len(), 1);
        assert!(to_create[0].ends_with("new.txt"));

        assert_eq!(to_update.len(), 1);
        assert!(to_update[0].ends_with("existing.txt"));
    }

    #[test]
    fn test_fs_writer_write_creates_files() {
        let temp_dir = TempDir::new().unwrap();
        let writer = FileSystemWriter::new(temp_dir.path());

        let output = create_test_output(vec![
            ("test.txt", "Test content"),
            ("nested/dir/file.txt", "Nested content"),
        ]);

        writer.write(&output).unwrap();

        assert!(temp_dir.path().join("test.txt").exists());
        assert!(temp_dir.path().join("nested/dir/file.txt").exists());

        let content = fs::read_to_string(temp_dir.path().join("test.txt")).unwrap();
        assert!(content.contains("Test content"));
        assert!(content.contains("@alioth-generated"));
    }

    #[test]
    fn test_fs_writer_no_overwrite() {
        let temp_dir = TempDir::new().unwrap();

        // 先写入一个文件
        fs::write(temp_dir.path().join("protected.txt"), "Original").unwrap();

        let writer = FileSystemWriter::with_options(temp_dir.path().to_path_buf(), true, false);

        let output = create_test_output(vec![("protected.txt", "Modified")]);

        writer.write(&output).unwrap();

        // 文件应该保持不变
        let content = fs::read_to_string(temp_dir.path().join("protected.txt")).unwrap();
        assert_eq!(content, "Original");
    }
}
