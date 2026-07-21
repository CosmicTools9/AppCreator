//! Module API Generator - Phase 47 / Phase 181
//!
//! 从 Module IR 生成 Rust 后端代码（Actix-web + SQLx）
//!
//! 生成文件结构：
//! ```text
//! Modules/{name}/backend/
//! ├── src/
//! │   ├── main.rs              # 服务入口
//! │   ├── lib.rs               # 库入口
//! │   ├── routes.rs            # 路由配置
//! │   ├── errors.rs            # 错误处理
//! │   ├── auth/                # NGAC PEP 授权中间件 (Phase 181)
//! │   │   ├── context.rs
//! │   │   └── mod.rs            # auth 模块入口
//! │   ├── models/
//! │   │   ├── mod.rs
//! │   │   └── {entity}.rs      # 实体模型
//! │   └── handlers/
//! │       ├── mod.rs
//! │       └── {entity}.rs      # CRUD handlers
//! ├── Cargo.toml               # 模块配置
//! └── migrations/               # DB schema migrations (Rust init/seed/migration)
//!     └── init.rs
//! ```

use crate::generator::ir::module::MetaModule;
#[allow(unused_imports)]
use crate::generator::ir::module::{MetaEntity, MetaField, MetaFieldType};
use crate::generator::{
    GenerateError, GeneratedFile, GeneratedOutput, GenerationMetadata, Generator,
};
use std::path::PathBuf;

/// Module API 生成器选项
#[derive(Debug, Clone)]
pub struct ModuleApiGenOptions {
    /// 是否生成搜索功能
    pub with_search: bool,
    /// 是否生成分页功能
    pub with_pagination: bool,
    /// 数据库连接池大小
    pub pool_size: u32,
    /// 服务端口
    pub port: u16,
}

impl Default for ModuleApiGenOptions {
    fn default() -> Self {
        Self {
            with_search: true,
            with_pagination: true,
            pool_size: 5,
            port: 8080,
        }
    }
}

/// Module API 生成器
pub struct ModuleApiGenerator {
    options: ModuleApiGenOptions,
}

impl ModuleApiGenerator {
    /// 创建新的 Module API 生成器
    pub fn new() -> Self {
        Self {
            options: ModuleApiGenOptions::default(),
        }
    }

    /// 使用自定义选项创建生成器
    pub fn with_options(options: ModuleApiGenOptions) -> Self {
        Self { options }
    }

