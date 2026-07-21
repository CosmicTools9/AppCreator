//! TypeScript AST 节点与渲染器
//!
//! 表示 TypeScript/JavaScript 代码的结构化 AST。

use super::{
    emit::{
        emit_comments, emit_literal, emit_type_ref, AstEmitter, CommentStyle, EmitContext,
        EmitError,
    },
    nodes::*,
    AstRoot,
};
use std::fmt::Write;

/// TypeScript 文件 AST 根节点
#[derive(Debug, Clone, PartialEq)]
pub struct TypeScriptAst {
    pub file_path: String,
    pub imports: Vec<TsImport>,
    pub statements: Vec<TsStatement>,
    pub comments: Vec<Comment>,
}

impl TypeScriptAst {
    pub fn new<S: Into<String>>(file_path: S) -> Self {
        Self {
            file_path: file_path.into(),
            imports: vec![],
            statements: vec![],
            comments: vec![],
        }
    }

    pub fn with_import(mut self, import: TsImport) -> Self {
        self.imports.push(import);
        self
    }

    pub fn with_statement(mut self, stmt: TsStatement) -> Self {
        self.statements.push(stmt);
        self
    }

    pub fn with_comment(mut self, comment: Comment) -> Self {
        self.comments.push(comment);
        self
    }
}

impl AstRoot for TypeScriptAst {
    fn file_extension(&self) -> &'static str {
        "ts"
    }

    fn emit(&self) -> Result<String, EmitError> {
        TypeScriptEmitter.emit(self)
    }
}

/// TypeScript 导入声明
#[derive(Debug, Clone, PartialEq)]
pub struct TsImport {
    pub module: String,
    pub default: Option<String>,
    pub named: Vec<TsImportNamed>,
    pub namespace: Option<String>,
    pub comments: Vec<Comment>,
}

impl TsImport {
    pub fn new<S: Into<String>>(module: S) -> Self {
        Self {
            module: module.into(),
            default: None,
            named: vec![],
            namespace: None,
            comments: vec![],
        }
    }

    pub fn default<S: Into<String>>(module: S, name: S) -> Self {
        Self {
            module: module.into(),
            default: Some(name.into()),
            named: vec![],
            namespace: None,
            comments: vec![],
        }
    }

    pub fn named<S: Into<String>>(module: S, imports: Vec<TsImportNamed>) -> Self {
        Self {
            module: module.into(),
            default: None,
            named: imports,
            namespace: None,
            comments: vec![],
        }
    }

    pub fn add_named(mut self, name: TsImportNamed) -> Self {
        self.named.push(name);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TsImportNamed {
    pub name: String,
    pub alias: Option<String>,
}

impl TsImportNamed {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            alias: None,
        }
    }

    pub fn aliased<S: Into<String>>(name: S, alias: S) -> Self {
        Self {
            name: name.into(),
            alias: Some(alias.into()),
        }
    }
}

/// TypeScript 语句
#[derive(Debug, Clone, PartialEq)]
pub enum TsStatement {
    /// 接口声明
    Interface(TsInterface),
    /// 类型别名
    TypeAlias(TsTypeAlias),
    /// 变量/常量声明
    Variable(TsVariable),
    /// 枚举声明
    Enum(TsEnum),
    /// 函数声明
    Function(TsFunction),
    /// 导出列表
    Export(Vec<String>),
    /// 表达式语句
    Expression(TsExpression),
    /// 注释块
    CommentBlock(Vec<Comment>),
}

/// TypeScript 接口声明
#[derive(Debug, Clone, PartialEq)]
pub struct TsInterface {
    pub name: String,
    pub extends: Vec<String>,
    pub properties: Vec<PropertyDef>,
    pub comments: Vec<Comment>,
    pub exported: bool,
}

impl TsInterface {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            extends: vec![],
            properties: vec![],
            comments: vec![],
            exported: true,
        }
    }

    pub fn extends<S: Into<String>>(mut self, base: S) -> Self {
        self.extends.push(base.into());
        self
    }

    pub fn property(mut self, prop: PropertyDef) -> Self {
        self.properties.push(prop);
        self
    }

    pub fn exported(mut self, v: bool) -> Self {
        self.exported = v;
        self
    }
}

/// TypeScript 类型别名
#[derive(Debug, Clone, PartialEq)]
pub struct TsTypeAlias {
    pub name: String,
    pub type_ref: TypeRef,
    pub exported: bool,
    pub comments: Vec<Comment>,
}

