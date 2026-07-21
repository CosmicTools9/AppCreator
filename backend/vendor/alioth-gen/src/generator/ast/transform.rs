//! IR-2 → AST 转换器
//!
//! 将结构化 IR-2 模型转换为语言特定的 AST 表示。
//! 这是代码生成的第一阶段，输出是可测试、可序列化的 AST。
//!
//! GeneratorModel ──► TypeScriptAst / RustAst ──► String
//! ```

use crate::generator::ir::{GeneratorEntity, GeneratorFieldType, GeneratorModel, PrimaryKeyType};

use super::{
    nodes::{Comment, LiteralValue, Parameter, PropertyDef, TypeRef},
    rust::{RustAst, RustExpression, RustFunction, RustItem, RustMatchArm, RustStatement, RustUse},
    ts::{
        TsEnum, TsEnumMember, TsExpression, TsImport, TsInterface, TsPropertyAssignment,
        TsStatement, TsVariable, TsVariableKind, TypeScriptAst,
    },
};

// ============================================================================
// TypeScript 转换器
// ============================================================================

/// 将 IR-2 实体转换为 TypeScript Zod Schema AST
pub fn entity_to_ts_zod_ast(entity: &GeneratorEntity) -> TypeScriptAst {
    let mut ast = TypeScriptAst::new(format!("{}.schema.ts", entity.name.kebab));

    // import { z } from 'zod';
    ast.imports.push(TsImport::default("zod", "z"));

    // Enum 类型声明
    for field in &entity.fields {
        if let GeneratorFieldType::Enum(name) = &field.field_type {
            ast.statements.push(TsStatement::Enum(TsEnum {
                name: name.clone(),
                members: vec![
                    // 占位成员，实际值应从 IR-2 扩展
                    TsEnumMember {
                        name: format!("{}Variant", name),
                        value: None,
                        comments: vec![Comment::line("TODO: define enum values")],
                    },
                ],
                exported: true,
                comments: vec![Comment::doc(format!("Enum type for {}", name))],
                is_const: true,
            }));
        }
    }

    // TypeScript interface
    let mut iface = TsInterface::new(&entity.name.pascal).exported(true);

    // ID field
    let id_type = match entity.primary_key_type {
        PrimaryKeyType::BigInt => TypeRef::named("bigint"),
        PrimaryKeyType::Uuid => TypeRef::named("string"),
    };
    iface = iface.property(PropertyDef::new("id").with_type(id_type).readonly());

    // Regular fields
    for field in &entity.fields {
        if field.name.snake == "id" {
            continue;
        }
        let ts_type = field_type_to_ts(&field.field_type);
        let mut prop = PropertyDef::new(&field.name.camel).with_type(ts_type);
        if field.nullable {
            prop = prop.optional();
        }
        if let Some(desc) = &field.description {
            prop = prop.with_comment(desc.clone());
        }
        iface = iface.property(prop);
    }

    // System fields
    iface = iface.property(
        PropertyDef::new("createdAt")
            .with_type(TypeRef::named("Date"))
            .readonly(),
    );
    iface = iface.property(
        PropertyDef::new("updatedAt")
            .with_type(TypeRef::named("Date"))
            .readonly(),
    );

    if let Some(desc) = &entity.description {
        iface.comments.push(Comment::doc(desc.clone()));
    }
    ast.statements.push(TsStatement::Interface(iface));

    // Input interface (for create/update)
    let mut input_iface = TsInterface::new(format!("{}Input", entity.name.pascal)).exported(true);

    for field in &entity.fields {
        if field.name.snake == "id"
            || field.name.snake.starts_with("created_")
            || field.name.snake.starts_with("updated_")
        {
            continue;
        }
        let ts_type = field_type_to_ts(&field.field_type);
        let mut prop = PropertyDef::new(&field.name.camel).with_type(ts_type);
        if field.nullable {
            prop = prop.optional();
        }
        if let Some(desc) = &field.description {
            prop = prop.with_comment(desc.clone());
        }
        input_iface = input_iface.property(prop);
    }
    ast.statements.push(TsStatement::Interface(input_iface));

    // Zod schema: export const UserSchema = z.object({ ... })
    let schema_expr = TsExpression::Call {
        callee: Box::new(TsExpression::MemberAccess {
            object: Box::new(TsExpression::Ident("z".to_string())),
            property: "object".to_string(),
        }),
        args: vec![TsExpression::ObjectLiteral(build_zod_schema_fields(entity))],
    };

    ast.statements.push(TsStatement::Variable(TsVariable {
        name: format!("{}Schema", entity.name.pascal),
        kind: TsVariableKind::Const,
        type_annotation: None,
        initializer: Some(schema_expr),
        exported: true,
        comments: vec![Comment::doc(format!("Zod schema for {}", entity.name.raw))],
    }));

    // Input schema: export const UserInputSchema = z.object({ ... })
    let input_schema_expr = TsExpression::Call {
        callee: Box::new(TsExpression::MemberAccess {
            object: Box::new(TsExpression::Ident("z".to_string())),
            property: "object".to_string(),
        }),
        args: vec![TsExpression::ObjectLiteral(build_zod_input_fields(entity))],
    };

    ast.statements.push(TsStatement::Variable(TsVariable {
        name: format!("{}InputSchema", entity.name.pascal),
        kind: TsVariableKind::Const,
        type_annotation: None,
        initializer: Some(input_schema_expr),
        exported: true,
        comments: vec![Comment::doc(format!(
            "Zod input schema for {}",
            entity.name.raw
        ))],
    }));

    // Type inference helpers
    ast.statements
        .push(TsStatement::CommentBlock(vec![Comment::line(
            "Type inference helpers",
        )]));

    // export type User = z.infer<typeof UserSchema>;
    ast.statements
        .push(TsStatement::TypeAlias(super::ts::TsTypeAlias {
            name: entity.name.pascal.clone(),
            type_ref: TypeRef::generic(
                "z.infer",
                vec![TypeRef::named(format!(
                    "typeof {}Schema",
                    entity.name.pascal
                ))],
            ),
            exported: true,
            comments: vec![],
        }));

    ast.statements
        .push(TsStatement::TypeAlias(super::ts::TsTypeAlias {
            name: format!("{}Input", entity.name.pascal),
            type_ref: TypeRef::generic(
                "z.infer",
                vec![TypeRef::named(format!(
                    "typeof {}InputSchema",
                    entity.name.pascal
                ))],
            ),
            exported: true,
            comments: vec![],
        }));

    // Safe parse helpers
    let parse_fn_expr = TsExpression::ArrowFunction {
        params: vec![Parameter::new("data").with_type(TypeRef::named("unknown"))],
        return_type: Some(TypeRef::named(format!(
            "z.SafeParseReturnType<unknown, {}>",
            entity.name.pascal
        ))),
        body: Box::new(TsExpression::Call {
            callee: Box::new(TsExpression::MemberAccess {
                object: Box::new(TsExpression::Ident(format!("{}Schema", entity.name.pascal))),
                property: "safeParse".to_string(),
            }),
            args: vec![TsExpression::Ident("data".to_string())],
        }),
    };

    ast.statements.push(TsStatement::Variable(TsVariable {
        name: format!("parse{}", entity.name.pascal),
        kind: TsVariableKind::Const,
        type_annotation: None,
        initializer: Some(parse_fn_expr),
        exported: true,
        comments: vec![],
    }));

    ast
}

