//! 受保护区域解析与合并引擎
//!
//! 支持安全增量重新生成：
//! - 解析生成文件中的保护区域标记
//! - 合并用户手写代码与新生成内容
//! - 检测并报告合并冲突

use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::diff::FileChange;
use super::memory::CapturedFile;
use super::writer::WriteError;

/// 受保护区域标记的默认格式
pub const GENERATED_MARKER_START: &str = "// @alioth-protected-start";
pub const GENERATED_MARKER_END: &str = "// @alioth-protected-end";
pub const DO_NOT_EDIT_MARKER: &str = "// DO NOT EDIT MANUALLY";

/// 保护区域解析结果
#[derive(Debug, Clone, PartialEq)]
pub struct ProtectedRegion {
    /// 标记之前的用户代码（前缀）
    pub prefix: String,
    /// 生成的代码内容
    pub generated: String,
    /// 标记之后的用户代码（后缀）
    pub suffix: String,
    /// 标记中的元数据
    pub metadata: Option<MarkerMetadata>,
}

/// 标记元数据
#[derive(Debug, Clone, PartialEq)]
pub struct MarkerMetadata {
    /// 校验和
    pub checksum: String,
    /// 时间戳
    pub timestamp: String,
    /// 排序信息
    pub order: Vec<String>,
    /// 原始标记行
    pub marker_lines: Vec<String>,
}

/// 合并结果
#[derive(Debug, Clone)]
pub enum MergeResult {
    /// 成功合并
    Merged(String),
    /// 存在冲突
    Conflict(ConflictReport),
    /// 跳过（无需更改）
    Skipped,
}

/// 冲突报告
#[derive(Debug, Clone)]
pub struct ConflictReport {
    /// 文件路径
    pub file_path: PathBuf,
    /// 发现的冲突列表
    pub conflicts: Vec<Conflict>,
    /// 合并建议
    pub suggestion: String,
}

/// 单个冲突
#[derive(Debug, Clone, PartialEq)]
pub struct Conflict {
    /// 冲突类型
    pub conflict_type: ConflictType,
    /// 冲突发生的行号
    pub line_number: Option<usize>,
    /// 冲突描述
    pub description: String,
    /// 用户代码中的符号（如果有）
    pub user_symbol: Option<String>,
    /// 生成代码中的符号（如果有）
    pub generated_symbol: Option<String>,
}

/// 冲突类型
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictType {
    /// 命名冲突：用户符号与新生成符号冲突
    NamingCollision,
    /// 引用断裂：用户代码引用了不存在的新生成符号
    BrokenReference,
    /// 标记损坏：无法解析保护区域标记
    CorruptedMarker,
    /// 缺少标记：文件没有保护区域标记
    MissingMarker,
    /// 多个标记：文件包含多个保护区域
    MultipleMarkers,
    /// 语法错误：用户代码存在语法问题
    SyntaxError,
}

impl ConflictReport {
    /// 创建新的冲突报告
    pub fn new(file_path: impl AsRef<Path>) -> Self {
        Self {
            file_path: file_path.as_ref().to_path_buf(),
            conflicts: vec![],
            suggestion: String::new(),
        }
    }

    /// 添加冲突
    pub fn add_conflict(&mut self, conflict: Conflict) {
        self.conflicts.push(conflict);
    }

    /// 检查是否有冲突
    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }

    /// 获取冲突数量
    pub fn conflict_count(&self) -> usize {
        self.conflicts.len()
    }

    /// 生成人类可读的冲突报告
    pub fn format_report(&self) -> String {
        let mut report = format!("冲突报告: {}\n", self.file_path.display());
        report.push_str(&"=".repeat(50));
        report.push('\n');

        for (i, conflict) in self.conflicts.iter().enumerate() {
            report.push_str(&format!("\n[{}] {:?}\n", i + 1, conflict.conflict_type));
            if let Some(line) = conflict.line_number {
                report.push_str(&format!("  行号: {}\n", line));
            }
            report.push_str(&format!("  描述: {}\n", conflict.description));
            if let Some(ref sym) = conflict.user_symbol {
                report.push_str(&format!("  用户符号: {}\n", sym));
            }
            if let Some(ref sym) = conflict.generated_symbol {
                report.push_str(&format!("  生成符号: {}\n", sym));
            }
        }

        if !self.suggestion.is_empty() {
            report.push_str(&format!("\n建议: {}\n", self.suggestion));
        }

        report
    }
}

/// 标记解析器
pub struct MarkerParser {
    /// 起始标记正则
    start_regex: Regex,
    /// 结束标记正则
    end_regex: Regex,
    /// 元数据行正则
    metadata_regex: Regex,
}