    /// 从 MetaModule 生成完整的 Rust 后端代码
    pub fn generate(&self, module: &MetaModule) -> Result<GeneratedOutput, GenerateError> {
        let _ = &self.options;
        let entity_count = module.entities.len();
        // 预分配容量：固定文件约 11 个 + 每个实体约 1 个文件 (model)
        let mut files = Vec::with_capacity(11 + entity_count);
        let module_name = &module.name;

        // 生成 Cargo.toml
        files.push(self.generate_cargo_toml(module));

        // 生成 src/lib.rs (Library crate，供 Gateway 集成)
        files.push(self.generate_lib_rs(module_name));

        // 生成 src/routes.rs
        files.push(self.generate_routes_rs(module, &module.entities));

        // 生成 src/errors.rs
        files.push(self.generate_errors_rs());

        // 生成 src/auth/ 模块 (Phase 181)
        files.push(self.generate_auth_mod_rs());
        files.push(self.generate_auth_context_rs());

        // 生成 models/mod.rs 和 models/{entity}.rs
        let models_mod = self.generate_models_mod_rs(&module.entities);
        files.push(models_mod);
        for entity in &module.entities {
            files.push(self.generate_entity_model_rs(entity, module_name));
        }

        let c_file_count = files.len();

        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "module_api".to_string(),
                entity_count: module.entities.len(),
                c_file_count,
            },
        })
    }

    /// 生成 Cargo.toml
    fn generate_cargo_toml(&self, module: &MetaModule) -> GeneratedFile {
        let module_name = &module.name;
        let has_decimal = module.entities.iter().any(|e| {
            e.fields
                .iter()
                .any(|f| matches!(f.field_type, MetaFieldType::Decimal))
        });
        let rust_decimal_dep = if has_decimal {
            r#"rust_decimal = { version = "1", features = ["serde"] }"#
        } else {
            r#"rust_decimal = "1""#
        };
        let content = format!(
            r#"[package]
name = "{module_name}-backend"
version = "0.1.0"
edition = "2021"

[lib]
name = "{module_name}_backend"
path = "src/lib.rs"
crate-type = ["lib"]

[dependencies]
actix-web = "4"
actix-rt = "2"
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
sqlx = {{ version = "0.8", features = ["runtime-tokio", "postgres", "macros", "chrono", "rust_decimal"] }}
tokio = {{ version = "1", features = ["full"] }}
chrono = {{ version = "0.4", features = ["serde"] }}
thiserror = "2"
log = "0.4"
uuid = {{ version = "1", features = ["v4", "serde"] }}
futures = "0.3"
jsonwebtoken = "9"
{rust_decimal_dep}
alioth-common = {{ path = "../../../Framework/backend/common" }}
alioth-crud = {{ path = "../../../Framework/backend/crud" }}

[dev-dependencies]
actix-rt = "2"
sqlx = {{ version = "0.8", features = ["runtime-tokio", "postgres", "macros", "chrono"] }}
"#,
            module_name = module_name,
            rust_decimal_dep = rust_decimal_dep
        );

        GeneratedFile {
            path: PathBuf::from("Cargo.toml"),
            content,
            checksum: String::new(),
        }
    }

    /// 生成 src/lib.rs
    fn generate_lib_rs(&self, module_name: &str) -> GeneratedFile {
        let content = format!(
            r#"//! {module_name} 后端库
//!
//! 由 AliothStudio Module Backend Generator 自动生成

pub mod auth;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod routes;
"#,
            module_name = module_name
        );

        GeneratedFile {
            path: PathBuf::from("src/lib.rs"),
            content,
            checksum: String::new(),
        }
    }

    /// 生成 src/routes.rs
    fn generate_routes_rs(&self, _module: &MetaModule, entities: &[MetaEntity]) -> GeneratedFile {
        let mut configure_blocks = Vec::new();

        for entity in entities {
            let entity_snake = to_snake_case(&entity.name);
            configure_blocks.push(format!(
                "        crate::handlers::{entity_snake}::config(cfg);",
                entity_snake = entity_snake
            ));
        }

        let content = format!(
            r#"//! 路由配置
//!
//! 由 AliothStudio Module Backend Generator 自动生成

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {{
{configure_blocks}
}}
"#,
            configure_blocks = configure_blocks.join("\n")
        );

        GeneratedFile {
            path: PathBuf::from("src/routes.rs"),
            content,
            checksum: String::new(),
        }
    }

    /// 生成 src/errors.rs
    fn generate_errors_rs(&self) -> GeneratedFile {
        let content = r#"//! 错误处理
//!
//! 由 AliothStudio Module Backend Generator 自动生成

use actix_web::{HttpResponse, ResponseError};
use std::fmt;

/// API 错误类型
#[derive(Debug)]
pub enum ApiError {
    /// 数据库错误
    Database(String),
    /// 找不到资源
    NotFound(String),
    /// 参数验证失败
    BadRequest(String),
    /// 内部服务器错误
    Internal(String),
    /// 未授权
    Unauthorized(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Database(msg) => write!(f, "数据库错误: {}", msg),
            ApiError::NotFound(msg) => write!(f, "未找到: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "验证失败: {}", msg),
            ApiError::Internal(msg) => write!(f, "内部错误: {}", msg),
            ApiError::Unauthorized(msg) => write!(f, "未授权: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::Database(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "database_error",
                    "message": msg
                }))
            }
            ApiError::NotFound(msg) => {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "not_found",
                    "message": msg
                }))
            }
            ApiError::BadRequest(msg) => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "validation_error",
                    "message": msg
                }))
            }
            ApiError::Internal(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "internal_error",
                    "message": msg
                }))
            }
            ApiError::Unauthorized(msg) => {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "unauthorized",
                    "message": msg
                }))
            }
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ApiError::NotFound("资源不存在".to_string()),
            sqlx::Error::Database(dbe) if dbe.constraint().is_some() => {
                ApiError::BadRequest(format!("约束违反: {}", dbe))
            }
            _ => ApiError::Database(err.to_string()),
        }
    }
}