/// TypeScript 变量/常量声明
#[derive(Debug, Clone, PartialEq)]
pub struct TsVariable {
    pub name: String,
    pub kind: TsVariableKind,
    pub type_annotation: Option<TypeRef>,
    pub initializer: Option<TsExpression>,
    pub exported: bool,
    pub comments: Vec<Comment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TsVariableKind {
    Const,
    Let,
    Var,
}

/// TypeScript 枚举声明
#[derive(Debug, Clone, PartialEq)]
pub struct TsEnum {
    pub name: String,
    pub members: Vec<TsEnumMember>,
    pub exported: bool,
    pub comments: Vec<Comment>,
    pub is_const: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TsEnumMember {
    pub name: String,
    pub value: Option<LiteralValue>,
    pub comments: Vec<Comment>,
}

/// TypeScript 函数声明
#[derive(Debug, Clone, PartialEq)]
pub struct TsFunction {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<TypeRef>,
    pub body: Vec<TsStatement>,
    pub exported: bool,
    pub async_: bool,
    pub comments: Vec<Comment>,
}

/// TypeScript 表达式
#[derive(Debug, Clone, PartialEq)]
pub enum TsExpression {
    /// 标识符引用
    Ident(String),
    /// 字面量
    Literal(LiteralValue),
    /// 属性访问: obj.prop
    MemberAccess {
        object: Box<TsExpression>,
        property: String,
    },
    /// 调用表达式: fn(args)
    Call {
        callee: Box<TsExpression>,
        args: Vec<TsExpression>,
    },
    /// 对象字面量
    ObjectLiteral(Vec<TsPropertyAssignment>),
    /// 箭头函数
    ArrowFunction {
        params: Vec<Parameter>,
        return_type: Option<TypeRef>,
        body: Box<TsExpression>,
    },
    /// 类型断言: expr as Type
    TypeAssertion {
        expr: Box<TsExpression>,
        type_ref: TypeRef,
    },
    /// 数组字面量
    ArrayLiteral(Vec<TsExpression>),
    /// 模板字符串（简化）
    TemplateString {
        parts: Vec<String>,
        expressions: Vec<TsExpression>,
    },
    /// typeof 表达式（用于 Zod schema 引用）
    TypeOf(Box<TsExpression>),
    /// 二元表达式
    Binary {
        left: Box<TsExpression>,
        op: TsBinaryOp,
        right: Box<TsExpression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TsPropertyAssignment {
    pub name: String,
    pub value: TsExpression,
    pub optional: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TsBinaryOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Add,
    Sub,
    Mul,
    Div,
}

// ============================================================================
// TypeScript 渲染器
// ============================================================================

pub struct TypeScriptEmitter;

impl TypeScriptEmitter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TypeScriptEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl AstEmitter<TypeScriptAst> for TypeScriptEmitter {
    fn emit(&self, ast: &TypeScriptAst) -> Result<String, EmitError> {
        self.emit_with_ctx(ast, &EmitContext::default())
    }

    fn emit_with_ctx(&self, ast: &TypeScriptAst, ctx: &EmitContext) -> Result<String, EmitError> {
        let mut output = String::new();

        // 文件级注释
        emit_comments(&mut output, ctx, &ast.comments, CommentStyle::SlashSlash)?;

        // 导入声明
        for import in &ast.imports {
            self.emit_import(&mut output, ctx, import)?;
        }
        if !ast.imports.is_empty() && !ast.statements.is_empty() {
            writeln!(&mut output)?;
        }

        // 语句
        for (i, stmt) in ast.statements.iter().enumerate() {
            if i > 0 {
                writeln!(&mut output)?;
            }
            self.emit_statement(&mut output, ctx, stmt)?;
        }

        Ok(output)
    }
}

impl TypeScriptEmitter {
    fn emit_import<W: Write>(
        &self,
        w: &mut W,
        ctx: &EmitContext,
        import: &TsImport,
    ) -> Result<(), EmitError> {
        emit_comments(w, ctx, &import.comments, CommentStyle::SlashSlash)?;
        ctx.write_indent(w)?;

        write!(w, "import ")?;

        if let Some(ns) = &import.namespace {
            write!(w, "* as {} from '{}'", ns, import.module)?;
        } else if let Some(default) = &import.default {
            if import.named.is_empty() {
                write!(w, "{} from '{}'", default, import.module)?;
            } else {
                write!(w, "{default}, {{ ")?;
                for (i, n) in import.named.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    if let Some(alias) = &n.alias {
                        write!(w, "{} as {}", n.name, alias)?;
                    } else {
                        write!(w, "{}", n.name)?;
                    }
                }
                write!(w, " }} from '{}'", import.module)?;
            }
        } else if !import.named.is_empty() {
            write!(w, "{{ ")?;
            for (i, n) in import.named.iter().enumerate() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                if let Some(alias) = &n.alias {
                    write!(w, "{} as {}", n.name, alias)?;
                } else {
                    write!(w, "{}", n.name)?;
                }
            }
            write!(w, " }} from '{}'", import.module)?;
        }

        writeln!(w, ";")?;
        Ok(())
    }