/// 将完整的 GeneratorModel 转换为 TypeScript schemas AST
pub fn model_to_ts_schemas_ast(model: &GeneratorModel) -> Vec<TypeScriptAst> {
    model.entities.iter().map(entity_to_ts_zod_ast).collect()
}

fn field_type_to_ts(field_type: &GeneratorFieldType) -> TypeRef {
    match field_type {
        GeneratorFieldType::Text => TypeRef::named("string"),
        GeneratorFieldType::Integer => TypeRef::named("number"),
        GeneratorFieldType::BigInt => TypeRef::named("bigint"),
        GeneratorFieldType::Decimal => TypeRef::named("number"),
        GeneratorFieldType::Boolean => TypeRef::named("boolean"),
        GeneratorFieldType::DateTime => TypeRef::named("Date"),
        GeneratorFieldType::Uuid => TypeRef::named("string"),
        GeneratorFieldType::Json => TypeRef::named("Record<string, unknown>"),
        GeneratorFieldType::Enum(name) => TypeRef::named(name.clone()),
        GeneratorFieldType::Reference(name) => TypeRef::named(name.clone()),
    }
}

fn build_zod_schema_fields(entity: &GeneratorEntity) -> Vec<TsPropertyAssignment> {
    let mut fields = Vec::new();

    // ID field
    let id_schema = match entity.primary_key_type {
        PrimaryKeyType::BigInt => TsExpression::MemberAccess {
            object: Box::new(TsExpression::Ident("z".to_string())),
            property: "bigint".to_string(),
        },
        PrimaryKeyType::Uuid => TsExpression::Call {
            callee: Box::new(TsExpression::MemberAccess {
                object: Box::new(TsExpression::MemberAccess {
                    object: Box::new(TsExpression::Ident("z".to_string())),
                    property: "string".to_string(),
                }),
                property: "uuid".to_string(),
            }),
            args: vec![],
        },
    };
    fields.push(TsPropertyAssignment {
        name: "id".to_string(),
        value: id_schema,
        optional: false,
    });

    for field in &entity.fields {
        if field.name.snake == "id" {
            continue;
        }
        let zod_expr = field_type_to_zod_expr(&field.field_type, field.nullable);
        fields.push(TsPropertyAssignment {
            name: field.name.camel.clone(),
            value: zod_expr,
            optional: false,
        });
    }

    // System fields
    fields.push(TsPropertyAssignment {
        name: "createdAt".to_string(),
        value: TsExpression::MemberAccess {
            object: Box::new(TsExpression::Ident("z".to_string())),
            property: "date".to_string(),
        },
        optional: false,
    });
    fields.push(TsPropertyAssignment {
        name: "updatedAt".to_string(),
        value: TsExpression::MemberAccess {
            object: Box::new(TsExpression::Ident("z".to_string())),
            property: "date".to_string(),
        },
        optional: false,
    });

    fields
}

fn build_zod_input_fields(entity: &GeneratorEntity) -> Vec<TsPropertyAssignment> {
    let mut fields = Vec::new();

    for field in &entity.fields {
        if field.name.snake == "id"
            || field.name.snake.starts_with("created_")
            || field.name.snake.starts_with("updated_")
        {
            continue;
        }
        let zod_expr = field_type_to_zod_expr(&field.field_type, field.nullable);
        fields.push(TsPropertyAssignment {
            name: field.name.camel.clone(),
            value: zod_expr,
            optional: field.nullable,
        });
    }

    fields
}

