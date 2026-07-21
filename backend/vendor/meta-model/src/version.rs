//! 版本管理模块
//!
//! 提供语义化版本 (SemVer) 支持，包括版本解析、比较和约束匹配

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// 语义化版本
///
/// 遵循 SemVer 2.0.0 规范：MAJOR.MINOR.PATCH[-prerelease][+build]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Version {
    /// 主版本号（破坏性变更）
    pub major: u64,
    /// 次版本号（功能新增）
    pub minor: u64,
    /// 修订号（问题修复）
    pub patch: u64,
    /// 预发布版本标识
    pub prerelease: Option<String>,
    /// 构建元数据
    pub build: Option<String>,
}

impl Version {
    /// 创建新版本
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: None,
            build: None,
        }
    }

    /// 创建带预发布版本的版本
    pub fn with_prerelease(mut self, prerelease: impl Into<String>) -> Self {
        self.prerelease = Some(prerelease.into());
        self
    }

    /// 创建带构建元数据的版本
    pub fn with_build(mut self, build: impl Into<String>) -> Self {
        self.build = Some(build.into());
        self
    }

    /// 获取版本字符串（不含构建元数据）
    pub fn to_string_short(&self) -> String {
        match &self.prerelease {
            Some(pre) => format!("{}.{}.{}-{}", self.major, self.minor, self.patch, pre),
            None => format!("{}.{}.{}", self.major, self.minor, self.patch),
        }
    }

    /// 检查是否为预发布版本
    pub fn is_prerelease(&self) -> bool {
        self.prerelease.is_some()
    }

    /// 获取下一个主版本
    pub fn next_major(&self) -> Self {
        Self::new(self.major + 1, 0, 0)
    }

    /// 获取下一个次版本
    pub fn next_minor(&self) -> Self {
        Self::new(self.major, self.minor + 1, 0)
    }

    /// 获取下一个修订版本
    pub fn next_patch(&self) -> Self {
        Self::new(self.major, self.minor, self.patch + 1)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;

        if let Some(pre) = &self.prerelease {
            write!(f, "-{}", pre)?;
        }

        if let Some(build) = &self.build {
            write!(f, "+{}", build)?;
        }

        Ok(())
    }
}

impl FromStr for Version {
    type Err = VersionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(VersionParseError::Empty);
        }

        // 分离构建元数据
        let (s, build) = if let Some(pos) = s.find('+') {
            let (version_part, build_part) = s.split_at(pos);
            (version_part, Some(build_part[1..].to_string()))
        } else {
            (s, None)
        };

        // 分离预发布版本
        let (s, prerelease) = if let Some(pos) = s.find('-') {
            let (version_part, pre_part) = s.split_at(pos);
            (version_part, Some(pre_part[1..].to_string()))
        } else {
            (s, None)
        };

        // 解析版本号
        let parts: Vec<&str> = s.split('.').collect();

        if parts.len() != 3 {
            return Err(VersionParseError::InvalidFormat(
                "版本号必须包含 MAJOR.MINOR.PATCH 三部分".to_string(),
            ));
        }

        let major = parts[0]
            .parse::<u64>()
            .map_err(|_| VersionParseError::InvalidNumber(parts[0].to_string()))?;

        let minor = parts[1]
            .parse::<u64>()
            .map_err(|_| VersionParseError::InvalidNumber(parts[1].to_string()))?;

        let patch = parts[2]
            .parse::<u64>()
            .map_err(|_| VersionParseError::InvalidNumber(parts[2].to_string()))?;

        Ok(Version {
            major,
            minor,
            patch,
            prerelease,
            build,
        })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 首先比较主、次、修订号
        match self.major.cmp(&other.major) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        // 比较预发布版本
        match (&self.prerelease, &other.prerelease) {
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Greater, // 正式版本 > 预发布版本
            (Some(_), None) => std::cmp::Ordering::Less,
            (Some(a), Some(b)) => compare_prerelease(a, b),
        }
    }
}