    fn emit_statement<W: Write>(
        &self,
        w: &mut W,
        ctx: &EmitContext,
        stmt: &TsStatement,
    ) -> Result<(), EmitError> {
        match stmt {
            TsStatement::Interface(iface) => self.emit_interface(w, ctx, iface)?,
            TsStatement::TypeAlias(alias) => self.emit_type_alias(w, ctx, alias)?,
            TsStatement::Variable(var) => self.emit_variable(w, ctx, var)?,
            TsStatement::Enum(enm) => self.emit_enum(w, ctx, enm)?,
            TsStatement::Function(func) => self.emit_function(w, ctx, func)?,
            TsStatement::Export(items) => {
                ctx.write_indent(w)?;
                writeln!(w, "export {{ {} }};", items.join(", "))?;
            }
            TsStatement::Expression(expr) => {
                ctx.write_indent(w)?;
                self.emit_expression(w, ctx, expr)?;
                writeln!(w, ";")?;
            }
            TsStatement::CommentBlock(comments) => {
                emit_comments(w, ctx, comments, CommentStyle::SlashSlash)?;
            }
        }
        Ok(())
    }

    fn emit_interface<W: Write>(
        &self,
        w: &mut W,
        ctx: &EmitContext,
        iface: &TsInterface,
    ) -> Result<(), EmitError> {
        emit_comments(w, ctx, &iface.comments, CommentStyle::JsDoc)?;
        ctx.write_indent(w)?;

        if iface.exported {
            write!(w, "export ")?;
        }
        write!(w, "interface {} ", iface.name)?;

        if !iface.extends.is_empty() {
            write!(w, "extends {} ", iface.extends.join(", "))?;
        }

        writeln!(w, "{{")?;

        let nested = ctx.nested();
        for prop in &iface.properties {
            self.emit_property(w, &nested, prop)?;
        }

        ctx.write_indent(w)?;
        writeln!(w, "}}")?;
        Ok(())
    }

    fn emit_property<W: Write>(
        &self,
        w: &mut W,
        ctx: &EmitContext,
        prop: &PropertyDef,
    ) -> Result<(), EmitError> {
        emit_comments(w, ctx, &prop.comments, CommentStyle::JsDoc)?;
        ctx.write_indent(w)?;

        if prop.readonly {
            write!(w, "readonly ")?;
        }
        write!(w, "{}", prop.name)?;
        if prop.optional {
            write!(w, "?")?;
        }
        if let Some(ty) = &prop.type_ref {
            write!(w, ": ")?;
            emit_type_ref(w, ty)?;
        }
        writeln!(w, ";")?;
        Ok(())
    }

    fn emit_type_alias<W: Write>(
        &self,
        w: &mut W,
        ctx: &EmitContext,
        alias: &TsTypeAlias,
    ) -> Result<(), EmitError> {
        emit_comments(w, ctx, &alias.comments, CommentStyle::JsDoc)?;
        ctx.write_indent(w)?;

        if alias.exported {
            write!(w, "export ")?;
        }
        write!(w, "type {} = ", alias.name)?;
        emit_type_ref(w, &alias.type_ref)?;
        writeln!(w, ";")?;
        Ok(())
    }