fn field_type_to_zod_expr(field_type: &GeneratorFieldType, nullable: bool) -> TsExpression {
    let base = match field_type {
        GeneratorFieldType::Text => TsExpression::MemberAccess {
            object: Box::new(TsExpression::Ident("z".to_string())),
            property: "string".to_string(),
        },
        GeneratorFieldType::Integer => TsExpression::MemberAccess {
            object: Box::new(TsExpression::Ident("z".to_string())),
            property: "number".to_string(),
        },
        GeneratorFieldType::BigInt => TsExpression::MemberAccess {
            object: Box::new(TsExpression::Ident("z".to_string())),
            property: "bigint".to_string(),
        },
        GeneratorFieldType::Decimal => TsExpression::MemberAccess {
            object: Box::new(TsExpression::Ident("z".to_string())),
            property: "number".to_string(),
        },
        GeneratorFieldType::Boolean => TsExpression::MemberAccess {
            object: Box::new(TsExpression::Ident("z".to_string())),
            property: "boolean".to_string(),
        },
        GeneratorFieldType::DateTime => TsExpression::MemberAccess {
            object: Box::new(TsExpression::Ident("z".to_string())),
            property: "date".to_string(),
        },
        GeneratorFieldType::Uuid => TsExpression::Call {
            callee: Box::new(TsExpression::MemberAccess {
                object: Box::new(TsExpression::MemberAccess {
                    object: Box::new(TsExpression::Ident("z".to_string())),
                    property: "string".to_string(),
                }),
                property: "uuid".to_string(),
            }),
            args: vec![],
        },
        GeneratorFieldType::Json => TsExpression::MemberAccess {
            object: Box::new(TsExpression::Ident("z".to_string())),
            property: "record".to_string(),
        },
        GeneratorFieldType::Enum(name) => TsExpression::Ident(name.clone()),
        GeneratorFieldType::Reference(name) => TsExpression::Ident(name.clone()),
    };

    if nullable {
        TsExpression::Call {
            callee: Box::new(TsExpression::MemberAccess {
                object: Box::new(base),
                property: "nullable".to_string(),
            }),
            args: vec![],
        }
    } else {
        base
    }
}

// ============================================================================
// Rust 转换器
// ============================================================================