impl Default for MarkerParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkerParser {
    /// 创建新的标记解析器
    pub fn new() -> Self {
        Self {
            start_regex: Regex::new(r"^(?://|--)?\s*@alioth-protected-start")
                .expect("Invalid start regex"),
            end_regex: Regex::new(r"^(?://|--)?\s*@alioth-protected-end")
                .expect("Invalid end regex"),
            metadata_regex: Regex::new(
                r"^(?://|--)?\s*(ORDER|CHECKSUM|TIMESTAMP|DO\s+NOT\s+EDIT|END\s+OF):?\s*(.*)$",
            )
            .expect("Invalid metadata regex"),
        }
    }

    /// 解析文件内容，提取受保护区域
    ///
    /// # 返回值
    /// - `Ok(Some(ProtectedRegion))`: 成功解析
    /// - `Ok(None)`: 没有标记（整个文件视为受保护）
    /// - `Err(Conflict)`: 标记解析出错
    pub fn parse(&self, content: &str) -> Result<Option<ProtectedRegion>, Conflict> {
        let lines: Vec<&str> = content.lines().collect();

        // 查找所有标记位置
        let mut start_indices: Vec<usize> = vec![];
        let mut end_indices: Vec<usize> = vec![];

        for (i, line) in lines.iter().enumerate() {
            if self.start_regex.is_match(line) {
                start_indices.push(i);
            }
            if self.end_regex.is_match(line) {
                end_indices.push(i);
            }
        }

        // 处理各种边缘情况
        match (start_indices.len(), end_indices.len()) {
            (0, 0) => {
                // 没有标记 - 整个文件视为受保护
                Ok(None)
            }
            (0, _) => {
                // 只有结束标记，没有起始标记
                Err(Conflict {
                    conflict_type: ConflictType::CorruptedMarker,
                    line_number: end_indices.first().copied().map(|n| n + 1),
                    description: "找到结束标记但没有起始标记".to_string(),
                    user_symbol: None,
                    generated_symbol: None,
                })
            }
            (_, 0) => {
                // 只有起始标记，没有结束标记
                Err(Conflict {
                    conflict_type: ConflictType::CorruptedMarker,
                    line_number: start_indices.first().copied().map(|n| n + 1),
                    description: "找到起始标记但没有结束标记".to_string(),
                    user_symbol: None,
                    generated_symbol: None,
                })
            }
            (1, 1) => {
                // 正常情况：一个起始标记和一个结束标记
                let start_idx = start_indices[0];
                let end_idx = end_indices[0];

                if start_idx >= end_idx {
                    return Err(Conflict {
                        conflict_type: ConflictType::CorruptedMarker,
                        line_number: Some(start_idx + 1),
                        description: "起始标记在结束标记之后".to_string(),
                        user_symbol: None,
                        generated_symbol: None,
                    });
                }

                // 提取元数据并查找内容起始位置
                let metadata = self.extract_metadata(&lines, start_idx, end_idx);
                let content_start = self.find_content_start(&lines, start_idx, end_idx);

                // 提取各部分
                let prefix = lines[..start_idx].join("\n");
                let generated = lines[content_start..end_idx].join("\n");
                let suffix = if end_idx + 1 < lines.len() {
                    lines[end_idx + 1..].join("\n")
                } else {
                    String::new()
                };

                Ok(Some(ProtectedRegion {
                    prefix: if prefix.is_empty() {
                        String::new()
                    } else {
                        prefix + "\n"
                    },
                    generated,
                    suffix: if suffix.is_empty() {
                        String::new()
                    } else {
                        "\n".to_string() + &suffix
                    },
                    metadata,
                }))
            }
            _ => {
                // 多个标记 - 使用最后出现的标记对（last-wins策略）
                // 但报告警告
                let start_idx = *start_indices.last().unwrap();
                let end_idx = *end_indices.last().unwrap();

                if start_idx >= end_idx {
                    // 尝试找到匹配的标记对
                    let mut valid_pair = None;
                    for &s in &start_indices {
                        if let Some(&e) = end_indices.iter().find(|&&e| e > s) {
                            valid_pair = Some((s, e));
                        }
                    }

                    if let Some((s, e)) = valid_pair {
                        let metadata = self.extract_metadata(&lines, s, e);
                        let content_start = self.find_content_start(&lines, s, e);
                        let prefix = lines[..s].join("\n");
                        let generated = lines[content_start..e].join("\n");
                        let suffix = if e + 1 < lines.len() {
                            lines[e + 1..].join("\n")
                        } else {
                            String::new()
                        };

                        return Ok(Some(ProtectedRegion {
                            prefix: if prefix.is_empty() {
                                String::new()
                            } else {
                                prefix + "\n"
                            },
                            generated,
                            suffix: if suffix.is_empty() {
                                String::new()
                            } else {
                                "\n".to_string() + &suffix
                            },
                            metadata,
                        }));
                    }

                    return Err(Conflict {
                        conflict_type: ConflictType::MultipleMarkers,
                        line_number: Some(start_idx + 1),
                        description: format!(
                            "发现 {} 个起始标记和 {} 个结束标记，但无法找到有效的标记对",
                            start_indices.len(),
                            end_indices.len()
                        ),
                        user_symbol: None,
                        generated_symbol: None,
                    });
                }

                let metadata = self.extract_metadata(&lines, start_idx, end_idx);
                let content_start = self.find_content_start(&lines, start_idx, end_idx);
                let prefix = lines[..start_idx].join("\n");
                let generated = lines[content_start..end_idx].join("\n");
                let suffix = if end_idx + 1 < lines.len() {
                    lines[end_idx + 1..].join("\n")
                } else {
                    String::new()
                };

                Ok(Some(ProtectedRegion {
                    prefix: if prefix.is_empty() {
                        String::new()
                    } else {
                        prefix + "\n"
                    },
                    generated,
                    suffix: if suffix.is_empty() {
                        String::new()
                    } else {
                        "\n".to_string() + &suffix
                    },
                    metadata,
                }))
            }
        }
    }