    fn emit_variable<W: Write>(
        &self,
        w: &mut W,
        ctx: &EmitContext,
        var: &TsVariable,
    ) -> Result<(), EmitError> {
        emit_comments(w, ctx, &var.comments, CommentStyle::JsDoc)?;
        ctx.write_indent(w)?;

        if var.exported {
            write!(w, "export ")?;
        }

        let kind_str = match var.kind {
            TsVariableKind::Const => "const",
            TsVariableKind::Let => "let",
            TsVariableKind::Var => "var",
        };
        write!(w, "{} {}", kind_str, var.name)?;

        if let Some(ty) = &var.type_annotation {
            write!(w, ": ")?;
            emit_type_ref(w, ty)?;
        }

        if let Some(init) = &var.initializer {
            write!(w, " = ")?;
            self.emit_expression(w, ctx, init)?;
        }

        writeln!(w, ";")?;
        Ok(())
    }

    fn emit_enum<W: Write>(
        &self,
        w: &mut W,
        ctx: &EmitContext,
        enm: &TsEnum,
    ) -> Result<(), EmitError> {
        emit_comments(w, ctx, &enm.comments, CommentStyle::JsDoc)?;
        ctx.write_indent(w)?;

        if enm.exported {
            write!(w, "export ")?;
        }
        if enm.is_const {
            write!(w, "const ")?;
        }
        writeln!(w, "enum {} {{", enm.name)?;

        let nested = ctx.nested();
        for member in &enm.members {
            emit_comments(w, &nested, &member.comments, CommentStyle::SlashSlash)?;
            nested.write_indent(w)?;
            write!(w, "{}", member.name)?;
            if let Some(val) = &member.value {
                write!(w, " = ")?;
                emit_literal(w, val)?;
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
        func: &TsFunction,
    ) -> Result<(), EmitError> {
        emit_comments(w, ctx, &func.comments, CommentStyle::JsDoc)?;
        ctx.write_indent(w)?;

        if func.exported {
            write!(w, "export ")?;
        }
        if func.async_ {
            write!(w, "async ")?;
        }
        write!(w, "function {}(", func.name)?;

        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                write!(w, ", ")?;
            }
            if param.mutable {
                // TS 中没有 mut 关键字
            }
            write!(w, "{}", param.name)?;
            if let Some(ty) = &param.type_ref {
                write!(w, ": ")?;
                emit_type_ref(w, ty)?;
            }
            if let Some(default) = &param.default_value {
                write!(w, " = ")?;
                emit_literal(w, default)?;
            }
        }

        write!(w, ")")?;
        if let Some(rt) = &func.return_type {
            write!(w, ": ")?;
            emit_type_ref(w, rt)?;
        }

        writeln!(w, " {{")?;

        let nested = ctx.nested();
        for stmt in &func.body {
            self.emit_statement(w, &nested, stmt)?;
        }

        ctx.write_indent(w)?;
        writeln!(w, "}}")?;
        Ok(())
    }

