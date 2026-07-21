//! AST 中间表示层
//!
//! 将代码生成拆分为两阶段：
//! 1. 结构化 IR-2 → AST 表示（可测试、可序列化）
//! 2. AST → 字符串（渲染/格式化）
//!
//! 设计原则：
//! - AST 节点是纯数据结构，与目标语言语义对齐
//! - 渲染器负责缩进、换行、括号等格式化细节
//! - AST 层可独立测试，不依赖字符串匹配

pub mod emit;
pub mod nodes;
pub mod rust;
pub mod transform;
pub mod ts;

pub use emit::{AstEmitter, EmitContext, EmitError};
pub use nodes::*;
pub use rust::{RustAst, RustEmitter};
pub use ts::{TypeScriptAst, TypeScriptEmitter};

/// 所有 AST 根节点实现的 trait
pub trait AstRoot: std::fmt::Debug + Clone + PartialEq {
    /// 目标文件扩展名
    fn file_extension(&self) -> &'static str;

    /// 使用默认渲染器生成字符串
    fn emit(&self) -> Result<String, EmitError>;
}

/// 通用 AST 构建辅助函数
pub mod builders {
    use super::nodes::*;

    /// 创建文档注释
    pub fn doc(lines: &[&str]) -> Vec<Comment> {
        lines.iter().map(|l| Comment::Doc(l.to_string())).collect()
    }

    /// 创建普通行注释
    pub fn line_comment(text: &str) -> Comment {
        Comment::Line(text.to_string())
    }

    /// 创建块注释
    pub fn block_comment(text: &str) -> Comment {
        Comment::Block(text.to_string())
    }

    /// 创建修饰符列表（public + export）
    pub fn pub_export() -> Vec<Modifier> {
        vec![Modifier::Public, Modifier::Export]
    }

    /// 创建 public 修饰符
    pub fn public() -> Vec<Modifier> {
        vec![Modifier::Public]
    }
}