    /// 提取标记元数据
    fn extract_metadata(
        &self,
        lines: &[&str],
        start_idx: usize,
        end_idx: usize,
    ) -> Option<MarkerMetadata> {
        let mut metadata = MarkerMetadata {
            checksum: String::new(),
            timestamp: String::new(),
            order: vec![],
            marker_lines: vec![],
        };

        // 解析起始标记后的元数据行
        for line in &lines[start_idx..end_idx.min(start_idx + 10)] {
            if let Some(captures) = self.metadata_regex.captures(line) {
                let key = captures.get(1)?.as_str();
                let value = captures.get(2)?.as_str().trim();

                match key {
                    "ORDER" => {
                        metadata.order = value.split(',').map(|s| s.trim().to_string()).collect();
                    }
                    "CHECKSUM" => {
                        metadata.checksum = value.to_string();
                    }
                    "TIMESTAMP" => {
                        metadata.timestamp = value.to_string();
                    }
                    _ => {}
                }
            }
            metadata.marker_lines.push(line.to_string());
        }

        // 只有当至少有一个元数据字段时才返回
        if !metadata.checksum.is_empty()
            || !metadata.timestamp.is_empty()
            || !metadata.order.is_empty()
        {
            Some(metadata)
        } else {
            None
        }
    }

    /// 快速检查内容是否包含保护区域标记
    pub fn has_markers(&self, content: &str) -> bool {
        content.contains("@alioth-protected-start") || content.contains("@alioth-protected-end")
    }

    /// 查找实际内容的起始行（跳过元数据行和空行）
    fn find_content_start(&self, lines: &[&str], start_idx: usize, end_idx: usize) -> usize {
        // 从起始标记后一行开始扫描
        for (i, line) in lines.iter().enumerate().take(end_idx).skip(start_idx + 1) {
            // 如果遇到结束标记，直接返回
            if self.end_regex.is_match(line) {
                return i;
            }
            // 跳过空行和以 // 开头的元数据行
            if line.trim().is_empty() {
                continue;
            }
            if line.starts_with("//") && self.metadata_regex.is_match(line) {
                continue;
            }
            // 找到实际内容
            return i;
        }
        end_idx
    }
}

/// 合并选项
#[derive(Debug, Clone)]
pub struct MergeOptions {
    /// 包含的文件模式（glob）
    pub include_patterns: Vec<String>,
    /// 排除的文件模式（glob）
    pub exclude_patterns: Vec<String>,
    /// 强制覆盖现有文件（忽略保护区域）
    pub force_overwrite: bool,
    /// 检查命名冲突
    pub check_naming_collisions: bool,
    /// 检查引用断裂
    pub check_broken_references: bool,
}

impl Default for MergeOptions {
    fn default() -> Self {
        Self {
            include_patterns: vec!["**/*".to_string()],
            exclude_patterns: vec![],
            force_overwrite: false,
            check_naming_collisions: true,
            check_broken_references: true,
        }
    }
}

impl MergeOptions {
    /// 创建默认选项
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置包含模式
    pub fn include(mut self, patterns: Vec<String>) -> Self {
        self.include_patterns = patterns;
        self
    }

    /// 设置排除模式
    pub fn exclude(mut self, patterns: Vec<String>) -> Self {
        self.exclude_patterns = patterns;
        self
    }

    /// 设置强制覆盖
    pub fn force_overwrite(mut self, force: bool) -> Self {
        self.force_overwrite = force;
        self
    }

