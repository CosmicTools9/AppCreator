//! AST → 字符串 渲染器框架
//!
//! 提供统一的渲染接口和通用格式化工具。

use std::fmt::Write;

/// 渲染错误
#[derive(Debug, thiserror::Error)]
pub enum EmitError {
    #[error("格式化错误: {0}")]
    Fmt(#[from] std::fmt::Error),

    #[error("不支持的 AST 节点: {0}")]
    Unsupported(String),

    #[error("渲染配置错误: {0}")]
    Config(String),
}

/// 渲染上下文（缩进、换行配置）
pub struct EmitContext {
    pub indent_size: usize,
    pub use_tabs: bool,
    pub line_width: usize,
    pub indent_level: usize,
}

impl Default for EmitContext {
    fn default() -> Self {
        Self {
            indent_size: 2,
            use_tabs: false,
            line_width: 100,
            indent_level: 0,
        }
    }
}

impl EmitContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取当前缩进字符串
    pub fn indent(&self) -> String {
        if self.use_tabs {
            "\t".repeat(self.indent_level)
        } else {
            " ".repeat(self.indent_level * self.indent_size)
        }
    }

    /// 增加缩进级别
    pub fn nested(&self) -> Self {
        Self {
            indent_level: self.indent_level + 1,
            ..self.clone()
        }
    }

    /// 在 writer 中写入当前缩进
    pub fn write_indent<W: Write>(&self, w: &mut W) -> Result<(), std::fmt::Error> {
        write!(w, "{}", self.indent())
    }

    /// 写入带缩进的行
    pub fn write_line<W: Write>(&self, w: &mut W, content: &str) -> Result<(), std::fmt::Error> {
        self.write_indent(w)?;
        writeln!(w, "{}", content)
    }

    /// 写入空行
    pub fn write_blank<W: Write>(&self, w: &mut W) -> Result<(), std::fmt::Error> {
        writeln!(w)
    }
}

impl Clone for EmitContext {
    fn clone(&self) -> Self {
        Self {
            indent_size: self.indent_size,
            use_tabs: self.use_tabs,
            line_width: self.line_width,
            indent_level: self.indent_level,
        }
    }
}

/// AST 渲染器 trait
pub trait AstEmitter<T> {
    /// 将 AST 渲染为字符串
    fn emit(&self, ast: &T) -> Result<String, EmitError>;

    /// 使用指定上下文渲染
    fn emit_with_ctx(&self, ast: &T, ctx: &EmitContext) -> Result<String, EmitError>;
}

/// 通用注释渲染辅助
pub fn emit_comments<W: Write>(
    w: &mut W,
    ctx: &EmitContext,
    comments: &[super::Comment],
    style: CommentStyle,
) -> Result<(), EmitError> {
    for comment in comments {
        match (comment, style) {
            (super::Comment::Line(text), CommentStyle::Line) => {
                ctx.write_indent(w)?;
                writeln!(w, "// {}", text)?;
            }
            (super::Comment::Line(text), CommentStyle::SlashSlash) => {
                ctx.write_indent(w)?;
                writeln!(w, "// {}", text)?;
            }
            (super::Comment::Doc(text), CommentStyle::TripleSlash) => {
                ctx.write_indent(w)?;
                writeln!(w, "/// {}", text)?;
            }
            (super::Comment::Doc(text), CommentStyle::JsDoc) => {
                if comments.len() == 1 {
                    ctx.write_indent(w)?;
                    writeln!(w, "/** {} */", text)?;
                } else {
                    // 多行 JSDoc
                    ctx.write_indent(w)?;
                    writeln!(w, "/**")?;
                    ctx.write_indent(w)?;
                    writeln!(w, " * {}", text)?;
                    ctx.write_indent(w)?;
                    writeln!(w, " */")?;
                }
            }
            (super::Comment::Block(text), CommentStyle::Block) => {
                ctx.write_indent(w)?;
                writeln!(w, "/* {} */", text)?;
            }
            (super::Comment::Doc(text), CommentStyle::Line) => {
                ctx.write_indent(w)?;
                writeln!(w, "// {}", text)?;
            }
            _ => {
                ctx.write_indent(w)?;
                writeln!(w, "// {:?}", comment)?;
            }
        }
    }
    Ok(())
}

/// 注释风格
#[derive(Debug, Clone, Copy)]
pub enum CommentStyle {
    /// //
    Line,
    /// // (别名)
    SlashSlash,
    /// ///
    TripleSlash,
    /// /** */
    JsDoc,
    /// /* */
    Block,
}