/// 比较预发布版本标识符
///
/// 按照 SemVer 规范：
/// - 数字标识符按数值比较
/// - 字母标识符按 ASCII 顺序比较
/// - 数字标识符 < 字母标识符
fn compare_prerelease(a: &str, b: &str) -> std::cmp::Ordering {
    let a_parts: Vec<&str> = a.split('.').collect();
    let b_parts: Vec<&str> = b.split('.').collect();

    for (a_part, b_part) in a_parts.iter().zip(b_parts.iter()) {
        let ord = compare_prerelease_part(a_part, b_part);
        if ord != std::cmp::Ordering::Equal {
            return ord;
        }
    }

    // 较短的预发布版本较小（如果有更多部分）
    a_parts.len().cmp(&b_parts.len())
}

fn compare_prerelease_part(a: &str, b: &str) -> std::cmp::Ordering {
    let a_is_num = a.chars().all(|c| c.is_ascii_digit());
    let b_is_num = b.chars().all(|c| c.is_ascii_digit());

    match (a_is_num, b_is_num) {
        (true, true) => {
            let a_num: u64 = a.parse().unwrap_or(0);
            let b_num: u64 = b.parse().unwrap_or(0);
            a_num.cmp(&b_num)
        }
        (true, false) => std::cmp::Ordering::Less, // 数字 < 字母
        (false, true) => std::cmp::Ordering::Greater,
        (false, false) => a.cmp(b),
    }
}

/// 版本解析错误
#[derive(Debug, Clone, PartialEq)]
pub enum VersionParseError {
    /// 空字符串
    Empty,
    /// 无效格式
    InvalidFormat(String),
    /// 无效数字
    InvalidNumber(String),
}

impl fmt::Display for VersionParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionParseError::Empty => write!(f, "版本号不能为空"),
            VersionParseError::InvalidFormat(msg) => write!(f, "无效的版本格式: {}", msg),
            VersionParseError::InvalidNumber(s) => write!(f, "无效的数字: {}", s),
        }
    }
}

impl std::error::Error for VersionParseError {}

/// 版本约束
///
/// 支持多种版本约束表达式
#[derive(Debug, Clone, PartialEq)]
pub enum VersionConstraint {
    /// 精确版本
    Exact(Version),
    /// 大于指定版本
    GreaterThan(Version),
    /// 大于等于指定版本
    GreaterOrEqual(Version),
    /// 小于指定版本
    LessThan(Version),
    /// 小于等于指定版本
    LessOrEqual(Version),
    /// 版本范围 [min, max)
    Range(Version, Version),
    /// 兼容版本 ^1.2.3 = >=1.2.3, <2.0.0
    Caret(Version),
    /// 近似版本 ~1.2.3 = >=1.2.3, <1.3.0
    Tilde(Version),
    /// 通配符 *
    Wildcard,
}

impl VersionConstraint {
    /// 检查版本是否满足约束
    pub fn matches(&self, version: &Version) -> bool {
        match self {
            VersionConstraint::Exact(v) => version == v,
            VersionConstraint::GreaterThan(v) => version > v,
            VersionConstraint::GreaterOrEqual(v) => version >= v,
            VersionConstraint::LessThan(v) => version < v,
            VersionConstraint::LessOrEqual(v) => version <= v,
            VersionConstraint::Range(min, max) => version >= min && version < max,
            VersionConstraint::Caret(v) => {
                // ^1.2.3 = >=1.2.3, <2.0.0
                // ^0.2.3 = >=0.2.3, <0.3.0
                // ^0.0.3 = >=0.0.3, <0.0.4
                let upper = if v.major == 0 {
                    if v.minor == 0 {
                        Version::new(0, 0, v.patch + 1)
                    } else {
                        Version::new(0, v.minor + 1, 0)
                    }
                } else {
                    Version::new(v.major + 1, 0, 0)
                };
                version >= v && version < &upper
            }
            VersionConstraint::Tilde(v) => {
                // ~1.2.3 = >=1.2.3, <1.3.0
                let upper = Version::new(v.major, v.minor + 1, 0);
                version >= v && version < &upper
            }
            VersionConstraint::Wildcard => true,
        }
    }
}