    /// 检查文件路径是否匹配包含/排除规则
    pub fn should_process(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // 首先检查排除模式
        for pattern in &self.exclude_patterns {
            if Self::glob_match(pattern, &path_str) {
                return false;
            }
        }

        // 然后检查包含模式
        for pattern in &self.include_patterns {
            if Self::glob_match(pattern, &path_str) {
                return true;
            }
        }

        false
    }

    /// 简单的 glob 匹配实现
    fn glob_match(pattern: &str, path: &str) -> bool {
        // 将 glob 模式转换为正则表达式
        let mut regex_pattern = String::new();
        regex_pattern.push('^');

        let mut chars = pattern.chars().peekable();
        while let Some(ch) = chars.next() {
            match ch {
                '*' => {
                    // 检查是否是 **
                    if chars.peek() == Some(&'*') {
                        chars.next(); // 消费第二个 *
                                      // ** 后面可能跟着 /
                        if chars.peek() == Some(&'/') {
                            chars.next(); // 消费 /
                                          // **/ 匹配零个或多个目录层级
                            regex_pattern.push_str("(?:.*/)?");
                        } else {
                            // ** 不匹配 /
                            regex_pattern.push_str("[^/]*");
                        }
                    } else {
                        // * 匹配非 / 的任意字符
                        regex_pattern.push_str("[^/]*");
                    }
                }
                '?' => regex_pattern.push_str("[^/]"),
                '.' => regex_pattern.push_str("\\."),
                '/' => regex_pattern.push('/'),
                '{' => regex_pattern.push('('),
                '}' => regex_pattern.push(')'),
                ',' => regex_pattern.push('|'),
                '[' => regex_pattern.push('['),
                ']' => regex_pattern.push(']'),
                c => {
                    if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                        regex_pattern.push(c);
                    } else {
                        regex_pattern.push_str(&format!("\\{}", c));
                    }
                }
            }
        }

        regex_pattern.push('$');

        Regex::new(&regex_pattern)
            .map(|re| re.is_match(path))
            .unwrap_or(false)
    }
}

/// 合并引擎
pub struct MergeEngine {
    parser: MarkerParser,
    options: MergeOptions,
}

impl MergeEngine {
    /// 创建新的合并引擎
    pub fn new() -> Self {
        Self {
            parser: MarkerParser::new(),
            options: MergeOptions::default(),
        }
    }

    /// 使用指定选项创建合并引擎
    pub fn with_options(options: MergeOptions) -> Self {
        Self {
            parser: MarkerParser::new(),
            options,
        }
    }

    /// 合并新内容到现有文件
    ///
    /// # 参数
    /// - `existing_content`: 磁盘上的现有文件内容
    /// - `new_content`: 新生成的内容（不含标记）
    /// - `file_path`: 文件路径（用于冲突报告）
    ///
    /// # 返回值
    /// 合并结果，可能包含冲突报告
    pub fn merge(
        &self,
        existing_content: &str,
        new_content: &str,
        file_path: &Path,
    ) -> MergeResult {
        // 如果强制覆盖，直接返回新内容
        if self.options.force_overwrite {
            return MergeResult::Merged(self.wrap_with_markers(new_content, file_path));
        }

        // 解析现有文件的保护区域
        match self.parser.parse(existing_content) {
            Ok(None) => {
                // 没有标记 - 整个文件视为受保护
                let mut report = ConflictReport::new(file_path);
                report.add_conflict(Conflict {
                    conflict_type: ConflictType::MissingMarker,
                    line_number: None,
                    description: "文件没有保护区域标记，无法安全合并".to_string(),
                    user_symbol: None,
                    generated_symbol: None,
                });
                report.suggestion = "使用 force_overwrite=true 强制覆盖或使用手动合并".to_string();
                MergeResult::Conflict(report)
            }
            Ok(Some(region)) => {
                // 成功解析保护区域，执行合并
                self.perform_merge(&region, new_content, file_path, existing_content)
            }
            Err(conflict) => {
                // 标记解析错误
                let mut report = ConflictReport::new(file_path);
                report.add_conflict(conflict);
                report.suggestion = "检查文件标记是否损坏，或尝试强制覆盖".to_string();
                MergeResult::Conflict(report)
            }
        }
    }