/// 将 IR-2 实体转换为 Rust Handler AST
pub fn entity_to_rust_handler_ast(entity: &GeneratorEntity) -> RustAst {
    let mut ast = RustAst::new(format!("handlers/{}.rs", entity.name.snake));

    let entity_name = &entity.name.pascal;
    let entity_snake = &entity.name.snake;
    let _entity_plural = &entity.name.plural_snake;
    let id_type = match entity.primary_key_type {
        PrimaryKeyType::BigInt => "i64",
        PrimaryKeyType::Uuid => "Uuid",
    };

    // use declarations
    ast.items.push(RustItem::Use(RustUse::items(
        "actix_web",
        vec![
            "web",
            "HttpRequest",
            "HttpResponse",
            "Result",
            "HttpMessage",
        ],
    )));
    ast.items
        .push(RustItem::Use(RustUse::simple("serde_json::json")));
    ast.items
        .push(RustItem::Use(RustUse::simple("sqlx::PgPool")));
    ast.items.push(RustItem::Use(RustUse::items(
        format!("crate::models::{}", entity_snake),
        vec![entity_name, &format!("{}Input", entity_name)],
    )));
    ast.items
        .push(RustItem::Use(RustUse::simple("crate::errors::ApiError")));
    ast.items.push(RustItem::Use(RustUse::items(
        format!("crate::repositories::{}", entity_snake),
        vec![&format!("{}Repository", entity_name)],
    )));

    // extract_user_id helper
    ast.items.push(RustItem::Function(
        RustFunction::new("extract_user_id")
            .param(Parameter::new("req").with_type(TypeRef::reference("HttpRequest", false)))
            .returns(TypeRef::optional(TypeRef::named("i64")))
            .body_stmt(RustStatement::Expression(RustExpression::MethodCall {
                receiver: Box::new(RustExpression::Call {
                    callee: Box::new(RustExpression::Path(vec![
                        "common".to_string(),
                        "context".to_string(),
                        "extract_user_id".to_string(),
                    ])),
                    args: vec![RustExpression::Ident("req".to_string())],
                }),
                method: "or_else".to_string(),
                args: vec![RustExpression::Closure {
                    params: vec![],
                    body: Box::new(RustExpression::MethodCall {
                        receiver: Box::new(RustExpression::MethodCall {
                            receiver: Box::new(RustExpression::MethodCall {
                                receiver: Box::new(RustExpression::Ident("req".to_string())),
                                method: "extensions".to_string(),
                                args: vec![],
                            }),
                            method: "get".to_string(),
                            args: vec![],
                        }),
                        method: "copied".to_string(),
                        args: vec![],
                    }),
                    async_: false,
                }],
            })),
    ));

    // require_auth helper
    ast.items.push(RustItem::Function(
        RustFunction::new("require_auth")
            .param(Parameter::new("req").with_type(TypeRef::reference("HttpRequest", false)))
            .returns(TypeRef::generic(
                "Result",
                vec![TypeRef::named("i64"), TypeRef::named("ApiError")],
            ))
            .body_stmt(RustStatement::Expression(RustExpression::MethodCall {
                receiver: Box::new(RustExpression::Call {
                    callee: Box::new(RustExpression::Ident("extract_user_id".to_string())),
                    args: vec![RustExpression::Ident("req".to_string())],
                }),
                method: "ok_or_else".to_string(),
                args: vec![RustExpression::Closure {
                    params: vec![],
                    body: Box::new(RustExpression::Call {
                        callee: Box::new(RustExpression::Path(vec![
                            "ApiError".to_string(),
                            "Unauthorized".to_string(),
                        ])),
                        args: vec![RustExpression::Literal(LiteralValue::String(
                            "Authentication required".to_string(),
                        ))],
                    }),
                    async_: false,
                }],
            })),
    ));

    // list handler
    ast.items.push(RustItem::Function(
        RustFunction::new(format!("list_{}", entity_snake))
            .async_()
            .param(Parameter::new("pool").with_type(TypeRef::generic(
                "web::Data",
                vec![TypeRef::named("PgPool")],
            )))
            .returns(TypeRef::generic(
                "Result",
                vec![TypeRef::named("HttpResponse"), TypeRef::named("ApiError")],
            ))
            .body_stmt(RustStatement::Let {
                name: "repo".to_string(),
                type_hint: None,
                value: RustExpression::Call {
                    callee: Box::new(RustExpression::Path(vec![
                        format!("{}Repository", entity_name),
                        "new".to_string(),
                    ])),
                    args: vec![RustExpression::MethodCall {
                        receiver: Box::new(RustExpression::Ident("pool".to_string())),
                        method: "get_ref".to_string(),
                        args: vec![],
                    }],
                },
                mutable: false,
            })
            .body_stmt(RustStatement::Let {
                name: "items".to_string(),
                type_hint: None,
                value: RustExpression::Await(Box::new(RustExpression::MethodCall {
                    receiver: Box::new(RustExpression::Ident("repo".to_string())),
                    method: "list".to_string(),
                    args: vec![],
                })),
                mutable: false,
            })
            .body_stmt(RustStatement::Return(Some(RustExpression::Call {
                callee: Box::new(RustExpression::MethodCall {
                    receiver: Box::new(RustExpression::Ident("HttpResponse".to_string())),
                    method: "Ok".to_string(),
                    args: vec![],
                }),
                args: vec![RustExpression::MethodCall {
                    receiver: Box::new(RustExpression::Ident("items".to_string())),
                    method: "json".to_string(),
                    args: vec![],
                }],
            }))),
    ));

    // get handler
    ast.items.push(RustItem::Function(
        RustFunction::new(format!("get_{}", entity_snake))
            .async_()
            .param(Parameter::new("pool").with_type(TypeRef::generic(
                "web::Data",
                vec![TypeRef::named("PgPool")],
            )))
            .param(
                Parameter::new("path")
                    .with_type(TypeRef::generic("web::Path", vec![TypeRef::named(id_type)])),
            )
            .returns(TypeRef::generic(
                "Result",
                vec![TypeRef::named("HttpResponse"), TypeRef::named("ApiError")],
            ))
            .body_stmt(RustStatement::Let {
                name: "id".to_string(),
                type_hint: None,
                value: RustExpression::MethodCall {
                    receiver: Box::new(RustExpression::Ident("path".to_string())),
                    method: "into_inner".to_string(),
                    args: vec![],
                },
                mutable: false,
            })
            .body_stmt(RustStatement::Let {
                name: "repo".to_string(),
                type_hint: None,
                value: RustExpression::Call {
                    callee: Box::new(RustExpression::Path(vec![
                        format!("{}Repository", entity_name),
                        "new".to_string(),
                    ])),
                    args: vec![RustExpression::MethodCall {
                        receiver: Box::new(RustExpression::Ident("pool".to_string())),
                        method: "get_ref".to_string(),
                        args: vec![],
                    }],
                },
                mutable: false,
            })
            .body_stmt(RustStatement::Let {
                name: "item".to_string(),
                type_hint: None,
                value: RustExpression::Await(Box::new(RustExpression::MethodCall {
                    receiver: Box::new(RustExpression::Ident("repo".to_string())),
                    method: "get_by_id".to_string(),
                    args: vec![RustExpression::Ident("id".to_string())],
                })),
                mutable: false,
            })
            .body_stmt(RustStatement::Expression(RustExpression::Match {
                expr: Box::new(RustExpression::Ident("item".to_string())),
                arms: vec![
                    RustMatchArm {
                        pattern: "Some(item)".to_string(),
                        guard: None,
                        body: RustExpression::Call {
                            callee: Box::new(RustExpression::MethodCall {
                                receiver: Box::new(RustExpression::Ident(
                                    "HttpResponse".to_string(),
                                )),
                                method: "Ok".to_string(),
                                args: vec![],
                            }),
                            args: vec![RustExpression::MethodCall {
                                receiver: Box::new(RustExpression::Ident("item".to_string())),
                                method: "json".to_string(),
                                args: vec![],
                            }],
                        },
                    },
                    RustMatchArm {
                        pattern: "None".to_string(),
                        guard: None,
                        body: RustExpression::Call {
                            callee: Box::new(RustExpression::Path(vec![
                                "ApiError".to_string(),
                                "NotFound".to_string(),
                            ])),
                            args: vec![RustExpression::Macro {
                                name: "format".to_string(),
                                tokens: format!("\"{} {{}} not found\", id", entity_name),
                            }],
                        },
                    },
                ],
            })),
    ));

    // create handler
    ast.items.push(RustItem::Function(
        RustFunction::new(format!("create_{}", entity_snake))
            .async_()
            .param(Parameter::new("pool").with_type(TypeRef::generic(
                "web::Data",
                vec![TypeRef::named("PgPool")],
            )))
            .param(Parameter::new("req").with_type(TypeRef::named("HttpRequest")))
            .param(Parameter::new("body").with_type(TypeRef::generic(
                "web::Json",
                vec![TypeRef::named(format!("{}Input", entity_name))],
            )))
            .returns(TypeRef::generic(
                "Result",
                vec![TypeRef::named("HttpResponse"), TypeRef::named("ApiError")],
            ))
            .body_stmt(RustStatement::Let {
                name: "fk_user".to_string(),
                type_hint: None,
                value: RustExpression::Call {
                    callee: Box::new(RustExpression::Ident("require_auth".to_string())),
                    args: vec![RustExpression::Reference {
                        mutable: false,
                        expr: Box::new(RustExpression::Ident("req".to_string())),
                    }],
                },
                mutable: false,
            })
            .body_stmt(RustStatement::Let {
                name: "input".to_string(),
                type_hint: None,
                value: RustExpression::MethodCall {
                    receiver: Box::new(RustExpression::Ident("body".to_string())),
                    method: "into_inner".to_string(),
                    args: vec![],
                },
                mutable: false,
            })
            .body_stmt(RustStatement::Let {
                name: "repo".to_string(),
                type_hint: None,
                value: RustExpression::Call {
                    callee: Box::new(RustExpression::Path(vec![
                        format!("{}Repository", entity_name),
                        "new".to_string(),
                    ])),
                    args: vec![RustExpression::MethodCall {
                        receiver: Box::new(RustExpression::Ident("pool".to_string())),
                        method: "get_ref".to_string(),
                        args: vec![],
                    }],
                },
                mutable: false,
            })
            .body_stmt(RustStatement::Let {
                name: "created".to_string(),
                type_hint: None,
                value: RustExpression::Await(Box::new(RustExpression::MethodCall {
                    receiver: Box::new(RustExpression::Ident("repo".to_string())),
                    method: "create".to_string(),
                    args: vec![
                        RustExpression::Ident("input".to_string()),
                        RustExpression::Ident("fk_user".to_string()),
                    ],
                })),
                mutable: false,
            })
            .body_stmt(RustStatement::Return(Some(RustExpression::Call {
                callee: Box::new(RustExpression::MethodCall {
                    receiver: Box::new(RustExpression::Ident("HttpResponse".to_string())),
                    method: "Created".to_string(),
                    args: vec![],
                }),
                args: vec![RustExpression::MethodCall {
                    receiver: Box::new(RustExpression::Ident("created".to_string())),
                    method: "json".to_string(),
                    args: vec![],
                }],
            }))),
    ));

    // update handler
    ast.items.push(RustItem::Function(
        RustFunction::new(format!("update_{}", entity_snake))
            .async_()
            .param(Parameter::new("pool").with_type(TypeRef::generic(
                "web::Data",
                vec![TypeRef::named("PgPool")],
            )))
            .param(Parameter::new("req").with_type(TypeRef::named("HttpRequest")))
            .param(
                Parameter::new("path")
                    .with_type(TypeRef::generic("web::Path", vec![TypeRef::named(id_type)])),
            )
            .param(Parameter::new("body").with_type(TypeRef::generic(
                "web::Json",
                vec![TypeRef::named(format!("{}Input", entity_name))],
            )))
            .returns(TypeRef::generic(
                "Result",
                vec![TypeRef::named("HttpResponse"), TypeRef::named("ApiError")],
            ))
            .body_stmt(RustStatement::Let {
                name: "id".to_string(),
                type_hint: None,
                value: RustExpression::MethodCall {
                    receiver: Box::new(RustExpression::Ident("path".to_string())),
                    method: "into_inner".to_string(),
                    args: vec![],
                },
                mutable: false,
            })
            .body_stmt(RustStatement::Let {
                name: "fk_user".to_string(),
                type_hint: None,
                value: RustExpression::Call {
                    callee: Box::new(RustExpression::Ident("require_auth".to_string())),
                    args: vec![RustExpression::Reference {
                        mutable: false,
                        expr: Box::new(RustExpression::Ident("req".to_string())),
                    }],
                },
                mutable: false,
            })
            .body_stmt(RustStatement::Let {
                name: "input".to_string(),
                type_hint: None,
                value: RustExpression::MethodCall {
                    receiver: Box::new(RustExpression::Ident("body".to_string())),
                    method: "into_inner".to_string(),
                    args: vec![],
                },
                mutable: false,
            })
            .body_stmt(RustStatement::Let {
                name: "repo".to_string(),
                type_hint: None,
                value: RustExpression::Call {
                    callee: Box::new(RustExpression::Path(vec![
                        format!("{}Repository", entity_name),
                        "new".to_string(),
                    ])),
                    args: vec![RustExpression::MethodCall {
                        receiver: Box::new(RustExpression::Ident("pool".to_string())),
                        method: "get_ref".to_string(),
                        args: vec![],
                    }],
                },
                mutable: false,
            })
            .body_stmt(RustStatement::Expression(RustExpression::Match {
                expr: Box::new(RustExpression::Await(Box::new(
                    RustExpression::MethodCall {
                        receiver: Box::new(RustExpression::Ident("repo".to_string())),
                        method: "update".to_string(),
                        args: vec![
                            RustExpression::Ident("id".to_string()),
                            RustExpression::Ident("input".to_string()),
                            RustExpression::Ident("fk_user".to_string()),
                        ],
                    },
                ))),
                arms: vec![
                    RustMatchArm {
                        pattern: "Some(item)".to_string(),
                        guard: None,
                        body: RustExpression::Call {
                            callee: Box::new(RustExpression::MethodCall {
                                receiver: Box::new(RustExpression::Ident(
                                    "HttpResponse".to_string(),
                                )),
                                method: "Ok".to_string(),
                                args: vec![],
                            }),
                            args: vec![RustExpression::MethodCall {
                                receiver: Box::new(RustExpression::Ident("item".to_string())),
                                method: "json".to_string(),
                                args: vec![],
                            }],
                        },
                    },
                    RustMatchArm {
                        pattern: "None".to_string(),
                        guard: None,
                        body: RustExpression::Call {
                            callee: Box::new(RustExpression::Path(vec![
                                "ApiError".to_string(),
                                "NotFound".to_string(),
                            ])),
                            args: vec![RustExpression::Macro {
                                name: "format".to_string(),
                                tokens: format!("\"{} {{}} not found\", id", entity_name),
                            }],
                        },
                    },
                ],
            })),
    ));

    // delete handler (soft delete)
    ast.items.push(RustItem::Function(
        RustFunction::new(format!("delete_{}", entity_snake))
            .async_()
            .param(Parameter::new("pool").with_type(TypeRef::generic(
                "web::Data",
                vec![TypeRef::named("PgPool")],
            )))
            .param(Parameter::new("req").with_type(TypeRef::named("HttpRequest")))
            .param(
                Parameter::new("path")
                    .with_type(TypeRef::generic("web::Path", vec![TypeRef::named(id_type)])),
            )
            .returns(TypeRef::generic(
                "Result",
                vec![TypeRef::named("HttpResponse"), TypeRef::named("ApiError")],
            ))
            .body_stmt(RustStatement::Let {
                name: "id".to_string(),
                type_hint: None,
                value: RustExpression::MethodCall {
                    receiver: Box::new(RustExpression::Ident("path".to_string())),
                    method: "into_inner".to_string(),
                    args: vec![],
                },
                mutable: false,
            })
            .body_stmt(RustStatement::Let {
                name: "fk_user".to_string(),
                type_hint: None,
                value: RustExpression::Call {
                    callee: Box::new(RustExpression::Ident("require_auth".to_string())),
                    args: vec![RustExpression::Reference {
                        mutable: false,
                        expr: Box::new(RustExpression::Ident("req".to_string())),
                    }],
                },
                mutable: false,
            })
            .body_stmt(RustStatement::Let {
                name: "repo".to_string(),
                type_hint: None,
                value: RustExpression::Call {
                    callee: Box::new(RustExpression::Path(vec![
                        format!("{}Repository", entity_name),
                        "new".to_string(),
                    ])),
                    args: vec![RustExpression::MethodCall {
                        receiver: Box::new(RustExpression::Ident("pool".to_string())),
                        method: "get_ref".to_string(),
                        args: vec![],
                    }],
                },
                mutable: false,
            })
            .body_stmt(RustStatement::Expression(RustExpression::Match {
                expr: Box::new(RustExpression::Await(Box::new(
                    RustExpression::MethodCall {
                        receiver: Box::new(RustExpression::Ident("repo".to_string())),
                        method: "soft_delete".to_string(),
                        args: vec![
                            RustExpression::Ident("id".to_string()),
                            RustExpression::Ident("fk_user".to_string()),
                        ],
                    },
                ))),
                arms: vec![
                    RustMatchArm {
                        pattern: "true".to_string(),
                        guard: None,
                        body: RustExpression::Call {
                            callee: Box::new(RustExpression::MethodCall {
                                receiver: Box::new(RustExpression::Ident(
                                    "HttpResponse".to_string(),
                                )),
                                method: "NoContent".to_string(),
                                args: vec![],
                            }),
                            args: vec![RustExpression::MethodCall {
                                receiver: Box::new(RustExpression::Ident(
                                    "HttpResponse".to_string(),
                                )),
                                method: "finish".to_string(),
                                args: vec![],
                            }],
                        },
                    },
                    RustMatchArm {
                        pattern: "false".to_string(),
                        guard: None,
                        body: RustExpression::Call {
                            callee: Box::new(RustExpression::Path(vec![
                                "ApiError".to_string(),
                                "NotFound".to_string(),
                            ])),
                            args: vec![RustExpression::Macro {
                                name: "format".to_string(),
                                tokens: format!("\"{} {{}} not found\", id", entity_name),
                            }],
                        },
                    },
                ],
            })),
    ));

    // hard_delete handler
    ast.items.push(RustItem::Function(
        RustFunction::new(format!("hard_delete_{}", entity_snake))
            .async_()
            .param(Parameter::new("pool").with_type(TypeRef::generic(
                "web::Data",
                vec![TypeRef::named("PgPool")],
            )))
            .param(Parameter::new("req").with_type(TypeRef::named("HttpRequest")))
            .param(
                Parameter::new("path")
                    .with_type(TypeRef::generic("web::Path", vec![TypeRef::named(id_type)])),
            )
            .returns(TypeRef::generic(
                "Result",
                vec![TypeRef::named("HttpResponse"), TypeRef::named("ApiError")],
            ))
            .body_stmt(RustStatement::Let {
                name: "id".to_string(),
                type_hint: None,
                value: RustExpression::MethodCall {
                    receiver: Box::new(RustExpression::Ident("path".to_string())),
                    method: "into_inner".to_string(),
                    args: vec![],
                },
                mutable: false,
            })
            .body_stmt(RustStatement::Let {
                name: "repo".to_string(),
                type_hint: None,
                value: RustExpression::Call {
                    callee: Box::new(RustExpression::Path(vec![
                        format!("{}Repository", entity_name),
                        "new".to_string(),
                    ])),
                    args: vec![RustExpression::MethodCall {
                        receiver: Box::new(RustExpression::Ident("pool".to_string())),
                        method: "get_ref".to_string(),
                        args: vec![],
                    }],
                },
                mutable: false,
            })
            .body_stmt(RustStatement::Expression(RustExpression::Match {
                expr: Box::new(RustExpression::Await(Box::new(
                    RustExpression::MethodCall {
                        receiver: Box::new(RustExpression::Ident("repo".to_string())),
                        method: "hard_delete".to_string(),
                        args: vec![RustExpression::Ident("id".to_string())],
                    },
                ))),
                arms: vec![
                    RustMatchArm {
                        pattern: "true".to_string(),
                        guard: None,
                        body: RustExpression::Call {
                            callee: Box::new(RustExpression::MethodCall {
                                receiver: Box::new(RustExpression::Ident(
                                    "HttpResponse".to_string(),
                                )),
                                method: "NoContent".to_string(),
                                args: vec![],
                            }),
                            args: vec![RustExpression::MethodCall {
                                receiver: Box::new(RustExpression::Ident(
                                    "HttpResponse".to_string(),
                                )),
                                method: "finish".to_string(),
                                args: vec![],
                            }],
                        },
                    },
                    RustMatchArm {
                        pattern: "false".to_string(),
                        guard: None,
                        body: RustExpression::Call {
                            callee: Box::new(RustExpression::Path(vec![
                                "ApiError".to_string(),
                                "NotFound".to_string(),
                            ])),
                            args: vec![RustExpression::Macro {
                                name: "format".to_string(),
                                tokens: format!("\"{} {{}} not found\", id", entity_name),
                            }],
                        },
                    },
                ],
            })),
    ));

    ast
}