impl From<crud::CrudError> for ApiError {
    fn from(err: crud::CrudError) -> Self {
        match err {
            crud::CrudError::Database(msg) => ApiError::Internal(msg),
            crud::CrudError::NotFound(msg) => ApiError::NotFound(msg),
            crud::CrudError::BadRequest(msg) => ApiError::BadRequest(msg),
            crud::CrudError::Unauthorized(msg) => ApiError::Unauthorized(msg),
            crud::CrudError::Internal(msg) => ApiError::Internal(msg),
        }
    }
}
"#;

        GeneratedFile {
            path: PathBuf::from("src/errors.rs"),
            content: content.to_string(),
            checksum: String::new(),
        }
    }

    /// 生成 src/auth/mod.rs
    fn generate_auth_mod_rs(&self) -> GeneratedFile {
        let content = r#"//! NGAC PEP Authorization Module
//!
//! Gateway enforces authorization centrally via NgacEnforcer.
//! Module middleware only performs JWT authentication.

pub use common::context::{RequestContext, RequestContextExt};
"#;

        GeneratedFile {
            path: PathBuf::from("src/auth/mod.rs"),
            content: content.to_string(),
            checksum: String::new(),
        }
    }

    /// 生成 src/auth/context.rs
    fn generate_auth_context_rs(&self) -> GeneratedFile {
        let content = r#"//! Request context for NGAC PEP middleware
//!
//! 模块统一使用 common::context::RequestContext，
//! 确保在 Gateway 集成和 Meta 独立测试时都能正确读取上下文。

pub use common::context::{extract_user_id, RequestContext, RequestContextExt};
"#;

        GeneratedFile {
            path: PathBuf::from("src/auth/context.rs"),
            content: content.to_string(),
            checksum: String::new(),
        }
    }

    fn generate_models_mod_rs(&self, entities: &[MetaEntity]) -> GeneratedFile {
        let mut module_declarations = Vec::new();

        for entity in entities {
            let entity_snake = to_snake_case(&entity.name);
            module_declarations.push(format!("pub mod {};", entity_snake));
        }

        let content = format!(
            r#"//! 数据模型
//!
//! 由 AliothStudio Module Backend Generator 自动生成

{}
"#,
            module_declarations.join("\n")
        );

        GeneratedFile {
            path: PathBuf::from("src/models/mod.rs"),
            content,
            checksum: String::new(),
        }
    }

    /// 生成单个实体的模型文件
    fn generate_entity_model_rs(&self, entity: &MetaEntity, _module_name: &str) -> GeneratedFile {
        let entity_name = &entity.name;
        let entity_snake = to_snake_case(entity_name);

        // 生成字段定义
        let mut struct_fields = Vec::new();
        let mut impl_fields = Vec::new();
        let mut select_fields = Vec::new();

        // 标准审计字段
        struct_fields.push("    /// 主键 ID".to_string());
        struct_fields.push("    #[serde(with = \"common::serde_zuid\")]".to_string());
        struct_fields.push("    pub id: i64,".to_string());
        impl_fields.push("        fields.push(\"id\");".to_string());
        select_fields.push("id".to_string());

        for field in &entity.fields {
            let field_name = &field.name;
            let field_snake = to_snake_case(field_name);
            let rust_type = meta_field_type_to_rust(&field.field_type, field.nullable);

            struct_fields.push(format!(
                "    /// {}",
                field.description.as_deref().unwrap_or(field_name)
            ));
            struct_fields.push(format!("    pub {}: {},", field_snake, rust_type));
            impl_fields.push(format!("        fields.push(\"{}\");", field_snake));
            select_fields.push(field_snake.clone());
        }

        // 关系字段：根据关系类型生成对应 Rust 字段
        for relation in &entity.relations {
            use crate::generator::ir::module::MetaRelationType;
            match relation.relation_type {
                MetaRelationType::OneToOne | MetaRelationType::ManyToOne => {
                    // belong 方向：当前表持有外键
                    let col_name = format!("fk_{}", to_snake_case(&relation.name));
                    let rust_type = if relation.nullable {
                        "Option<i64>".to_string()
                    } else {
                        "i64".to_string()
                    };
                    struct_fields.push(format!("    /// 关联: {}", relation.target_entity));
                    struct_fields.push(format!("    pub {}: {},", col_name, rust_type));
                    impl_fields.push(format!("        fields.push(\"{}\");", col_name));
                    select_fields.push(col_name);
                }
                MetaRelationType::ManyHasMany => {
                    // 本地数组方向：当前表持有 bigint[]
                    let col_name = format!("ak_{}s", to_snake_case(&relation.name));
                    struct_fields.push(format!("    /// 关联数组: {}", relation.target_entity));
                    struct_fields.push(format!("    pub {}: Option<Vec<i64>>,", col_name));
                    impl_fields.push(format!("        fields.push(\"{}\");", col_name));
                    select_fields.push(col_name);
                }
                // OneToMany / ManyToMany：不在当前实体生成字段（由关联表/中间表维护）
                _ => {}
            }
        }

        // 标准审计字段（避免与用户自定义字段重复）
        let has_created_at = entity.fields.iter().any(|f| f.name == "created_at");
        let has_updated_at = entity.fields.iter().any(|f| f.name == "updated_at");
        if !has_created_at {
            struct_fields.push("    /// 创建时间".to_string());
            struct_fields.push("    pub created_at: chrono::DateTime<chrono::Utc>,".to_string());
            impl_fields.push("        fields.push(\"created_at\");".to_string());
            select_fields.push("created_at".to_string());
        }
        if !has_updated_at {
            struct_fields.push("    /// 更新时间".to_string());
            struct_fields.push("    pub updated_at: chrono::DateTime<chrono::Utc>,".to_string());
            impl_fields.push("        fields.push(\"updated_at\");".to_string());
            select_fields.push("updated_at".to_string());
        }

        let select_clause = select_fields.join(", ");

        let content = format!(
            r#"//! {entity_name} 实体模型
//!
//! 由 AliothStudio Module Backend Generator 自动生成

use serde::{{Deserialize, Serialize}};
use sqlx::{{FromRow}};

/// {entity_name} 数据模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct {entity_name} {{
{struct_fields}
}}

/// {entity_name} 创建/更新输入
#[derive(Debug, Clone, Deserialize)]
pub struct {entity_name}Input {{
{input_fields}
}}

impl {entity_name} {{
    /// 获取所有字段名
    pub fn fields() -> Vec<&'static str> {{
        let mut fields = Vec::new();
{impl_fields}
        fields
    }}

    /// 获取 SELECT 字段列表
    pub fn select_fields() -> &'static str {{
        "{select_clause}"
    }}
}}

impl crud::Identifiable for {entity_name} {{
    fn id(&self) -> i64 {{
        self.id
    }}
}}