    fn emit_expression<W: Write>(
        &self,
        w: &mut W,
        _ctx: &EmitContext,
        expr: &TsExpression,
    ) -> Result<(), EmitError> {
        match expr {
            TsExpression::Ident(name) => {
                write!(w, "{}", name)?;
            }
            TsExpression::Literal(lit) => {
                emit_literal(w, lit)?;
            }
            TsExpression::MemberAccess { object, property } => {
                self.emit_expression(w, _ctx, object)?;
                write!(w, ".{}", property)?;
            }
            TsExpression::Call { callee, args } => {
                self.emit_expression(w, _ctx, callee)?;
                write!(w, "(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    self.emit_expression(w, _ctx, arg)?;
                }
                write!(w, ")")?;
            }
            TsExpression::ObjectLiteral(props) => {
                if props.is_empty() {
                    write!(w, "{{}}")?;
                } else {
                    writeln!(w, "{{")?;
                    let nested = _ctx.nested();
                    for (i, prop) in props.iter().enumerate() {
                        nested.write_indent(w)?;
                        write!(w, "{}", prop.name)?;
                        if prop.optional {
                            write!(w, "?")?;
                        }
                        write!(w, ": ")?;
                        self.emit_expression(w, &nested, &prop.value)?;
                        if i < props.len() - 1 {
                            writeln!(w, ",")?;
                        } else {
                            writeln!(w)?;
                        }
                    }
                    _ctx.write_indent(w)?;
                    write!(w, "}}")?;
                }
            }
            TsExpression::ArrowFunction {
                params,
                return_type,
                body,
            } => {
                write!(w, "(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    write!(w, "{}", param.name)?;
                    if let Some(ty) = &param.type_ref {
                        write!(w, ": ")?;
                        emit_type_ref(w, ty)?;
                    }
                }
                write!(w, ")")?;
                if let Some(rt) = return_type {
                    write!(w, ": ")?;
                    emit_type_ref(w, rt)?;
                }
                write!(w, " => ")?;
                self.emit_expression(w, _ctx, body)?;
            }
            TsExpression::TypeAssertion { expr, type_ref } => {
                write!(w, "(")?;
                self.emit_expression(w, _ctx, expr)?;
                write!(w, " as ")?;
                emit_type_ref(w, type_ref)?;
                write!(w, ")")?;
            }
            TsExpression::ArrayLiteral(items) => {
                write!(w, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    self.emit_expression(w, _ctx, item)?;
                }
                write!(w, "]")?;
            }
            TsExpression::TemplateString { parts, expressions } => {
                write!(w, "`")?;
                for (i, part) in parts.iter().enumerate() {
                    write!(w, "{}", part)?;
                    if let Some(expr) = expressions.get(i) {
                        write!(w, "${{")?;
                        self.emit_expression(w, _ctx, expr)?;
                        write!(w, "}}")?;
                    }
                }
                write!(w, "`")?;
            }
            TsExpression::TypeOf(expr) => {
                write!(w, "typeof ")?;
                self.emit_expression(w, _ctx, expr)?;
            }
            TsExpression::Binary { left, op, right } => {
                self.emit_expression(w, _ctx, left)?;
                let op_str = match op {
                    TsBinaryOp::Eq => " === ",
                    TsBinaryOp::Ne => " !== ",
                    TsBinaryOp::Lt => " < ",
                    TsBinaryOp::Le => " <= ",
                    TsBinaryOp::Gt => " > ",
                    TsBinaryOp::Ge => " >= ",
                    TsBinaryOp::And => " && ",
                    TsBinaryOp::Or => " || ",
                    TsBinaryOp::Add => " + ",
                    TsBinaryOp::Sub => " - ",
                    TsBinaryOp::Mul => " * ",
                    TsBinaryOp::Div => " / ",
                };
                write!(w, "{}", op_str)?;
                self.emit_expression(w, _ctx, right)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_simple_import() {
        let ast = TypeScriptAst::new("test.ts").with_import(TsImport::default("zod", "z"));

        let emitter = TypeScriptEmitter;
        let output = emitter.emit(&ast).unwrap();
        assert!(output.contains("import z from 'zod';"));
    }

    #[test]
    fn test_emit_interface() {
        let ast = TypeScriptAst::new("test.ts").with_statement(TsStatement::Interface(
            TsInterface::new("User")
                .property(PropertyDef::new("id").with_type(TypeRef::named("string")))
                .property(PropertyDef::new("email").with_type(TypeRef::named("string"))),
        ));

        let emitter = TypeScriptEmitter;
        let output = emitter.emit(&ast).unwrap();
        assert!(output.contains("export interface User {"));
        assert!(output.contains("id: string;"));
        assert!(output.contains("email: string;"));
    }

    #[test]
    fn test_emit_variable_with_initializer() {
        let ast = TypeScriptAst::new("test.ts").with_statement(TsStatement::Variable(TsVariable {
            name: "UserSchema".to_string(),
            kind: TsVariableKind::Const,
            type_annotation: None,
            initializer: Some(TsExpression::Call {
                callee: Box::new(TsExpression::MemberAccess {
                    object: Box::new(TsExpression::Ident("z".to_string())),
                    property: "object".to_string(),
                }),
                args: vec![TsExpression::ObjectLiteral(vec![])],
            }),
            exported: true,
            comments: vec![],
        }));

        let emitter = TypeScriptEmitter;
        let output = emitter.emit(&ast).unwrap();
        assert!(output.contains("export const UserSchema = z.object({"));
    }

    #[test]
    fn test_ast_equality() {
        let ast1 = TypeScriptAst::new("a.ts")
            .with_import(TsImport::new("zod").add_named(TsImportNamed::new("z")))
            .with_statement(TsStatement::Interface(TsInterface::new("User")));

        let ast2 = TypeScriptAst::new("a.ts")
            .with_import(TsImport::new("zod").add_named(TsImportNamed::new("z")))
            .with_statement(TsStatement::Interface(TsInterface::new("User")));

        assert_eq!(ast1, ast2);
    }
}