/// 生成 Rust routes AST
pub fn entity_to_rust_routes_ast(entity: &GeneratorEntity) -> RustAst {
    let mut ast = RustAst::new(format!("routes/{}.rs", entity.name.snake));

    let entity_snake = &entity.name.snake;
    let plural_kebab = &entity.name.plural_kebab;

    ast.items
        .push(RustItem::Use(RustUse::simple("actix_web::web")));
    ast.items.push(RustItem::Use(RustUse::items(
        format!("crate::handlers::{}", entity_snake),
        vec![
            format!("list_{}", entity_snake).as_str(),
            format!("get_{}", entity_snake).as_str(),
            format!("create_{}", entity_snake).as_str(),
            format!("update_{}", entity_snake).as_str(),
            format!("delete_{}", entity_snake).as_str(),
            format!("hard_delete_{}", entity_snake).as_str(),
        ],
    )));

    // Build chain: web::scope("/{plural_kebab}").route("", web::get().to(list)).route("", web::post().to(create))...
    let scope_expr = RustExpression::Call {
        callee: Box::new(RustExpression::Path(vec![
            "web".to_string(),
            "scope".to_string(),
        ])),
        args: vec![RustExpression::Literal(LiteralValue::String(format!(
            "/{}",
            plural_kebab
        )))],
    };

    let routes: Vec<(String, String, String)> = vec![
        (
            "".to_string(),
            "get".to_string(),
            format!("list_{}", entity_snake),
        ),
        (
            "".to_string(),
            "post".to_string(),
            format!("create_{}", entity_snake),
        ),
        (
            "{id}".to_string(),
            "get".to_string(),
            format!("get_{}", entity_snake),
        ),
        (
            "{id}".to_string(),
            "put".to_string(),
            format!("update_{}", entity_snake),
        ),
        (
            "{id}".to_string(),
            "delete".to_string(),
            format!("delete_{}", entity_snake),
        ),
        (
            "{id}/hard".to_string(),
            "delete".to_string(),
            format!("hard_delete_{}", entity_snake),
        ),
    ];

    let mut scope_chain = scope_expr;
    for (path, method, handler) in routes {
        scope_chain = RustExpression::MethodCall {
            receiver: Box::new(scope_chain),
            method: "route".to_string(),
            args: vec![
                RustExpression::Literal(LiteralValue::String(path)),
                RustExpression::Call {
                    callee: Box::new(RustExpression::MethodCall {
                        receiver: Box::new(RustExpression::Path(vec![
                            "web".to_string(),
                            method.clone(),
                        ])),
                        method: "to".to_string(),
                        args: vec![RustExpression::Ident(handler)],
                    }),
                    args: vec![],
                },
            ],
        };
    }

    // pub fn config(cfg: &mut web::ServiceConfig)
    ast.items.push(RustItem::Function(
        RustFunction::new("config")
            .param(
                Parameter::new("cfg").with_type(TypeRef::reference("mut web::ServiceConfig", true)),
            )
            .body_stmt(RustStatement::Expression(RustExpression::MethodCall {
                receiver: Box::new(RustExpression::Ident("cfg".to_string())),
                method: "service".to_string(),
                args: vec![scope_chain],
            })),
    ));

    ast
}