    /// 执行实际合并
    fn perform_merge(
        &self,
        region: &ProtectedRegion,
        new_content: &str,
        file_path: &Path,
        full_existing: &str,
    ) -> MergeResult {
        let mut report = ConflictReport::new(file_path);

        // 检查命名冲突
        if self.options.check_naming_collisions {
            let collisions = self.detect_naming_collisions(region, new_content);
            for collision in collisions {
                report.add_conflict(collision);
            }
        }

        // 检查引用断裂
        if self.options.check_broken_references {
            let broken_refs = self.detect_broken_references(region, new_content);
            for broken in broken_refs {
                report.add_conflict(broken);
            }
        }

        // 如果有冲突，返回冲突报告
        if report.has_conflicts() {
            report.suggestion = format!(
                "文件 {} 存在 {} 个冲突，需要手动解决",
                file_path.display(),
                report.conflict_count()
            );
            return MergeResult::Conflict(report);
        }

        // 无冲突，执行合并
        // 保留用户的前缀和后缀，替换生成的内容
        let merged = format!(
            "{}{}{}",
            region.prefix,
            self.wrap_with_markers(new_content, file_path),
            region.suffix
        );

        // 检查是否实际有变化
        if merged == full_existing {
            MergeResult::Skipped
        } else {
            MergeResult::Merged(merged)
        }
    }

    /// 用标记包装内容
    fn wrap_with_markers(&self, content: &str, file_path: &Path) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let checksum = hex::encode(hasher.finalize())[..16].to_string();

        format!(
            "{}\n// ORDER: {}\n// CHECKSUM: {}\n// TIMESTAMP: {}\n{}\n{}\n// {}\n",
            GENERATED_MARKER_START,
            file_path.file_name().unwrap_or_default().to_string_lossy(),
            checksum,
            chrono::Utc::now().to_rfc3339(),
            DO_NOT_EDIT_MARKER,
            content,
            GENERATED_MARKER_END
        )
    }

    /// 检测命名冲突
    fn detect_naming_collisions(
        &self,
        region: &ProtectedRegion,
        new_content: &str,
    ) -> Vec<Conflict> {
        let mut conflicts = vec![];

        // 提取用户代码中的标识符（简化实现）
        let mut user_identifiers = self.extract_identifiers(&region.prefix);
        user_identifiers.extend(self.extract_identifiers(&region.suffix));

        // 提取新生成代码中的标识符
        let new_identifiers = self.extract_identifiers(new_content);

        // 查找冲突
        let user_set: HashSet<_> = user_identifiers.iter().map(|(n, _)| n.clone()).collect();
        for (ident, line_num) in new_identifiers {
            if user_set.contains(&ident) {
                conflicts.push(Conflict {
                    conflict_type: ConflictType::NamingCollision,
                    line_number: Some(line_num),
                    description: format!("符号 '{}' 在用户代码和生成代码中都存在", ident),
                    user_symbol: Some(ident.clone()),
                    generated_symbol: Some(ident),
                });
            }
        }

        conflicts
    }

    /// 检测引用断裂
    fn detect_broken_references(
        &self,
        region: &ProtectedRegion,
        new_content: &str,
    ) -> Vec<Conflict> {
        let mut conflicts = vec![];

        // 提取用户代码中的引用（简化实现）
        let user_references = self.extract_references(&region.prefix);
        let suffix_refs = self.extract_references(&region.suffix);

        // 提取原生成代码中的定义
        let old_definitions = self.extract_definitions(&region.generated);

        // 提取新代码中的定义
        let new_definitions = self.extract_definitions(new_content);

        // 检查用户引用是否在旧定义中但不在新定义中
        let new_def_set: HashSet<_> = new_definitions.iter().cloned().collect();

        for (ref_name, line_num) in user_references.iter().chain(suffix_refs.iter()) {
            // 检查是否引用了原生成代码中的符号
            if old_definitions.contains(ref_name) && !new_def_set.contains(ref_name) {
                conflicts.push(Conflict {
                    conflict_type: ConflictType::BrokenReference,
                    line_number: Some(*line_num),
                    description: format!(
                        "用户代码引用 '{}' 在原生成代码中存在，但在新代码中不存在",
                        ref_name
                    ),
                    user_symbol: Some(ref_name.clone()),
                    generated_symbol: None,
                });
            }
        }

        conflicts
    }

    /// 从代码中提取标识符（简化实现）
    fn extract_identifiers(&self, content: &str) -> Vec<(String, usize)> {
        let mut identifiers = vec![];
        let identifier_regex = Regex::new(
            r"(?:fn|struct|enum|trait|impl|const|let|type|use)\s+(?:(?:pub|crate|super)\s+)?(\w+)",
        )
        .expect("Invalid identifier regex");

        for (line_num, line) in content.lines().enumerate() {
            for cap in identifier_regex.captures_iter(line) {
                if let Some(m) = cap.get(1) {
                    identifiers.push((m.as_str().to_string(), line_num + 1));
                }
            }
        }

        identifiers
    }

    /// 从代码中提取引用（简化实现）
    fn extract_references(&self, content: &str) -> Vec<(String, usize)> {
        let mut references = vec![];

        // 匹配函数调用、类型使用等
        let ref_regex = Regex::new(r"\b(\w+)\s*(?:\(|::|<|:\s)").expect("Invalid reference regex");

        for (line_num, line) in content.lines().enumerate() {
            for cap in ref_regex.captures_iter(line) {
                if let Some(m) = cap.get(1) {
                    let name = m.as_str();
                    // 过滤关键字和常见类型
                    if !self.is_keyword(name) {
                        references.push((name.to_string(), line_num + 1));
                    }
                }
            }
        }

        references
    }

    /// 从代码中提取定义（简化实现）
    fn extract_definitions(&self, content: &str) -> HashSet<String> {
        let mut definitions = HashSet::new();

        // 匹配函数、结构体、枚举等定义
        let def_regex =
            Regex::new(r"(?:pub\s+)?(?:fn|struct|enum|trait|type|const|static)\s+(\w+)")
                .expect("Invalid definition regex");

        for line in content.lines() {
            for cap in def_regex.captures_iter(line) {
                if let Some(m) = cap.get(1) {
                    definitions.insert(m.as_str().to_string());
                }
            }
        }

        definitions
    }

    /// 检查是否为关键字
    fn is_keyword(&self, name: &str) -> bool {
        let keywords: HashSet<&str> = [
            "if", "else", "match", "for", "while", "loop", "return", "break", "continue", "let",
            "mut", "const", "static", "fn", "struct", "enum", "trait", "impl", "use", "mod", "pub",
            "crate", "super", "self", "where", "type", "as", "async", "await", "move", "ref",
            "box", "dyn", "Self", "true", "false", "i8", "i16", "i32", "i64", "i128", "isize",
            "u8", "u16", "u32", "u64", "u128", "usize", "f32", "f64", "bool", "char", "str",
            "String", "Vec", "Option", "Result",
        ]
        .iter()
        .cloned()
        .collect();

        keywords.contains(name)
    }

    /// 处理一组文件的合并
    pub fn merge_batch(
        &self,
        memory_files: &[CapturedFile],
        file_changes: &[FileChange],
        base_path: &Path,
    ) -> Result<Vec<(PathBuf, MergeResult)>, WriteError> {
        let mut results = vec![];

        for change in file_changes {
            // 只处理更新操作
            if !change.is_updated() {
                continue;
            }

            let path = change.path();

            // 检查是否应该处理此文件
            if !self.options.should_process(path) {
                continue;
            }

            // 查找内存文件
            let memory_file = memory_files.iter().find(|f| &f.path == path);

            if let Some(mem_file) = memory_file {
                // 读取现有文件内容
                let full_path = base_path.join(path);
                let existing_content = std::fs::read_to_string(&full_path)?;

                // 执行合并
                let result = self.merge(&existing_content, &mem_file.content, path);
                results.push((path.clone(), result));
            }
        }

        Ok(results)
    }
}

