//! 通用 AST 节点定义
//!
//! 这些节点在不同目标语言的 AST 之间共享概念，但语义由各语言 AST 解释。

/// 注释类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Comment {
    /// 行注释: // text
    Line(String),
    /// 块注释: /* text */
    Block(String),
    /// 文档注释: /// text 或 /** text */
    Doc(String),
}

impl Default for Comment {
    fn default() -> Self {
        Comment::Doc(String::new())
    }
}

impl Comment {
    pub fn line<S: Into<String>>(text: S) -> Self {
        Comment::Line(text.into())
    }

    pub fn block<S: Into<String>>(text: S) -> Self {
        Comment::Block(text.into())
    }

    pub fn doc<S: Into<String>>(text: S) -> Self {
        Comment::Doc(text.into())
    }
}

/// 可见性/导出修饰符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    /// public (Rust pub / TS 默认)
    Public,
    /// private (显式私有)
    Private,
    /// export (模块导出)
    Export,
    /// async
    Async,
    /// static / const
    Const,
    /// readonly
    Readonly,
    /// mut (Rust mutable)
    Mut,
    /// derive / macro 属性
    Derive,
}

/// 通用标识符
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    pub name: String,
}

impl Ident {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }
}

impl From<&str> for Ident {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// 通用类型引用
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeRef {
    /// 命名类型: string, i64, User
    Named(String),
    /// 泛型类型: Vec<T>, Promise<T>, Array<T>
    Generic { base: String, args: Vec<TypeRef> },
    /// 可选类型: T | null / Option<T>
    Optional(Box<TypeRef>),
    /// 函数类型: (T, U) => V
    Function {
        params: Vec<TypeRef>,
        return_type: Box<TypeRef>,
    },
    /// 字面量类型: "success" | "error"
    Literal(String),
}

impl TypeRef {
    pub fn named<S: Into<String>>(name: S) -> Self {
        TypeRef::Named(name.into())
    }

    pub fn generic<S: Into<String>>(base: S, args: Vec<TypeRef>) -> Self {
        TypeRef::Generic {
            base: base.into(),
            args,
        }
    }

    pub fn optional(inner: TypeRef) -> Self {
        TypeRef::Optional(Box::new(inner))
    }

    pub fn func(params: Vec<TypeRef>, return_type: TypeRef) -> Self {
        TypeRef::Function {
            params,
            return_type: Box::new(return_type),
        }
    }

    pub fn reference<S: Into<String>>(name: S, _mutable: bool) -> Self {
        // Rust 引用类型在 AST 中表示为命名类型，实际由 Rust 渲染器处理
        TypeRef::Named(name.into())
    }
}

/// 字面量值
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    Array(Vec<LiteralValue>),
    Object(Vec<(String, LiteralValue)>),
}

impl LiteralValue {
    pub fn string<S: Into<String>>(s: S) -> Self {
        LiteralValue::String(s.into())
    }

    pub fn int(v: i64) -> Self {
        LiteralValue::Int(v)
    }

    pub fn bool(v: bool) -> Self {
        LiteralValue::Bool(v)
    }

    pub fn null() -> Self {
        LiteralValue::Null
    }
}

/// 属性/字段定义（通用）
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyDef {
    pub name: String,
    pub type_ref: Option<TypeRef>,
    pub optional: bool,
    pub default_value: Option<LiteralValue>,
    pub annotations: Vec<String>,
    pub comments: Vec<Comment>,
    pub readonly: bool,
}

impl PropertyDef {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            type_ref: None,
            optional: false,
            default_value: None,
            annotations: vec![],
            comments: vec![],
            readonly: false,
        }
    }

    pub fn with_type(mut self, ty: TypeRef) -> Self {
        self.type_ref = Some(ty);
        self
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    pub fn with_default(mut self, val: LiteralValue) -> Self {
        self.default_value = Some(val);
        self
    }

    pub fn with_comment<S: Into<String>>(mut self, text: S) -> Self {
        self.comments.push(Comment::Line(text.into()));
        self
    }

    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }
}

/// 函数参数
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_ref: Option<TypeRef>,
    pub default_value: Option<LiteralValue>,
    pub mutable: bool,
    pub comments: Vec<Comment>,
}

impl Parameter {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            type_ref: None,
            default_value: None,
            mutable: false,
            comments: vec![],
        }
    }

    pub fn with_type(mut self, ty: TypeRef) -> Self {
        self.type_ref = Some(ty);
        self
    }
}

/// 属性访问修饰符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    #[default]
    Default,
    Public,
    Private,
    Protected,
    Crate,
    Super,
}
