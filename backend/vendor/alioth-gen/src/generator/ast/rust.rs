//! Rust AST 节点与渲染器
//!
//! 表示 Rust 代码的结构化 AST。

use super::{
    emit::{
        emit_comments, emit_rust_literal, emit_rust_type_ref, AstEmitter, CommentStyle,
        EmitContext, EmitError,
    },
    nodes::*,
    AstRoot,
};
use std::fmt::Write;

/// Rust 文件 AST 根节点
#[derive(Debug, Clone, PartialEq)]
pub struct RustAst {
    pub file_path: String,
    pub attributes: Vec<RustAttribute>,
    pub items: Vec<RustItem>,
    pub comments: Vec<Comment>,
}

impl RustAst {
    pub fn new<S: Into<String>>(file_path: S) -> Self {
        Self {
            file_path: file_path.into(),
            attributes: vec![],
            items: vec![],
            comments: vec![],
        }
    }

    pub fn with_attribute(mut self, attr: RustAttribute) -> Self {
        self.attributes.push(attr);
        self
    }

    pub fn with_item(mut self, item: RustItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn with_comment(mut self, comment: Comment) -> Self {
        self.comments.push(comment);
        self
    }
}

impl AstRoot for RustAst {
    fn file_extension(&self) -> &'static str {
        "rs"
    }

    fn emit(&self) -> Result<String, EmitError> {
        RustEmitter.emit(self)
    }
}

/// Rust 属性
#[derive(Debug, Clone, PartialEq)]
pub struct RustAttribute {
    pub name: String,
    pub args: Vec<String>,
    pub inner: bool, // #![...] vs #[...]
}

impl RustAttribute {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            args: vec![],
            inner: false,
        }
    }

    pub fn derive(traits: Vec<String>) -> Self {
        Self {
            name: "derive".to_string(),
            args: traits,
            inner: false,
        }
    }

    pub fn arg<S: Into<String>>(mut self, arg: S) -> Self {
        self.args.push(arg.into());
        self
    }
}

/// Rust 模块项
#[derive(Debug, Clone, PartialEq)]
pub enum RustItem {
    /// use 声明
    Use(RustUse),
    /// mod 声明
    ModDecl { name: String, public: bool },
    /// extern crate (保留)
    ExternCrate { name: String, alias: Option<String> },
    /// 结构体定义
    Struct(RustStruct),
    /// 枚举定义
    Enum(RustEnum),
    /// 类型别名
    TypeAlias {
        name: String,
        type_ref: TypeRef,
        public: bool,
    },
    /// 常量定义
    Const {
        name: String,
        type_ref: TypeRef,
        value: LiteralValue,
        public: bool,
    },
    /// 静态变量
    Static {
        name: String,
        type_ref: TypeRef,
        value: LiteralValue,
        mutable: bool,
        public: bool,
    },
    /// 函数定义
    Function(RustFunction),
    /// impl 块
    Impl(RustImpl),
    /// trait 定义
    Trait(RustTrait),
    /// 宏调用/声明
    Macro { name: String, contents: String },
    /// 注释块
    CommentBlock(Vec<Comment>),
}

/// use 声明
#[derive(Debug, Clone, PartialEq)]
pub struct RustUse {
    pub path: String,
    pub items: Vec<RustUseItem>,
    pub alias: Option<String>,
    pub public: bool,
    pub glob: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RustUseItem {
    Simple(String),
    Renamed { name: String, alias: String },
    Nested(Vec<RustUseItem>),
}

impl RustUse {
    pub fn simple<S: Into<String>>(path: S) -> Self {
        Self {
            path: path.into(),
            items: vec![],
            alias: None,
            public: false,
            glob: false,
        }
    }

    pub fn glob<S: Into<String>>(path: S) -> Self {
        Self {
            path: path.into(),
            items: vec![],
            alias: None,
            public: false,
            glob: true,
        }
    }

    pub fn items<S: Into<String>>(path: S, items: Vec<&str>) -> Self {
        Self {
            path: path.into(),
            items: items
                .into_iter()
                .map(|s| RustUseItem::Simple(s.to_string()))
                .collect(),
            alias: None,
            public: false,
            glob: false,
        }
    }

    pub fn public(mut self) -> Self {
        self.public = true;
        self
    }
}

/// 结构体定义
#[derive(Debug, Clone, PartialEq)]
pub struct RustStruct {
    pub name: String,
    pub fields: Vec<RustField>,
    pub generics: Vec<String>,
    pub attributes: Vec<RustAttribute>,
    pub public: bool,
    pub comments: Vec<Comment>,
    pub is_tuple: bool,
}

impl RustStruct {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            fields: vec![],
            generics: vec![],
            attributes: vec![],
            public: true,
            comments: vec![],
            is_tuple: false,
        }
    }

    pub fn field(mut self, field: RustField) -> Self {
        self.fields.push(field);
        self
    }

    pub fn derive(mut self, traits: Vec<String>) -> Self {
        self.attributes.push(RustAttribute::derive(traits));
        self
    }
}

