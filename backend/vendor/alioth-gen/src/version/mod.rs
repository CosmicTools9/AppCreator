//! 版本管理模块
//!
//! 提供语义化版本 (SemVer) 支持，包括版本解析、比较和约束匹配

pub mod compat;
pub mod manager;

pub use meta_model::version::{Version, VersionConstraint, VersionParseError};