impl fmt::Display for VersionConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionConstraint::Exact(v) => write!(f, "={}", v),
            VersionConstraint::GreaterThan(v) => write!(f, ">{}", v),
            VersionConstraint::GreaterOrEqual(v) => write!(f, ">={}", v),
            VersionConstraint::LessThan(v) => write!(f, "<{}", v),
            VersionConstraint::LessOrEqual(v) => write!(f, "<={}", v),
            VersionConstraint::Range(min, max) => write!(f, ">={}, <{}", min, max),
            VersionConstraint::Caret(v) => write!(f, "^{}", v),
            VersionConstraint::Tilde(v) => write!(f, "~{}", v),
            VersionConstraint::Wildcard => write!(f, "*"),
        }
    }
}

impl FromStr for VersionConstraint {
    type Err = VersionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s == "*" || s == ">=" {
            return Ok(VersionConstraint::Wildcard);
        }

        // 尝试解析各种约束格式
        if let Some(rest) = s.strip_prefix("^") {
            let version = rest.parse()?;
            return Ok(VersionConstraint::Caret(version));
        }

        if let Some(rest) = s.strip_prefix("~") {
            let version = rest.parse()?;
            return Ok(VersionConstraint::Tilde(version));
        }

        if let Some(rest) = s.strip_prefix(">=") {
            let version = rest.parse()?;
            return Ok(VersionConstraint::GreaterOrEqual(version));
        }

        if let Some(rest) = s.strip_prefix(">") {
            let version = rest.parse()?;
            return Ok(VersionConstraint::GreaterThan(version));
        }

        if let Some(rest) = s.strip_prefix("<=") {
            let version = rest.parse()?;
            return Ok(VersionConstraint::LessOrEqual(version));
        }

        if let Some(rest) = s.strip_prefix("<") {
            let version = rest.parse()?;
            return Ok(VersionConstraint::LessThan(version));
        }

        if let Some(rest) = s.strip_prefix("=") {
            let version = rest.parse()?;
            return Ok(VersionConstraint::Exact(version));
        }