/// Rust 字段定义
#[derive(Debug, Clone, PartialEq)]
pub struct RustField {
    pub name: String,
    pub type_ref: TypeRef,
    pub public: bool,
    pub attributes: Vec<RustAttribute>,
    pub comments: Vec<Comment>,
}

impl RustField {
    pub fn new<S: Into<String>>(name: S, type_ref: TypeRef) -> Self {
        Self {
            name: name.into(),
            type_ref,
            public: false,
            attributes: vec![],
            comments: vec![],
        }
    }

    pub fn public(mut self) -> Self {
        self.public = true;
        self
    }
}

/// 枚举定义
#[derive(Debug, Clone, PartialEq)]
pub struct RustEnum {
    pub name: String,
    pub variants: Vec<RustEnumVariant>,
    pub generics: Vec<String>,
    pub attributes: Vec<RustAttribute>,
    pub public: bool,
    pub comments: Vec<Comment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RustEnumVariant {
    pub name: String,
    pub fields: Vec<RustField>,
    pub discriminant: Option<LiteralValue>,
    pub comments: Vec<Comment>,
    pub is_tuple: bool,
}

impl RustEnumVariant {
    pub fn unit<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            fields: vec![],
            discriminant: None,
            comments: vec![],
            is_tuple: false,
        }
    }

    pub fn named<S: Into<String>>(name: S, fields: Vec<RustField>) -> Self {
        Self {
            name: name.into(),
            fields,
            discriminant: None,
            comments: vec![],
            is_tuple: false,
        }
    }

    pub fn tuple<S: Into<String>>(name: S, fields: Vec<RustField>) -> Self {
        Self {
            name: name.into(),
            fields,
            discriminant: None,
            comments: vec![],
            is_tuple: true,
        }
    }
}

/// 函数定义
#[derive(Debug, Clone, PartialEq)]
pub struct RustFunction {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<TypeRef>,
    pub body: Vec<RustStatement>,
    pub attributes: Vec<RustAttribute>,
    pub public: bool,
    pub async_: bool,
    pub generics: Vec<String>,
    pub comments: Vec<Comment>,
    pub is_method: bool,
    pub self_param: Option<RustSelfParam>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RustSelfParam {
    pub mutable: bool,
    pub reference: bool,
    pub lifetime: Option<String>,
}

impl RustFunction {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            params: vec![],
            return_type: None,
            body: vec![],
            attributes: vec![],
            public: true,
            async_: false,
            generics: vec![],
            comments: vec![],
            is_method: false,
            self_param: None,
        }
    }

    pub fn param(mut self, param: Parameter) -> Self {
        self.params.push(param);
        self
    }

    pub fn returns(mut self, ty: TypeRef) -> Self {
        self.return_type = Some(ty);
        self
    }

    pub fn body_stmt(mut self, stmt: RustStatement) -> Self {
        self.body.push(stmt);
        self
    }

    pub fn async_(mut self) -> Self {
        self.async_ = true;
        self
    }

    pub fn method(mut self) -> Self {
        self.is_method = true;
        self
    }

    pub fn with_self(mut self, mutable: bool, reference: bool) -> Self {
        self.self_param = Some(RustSelfParam {
            mutable,
            reference,
            lifetime: None,
        });
        self
    }
}