/// 创建请求类型别名
pub type Create{entity_name}Request = {entity_name}Input;
/// 更新请求类型别名
pub type Update{entity_name}Request = {entity_name}Input;
"#,
            entity_name = entity_name,
            struct_fields = struct_fields.join("\n"),
            impl_fields = impl_fields.join("\n"),
            select_clause = select_clause,
            input_fields = {
                let mut input_lines: Vec<String> = entity
                    .fields
                    .iter()
                    .map(|f| {
                        format!(
                            "    pub {}: {},",
                            to_snake_case(&f.name),
                            meta_field_type_to_rust(&f.field_type, false)
                        )
                    })
                    .collect();
                // 关系字段也加入 Input struct
                for relation in &entity.relations {
                    use crate::generator::ir::module::MetaRelationType;
                    match relation.relation_type {
                        MetaRelationType::OneToOne | MetaRelationType::ManyToOne => {
                            let col_name = format!("fk_{}", to_snake_case(&relation.name));
                            input_lines.push(format!("    pub {}: Option<i64>,", col_name));
                        }
                        MetaRelationType::ManyHasMany => {
                            let col_name = format!("ak_{}s", to_snake_case(&relation.name));
                            input_lines.push(format!("    pub {}: Option<Vec<i64>>,", col_name));
                        }
                        _ => {}
                    }
                }
                input_lines.join("\n")
            }
        );

        GeneratedFile {
            path: PathBuf::from(format!("src/models/{}.rs", entity_snake)),
            content,
            checksum: String::new(),
        }
    }

    #[allow(dead_code)]
    /// 生成 handlers/mod.rs
    fn generate_handlers_mod_rs(&self, entities: &[MetaEntity]) -> GeneratedFile {
        let mut module_declarations = Vec::new();
        let mut config_calls = Vec::new();

        for entity in entities {
            let entity_snake = to_snake_case(&entity.name);
            module_declarations.push(format!("pub mod {};", entity_snake));
            config_calls.push(format!("            {}::config(cfg);", entity_snake));
        }

        let content = format!(
            r#"//! API 处理器
//!
//! 由 AliothStudio Module Backend Generator 自动生成

use actix_web::web;

{}

/// 配置所有模块路由
pub fn config(cfg: &mut web::ServiceConfig) {{
{}
}}
"#,
            module_declarations.join("\n"),
            config_calls.join("\n")
        );

        GeneratedFile {
            path: PathBuf::from("src/handlers/mod.rs"),
            content,
            checksum: String::new(),
        }
    }

    #[allow(dead_code)]
    /// 生成单个实体的 handlers
    fn generate_entity_handlers_rs(
        &self,
        entity: &MetaEntity,
        options: &ModuleApiGenOptions,
    ) -> GeneratedFile {
        let entity_name = &entity.name;
        let entity_snake = to_snake_case(entity_name);
        let plural_snake = to_plural_snake(entity_name);
        let plural_kebab = to_kebab_case(&to_plural_case(entity_name));
        let has_state_machine = entity.state_machine.enabled;

        let list_impl = if options.with_pagination {
            format!(
                r#"/// 列出所有 {entity_name} (分页)
pub async fn list_{entity_snake}(
    pool: web::Data<sqlx::PgPool>,
    query: web::Query<crud::PaginationQuery>,
) -> Result<HttpResponse, ApiError> {{
    let offset = query.offset();

    let items = sqlx::query_as::<_, {entity_name}>(AssertSqlSafe(format!(
        "SELECT {{}} FROM {plural_snake} ORDER BY id DESC LIMIT $1 OFFSET $2",
        {entity_name}::select_fields()
    )))
    .bind(query.page_size)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM {plural_snake}")
        .fetch_one(pool.get_ref())
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({{
        "items": items,
        "total": total.0,
        "page": query.page,
        "page_size": query.page_size
    }})))
}}"#,
                entity_name = entity_name,
                entity_snake = entity_snake,
                plural_snake = plural_snake
            )
        } else {
            format!(
                r#"/// 列出所有 {entity_name}
pub async fn list_{entity_snake}(
    pool: web::Data<sqlx::PgPool>,
) -> Result<HttpResponse, ApiError> {{
    let items = sqlx::query_as::<_, {entity_name}>(AssertSqlSafe(format!(
        "SELECT {{}} FROM {plural_snake} ORDER BY id DESC",
        {entity_name}::select_fields()
    )))
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(items))
}}"#,
                entity_name = entity_name,
                entity_snake = entity_snake,
                plural_snake = plural_snake
            )
        };

        let state_machine_imports = if has_state_machine {
            r#"use meta_services::runtime::{
    StateMachineEngine, TransitionContext, EngineError,
};
"#
            .to_string()
        } else {
            String::new()
        };

        let transition_handlers = if has_state_machine {
            format!(
                r#"/// Transition {entity_name} to a new state
#[derive(Debug, Deserialize)]
pub struct TransitionRequest {{
    pub event: String,
    pub context: Option<serde_json::Value>,
}}

pub async fn transition_{entity_snake}(
    pool: web::Data<PgPool>,
    path: web::Path<i64>,
    body: web::Json<TransitionRequest>,
) -> Result<HttpResponse, ApiError> {{
    let id = path.into_inner();
    let req = body.into_inner();

    // Build engine from static behavior definition
    let engine = build_{entity_snake}_state_machine_engine();

    // Fetch current state (placeholder: in generated code this will query zc_id_lifecycle_r_primary_status)
    let current_state: Option<String> = None;

    let ctx = TransitionContext {{
        triggered_by: None,
        payload: std::collections::HashMap::new(),
    }};

    match engine.execute_transition(current_state.as_deref(), &req.event, &ctx) {{
        Ok(result) => {{
            // TODO: persist state change and history within SQLx transaction
            Ok(HttpResponse::Ok().json(serde_json::json!({{
                "id": id,
                "from_state": result.from_state,
                "to_state": result.to_state,
                "event": result.event,
                "hooks": result.hooks_executed,
            }})))
        }}
        Err(e) => Err(ApiError::BadRequest(e.to_string())),
    }}
}}

pub async fn get_available_transitions_{entity_snake}(
    _pool: web::Data<PgPool>,
    path: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {{
    let _id = path.into_inner();
    let engine = build_{entity_snake}_state_machine_engine();
    // Placeholder current state; real implementation will query from DB
    let current_state = "";
    let events = engine.available_events(current_state);
    Ok(HttpResponse::Ok().json(serde_json::json!({{ "events": events }})))
}}
"#,
                entity_name = entity_name,
                entity_snake = entity_snake
            )
        } else {
            String::new()
        };

        let build_engine_helper = if has_state_machine {
            let states: Vec<String> = entity
                .state_machine
                .states
                .iter()
                .map(|s| format!(r#"State::new("{}")"#, s))
                .collect();
            let states_str = states.join(", ");
            let initial_state = entity.state_machine.initial_state.as_deref().unwrap_or("");

            let mut transition_adds = Vec::new();
            for t in &entity.transitions {
                if t.from.len() == 1 {
                    transition_adds.push(format!(
                        r#"tt.add(Transition::new("{}", "{}", "{}"));"#,
                        t.event, t.from[0], t.to
                    ));
                } else {
                    let froms: Vec<String> = t.from.iter().map(|f| format!(r#""{}""#, f)).collect();
                    transition_adds.push(format!(
                        r#"tt.add(Transition::new_multi_from("{}", vec![{}], "{}"));"#,
                        t.event,
                        froms.join(", "),
                        t.to
                    ));
                }
            }
            let transition_adds_str = transition_adds.join("\n    ");

            format!(
                r#"fn build_{entity_snake}_state_machine_engine() -> StateMachineEngine {{
    let mut sm = runtime_contract::behavior::StateMachine::default();
    sm.enabled = true;
    sm.states = vec![{states}];
    sm.initial_state = Some("{initial}".to_string());

    let mut tt = runtime_contract::behavior::TransitionTable::new();
    {transition_adds}

    let hooks = runtime_contract::behavior::LifecycleHooks::default();
    let rules = runtime_contract::behavior::BusinessRules::default();

    StateMachineEngine::new(sm, tt, hooks, rules)
}}
"#,
                entity_snake = entity_snake,
                states = states_str,
                initial = initial_state,
                transition_adds = transition_adds_str
            )
        } else {
            String::new()
        };

        let extra_routes = if has_state_machine {
            format!(
                r#"            .route("/{{id}}/transitions", web::post().to(transition_{entity_snake}))
            .route("/{{id}}/available-transitions", web::get().to(get_available_transitions_{entity_snake}))"#,
                entity_snake = entity_snake
            )
        } else {
            String::new()
        };

        let content = format!(
            r#"//! {entity_name} CRUD 处理器
//!
//! 由 AliothStudio Module Backend Generator 自动生成
//!
//! # Authorization
//! 所有变更操作 (create, update, delete) 需要身份认证。

use actix_web::*;
use serde::Deserialize;
use sqlx::{{AssertSqlSafe, PgPool}};

use crate::errors::ApiError;
use crate::models::{entity_snake}::{{{entity_name}, {entity_name}Input}};
{state_machine_imports}
{list_impl}

/// Extract fk_user from request extensions
/// This is set by the Gateway PEP middleware or SSO authentication
fn extract_user_id(req: &HttpRequest) -> Option<i64> {{
    common::context::extract_user_id(req)
        .or_else(|| req.extensions().get::<i64>().copied())
}}

/// Require authentication - returns fk_user or Unauthorized error
fn require_auth(req: &HttpRequest) -> Result<i64, ApiError> {{
    extract_user_id(req).ok_or_else(|| ApiError::Unauthorized("Authentication required".to_string()))
}}

/// 获取单个 {entity_name} (public read - no auth required)
pub async fn get_{entity_snake}(
    pool: web::Data<PgPool>,
    path: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {{
    let id = path.into_inner();

    let item = sqlx::query_as::<_, {entity_name}>(AssertSqlSafe(format!(
        "SELECT {{}} FROM {plural_snake} WHERE id = $1",
        {entity_name}::select_fields()
    )))
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;

    match item {{
        Some(item) => Ok(HttpResponse::Ok().json(item)),
        None => Err(ApiError::NotFound(format!("{entity_name} {{}} not found", id))),
    }}
}}

/// 创建新的 {entity_name}
///
/// # Authorization
/// Requires authentication.
pub async fn create_{entity_snake}(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    body: web::Json<{entity_name}Input>,
) -> Result<HttpResponse, ApiError> {{
    let _user_id = require_auth(&req)?;
    let input = body.into_inner();

    let fields = {entity_name}::fields();
    let field_names: Vec<&str> = fields.iter().filter(|&&f| f != "id" && f != "created_at" && f != "updated_at").copied().collect();
    let placeholders: Vec<String> = (1..=field_names.len()).map(|i| format!("${{}}", i)).collect();
    let returning_fields = format!("{{}}", {entity_name}::select_fields());

    let query = format!(
        "INSERT INTO {plural_snake} ({{}}) VALUES ({{}}) RETURNING {{}}",
        field_names.join(", "),
        placeholders.join(", "),
        returning_fields
    );

    let item = sqlx::query_as::<_, {entity_name}>(&query)
{insert_binds}
        .fetch_one(pool.get_ref())
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    Ok(HttpResponse::Created().json(item))
}}

/// 更新 {entity_name}
///
/// # Authorization
/// Requires authentication.
pub async fn update_{entity_snake}(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<i64>,
    body: web::Json<{entity_name}Input>,
) -> Result<HttpResponse, ApiError> {{
    let id = path.into_inner();
    let _user_id = require_auth(&req)?;
    let input = body.into_inner();

    let fields = {entity_name}::fields();
    let update_fields: Vec<&str> = fields.iter().filter(|&&f| f != "id" && f != "created_at" && f != "updated_at").copied().collect();
    let set_clauses: Vec<String> = update_fields.iter().enumerate()
        .map(|(i, f)| format!("{{}} = ${{}}", f, i + 1))
        .collect();

    let query = format!(
        "UPDATE {plural_snake} SET {{}} WHERE id = ${{}} RETURNING {{}}",
        set_clauses.join(", "),
        update_fields.len() + 1,
        {entity_name}::select_fields()
    );

    let mut q = sqlx::query_as::<_, {entity_name}>(&query);
{update_binds}
    q = q.bind(id);

    let item = q
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    match item {{
        Some(item) => Ok(HttpResponse::Ok().json(item)),
        None => Err(ApiError::NotFound(format!("{entity_name} {{}} not found", id))),
    }}
}}

/// 删除 {entity_name}
///
/// # Authorization
/// Requires authentication.
pub async fn delete_{entity_snake}(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<i64>,
) -> Result<HttpResponse, ApiError> {{
    let id = path.into_inner();
    let _user_id = require_auth(&req)?;

    let result = sqlx::query("DELETE FROM {plural_snake} WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {{
        return Err(ApiError::NotFound(format!("{entity_name} {{}} not found", id)));
    }}

    Ok(HttpResponse::NoContent().finish())
}}

{transition_handlers}
{build_engine_helper}
/// 配置 {entity_name} 路由
pub fn config(cfg: &mut web::ServiceConfig) {{
    cfg.service(
        web::scope("/{plural_kebab}")
            .route("", web::get().to(list_{entity_snake}))
            .route("", web::post().to(create_{entity_snake}))
            .route("/{{id}}", web::get().to(get_{entity_snake}))
            .route("/{{id}}", web::put().to(update_{entity_snake}))
            .route("/{{id}}", web::delete().to(delete_{entity_snake}))
{extra_routes}
    );
}}
"#,
            entity_name = entity_name,
            entity_snake = entity_snake,
            plural_snake = plural_snake,
            plural_kebab = plural_kebab,
            list_impl = list_impl,
            state_machine_imports = state_machine_imports,
            transition_handlers = transition_handlers,
            build_engine_helper = build_engine_helper,
            extra_routes = extra_routes,
            insert_binds = self.generate_insert_binds(entity),
            update_binds = self.generate_update_binds(entity)
        );

        GeneratedFile {
            path: PathBuf::from(format!("src/handlers/{}.rs", entity_snake)),
            content,
            checksum: String::new(),
        }
    }

    #[allow(dead_code)]
    /// 生成单个实体的 handler 单元测试
    fn generate_entity_handler_tests_rs(
        &self,
        entity: &MetaEntity,
        _options: &ModuleApiGenOptions,
    ) -> GeneratedFile {
        let entity_name = &entity.name;
        let entity_snake = to_snake_case(entity_name);
        let plural_snake = to_plural_snake(entity_name);

        let list_test = format!(
            r#"#[actix_rt::test]
async fn test_list_{entity_snake}() {{
    // TODO: Initialize test database pool
    // let pool = setup_test_pool().await;
    // let app = test::init_service(
    //     App::new().app_data(web::Data::new(pool.clone())).configure(config)
    // ).await;
    // let req = test::TestRequest::get().uri("/{plural_snake}").to_request();
    // let resp = test::call_service(&app, req).await;
    // assert!(resp.status().is_success());
}}"#
        );

        let content = format!(
            r#"//! {entity_name} Handler 单元测试
//!
//! 由 AliothStudio Module Backend Generator 自动生成

#[cfg(test)]
mod tests {{
    use actix_web::{{test, web, App}};
    use crate::handlers::{entity_snake}::config;

{list_test}

    #[actix_rt::test]
    async fn test_get_{entity_snake}() {{
        // TODO: Setup test pool and test GET /{{id}}
    }}

    #[actix_rt::test]
    async fn test_create_{entity_snake}() {{
        // TODO: Setup test pool and test POST
    }}

    #[actix_rt::test]
    async fn test_update_{entity_snake}() {{
        // TODO: Setup test pool and test PUT /{{id}}
    }}

    #[actix_rt::test]
    async fn test_delete_{entity_snake}() {{
        // TODO: Setup test pool and test DELETE /{{id}}
    }}
}}"#,
            entity_name = entity_name,
            entity_snake = entity_snake,
            list_test = list_test
        );

        GeneratedFile {
            path: PathBuf::from(format!("src/handlers/{}_test.rs", entity_snake)),
            content,
            checksum: String::new(),
        }
    }

    #[allow(dead_code)]
    /// 生成单个实体的集成测试
    fn generate_integration_tests_rs(
        &self,
        entity: &MetaEntity,
        _options: &ModuleApiGenOptions,
    ) -> GeneratedFile {
        let entity_name = &entity.name;
        let entity_snake = to_snake_case(entity_name);
        let plural_kebab = to_kebab_case(&to_plural_case(entity_name));

        let content = format!(
            r#"//! {entity_name} 集成测试
//!
//! 由 AliothStudio Module Backend Generator 自动生成
//!
//! 使用 tokio::test + PgPool::connect 进行数据库集成测试。

use actix_web::{{test, web, App}};

async fn setup_pool() -> sqlx::PgPool {{
    sqlx::PgPool::connect("postgres://localhost:5432/aliothstudio_test")
        .await
        .expect("Failed to connect to test database")
}}

/// {entity_name} CRUD 集成测试模板
#[tokio::test]
async fn test_{entity_snake}_crud() {{
    let pool = setup_pool().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .configure(crate::routes::configure_routes)
    ).await;

    // 1. 创建
    // let create_req = test::TestRequest::post()
    //     .uri("/{plural_kebab}")
    //     .set_json(serde_json::json!({{ /* input fields */ }}))
    //     .to_request();
    // let create_resp = test::call_service(&app, create_req).await;
    // assert!(create_resp.status().is_success());

    // 2. 列表查询
    let list_req = test::TestRequest::get()
        .uri("/{plural_kebab}")
        .to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert!(list_resp.status().is_success());

    // 3. 详情查询
    // let get_req = test::TestRequest::get()
    //     .uri("/{plural_kebab}/1")
    //     .to_request();
    // let get_resp = test::call_service(&app, get_req).await;
    // assert!(get_resp.status().is_success());

    // 4. 更新
    // let update_req = test::TestRequest::put()
    //     .uri("/{plural_kebab}/1")
    //     .set_json(serde_json::json!({{ /* updated fields */ }}))
    //     .to_request();
    // let update_resp = test::call_service(&app, update_req).await;
    // assert!(update_resp.status().is_success());

    // 5. 删除
    // let delete_req = test::TestRequest::delete()
    //     .uri("/{plural_kebab}/1")
    //     .to_request();
    // let delete_resp = test::call_service(&app, delete_req).await;
    // assert!(delete_resp.status().is_success());
}}

/// {entity_name} 列表查询测试
#[tokio::test]
async fn test_{entity_snake}_list() {{
    let pool = setup_pool().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .configure(crate::routes::configure_routes)
    ).await;

    let req = test::TestRequest::get()
        .uri("/{plural_kebab}")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}}"#,
            entity_name = entity_name,
            entity_snake = entity_snake,
            plural_kebab = plural_kebab
        );

        GeneratedFile {
            path: PathBuf::from(format!("tests/integration/{}.rs", entity_snake)),
            content,
            checksum: String::new(),
        }
    }

    /// 生成 INSERT 绑定的参数
    fn generate_insert_binds(&self, entity: &MetaEntity) -> String {
        let mut binds = Vec::new();
        let fields: Vec<_> = entity
            .fields
            .iter()
            .filter(|f| f.name != "id" && f.name != "created_at" && f.name != "updated_at")
            .collect();

        for field in &fields {
            let field_snake = to_snake_case(&field.name);
            binds.push(format!("        .bind(&input.{})", field_snake));
        }

        binds.join("\n")
    }

    /// 生成 UPDATE 绑定的参数
    fn generate_update_binds(&self, entity: &MetaEntity) -> String {
        let mut binds = Vec::new();
        let fields: Vec<_> = entity
            .fields
            .iter()
            .filter(|f| f.name != "id" && f.name != "created_at" && f.name != "updated_at")
            .collect();

        for field in &fields {
            let field_snake = to_snake_case(&field.name);
            binds.push(format!("    q = q.bind(&input.{});", field_snake));
        }

        binds.join("\n")
    }
}