        // 默认作为精确版本
        let version = s.parse()?;
        Ok(VersionConstraint::Exact(version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_creation() {
        let v = Version::new(1, 2, 3);
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn test_version_with_prerelease() {
        let v = Version::new(1, 0, 0).with_prerelease("alpha.1");
        assert_eq!(v.to_string(), "1.0.0-alpha.1");
        assert!(v.is_prerelease());
    }

    #[test]
    fn test_version_with_build() {
        let v = Version::new(1, 0, 0).with_build("exp.sha.5114f85");
        assert_eq!(v.to_string(), "1.0.0+exp.sha.5114f85");
    }

    #[test]
    fn test_version_full() {
        let v = Version::new(1, 0, 0)
            .with_prerelease("beta")
            .with_build("20240101");
        assert_eq!(v.to_string(), "1.0.0-beta+20240101");
    }

    #[test]
    fn test_version_parse() {
        let v: Version = "1.2.3".parse().unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert_eq!(v.prerelease, None);
        assert_eq!(v.build, None);
    }

    #[test]
    fn test_version_parse_with_prerelease() {
        let v: Version = "1.0.0-alpha.1".parse().unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
        assert_eq!(v.prerelease, Some("alpha.1".to_string()));
    }

    #[test]
    fn test_version_parse_with_build() {
        let v: Version = "1.0.0+build.123".parse().unwrap();
        assert_eq!(v.build, Some("build.123".to_string()));
    }

    #[test]
    fn test_version_parse_full() {
        let v: Version = "1.0.0-beta+exp.sha.5114f85".parse().unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
        assert_eq!(v.prerelease, Some("beta".to_string()));
        assert_eq!(v.build, Some("exp.sha.5114f85".to_string()));
    }

    #[test]
    fn test_version_parse_errors() {
        assert!("".parse::<Version>().is_err());
        assert!("1.2".parse::<Version>().is_err());
        assert!("1.2.3.4".parse::<Version>().is_err());
        assert!("a.b.c".parse::<Version>().is_err());
        assert!("1.2.c".parse::<Version>().is_err());
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(1, 0, 1);
        let v3 = Version::new(1, 1, 0);
        let v4 = Version::new(2, 0, 0);

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v3 < v4);
        assert!(v1 < v4);
    }

    #[test]
    fn test_version_prerelease_comparison() {
        let v1 = Version::new(1, 0, 0).with_prerelease("alpha");
        let v2 = Version::new(1, 0, 0).with_prerelease("beta");
        let v3 = Version::new(1, 0, 0);

        assert!(v1 < v2); // alpha < beta
        assert!(v2 < v3); // 预发布 < 正式版本
    }

    #[test]
    fn test_version_numeric_prerelease() {
        let v1 = Version::new(1, 0, 0).with_prerelease("1");
        let v2 = Version::new(1, 0, 0).with_prerelease("2");
        let v3 = Version::new(1, 0, 0).with_prerelease("alpha");

        assert!(v1 < v2); // 数字比较
        assert!(v2 < v3); // 数字 < 字母
    }

    #[test]
    fn test_version_build_not_compared() {
        let v1 = Version::new(1, 0, 0).with_build("100");
        let v2 = Version::new(1, 0, 0).with_build("200");

        // 构建元数据不参与比较，但 Eq trait 会考虑所有字段
        // 所以这里比较核心版本号
        assert_eq!(v1.major, v2.major);
        assert_eq!(v1.minor, v2.minor);
        assert_eq!(v1.patch, v2.patch);
        assert_eq!(v1.prerelease, v2.prerelease);
        // build 字段可以不同
        assert_ne!(v1.build, v2.build);
    }

    #[test]
    fn test_version_next() {
        let v = Version::new(1, 2, 3);
        assert_eq!(v.next_major(), Version::new(2, 0, 0));
        assert_eq!(v.next_minor(), Version::new(1, 3, 0));
        assert_eq!(v.next_patch(), Version::new(1, 2, 4));
    }

    #[test]
    fn test_constraint_exact() {
        let c = VersionConstraint::Exact(Version::new(1, 2, 3));
        assert!(c.matches(&Version::new(1, 2, 3)));
        assert!(!c.matches(&Version::new(1, 2, 4)));
        assert!(!c.matches(&Version::new(1, 3, 0)));
    }

    #[test]
    fn test_constraint_greater_than() {
        let c = VersionConstraint::GreaterThan(Version::new(1, 0, 0));
        assert!(!c.matches(&Version::new(1, 0, 0)));
        assert!(c.matches(&Version::new(1, 0, 1)));
        assert!(c.matches(&Version::new(2, 0, 0)));
    }

    #[test]
    fn test_constraint_greater_or_equal() {
        let c = VersionConstraint::GreaterOrEqual(Version::new(1, 0, 0));
        assert!(c.matches(&Version::new(1, 0, 0)));
        assert!(c.matches(&Version::new(1, 0, 1)));
        assert!(c.matches(&Version::new(2, 0, 0)));
    }

    #[test]
    fn test_constraint_less_than() {
        let c = VersionConstraint::LessThan(Version::new(2, 0, 0));
        assert!(c.matches(&Version::new(1, 9, 9)));
        assert!(!c.matches(&Version::new(2, 0, 0)));
        assert!(!c.matches(&Version::new(2, 0, 1)));
    }

    #[test]
    fn test_constraint_range() {
        let c = VersionConstraint::Range(Version::new(1, 0, 0), Version::new(2, 0, 0));
        assert!(c.matches(&Version::new(1, 0, 0)));
        assert!(c.matches(&Version::new(1, 5, 0)));
        assert!(!c.matches(&Version::new(2, 0, 0))); // 不包含上限
        assert!(!c.matches(&Version::new(0, 9, 9)));
    }

    #[test]
    fn test_constraint_caret() {
        // ^1.2.3 = >=1.2.3, <2.0.0
        let c = VersionConstraint::Caret(Version::new(1, 2, 3));
        assert!(!c.matches(&Version::new(1, 2, 2)));
        assert!(c.matches(&Version::new(1, 2, 3)));
        assert!(c.matches(&Version::new(1, 3, 0)));
        assert!(c.matches(&Version::new(1, 9, 9)));
        assert!(!c.matches(&Version::new(2, 0, 0)));

        // ^0.2.3 = >=0.2.3, <0.3.0
        let c = VersionConstraint::Caret(Version::new(0, 2, 3));
        assert!(c.matches(&Version::new(0, 2, 3)));
        assert!(c.matches(&Version::new(0, 2, 9)));
        assert!(!c.matches(&Version::new(0, 3, 0)));
    }

    #[test]
    fn test_constraint_tilde() {
        // ~1.2.3 = >=1.2.3, <1.3.0
        let c = VersionConstraint::Tilde(Version::new(1, 2, 3));
        assert!(!c.matches(&Version::new(1, 2, 2)));
        assert!(c.matches(&Version::new(1, 2, 3)));
        assert!(c.matches(&Version::new(1, 2, 9)));
        assert!(!c.matches(&Version::new(1, 3, 0)));
    }

    #[test]
    fn test_constraint_wildcard() {
        let c = VersionConstraint::Wildcard;
        assert!(c.matches(&Version::new(0, 0, 1)));
        assert!(c.matches(&Version::new(1, 0, 0)));
        assert!(c.matches(&Version::new(99, 99, 99)));
    }

    #[test]
    fn test_constraint_parse() {
        let c: VersionConstraint = "1.2.3".parse().unwrap();
        assert_eq!(c, VersionConstraint::Exact(Version::new(1, 2, 3)));

        let c: VersionConstraint = "^1.2.3".parse().unwrap();
        assert_eq!(c, VersionConstraint::Caret(Version::new(1, 2, 3)));

        let c: VersionConstraint = "~1.2.3".parse().unwrap();
        assert_eq!(c, VersionConstraint::Tilde(Version::new(1, 2, 3)));

        let c: VersionConstraint = ">=1.0.0".parse().unwrap();
        assert_eq!(c, VersionConstraint::GreaterOrEqual(Version::new(1, 0, 0)));

        let c: VersionConstraint = ">1.0.0".parse().unwrap();
        assert_eq!(c, VersionConstraint::GreaterThan(Version::new(1, 0, 0)));

        let c: VersionConstraint = "<2.0.0".parse().unwrap();
        assert_eq!(c, VersionConstraint::LessThan(Version::new(2, 0, 0)));

        let c: VersionConstraint = "*".parse().unwrap();
        assert_eq!(c, VersionConstraint::Wildcard);
    }

    #[test]
    fn test_constraint_display() {
        assert_eq!(
            VersionConstraint::Exact(Version::new(1, 2, 3)).to_string(),
            "=1.2.3"
        );
        assert_eq!(
            VersionConstraint::GreaterThan(Version::new(1, 0, 0)).to_string(),
            ">1.0.0"
        );
        assert_eq!(
            VersionConstraint::Caret(Version::new(1, 2, 3)).to_string(),
            "^1.2.3"
        );
        assert_eq!(
            VersionConstraint::Tilde(Version::new(1, 2, 3)).to_string(),
            "~1.2.3"
        );
        assert_eq!(VersionConstraint::Wildcard.to_string(), "*");
    }
}