/// Rust 语句
#[derive(Debug, Clone, PartialEq)]
pub enum RustStatement {
    /// let 绑定
    Let {
        name: String,
        type_hint: Option<TypeRef>,
        value: RustExpression,
        mutable: bool,
    },
    /// 表达式语句
    Expression(RustExpression),
    /// return expr;
    Return(Option<RustExpression>),
    /// if / else if / else
    If {
        condition: RustExpression,
        then_branch: Vec<RustStatement>,
        else_branch: Option<Vec<RustStatement>>,
    },
    /// match
    Match {
        expr: RustExpression,
        arms: Vec<RustMatchArm>,
    },
    /// for 循环
    For {
        pat: String,
        expr: RustExpression,
        body: Vec<RustStatement>,
    },
    /// while 循环
    While {
        condition: RustExpression,
        body: Vec<RustStatement>,
    },
    /// 注释
    Comment(Vec<Comment>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RustMatchArm {
    pub pattern: String,
    pub guard: Option<RustExpression>,
    pub body: RustExpression,
}

/// Rust 表达式
#[derive(Debug, Clone, PartialEq)]
pub enum RustExpression {
    /// 标识符
    Ident(String),
    /// 字面量
    Literal(LiteralValue),
    /// 方法调用: expr.method(args)
    MethodCall {
        receiver: Box<RustExpression>,
        method: String,
        args: Vec<RustExpression>,
    },
    /// 函数调用: fn(args)
    Call {
        callee: Box<RustExpression>,
        args: Vec<RustExpression>,
    },
    /// 字段访问: expr.field
    FieldAccess {
        object: Box<RustExpression>,
        field: String,
    },
    /// 块表达式
    Block(Vec<RustStatement>),
    /// 闭包
    Closure {
        params: Vec<Parameter>,
        body: Box<RustExpression>,
        async_: bool,
    },
    /// 路径: std::vec::Vec
    Path(Vec<String>),
    /// 二元运算
    Binary {
        left: Box<RustExpression>,
        op: RustBinaryOp,
        right: Box<RustExpression>,
    },
    /// 一元运算
    Unary {
        op: RustUnaryOp,
        expr: Box<RustExpression>,
    },
    /// match 表达式
    Match {
        expr: Box<RustExpression>,
        arms: Vec<RustMatchArm>,
    },
    /// if 表达式
    If {
        condition: Box<RustExpression>,
        then_branch: Vec<RustStatement>,
        else_branch: Option<Box<RustExpression>>,
    },
    /// await: expr.await
    Await(Box<RustExpression>),
    /// 元组构造
    Tuple(Vec<RustExpression>),
    /// 结构体/枚举构造: Name { fields }
    StructInit {
        name: String,
        fields: Vec<(String, RustExpression)>,
    },
    /// 引用: &expr / &mut expr
    Reference {
        mutable: bool,
        expr: Box<RustExpression>,
    },
    /// 解引用: *expr
    Deref(Box<RustExpression>),
    /// 宏调用: macro!(...)
    Macro { name: String, tokens: String },
    /// Ok(expr), Err(expr), Some(expr), None
    ResultCtor {
        variant: String,
        expr: Option<Box<RustExpression>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustBinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Assign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustUnaryOp {
    Neg,
    Not,
    Deref,
    Ref,
    RefMut,
}

/// impl 块
#[derive(Debug, Clone, PartialEq)]
pub struct RustImpl {
    pub trait_name: Option<String>,
    pub struct_name: String,
    pub generics: Vec<String>,
    pub items: Vec<RustImplItem>,
    pub comments: Vec<Comment>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RustImplItem {
    Method(RustFunction),
    Const {
        name: String,
        type_ref: TypeRef,
        value: LiteralValue,
    },
    TypeAlias {
        name: String,
        type_ref: TypeRef,
    },
    Comment(Vec<Comment>),
}

/// trait 定义
#[derive(Debug, Clone, PartialEq)]
pub struct RustTrait {
    pub name: String,
    pub generics: Vec<String>,
    pub items: Vec<RustTraitItem>,
    pub public: bool,
    pub comments: Vec<Comment>,
    pub supertraits: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RustTraitItem {
    MethodSignature {
        name: String,
        params: Vec<Parameter>,
        return_type: Option<TypeRef>,
        async_: bool,
    },
    Type {
        name: String,
        bounds: Vec<String>,
    },
    Const {
        name: String,
        type_ref: TypeRef,
    },
    Comment(Vec<Comment>),
}

// ============================================================================
// Rust 渲染器
// ============================================================================

pub struct RustEmitter;

impl RustEmitter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RustEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl AstEmitter<RustAst> for RustEmitter {
    fn emit(&self, ast: &RustAst) -> Result<String, EmitError> {
        self.emit_with_ctx(ast, &EmitContext::default())
    }

    fn emit_with_ctx(&self, ast: &RustAst, ctx: &EmitContext) -> Result<String, EmitError> {
        let mut output = String::new();

        // 文件级注释
        emit_comments(&mut output, ctx, &ast.comments, CommentStyle::TripleSlash)?;

        // 文件级属性
        for attr in &ast.attributes {
            self.emit_attribute(&mut output, ctx, attr, true)?;
        }

        // 模块项
        for (i, item) in ast.items.iter().enumerate() {
            if i > 0 {
                writeln!(&mut output)?;
            }
            self.emit_item(&mut output, ctx, item)?;
        }

        Ok(output)
    }
}

impl RustEmitter {
    fn emit_attribute<W: Write>(
        &self,
        w: &mut W,
        ctx: &EmitContext,
        attr: &RustAttribute,
        _file_level: bool,
    ) -> Result<(), EmitError> {
        ctx.write_indent(w)?;
        if attr.inner {
            write!(w, "#!")?;
        } else {
            write!(w, "#")?;
        }
        if attr.args.is_empty() {
            writeln!(w, "[{}]", attr.name)?;
        } else {
            writeln!(w, "[{}({})]", attr.name, attr.args.join(", "))?;
        }
        Ok(())
    }

    fn emit_item<W: Write>(
        &self,
        w: &mut W,
        ctx: &EmitContext,
        item: &RustItem,
    ) -> Result<(), EmitError> {
        match item {
            RustItem::Use(u) => self.emit_use(w, ctx, u)?,
            RustItem::ModDecl { name, public } => {
                ctx.write_indent(w)?;
                if *public {
                    write!(w, "pub ")?;
                }
                writeln!(w, "mod {};", name)?;
            }
            RustItem::ExternCrate { name, alias } => {
                ctx.write_indent(w)?;
                write!(w, "extern crate {}", name)?;
                if let Some(a) = alias {
                    write!(w, " as {}", a)?;
                }
                writeln!(w, ";")?;
            }
            RustItem::Struct(s) => self.emit_struct(w, ctx, s)?,
            RustItem::Enum(e) => self.emit_enum(w, ctx, e)?,
            RustItem::TypeAlias {
                name,
                type_ref,
                public,
            } => {
                ctx.write_indent(w)?;
                if *public {
                    write!(w, "pub ")?;
                }
                write!(w, "type {} = ", name)?;
                emit_rust_type_ref(w, type_ref)?;
                writeln!(w, ";")?;
            }
            RustItem::Const {
                name,
                type_ref,
                value,
                public,
            } => {
                ctx.write_indent(w)?;
                if *public {
                    write!(w, "pub ")?;
                }
                write!(w, "const {}: ", name)?;
                emit_rust_type_ref(w, type_ref)?;
                write!(w, " = ")?;
                emit_rust_literal(w, value)?;
                writeln!(w, ";")?;
            }
            RustItem::Static {
                name,
                type_ref,
                value,
                mutable,
                public,
            } => {
                ctx.write_indent(w)?;
                if *public {
                    write!(w, "pub ")?;
                }
                write!(w, "static ")?;
                if *mutable {
                    write!(w, "mut ")?;
                }
                write!(w, "{}: ", name)?;
                emit_rust_type_ref(w, type_ref)?;
                write!(w, " = ")?;
                emit_rust_literal(w, value)?;
                writeln!(w, ";")?;
            }
            RustItem::Function(f) => self.emit_function(w, ctx, f)?,
            RustItem::Impl(i) => self.emit_impl(w, ctx, i)?,
            RustItem::Trait(t) => self.emit_trait(w, ctx, t)?,
            RustItem::Macro { name, contents } => {
                ctx.write_indent(w)?;
                writeln!(w, "{}!{{\n{}\n}}", name, contents)?;
            }
            RustItem::CommentBlock(comments) => {
                emit_comments(w, ctx, comments, CommentStyle::TripleSlash)?;
            }
        }
        Ok(())
    }

    fn emit_use<W: Write>(
        &self,
        w: &mut W,
        ctx: &EmitContext,
        u: &RustUse,
    ) -> Result<(), EmitError> {
        ctx.write_indent(w)?;
        if u.public {
            write!(w, "pub ")?;
        }
        write!(w, "use {}", u.path)?;

        if u.glob {
            writeln!(w, "::*;")?;
        } else if !u.items.is_empty() {
            write!(w, "::{{ ")?;
            for (i, item) in u.items.iter().enumerate() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                self.emit_use_item(w, item)?;
            }
            writeln!(w, " }};")?;
        } else if let Some(alias) = &u.alias {
            writeln!(w, " as {};", alias)?;
        } else {
            writeln!(w, ";")?;
        }
        Ok(())
    }

    fn emit_use_item<W: Write>(&self, w: &mut W, item: &RustUseItem) -> Result<(), EmitError> {
        match item {
            RustUseItem::Simple(name) => write!(w, "{}", name)?,
            RustUseItem::Renamed { name, alias } => write!(w, "{} as {}", name, alias)?,
            RustUseItem::Nested(items) => {
                write!(w, "{{ ")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    self.emit_use_item(w, item)?;
                }
                write!(w, " }}")?;
            }
        }
        Ok(())
    }

    fn emit_struct<W: Write>(
        &self,
        w: &mut W,
        ctx: &EmitContext,
        s: &RustStruct,
    ) -> Result<(), EmitError> {
        emit_comments(w, ctx, &s.comments, CommentStyle::TripleSlash)?;
        for attr in &s.attributes {
            self.emit_attribute(w, ctx, attr, false)?;
        }
        ctx.write_indent(w)?;
        if s.public {
            write!(w, "pub ")?;
        }
        write!(w, "struct {}{}", s.name, self.format_generics(&s.generics))?;

        if s.is_tuple {
            writeln!(w, "(")?;
            let nested = ctx.nested();
            for field in &s.fields {
                nested.write_indent(w)?;
                for attr in &field.attributes {
                    self.emit_attribute(w, &nested, attr, false)?;
                }
                emit_comments(w, &nested, &field.comments, CommentStyle::TripleSlash)?;
                nested.write_indent(w)?;
                emit_rust_type_ref(w, &field.type_ref)?;
                writeln!(w, ",")?;
            }
            ctx.write_indent(w)?;
            writeln!(w, ");")?;
        } else if s.fields.is_empty() {
            writeln!(w, " {{}}")?;
        } else {
            writeln!(w, " {{")?;
            let nested = ctx.nested();
            for field in &s.fields {
                for attr in &field.attributes {
                    self.emit_attribute(w, &nested, attr, false)?;
                }
                emit_comments(w, &nested, &field.comments, CommentStyle::TripleSlash)?;
                nested.write_indent(w)?;
                if field.public {
                    write!(w, "pub ")?;
                }
                write!(w, "{}: ", field.name)?;
                emit_rust_type_ref(w, &field.type_ref)?;
                writeln!(w, ",")?;
            }
            ctx.write_indent(w)?;
            writeln!(w, "}}")?;
        }
        Ok(())
    }

    fn emit_enum<W: Write>(
        &self,
        w: &mut W,
        ctx: &EmitContext,
        e: &RustEnum,
    ) -> Result<(), EmitError> {
        emit_comments(w, ctx, &e.comments, CommentStyle::TripleSlash)?;
        for attr in &e.attributes {
            self.emit_attribute(w, ctx, attr, false)?;
        }
        ctx.write_indent(w)?;
        if e.public {
            write!(w, "pub ")?;
        }
        writeln!(w, "enum {}{} {{", e.name, self.format_generics(&e.generics))?;

        let nested = ctx.nested();
        for variant in &e.variants {
            emit_comments(w, &nested, &variant.comments, CommentStyle::TripleSlash)?;
            nested.write_indent(w)?;
            write!(w, "{}", variant.name)?;

            if variant.is_tuple && !variant.fields.is_empty() {
                write!(w, "(")?;
                for (i, field) in variant.fields.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    emit_rust_type_ref(w, &field.type_ref)?;
                }
                write!(w, ")")?;
            } else if !variant.fields.is_empty() {
                write!(w, " {{")?;
                for (i, field) in variant.fields.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    if field.public {
                        write!(w, "pub ")?;
                    }
                    write!(w, "{}: ", field.name)?;
                    emit_rust_type_ref(w, &field.type_ref)?;
                }
                write!(w, " }}")?;
            }

            if let Some(disc) = &variant.discriminant {
                write!(w, " = ")?;
                emit_rust_literal(w, disc)?;
            }
            writeln!(w, ",")?;
        }

        ctx.write_indent(w)?;
        writeln!(w, "}}")?;
        Ok(())
    }

    fn emit_function<W: Write>(
        &self,
        w: &mut W,
        ctx: &EmitContext,
        f: &RustFunction,
    ) -> Result<(), EmitError> {
        emit_comments(w, ctx, &f.comments, CommentStyle::TripleSlash)?;
        for attr in &f.attributes {
            self.emit_attribute(w, ctx, attr, false)?;
        }
        ctx.write_indent(w)?;
        if f.public && !f.is_method {
            write!(w, "pub ")?;
        }
        if f.async_ {
            write!(w, "async ")?;
        }
        write!(w, "fn {}", f.name)?;
        write!(w, "{}", self.format_generics(&f.generics))?;
        write!(w, "(")?;

        let mut first = true;
        if let Some(self_param) = &f.self_param {
            if self_param.reference {
                write!(w, "&")?;
            }
            if self_param.mutable {
                write!(w, "mut ")?;
            }
            write!(w, "self")?;
            first = false;
        }

        for param in &f.params {
            if !first {
                write!(w, ", ")?;
            }
            first = false;
            if param.mutable {
                write!(w, "mut ")?;
            }
            write!(w, "{}", param.name)?;
            if let Some(ty) = &param.type_ref {
                write!(w, ": ")?;
                emit_rust_type_ref(w, ty)?;
            }
        }

        write!(w, ")")?;
        if let Some(rt) = &f.return_type {
            write!(w, " -> ")?;
            emit_rust_type_ref(w, rt)?;
        }

        writeln!(w, " {{")?;
        let nested = ctx.nested();
        for stmt in &f.body {
            self.emit_statement(w, &nested, stmt)?;
        }
        ctx.write_indent(w)?;
        writeln!(w, "}}")?;
        Ok(())
    }

    fn emit_impl<W: Write>(
        &self,
        w: &mut W,
        ctx: &EmitContext,
        i: &RustImpl,
    ) -> Result<(), EmitError> {
        ctx.write_indent(w)?;
        write!(w, "impl{}", self.format_generics(&i.generics))?;
        if let Some(trait_name) = &i.trait_name {
            write!(w, " {} for", trait_name)?;
        }
        writeln!(w, " {} {{", i.struct_name)?;

        let nested = ctx.nested();
        for item in &i.items {
            match item {
                RustImplItem::Method(m) => self.emit_function(w, &nested, m)?,
                RustImplItem::Const {
                    name,
                    type_ref,
                    value,
                } => {
                    nested.write_indent(w)?;
                    write!(w, "const {}: ", name)?;
                    emit_rust_type_ref(w, type_ref)?;
                    write!(w, " = ")?;
                    emit_rust_literal(w, value)?;
                    writeln!(w, ";")?;
                }
                RustImplItem::TypeAlias { name, type_ref } => {
                    nested.write_indent(w)?;
                    write!(w, "type {} = ", name)?;
                    emit_rust_type_ref(w, type_ref)?;
                    writeln!(w, ";")?;
                }
                RustImplItem::Comment(comments) => {
                    emit_comments(w, &nested, comments, CommentStyle::TripleSlash)?;
                }
            }
        }

        ctx.write_indent(w)?;
        writeln!(w, "}}")?;
        Ok(())
    }

    fn emit_trait<W: Write>(
        &self,
        w: &mut W,
        ctx: &EmitContext,
        t: &RustTrait,
    ) -> Result<(), EmitError> {
        emit_comments(w, ctx, &t.comments, CommentStyle::TripleSlash)?;
        ctx.write_indent(w)?;
        if t.public {
            write!(w, "pub ")?;
        }
        write!(w, "trait {}{}", t.name, self.format_generics(&t.generics))?;
        if !t.supertraits.is_empty() {
            write!(w, ": {}", t.supertraits.join(" + "))?;
        }
        writeln!(w, " {{")?;

        let nested = ctx.nested();
        for item in &t.items {
            match item {
                RustTraitItem::MethodSignature {
                    name,
                    params,
                    return_type,
                    async_,
                } => {
                    nested.write_indent(w)?;
                    if *async_ {
                        write!(w, "async ")?;
                    }
                    write!(w, "fn {}(", name)?;
                    for (i, p) in params.iter().enumerate() {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        write!(w, "{}", p.name)?;
                        if let Some(ty) = &p.type_ref {
                            write!(w, ": ")?;
                            emit_rust_type_ref(w, ty)?;
                        }
                    }
                    write!(w, ")")?;
                    if let Some(rt) = return_type {
                        write!(w, " -> ")?;
                        emit_rust_type_ref(w, rt)?;
                    }
                    writeln!(w, ";")?;
                }
                RustTraitItem::Type { name, bounds } => {
                    nested.write_indent(w)?;
                    write!(w, "type {}", name)?;
                    if !bounds.is_empty() {
                        write!(w, ": {}", bounds.join(" + "))?;
                    }
                    writeln!(w, ";")?;
                }
                RustTraitItem::Const { name, type_ref } => {
                    nested.write_indent(w)?;
                    write!(w, "const {}: ", name)?;
                    emit_rust_type_ref(w, type_ref)?;
                    writeln!(w, ";")?;
                }
                RustTraitItem::Comment(comments) => {
                    emit_comments(w, &nested, comments, CommentStyle::TripleSlash)?;
                }
            }
        }

        ctx.write_indent(w)?;
        writeln!(w, "}}")?;
        Ok(())
    }

    fn emit_statement<W: Write>(
        &self,
        w: &mut W,
        ctx: &EmitContext,
        stmt: &RustStatement,
    ) -> Result<(), EmitError> {
        match stmt {
            RustStatement::Let {
                name,
                type_hint,
                value,
                mutable,
            } => {
                ctx.write_indent(w)?;
                write!(w, "let ")?;
                if *mutable {
                    write!(w, "mut ")?;
                }
                write!(w, "{}", name)?;
                if let Some(ty) = type_hint {
                    write!(w, ": ")?;
                    emit_rust_type_ref(w, ty)?;
                }
                write!(w, " = ")?;
                self.emit_expression(w, ctx, value)?;
                writeln!(w, ";")?;
            }
            RustStatement::Expression(expr) => {
                ctx.write_indent(w)?;
                self.emit_expression(w, ctx, expr)?;
                writeln!(w, ";")?;
            }
            RustStatement::Return(expr) => {
                ctx.write_indent(w)?;
                write!(w, "return")?;
                if let Some(e) = expr {
                    write!(w, " ")?;
                    self.emit_expression(w, ctx, e)?;
                }
                writeln!(w, ";")?;
            }
            RustStatement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                ctx.write_indent(w)?;
                write!(w, "if ")?;
                self.emit_expression(w, ctx, condition)?;
                writeln!(w, " {{")?;
                let nested = ctx.nested();
                for stmt in then_branch {
                    self.emit_statement(w, &nested, stmt)?;
                }
                ctx.write_indent(w)?;
                write!(w, "}}")?;
                if let Some(else_stmts) = else_branch {
                    writeln!(w, " else {{")?;
                    let nested = ctx.nested();
                    for stmt in else_stmts {
                        self.emit_statement(w, &nested, stmt)?;
                    }
                    ctx.write_indent(w)?;
                    writeln!(w, "}}")?;
                } else {
                    writeln!(w)?;
                }
            }
            RustStatement::Match { expr, arms } => {
                ctx.write_indent(w)?;
                write!(w, "match ")?;
                self.emit_expression(w, ctx, expr)?;
                writeln!(w, " {{")?;
                let nested = ctx.nested();
                for arm in arms {
                    nested.write_indent(w)?;
                    write!(w, "{}", arm.pattern)?;
                    if let Some(guard) = &arm.guard {
                        write!(w, " if ")?;
                        self.emit_expression(w, &nested, guard)?;
                    }
                    write!(w, " => ")?;
                    self.emit_expression(w, &nested, &arm.body)?;
                    writeln!(w, ",")?;
                }
                ctx.write_indent(w)?;
                writeln!(w, "}}")?;
            }
            RustStatement::For { pat, expr, body } => {
                ctx.write_indent(w)?;
                write!(w, "for {} in ", pat)?;
                self.emit_expression(w, ctx, expr)?;
                writeln!(w, " {{")?;
                let nested = ctx.nested();
                for stmt in body {
                    self.emit_statement(w, &nested, stmt)?;
                }
                ctx.write_indent(w)?;
                writeln!(w, "}}")?;
            }
            RustStatement::While { condition, body } => {
                ctx.write_indent(w)?;
                write!(w, "while ")?;
                self.emit_expression(w, ctx, condition)?;
                writeln!(w, " {{")?;
                let nested = ctx.nested();
                for stmt in body {
                    self.emit_statement(w, &nested, stmt)?;
                }
                ctx.write_indent(w)?;
                writeln!(w, "}}")?;
            }
            RustStatement::Comment(comments) => {
                emit_comments(w, ctx, comments, CommentStyle::TripleSlash)?;
            }
        }
        Ok(())
    }

    fn emit_expression<W: Write>(
        &self,
        w: &mut W,
        ctx: &EmitContext,
        expr: &RustExpression,
    ) -> Result<(), EmitError> {
        match expr {
            RustExpression::Ident(name) => write!(w, "{}", name)?,
            RustExpression::Literal(lit) => emit_rust_literal(w, lit)?,
            RustExpression::MethodCall {
                receiver,
                method,
                args,
            } => {
                self.emit_expression(w, ctx, receiver)?;
                write!(w, ".{}(", method)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    self.emit_expression(w, ctx, arg)?;
                }
                write!(w, ")")?;
            }
            RustExpression::Call { callee, args } => {
                self.emit_expression(w, ctx, callee)?;
                write!(w, "(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    self.emit_expression(w, ctx, arg)?;
                }
                write!(w, ")")?;
            }
            RustExpression::FieldAccess { object, field } => {
                self.emit_expression(w, ctx, object)?;
                write!(w, ".{}", field)?;
            }
            RustExpression::Block(stmts) => {
                write!(w, "{{")?;
                if !stmts.is_empty() {
                    writeln!(w)?;
                    let nested = ctx.nested();
                    for stmt in stmts {
                        self.emit_statement(w, &nested, stmt)?;
                    }
                    ctx.write_indent(w)?;
                }
                write!(w, "}}")?;
            }
            RustExpression::Closure {
                params,
                body,
                async_,
            } => {
                if *async_ {
                    write!(w, "async ")?;
                }
                write!(w, "|")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    write!(w, "{}", p.name)?;
                    if let Some(ty) = &p.type_ref {
                        write!(w, ": ")?;
                        emit_rust_type_ref(w, ty)?;
                    }
                }
                write!(w, "|")?;
                write!(w, " ")?;
                self.emit_expression(w, ctx, body)?;
            }
            RustExpression::Path(segments) => {
                write!(w, "{}", segments.join("::"))?;
            }
            RustExpression::Binary { left, op, right } => {
                self.emit_expression(w, ctx, left)?;
                let op_str = match op {
                    RustBinaryOp::Add => " + ",
                    RustBinaryOp::Sub => " - ",
                    RustBinaryOp::Mul => " * ",
                    RustBinaryOp::Div => " / ",
                    RustBinaryOp::Rem => " % ",
                    RustBinaryOp::And => " && ",
                    RustBinaryOp::Or => " || ",
                    RustBinaryOp::BitAnd => " & ",
                    RustBinaryOp::BitOr => " | ",
                    RustBinaryOp::BitXor => " ^ ",
                    RustBinaryOp::Shl => " << ",
                    RustBinaryOp::Shr => " >> ",
                    RustBinaryOp::Eq => " == ",
                    RustBinaryOp::Ne => " != ",
                    RustBinaryOp::Lt => " < ",
                    RustBinaryOp::Le => " <= ",
                    RustBinaryOp::Gt => " > ",
                    RustBinaryOp::Ge => " >= ",
                    RustBinaryOp::Assign => " = ",
                };
                write!(w, "{}", op_str)?;
                self.emit_expression(w, ctx, right)?;
            }
            RustExpression::Unary { op, expr } => {
                let op_str = match op {
                    RustUnaryOp::Neg => "-",
                    RustUnaryOp::Not => "!",
                    RustUnaryOp::Deref => "*",
                    RustUnaryOp::Ref => "&",
                    RustUnaryOp::RefMut => "&mut ",
                };
                write!(w, "{}", op_str)?;
                self.emit_expression(w, ctx, expr)?;
            }
            RustExpression::Match { expr, arms } => {
                write!(w, "match ")?;
                self.emit_expression(w, ctx, expr)?;
                write!(w, " {{")?;
                writeln!(w)?;
                let nested = ctx.nested();
                for arm in arms {
                    nested.write_indent(w)?;
                    write!(w, "{}", arm.pattern)?;
                    if let Some(guard) = &arm.guard {
                        write!(w, " if ")?;
                        self.emit_expression(w, &nested, guard)?;
                    }
                    write!(w, " => ")?;
                    self.emit_expression(w, &nested, &arm.body)?;
                    writeln!(w, ",")?;
                }
                ctx.write_indent(w)?;
                write!(w, "}}")?;
            }
            RustExpression::If {
                condition,
                then_branch,
                else_branch,
            } => {
                write!(w, "if ")?;
                self.emit_expression(w, ctx, condition)?;
                write!(w, " {{")?;
                writeln!(w)?;
                let nested = ctx.nested();
                for stmt in then_branch {
                    self.emit_statement(w, &nested, stmt)?;
                }
                ctx.write_indent(w)?;
                write!(w, "}}")?;
                if let Some(else_expr) = else_branch {
                    write!(w, " else ")?;
                    self.emit_expression(w, ctx, else_expr)?;
                }
            }
            RustExpression::Await(expr) => {
                self.emit_expression(w, ctx, expr)?;
                write!(w, ".await")?;
            }
            RustExpression::Tuple(items) => {
                write!(w, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    self.emit_expression(w, ctx, item)?;
                }
                write!(w, ")")?;
            }
            RustExpression::StructInit { name, fields } => {
                write!(w, "{} {{", name)?;
                for (i, (k, v)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    write!(w, "{}: ", k)?;
                    self.emit_expression(w, ctx, v)?;
                }
                write!(w, "}}")?;
            }
            RustExpression::Reference { mutable, expr } => {
                write!(w, "&")?;
                if *mutable {
                    write!(w, "mut ")?;
                }
                self.emit_expression(w, ctx, expr)?;
            }
            RustExpression::Deref(expr) => {
                write!(w, "*")?;
                self.emit_expression(w, ctx, expr)?;
            }
            RustExpression::Macro { name, tokens } => {
                write!(w, "{}!({})", name, tokens)?;
            }
            RustExpression::ResultCtor { variant, expr } => {
                write!(w, "{}", variant)?;
                if let Some(e) = expr {
                    write!(w, "(")?;
                    self.emit_expression(w, ctx, e)?;
                    write!(w, ")")?;
                }
            }
        }
        Ok(())
    }

    fn format_generics(&self, generics: &[String]) -> String {
        if generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", generics.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_simple_struct() {
        let ast = RustAst::new("test.rs").with_item(RustItem::Struct(
            RustStruct::new("User")
                .derive(vec!["Debug".to_string(), "Clone".to_string()])
                .field(RustField::new("id", TypeRef::named("i64")).public())
                .field(RustField::new("email", TypeRef::named("String")).public()),
        ));

        let emitter = RustEmitter;
        let output = emitter.emit(&ast).unwrap();
        assert!(output.contains("#[derive(Debug, Clone)]"));
        assert!(output.contains("pub struct User {"));
        assert!(output.contains("pub id: i64,"));
        assert!(output.contains("pub email: String,"));
    }

    #[test]
    fn test_emit_function() {
        let ast = RustAst::new("test.rs").with_item(RustItem::Function(
            RustFunction::new("get_user")
                .param(Parameter::new("id").with_type(TypeRef::named("i64")))
                .returns(TypeRef::generic("Option", vec![TypeRef::named("User")]))
                .body_stmt(RustStatement::Return(Some(RustExpression::ResultCtor {
                    variant: "None".to_string(),
                    expr: None,
                }))),
        ));

        let emitter = RustEmitter;
        let output = emitter.emit(&ast).unwrap();
        assert!(output.contains("pub fn get_user(id: i64) -> Option<User> {"));
        assert!(output.contains("return None;"));
    }

    #[test]
    fn test_emit_async_function() {
        let ast = RustAst::new("test.rs").with_item(RustItem::Function(
            RustFunction::new("list_users")
                .async_()
                .param(
                    Parameter::new("pool")
                        .with_type(TypeRef::generic("Data", vec![TypeRef::named("PgPool")])),
                )
                .returns(TypeRef::generic(
                    "Result",
                    vec![TypeRef::named("HttpResponse"), TypeRef::named("ApiError")],
                ))
                .body_stmt(RustStatement::Expression(RustExpression::Await(Box::new(
                    RustExpression::MethodCall {
                        receiver: Box::new(RustExpression::Ident("repo".to_string())),
                        method: "list".to_string(),
                        args: vec![],
                    },
                )))),
        ));

        let emitter = RustEmitter;
        let output = emitter.emit(&ast).unwrap();
        assert!(output.contains(
            "pub async fn list_users(pool: Data<PgPool>) -> Result<HttpResponse, ApiError> {"
        ));
        assert!(output.contains("repo.list().await;"));
    }

    #[test]
    fn test_ast_equality() {
        let ast1 = RustAst::new("a.rs").with_item(RustItem::Use(
            RustUse::simple("std::collections::HashMap").public(),
        ));

        let ast2 = RustAst::new("a.rs").with_item(RustItem::Use(
            RustUse::simple("std::collections::HashMap").public(),
        ));

        assert_eq!(ast1, ast2);
    }
}