/// 通用类型引用渲染
pub fn emit_type_ref<W: Write>(w: &mut W, type_ref: &super::TypeRef) -> Result<(), EmitError> {
    match type_ref {
        super::TypeRef::Named(name) => {
            write!(w, "{}", name)?;
        }
        super::TypeRef::Generic { base, args } => {
            write!(w, "{}<", base)?;
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                emit_type_ref(w, arg)?;
            }
            write!(w, ">")?;
        }
        super::TypeRef::Optional(inner) => {
            emit_type_ref(w, inner)?;
            write!(w, " | null")?;
        }
        super::TypeRef::Function {
            params,
            return_type,
        } => {
            write!(w, "(")?;
            for (i, p) in params.iter().enumerate() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                emit_type_ref(w, p)?;
            }
            write!(w, ") => ")?;
            emit_type_ref(w, return_type)?;
        }
        super::TypeRef::Literal(val) => {
            write!(w, "\"{}\"", val)?;
        }
    }
    Ok(())
}

/// 通用字面量渲染
pub fn emit_literal<W: Write>(w: &mut W, literal: &super::LiteralValue) -> Result<(), EmitError> {
    match literal {
        super::LiteralValue::String(s) => {
            write!(w, "\"{}\"", s)?;
        }
        super::LiteralValue::Int(i) => {
            write!(w, "{}", i)?;
        }
        super::LiteralValue::Float(f) => {
            write!(w, "{}", f)?;
        }
        super::LiteralValue::Bool(b) => {
            write!(w, "{}", b)?;
        }
        super::LiteralValue::Null => {
            write!(w, "null")?;
        }
        super::LiteralValue::Array(items) => {
            write!(w, "[")?;
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                emit_literal(w, item)?;
            }
            write!(w, "]")?;
        }
        super::LiteralValue::Object(props) => {
            write!(w, "{{ ")?;
            for (i, (k, v)) in props.iter().enumerate() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                write!(w, "{}: ", k)?;
                emit_literal(w, v)?;
            }
            write!(w, " }}")?;
        }
    }
    Ok(())
}

/// Rust 风格类型引用渲染
pub fn emit_rust_type_ref<W: Write>(w: &mut W, type_ref: &super::TypeRef) -> Result<(), EmitError> {
    match type_ref {
        super::TypeRef::Named(name) => {
            write!(w, "{}", name)?;
        }
        super::TypeRef::Generic { base, args } => {
            write!(w, "{}<", base)?;
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                emit_rust_type_ref(w, arg)?;
            }
            write!(w, ">")?;
        }
        super::TypeRef::Optional(inner) => {
            write!(w, "Option<")?;
            emit_rust_type_ref(w, inner)?;
            write!(w, ">")?;
        }
        super::TypeRef::Function {
            params,
            return_type,
        } => {
            // Rust 函数指针类型: fn(T, U) -> V
            write!(w, "fn(")?;
            for (i, p) in params.iter().enumerate() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                emit_rust_type_ref(w, p)?;
            }
            write!(w, ") -> ")?;
            emit_rust_type_ref(w, return_type)?;
        }
        super::TypeRef::Literal(val) => {
            write!(w, "&'static str")?;
            // 字面量类型在 Rust 中通常表示为 &'static str
            let _ = val;
        }
    }
    Ok(())
}

/// Rust 风格字面量渲染
pub fn emit_rust_literal<W: Write>(
    w: &mut W,
    literal: &super::LiteralValue,
) -> Result<(), EmitError> {
    match literal {
        super::LiteralValue::String(s) => {
            write!(w, "\"{}\"", s)?;
        }
        super::LiteralValue::Int(i) => {
            write!(w, "{}", i)?;
        }
        super::LiteralValue::Float(f) => {
            write!(w, "{}", f)?;
        }
        super::LiteralValue::Bool(b) => {
            write!(w, "{}", b)?;
        }
        super::LiteralValue::Null => {
            write!(w, "None")?;
        }
        super::LiteralValue::Array(items) => {
            write!(w, "vec![")?;
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                emit_rust_literal(w, item)?;
            }
            write!(w, "]")?;
        }
        super::LiteralValue::Object(props) => {
            // Rust 中对象字面量通常用 struct 初始化，这里用 HashMap 近似
            write!(w, "[")?;
            for (i, (k, v)) in props.iter().enumerate() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                write!(w, "(\"{}\".to_string(), ", k)?;
                emit_rust_literal(w, v)?;
                write!(w, ")")?;
            }
            write!(
                w,
                "].into_iter().collect::<std::collections::HashMap<_, _>>()"
            )?;
        }
    }
    Ok(())
}