/// 生成 handlers/mod.rs AST
pub fn model_to_rust_handlers_mod_ast(model: &GeneratorModel) -> RustAst {
    let mut ast = RustAst::new("handlers/mod.rs");

    for entity in &model.entities {
        ast.items.push(RustItem::ModDecl {
            name: entity.name.snake.clone(),
            public: true,
        });
    }

    ast
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ast::AstRoot;
    use crate::generator::ir::{
        EntityName, FieldName, GeneratorField, GeneratorFieldType, PrimaryKeyType,
    };

    fn create_test_entity() -> GeneratorEntity {
        GeneratorEntity {
            name: EntityName {
                raw: "Order".to_string(),
                snake: "order".to_string(),
                camel: "order".to_string(),
                pascal: "Order".to_string(),
                kebab: "order".to_string(),
                screaming_snake: "ORDER".to_string(),
                plural_snake: "orders".to_string(),
                plural_pascal: "Orders".to_string(),
                plural_kebab: "orders".to_string(),
            },
            description: Some("Order entity".to_string()),
            fields: vec![GeneratorField {
                name: FieldName {
                    raw: "code".to_string(),
                    snake: "code".to_string(),
                    camel: "code".to_string(),
                    pascal: "Code".to_string(),
                },
                field_type: GeneratorFieldType::Text,
                description: Some("Order code".to_string()),
                nullable: false,
                unique: true,
                indexed: false,
                default_value: None,
                validations: vec![],
                annotations: vec![],
                ..Default::default()
            }],
            relations: vec![],
            annotations: vec![],
            primary_key_type: PrimaryKeyType::BigInt,
            ..Default::default()
        }
    }

    #[test]
    fn test_entity_to_ts_zod_ast_structure() {
        let entity = create_test_entity();
        let ast = entity_to_ts_zod_ast(&entity);

        // 验证 AST 结构而非字符串
        assert_eq!(ast.imports.len(), 1);
        assert_eq!(ast.imports[0].default, Some("z".to_string()));
        assert_eq!(ast.imports[0].module, "zod");

        // 应该有: interface Order, interface OrderInput, const OrderSchema, const OrderInputSchema, type alias, type alias, const parseOrder
        assert!(ast.statements.len() >= 6);

        // 验证 interface Order 存在
        let has_order_iface = ast
            .statements
            .iter()
            .any(|s| matches!(s, TsStatement::Interface(i) if i.name == "Order"));
        assert!(has_order_iface);

        // 验证 OrderSchema 存在
        let has_schema = ast
            .statements
            .iter()
            .any(|s| matches!(s, TsStatement::Variable(v) if v.name == "OrderSchema"));
        assert!(has_schema);
    }

    #[test]
    fn test_ts_ast_roundtrip() {
        let entity = create_test_entity();
        let ast = entity_to_ts_zod_ast(&entity);

        // 第一阶段验证: AST 结构可精确比较
        let ast2 = entity_to_ts_zod_ast(&entity);
        assert_eq!(ast, ast2);

        // 第二阶段验证: 渲染为字符串
        let output = ast.emit().unwrap();
        assert!(output.contains("export interface Order {"));
        assert!(output.contains("export const OrderSchema"));
        assert!(output.contains("z.object"));
    }

    #[test]
    fn test_entity_to_rust_handler_ast_structure() {
        let entity = create_test_entity();
        let ast = entity_to_rust_handler_ast(&entity);

        // 验证 use 声明
        let has_actix_use = ast
            .items
            .iter()
            .any(|i| matches!(i, RustItem::Use(u) if u.path == "actix_web"));
        assert!(has_actix_use);

        // 验证函数存在
        let has_list = ast
            .items
            .iter()
            .any(|i| matches!(i, RustItem::Function(f) if f.name == "list_order"));
        assert!(has_list);

        let has_create = ast
            .items
            .iter()
            .any(|i| matches!(i, RustItem::Function(f) if f.name == "create_order"));
        assert!(has_create);
    }

    #[test]
    fn test_rust_ast_roundtrip() {
        let entity = create_test_entity();
        let ast = entity_to_rust_handler_ast(&entity);

        // AST 可精确比较
        let ast2 = entity_to_rust_handler_ast(&entity);
        assert_eq!(ast, ast2);

        // 渲染为字符串
        let output = ast.emit().unwrap();
        assert!(output.contains("pub async fn list_order("));
        assert!(output.contains("pub async fn create_order("));
        assert!(output.contains("use actix_web"));
    }
}