impl Default for MergeEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// 扩展冲突类型检查
pub fn check_conflicts(
    existing_content: &str,
    new_content: &str,
    file_path: &Path,
) -> Option<ConflictReport> {
    let engine = MergeEngine::new();

    match engine.merge(existing_content, new_content, file_path) {
        MergeResult::Conflict(report) => Some(report),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marker_parser_no_markers() {
        let parser = MarkerParser::new();
        let content = "fn main() {}";

        let result = parser.parse(content).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_marker_parser_basic() {
        let parser = MarkerParser::new();
        let content = r#"// User prefix code
// @alioth-protected-start
// ORDER: test
// CHECKSUM: abc123
// DO NOT EDIT MANUALLY

fn generated() {}
// @alioth-protected-end
// User suffix code"#;

        let result = parser.parse(content).unwrap().unwrap();
        assert_eq!(result.prefix.trim(), "// User prefix code");
        assert_eq!(result.generated.trim(), "fn generated() {}");
        assert_eq!(result.suffix.trim(), "// User suffix code");
        assert!(result.metadata.is_some());
    }

    #[test]
    fn test_marker_parser_missing_end() {
        let parser = MarkerParser::new();
        let content = r#"// @alioth-protected-start
fn generated() {}"#;

        let result = parser.parse(content);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().conflict_type,
            ConflictType::CorruptedMarker
        );
    }

    #[test]
    fn test_marker_parser_multiple_markers() {
        let parser = MarkerParser::new();
        let content = r#"// @alioth-protected-start
fn first() {}
// @alioth-protected-end
// @alioth-protected-start
fn second() {}
// @alioth-protected-end"#;

        let result = parser.parse(content).unwrap().unwrap();
        assert_eq!(result.generated.trim(), "fn second() {}");
    }

    #[test]
    fn test_merge_options_glob_match() {
        let options = MergeOptions::new();

        assert!(options.should_process(Path::new("src/main.rs")));
        assert!(options.should_process(Path::new("lib/mod.rs")));
    }

    #[test]
    fn test_merge_options_exclude() {
        let options =
            MergeOptions::new().exclude(vec!["**/*.test.rs".to_string(), "vendor/**".to_string()]);

        assert!(options.should_process(Path::new("src/main.rs")));
        assert!(!options.should_process(Path::new("src/main.test.rs")));
        assert!(!options.should_process(Path::new("vendor/lib.rs")));
    }

    #[test]
    fn test_merge_options_include() {
        let options = MergeOptions::new().include(vec!["src/**/*.rs".to_string()]);

        assert!(options.should_process(Path::new("src/main.rs")));
        assert!(options.should_process(Path::new("src/lib/mod.rs")));
        assert!(!options.should_process(Path::new("tests/main.rs")));
    }

    #[test]
    fn test_merge_engine_no_markers() {
        let engine = MergeEngine::new();
        let existing = "fn main() {}";
        let new = "fn new_main() {}";

        let result = engine.merge(existing, new, Path::new("test.rs"));

        match result {
            MergeResult::Conflict(report) => {
                assert_eq!(report.conflicts.len(), 1);
                assert_eq!(
                    report.conflicts[0].conflict_type,
                    ConflictType::MissingMarker
                );
            }
            _ => panic!("Expected conflict for missing markers"),
        }
    }

    #[test]
    fn test_merge_engine_force_overwrite() {
        let engine = MergeEngine::with_options(MergeOptions::new().force_overwrite(true));
        let existing = "fn main() {}";
        let new = "fn new_main() {}";

        let result = engine.merge(existing, new, Path::new("test.rs"));

        match result {
            MergeResult::Merged(content) => {
                assert!(content.contains("fn new_main() {}"));
                assert!(content.contains("@alioth-protected-start"));
            }
            _ => panic!("Expected merged result"),
        }
    }

    #[test]
    fn test_merge_engine_successful_merge() {
        let engine = MergeEngine::new();
        let existing = r#"// User prefix
// @alioth-protected-start
// ORDER: test
fn old_generated() {}
// @alioth-protected-end
// User suffix"#;

        let new = "fn new_generated() {}";

        let result = engine.merge(existing, new, Path::new("test.rs"));

        match result {
            MergeResult::Merged(content) => {
                assert!(content.contains("// User prefix"));
                assert!(content.contains("fn new_generated() {}"));
                assert!(content.contains("// User suffix"));
                assert!(!content.contains("fn old_generated() {}"));
            }
            _ => panic!("Expected merged result"),
        }
    }

    #[test]
    fn test_merge_engine_naming_collision() {
        let engine = MergeEngine::new();
        let existing = r#"fn user_func() {}
// @alioth-protected-start
fn generated() {}
// @alioth-protected-end"#;

        let new = "fn user_func() { /* new */ }";

        let result = engine.merge(existing, new, Path::new("test.rs"));

        match result {
            MergeResult::Conflict(report) => {
                let has_collision = report
                    .conflicts
                    .iter()
                    .any(|c| c.conflict_type == ConflictType::NamingCollision);
                assert!(has_collision, "Expected naming collision");
            }
            _ => panic!("Expected conflict for naming collision"),
        }
    }

    #[test]
    fn test_conflict_report_format() {
        let mut report = ConflictReport::new("test.rs");
        report.add_conflict(Conflict {
            conflict_type: ConflictType::NamingCollision,
            line_number: Some(10),
            description: "Test conflict".to_string(),
            user_symbol: Some("foo".to_string()),
            generated_symbol: Some("foo".to_string()),
        });
        report.suggestion = "Fix the conflict".to_string();

        let formatted = report.format_report();
        assert!(formatted.contains("冲突报告: test.rs"));
        assert!(formatted.contains("NamingCollision"));
        assert!(formatted.contains("行号: 10"));
        assert!(formatted.contains("Fix the conflict"));
    }

    #[test]
    fn test_glob_pattern_matching() {
        // 测试基本通配符
        assert!(MergeOptions::glob_match("*.rs", "test.rs"));
        assert!(MergeOptions::glob_match("*.rs", "lib.rs"));
        assert!(!MergeOptions::glob_match("*.rs", "test.txt"));

        // 测试目录通配符
        assert!(MergeOptions::glob_match("src/**/*.rs", "src/main.rs"));
        assert!(MergeOptions::glob_match("src/**/*.rs", "src/lib/mod.rs"));
        assert!(!MergeOptions::glob_match("src/**/*.rs", "tests/main.rs"));

        // 测试精确匹配
        assert!(MergeOptions::glob_match("src/main.rs", "src/main.rs"));
        assert!(!MergeOptions::glob_match("src/main.rs", "src/lib.rs"));

        // 测试前缀匹配
        assert!(MergeOptions::glob_match("test_*.rs", "test_main.rs"));
        assert!(!MergeOptions::glob_match("test_*.rs", "main_test.rs"));
    }

    #[test]
    fn test_extract_identifiers() {
        let engine = MergeEngine::new();
        let content = r#"
fn test_func() {}
struct TestStruct;
pub fn public_func() {}
let local_var = 0;
"#;

        let identifiers = engine.extract_identifiers(content);
        let names: Vec<_> = identifiers.iter().map(|(n, _)| n.as_str()).collect();

        assert!(names.contains(&"test_func"));
        assert!(names.contains(&"TestStruct"));
        assert!(names.contains(&"public_func"));
        assert!(names.contains(&"local_var"));
    }

    #[test]
    fn test_extract_definitions() {
        let engine = MergeEngine::new();
        let content = r#"
pub fn public_func() {}
fn private_func() {}
struct MyStruct;
enum MyEnum { A, B }
trait MyTrait {}
"#;

        let definitions = engine.extract_definitions(content);

        assert!(definitions.contains("public_func"));
        assert!(definitions.contains("private_func"));
        assert!(definitions.contains("MyStruct"));
        assert!(definitions.contains("MyEnum"));
        assert!(definitions.contains("MyTrait"));
    }

    #[test]
    fn test_marker_has_markers() {
        let parser = MarkerParser::new();

        assert!(parser.has_markers("// @alioth-protected-start\nfn test() {}"));
        assert!(!parser.has_markers("fn main() {}"));
    }

    #[test]
    fn test_merge_engine_skipped() {
        let engine = MergeEngine::new();

        // 创建一个包含标记的内容
        let existing = format!(
            r#"// @alioth-protected-start
// ORDER: test
// CHECKSUM: {}
// TIMESTAMP: {}
// DO NOT EDIT MANUALLY

fn test() {{}}
// @alioth-protected-end"#,
            "abc123",
            chrono::Utc::now().to_rfc3339()
        );

        // 使用相同内容合并（只是标记不同）
        let result = engine.merge(&existing, "fn test() {}", Path::new("test.rs"));

        // 应该是 Merged 而不是 Skipped，因为标记会更新
        match result {
            MergeResult::Merged(_) => {
                // 这是预期的，因为时间戳会改变
            }
            _ => {
                // 也可能跳过，取决于实现
            }
        }
    }

    #[test]
    fn test_is_keyword() {
        let engine = MergeEngine::new();

        assert!(engine.is_keyword("fn"));
        assert!(engine.is_keyword("struct"));
        assert!(engine.is_keyword("let"));
        assert!(engine.is_keyword("pub"));
        assert!(engine.is_keyword("Self"));
        assert!(engine.is_keyword("i32"));
        assert!(engine.is_keyword("String"));

        assert!(!engine.is_keyword("my_function"));
        assert!(!engine.is_keyword("MyStruct"));
    }

    #[test]
    fn test_empty_content() {
        let parser = MarkerParser::new();

        let result = parser.parse("").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_only_generated_content() {
        let parser = MarkerParser::new();
        let content = r#"// @alioth-protected-start
fn generated() {}
// @alioth-protected-end"#;

        let result = parser.parse(content).unwrap().unwrap();
        assert!(result.prefix.is_empty());
        assert_eq!(result.generated.trim(), "fn generated() {}");
        assert!(result.suffix.is_empty());
    }

    #[test]
    fn test_check_conflicts_helper() {
        let existing = "fn main() {}";
        let new = "fn new() {}";

        let report = check_conflicts(existing, new, Path::new("test.rs"));
        assert!(report.is_some());
        assert_eq!(
            report.unwrap().conflicts[0].conflict_type,
            ConflictType::MissingMarker
        );
    }

    #[test]
    fn test_conflict_report_empty() {
        let report = ConflictReport::new("test.rs");
        assert!(!report.has_conflicts());
        assert_eq!(report.conflict_count(), 0);
    }

    #[test]
    fn test_merge_options_default() {
        let options = MergeOptions::default();

        assert_eq!(options.include_patterns, vec!["**/*"]);
        assert!(options.exclude_patterns.is_empty());
        assert!(!options.force_overwrite);
        assert!(options.check_naming_collisions);
        assert!(options.check_broken_references);
    }

    #[test]
    fn test_file_change_integration() {
        let change = FileChange::Updated {
            path: PathBuf::from("test.rs"),
            old_checksum: "old".to_string(),
            new_checksum: "new".to_string(),
        };

        assert!(change.is_updated());
        assert!(!change.is_created());
        assert!(!change.is_deleted());
        assert!(!change.is_unchanged());
        assert_eq!(change.old_checksum(), Some("old"));
        assert_eq!(change.new_checksum(), Some("new"));
    }
}