impl Default for ModuleApiGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator for ModuleApiGenerator {
    fn name(&self) -> &'static str {
        "module_api"
    }

    fn generate(
        &self,
        _model: &crate::generator::ir::GeneratorModel,
    ) -> Result<GeneratedOutput, GenerateError> {
        Err(GenerateError::Validation(
            "ModuleApiGenerator 需要使用 generate_module() 方法".to_string(),
        ))
    }

    fn validate(
        &self,
        _model: &crate::generator::ir::GeneratorModel,
    ) -> Result<(), crate::generator::ValidationError> {
        Ok(())
    }

    fn supports_incremental(&self) -> bool {
        false
    }

    fn file_extensions(&self) -> Vec<&'static str> {
        vec!["rs", "toml", "sql"]
    }
}

// ============== 辅助函数 ==============

/// 将字符串转换为 snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

/// 将字符串转换为复数形式 (简单处理)
fn to_plural_snake(s: &str) -> String {
    let snake = to_snake_case(s);
    if snake.ends_with('s') || snake.ends_with('x') || snake.ends_with('z') {
        format!("{}es", snake)
    } else if snake.ends_with('y') {
        format!("{}ies", &snake[..snake.len() - 1])
    } else {
        format!("{}s", snake)
    }
}

/// 将字符串转换为 kebab-case
#[allow(dead_code)]
fn to_kebab_case(s: &str) -> String {
    let snake = to_snake_case(s);
    snake.replace('_', "-")
}

/// 将字符串首字母大写
#[allow(dead_code)]
fn to_plural_case(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// 将 MetaFieldType 转换为 Rust 类型
fn meta_field_type_to_rust(field_type: &MetaFieldType, nullable: bool) -> String {
    let base_type = match field_type {
        MetaFieldType::String => "String",
        MetaFieldType::Integer => "i32",
        MetaFieldType::Long => "i64",
        MetaFieldType::Decimal => "rust_decimal::Decimal",
        MetaFieldType::Boolean => "bool",
        MetaFieldType::DateTime => "chrono::DateTime<chrono::Utc>",
        MetaFieldType::Uuid => "uuid::Uuid",
        MetaFieldType::Json => "serde_json::Value",
        MetaFieldType::Enum(name) => name,
        MetaFieldType::Reference(_) => "i64",
    };

    if nullable {
        format!("Option<{}>", base_type)
    } else {
        base_type.to_string()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_module() -> MetaModule {
        let order_entity = MetaEntity {
            name: "Order".to_string(),
            description: Some("订单实体".to_string()),
            fields: vec![
                MetaField {
                    name: "order_number".to_string(),
                    field_type: MetaFieldType::String,
                    description: Some("订单编号".to_string()),
                    nullable: false,
                    unique: true,
                    indexed: true,
                    default_value: None,
                    validations: vec![],
                    annotations: vec![],
                    domain: None,
                    range: None,
                    min_cardinality: None,
                    max_cardinality: None,
                    is_functional: false,
                    constraints: vec![],
                    field_permission: Default::default(),
                    throws_clauses: vec![],
                    quality_rules: vec![],
                },
                MetaField {
                    name: "total_amount".to_string(),
                    field_type: MetaFieldType::Decimal,
                    description: Some("总金额".to_string()),
                    nullable: false,
                    unique: false,
                    indexed: false,
                    default_value: Some("0".to_string()),
                    validations: vec![],
                    annotations: vec![],
                    domain: None,
                    range: None,
                    min_cardinality: None,
                    max_cardinality: None,
                    is_functional: false,
                    constraints: vec![],
                    field_permission: Default::default(),
                    throws_clauses: vec![],
                    quality_rules: vec![],
                },
            ],
            relations: vec![],
            annotations: vec![],
            parent_classes: vec![],
            equivalent_classes: vec![],
            disjoint_classes: vec![],
            is_abstract: false,
            state_machine: Default::default(),
            transitions: vec![],
            lifecycle_hooks: vec![],
            business_rules: vec![],
            swrl_rules: vec![],
            constraints: vec![],
            permission_config: Default::default(),
            permission_inheritance: Default::default(),
            permission_conflict_resolution: Default::default(),
            quality_rules: vec![],
        };

        let mut module = MetaModule::new("orders");
        module.add_entity(order_entity);
        module
    }

    #[test]
    fn test_module_api_generator() {
        let module = create_test_module();
        let generator = ModuleApiGenerator::new();
        let output = generator.generate(&module).unwrap();

        // 检查关键文件存在
        let paths: Vec<String> = output
            .files
            .iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect();

        assert!(paths.contains(&"Cargo.toml".to_string()));
        assert!(
            !paths.contains(&"src/main.rs".to_string()),
            "Library crate should not generate main.rs"
        );
        assert!(paths.contains(&"src/lib.rs".to_string()));
        assert!(paths.contains(&"src/routes.rs".to_string()));
        assert!(paths.contains(&"src/errors.rs".to_string()));
        assert!(paths.contains(&"src/auth/mod.rs".to_string()));
        assert!(paths.contains(&"src/auth/context.rs".to_string()));
        assert!(paths.contains(&"src/models/mod.rs".to_string()));
        assert!(paths.contains(&"src/models/order.rs".to_string()));
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("OrderNumber"), "order_number");
        assert_eq!(to_snake_case("TotalAmount"), "total_amount");
    }

    #[test]
    fn test_to_plural_snake() {
        assert_eq!(to_plural_snake("Order"), "orders");
        assert_eq!(to_plural_snake("Business"), "businesses");
        assert_eq!(to_plural_snake("Box"), "boxes");
    }

    #[test]
    fn test_meta_field_to_rust_type() {
        assert_eq!(
            meta_field_type_to_rust(&MetaFieldType::String, false),
            "String"
        );
        assert_eq!(
            meta_field_type_to_rust(&MetaFieldType::Integer, false),
            "i32"
        );
        assert_eq!(
            meta_field_type_to_rust(&MetaFieldType::Long, true),
            "Option<i64>"
        );
        assert_eq!(
            meta_field_type_to_rust(&MetaFieldType::Boolean, false),
            "bool"
        );
    }
}
