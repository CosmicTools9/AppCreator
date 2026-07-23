//! Chat session management + AppAgent integration.
//!
//! Stores sessions/messages in `isahl_meta.meta_chat_sessions` / `meta_chat_messages`
//! and drives `app_agent::AppAgent` for LLM-based app creation.

use actix_web::{web, HttpRequest, HttpResponse};
use app_agent::llm::{
    GenerationOverrides, LlmError as AgentLlmError, LlmService as AgentLlmService,
};
use app_agent::{AgentState, AppAgent, ConversationContext};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::progress::{progress_channel, sse_response, ProgressEvent};
use common::{ApiResponse, ErrorResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;

use crate::handlers::require_auth;

/// Environment-driven LLM adapter for AppAgent.
pub struct EnvLlmAdapter {
    inner: web::Data<llm::LlmService>,
}

impl EnvLlmAdapter {
    pub fn new(inner: web::Data<llm::LlmService>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl AgentLlmService for EnvLlmAdapter {
    async fn generate(&self, prompt: &str) -> Result<String, AgentLlmError> {
        self.inner
            .generate(prompt)
            .await
            .map_err(|e| AgentLlmError {
                message: e.to_string(),
            })
    }

    async fn generate_with_system(
        &self,
        system: &str,
        prompt: &str,
    ) -> Result<String, AgentLlmError> {
        self.inner
            .generate_with_system_preamble(system, prompt, None, None, None, None, None)
            .await
            .map_err(|e| AgentLlmError {
                message: e.to_string(),
            })
    }

    async fn generate_with_params(
        &self,
        system: &str,
        prompt: &str,
        _overrides: GenerationOverrides,
    ) -> Result<String, AgentLlmError> {
        self.inner
            .generate_with_system_preamble(system, prompt, None, None, None, None, None)
            .await
            .map_err(|e| AgentLlmError {
                message: e.to_string(),
            })
    }
}

// ── Types ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub title: Option<String>,
    pub app_instance_id: Option<i64>,
    pub namespace: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ChatSessionRow {
    pub id: i64,
    pub title: String,
    pub app_instance_id: Option<i64>,
    pub namespace: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ChatMessageRow {
    pub id: i64,
    pub session_id: i64,
    pub role: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ChatSessionResponse {
    pub id: i64,
    pub title: String,
    pub app_instance_id: Option<i64>,
    pub namespace: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<ChatMessageResponse>,
}

#[derive(Debug, Serialize)]
pub struct ChatMessageResponse {
    pub id: i64,
    pub session_id: i64,
    pub role: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AddMessageRequest {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct StepResponse {
    pub state_before: String,
    pub state_after: String,
    pub is_terminal: bool,
    pub progress_percent: u8,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetStateRequest {
    pub target_state: Option<String>,
}

// ── Repository ─────────────────────────────────────────

pub async fn create_session(
    pool: &PgPool,
    title: &str,
    app_instance_id: Option<i64>,
    namespace: &str,
) -> Result<ChatSessionRow, sqlx::Error> {
    sqlx::query_as::<_, ChatSessionRow>(
        r#"
        INSERT INTO isahl_meta.meta_chat_sessions (title, app_instance_id, namespace, status)
        VALUES ($1, $2, $3, 'active')
        RETURNING id, title, app_instance_id, namespace, status::text, created_at, updated_at
        "#,
    )
    .bind(title)
    .bind(app_instance_id)
    .bind(namespace)
    .fetch_one(pool)
    .await
}

pub async fn get_session(pool: &PgPool, id: i64) -> Result<Option<ChatSessionRow>, sqlx::Error> {
    sqlx::query_as::<_, ChatSessionRow>(
        r#"
        SELECT id, title, app_instance_id, namespace, status::text, created_at, updated_at
        FROM isahl_meta.meta_chat_sessions
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// 列出最近会话（按 updated_at 倒序）。
///
/// 注意：meta_chat_sessions 表无 owner 列（schema 归 Meta 管理），
/// 当前不做用户隔离 —— 登录用户可见同 namespace 下所有会话。
/// TODO(user-isolation): Meta 侧 DDL 增加 owner 列后（需 spec-audit），此处按用户过滤。
pub async fn list_sessions(
    pool: &PgPool,
    namespace: Option<&str>,
    limit: i64,
) -> Result<Vec<ChatSessionRow>, sqlx::Error> {
    sqlx::query_as::<_, ChatSessionRow>(
        r#"
        SELECT id, title, app_instance_id, namespace, status::text, created_at, updated_at
        FROM isahl_meta.meta_chat_sessions
        WHERE deleted_at IS NULL
          AND ($1::text IS NULL OR namespace = $1)
        ORDER BY updated_at DESC
        LIMIT $2
        "#,
    )
    .bind(namespace)
    .bind(limit.clamp(1, 100))
    .fetch_all(pool)
    .await
}

pub async fn list_messages(
    pool: &PgPool,
    session_id: i64,
) -> Result<Vec<ChatMessageRow>, sqlx::Error> {
    sqlx::query_as::<_, ChatMessageRow>(
        r#"
        SELECT id, session_id, role, content, created_at
        FROM isahl_meta.meta_chat_messages
        WHERE session_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
}

pub async fn add_message(
    pool: &PgPool,
    session_id: i64,
    role: &str,
    content: &str,
) -> Result<ChatMessageRow, sqlx::Error> {
    sqlx::query_as::<_, ChatMessageRow>(
        r#"
        INSERT INTO isahl_meta.meta_chat_messages (session_id, role, content)
        VALUES ($1, $2, $3)
        RETURNING id, session_id, role, content, created_at
        "#,
    )
    .bind(session_id)
    .bind(role)
    .bind(content)
    .fetch_one(pool)
    .await
}

pub async fn load_agent_context(
    pool: &PgPool,
    session_id: i64,
) -> Result<Option<ConversationContext>, sqlx::Error> {
    let row: Option<(Option<serde_json::Value>,)> = sqlx::query_as(
        "SELECT agent_state FROM isahl_meta.meta_chat_sessions WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await?;

    Ok(match row {
        Some((Some(json),)) => serde_json::from_value(json).ok(),
        _ => None,
    })
}

pub async fn save_agent_context(
    pool: &PgPool,
    session_id: i64,
    ctx: &ConversationContext,
) -> Result<(), sqlx::Error> {
    let json = serde_json::to_value(ctx).unwrap_or_default();
    sqlx::query(
        "UPDATE isahl_meta.meta_chat_sessions SET agent_state = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(json)
    .bind(session_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_session_status(
    pool: &PgPool,
    session_id: i64,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE isahl_meta.meta_chat_sessions SET status = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(status)
    .bind(session_id)
    .execute(pool)
    .await?;
    Ok(())
}

// ── Response helpers ───────────────────────────────────

fn row_to_session(row: ChatSessionRow, messages: Vec<ChatMessageRow>) -> ChatSessionResponse {
    ChatSessionResponse {
        id: row.id,
        title: row.title,
        app_instance_id: row.app_instance_id,
        namespace: row.namespace,
        status: row.status,
        created_at: row.created_at,
        updated_at: row.updated_at,
        messages: messages.into_iter().map(row_to_message).collect(),
    }
}

fn row_to_message(row: ChatMessageRow) -> ChatMessageResponse {
    ChatMessageResponse {
        id: row.id,
        session_id: row.session_id,
        role: row.role,
        content: row.content,
        created_at: row.created_at,
    }
}

fn error_response(code: &str, message: impl Into<String>) -> ErrorResponse {
    ErrorResponse {
        code: code.to_string(),
        message: message.into(),
        details: None,
    }
}

// ── Handlers ───────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListSessionsQuery {
    pub namespace: Option<String>,
    pub limit: Option<i64>,
}

pub async fn list_sessions_handler(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    query: web::Query<ListSessionsQuery>,
) -> HttpResponse {
    let _user = match require_auth(&req) {
        Ok(u) => u,
        Err(r) => return r,
    };

    let ns = crate::middleware::extract_namespace(&req);
    match list_sessions(
        pool.get_ref(),
        if ns.is_empty() { None } else { Some(&ns) },
        query.limit.unwrap_or(20),
    )
    .await
    {
        Ok(rows) => {
            let sessions: Vec<ChatSessionResponse> = rows
                .into_iter()
                .map(|r| row_to_session(r, vec![]))
                .collect();
            HttpResponse::Ok().json(ApiResponse::success(json!({ "sessions": sessions })))
        }
        Err(e) => {
            log::error!("list_sessions failed: {}", e);
            HttpResponse::InternalServerError().json(error_response("DB_ERROR", e.to_string()))
        }
    }
}

pub async fn create_session_handler(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<CreateSessionRequest>,
) -> HttpResponse {
    let user = match require_auth(&req) {
        Ok(u) => u,
        Err(r) => return r,
    };

    let namespace = match crate::middleware::extract_namespace(&req) {
        ns if !ns.is_empty() => ns,
        _ => {
            return HttpResponse::BadRequest().json(error_response(
                "MISSING_NAMESPACE",
                "namespace not available from auth context",
            ));
        }
    };

    let title = body.title.as_deref().unwrap_or("New session").trim();
    let title = if title.is_empty() {
        "New session"
    } else {
        title
    };

    match create_session(pool.get_ref(), title, body.app_instance_id, &namespace).await {
        Ok(row) => {
            let mut ctx =
                ConversationContext::new(row.id, user.username.clone(), namespace.to_string());
            ctx.user_description = "".to_string();
            let _ = save_agent_context(pool.get_ref(), row.id, &ctx).await;
            HttpResponse::Created().json(ApiResponse::success(row_to_session(row, vec![])))
        }
        Err(e) => {
            log::error!("create_session failed: {}", e);
            HttpResponse::InternalServerError().json(error_response("DB_ERROR", e.to_string()))
        }
    }
}

pub async fn get_session_handler(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<i64>,
) -> HttpResponse {
    let _user = match require_auth(&req) {
        Ok(u) => u,
        Err(r) => return r,
    };

    let session_id = path.into_inner();
    match get_session(pool.get_ref(), session_id).await {
        Ok(Some(row)) => match list_messages(pool.get_ref(), session_id).await {
            Ok(msgs) => HttpResponse::Ok().json(ApiResponse::success(row_to_session(row, msgs))),
            Err(e) => {
                HttpResponse::InternalServerError().json(error_response("DB_ERROR", e.to_string()))
            }
        },
        Ok(None) => HttpResponse::NotFound().json(error_response("NOT_FOUND", "Session not found")),
        Err(e) => {
            HttpResponse::InternalServerError().json(error_response("DB_ERROR", e.to_string()))
        }
    }
}

pub async fn add_message_handler(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<i64>,
    body: web::Json<AddMessageRequest>,
) -> HttpResponse {
    let _user = match require_auth(&req) {
        Ok(u) => u,
        Err(r) => return r,
    };

    let session_id = path.into_inner();
    let content = body.content.trim().to_string();
    if content.is_empty() {
        return HttpResponse::BadRequest()
            .json(error_response("EMPTY_MESSAGE", "content is required"));
    }

    match add_message(pool.get_ref(), session_id, "user", &content).await {
        Ok(msg) => HttpResponse::Ok().json(ApiResponse::success(row_to_message(msg))),
        Err(e) => {
            HttpResponse::InternalServerError().json(error_response("DB_ERROR", e.to_string()))
        }
    }
}

pub async fn generate_response_handler(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    llm: web::Data<llm::LlmService>,
    path: web::Path<i64>,
) -> HttpResponse {
    let _user = match require_auth(&req) {
        Ok(u) => u,
        Err(r) => return r,
    };

    let session_id = path.into_inner();
    let pool: Arc<PgPool> = pool.into_inner();

    let mut ctx = match load_agent_context(pool.as_ref(), session_id).await {
        Ok(Some(ctx)) => ctx,
        Ok(None) => {
            return HttpResponse::NotFound().json(error_response("NOT_FOUND", "Session not found"))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(error_response("DB_ERROR", e.to_string()))
        }
    };

    let adapter = EnvLlmAdapter::new(llm);
    let agent = AppAgent::new(pool.clone(), Box::new(adapter));

    match agent
        .run_single_step(&mut ctx, None::<&fn(app_agent::AgentProgress)>)
        .await
    {
        Ok(result) => {
            let _ = save_agent_context(pool.as_ref(), session_id, &ctx).await;

            let assistant_text = if result.is_terminal {
                result.message.clone()
            } else {
                format!("[{}] {}", app_agent::state_name(&ctx.state), result.message)
            };
            let _ = add_message(pool.as_ref(), session_id, "assistant", &assistant_text).await;

            // 幂等回填 app_instance_id：只要 FS 已有 app.json 且未关联即写入
            if let (Some(ns), Some(an)) = (ctx.namespace.as_deref(), ctx.app_name.as_deref()) {
                let apps_root = crate::app_repository::apps_dir(ns);
                if let Some((_, val)) = crate::app_repository::find_app_by_code(&apps_root, ns, an)
                {
                    if let Some(aid) = crate::app_repository::app_id_from_json(&val) {
                        if let Err(e) = sqlx::query(
                            "UPDATE isahl_meta.meta_chat_sessions \
                             SET app_instance_id = $1 WHERE id = $2 \
                             AND (app_instance_id IS NULL OR app_instance_id != $1)",
                        )
                        .bind(aid)
                        .bind(session_id)
                        .execute(pool.as_ref())
                        .await
                        {
                            log::warn!(
                                "Failed to update app_instance_id for session {session_id}: {e}"
                            );
                        }
                    }
                }
            }

            let status = if result.is_terminal {
                "completed"
            } else {
                "active"
            };
            let _ = update_session_status(pool.as_ref(), session_id, status).await;

            HttpResponse::Ok().json(ApiResponse::success(StepResponse {
                state_before: app_agent::state_name(&result.state_before).to_string(),
                state_after: app_agent::state_name(&result.state_after).to_string(),
                is_terminal: result.is_terminal,
                progress_percent: app_agent::progress_percent(&result.state_after),
                message: result.message,
            }))
        }
        Err(e) => {
            let _ = update_session_status(pool.as_ref(), session_id, "abandoned").await;
            HttpResponse::InternalServerError().json(error_response("AGENT_ERROR", e))
        }
    }
}

pub async fn interrupt_handler(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<i64>,
) -> HttpResponse {
    let _user = match require_auth(&req) {
        Ok(u) => u,
        Err(r) => return r,
    };

    let session_id = path.into_inner();
    let mut ctx = match load_agent_context(pool.get_ref(), session_id).await {
        Ok(Some(ctx)) => ctx,
        Ok(None) => {
            return HttpResponse::NotFound().json(error_response("NOT_FOUND", "Session not found"))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(error_response("DB_ERROR", e.to_string()))
        }
    };

    AppAgent::request_interrupt(&mut ctx);
    let _ = save_agent_context(pool.get_ref(), session_id, &ctx).await;
    let _ = update_session_status(pool.get_ref(), session_id, "active").await;

    HttpResponse::Ok().json(ApiResponse::success(
        json!({"status": "interrupt_requested"}),
    ))
}

pub async fn resume_handler(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    llm: web::Data<llm::LlmService>,
    path: web::Path<i64>,
) -> HttpResponse {
    generate_response_handler(req, pool, llm, path).await
}

pub async fn reset_state_handler(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<i64>,
    body: web::Json<ResetStateRequest>,
) -> HttpResponse {
    let _user = match require_auth(&req) {
        Ok(u) => u,
        Err(r) => return r,
    };

    let session_id = path.into_inner();
    let mut ctx = match load_agent_context(pool.get_ref(), session_id).await {
        Ok(Some(ctx)) => ctx,
        Ok(None) => {
            return HttpResponse::NotFound().json(error_response("NOT_FOUND", "Session not found"))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(error_response("DB_ERROR", e.to_string()))
        }
    };

    let target = match body.target_state.as_deref() {
        Some("semantic_analysis") => AgentState::SemanticAnalysis,
        Some("function_decomposition") => AgentState::FunctionDecomposition,
        Some("ontology_analysis") => AgentState::OntologyAnalysis { ontology_round: 0 },
        Some("module_creation") => AgentState::ModuleCreation,
        Some("block_creation") => AgentState::BlockCreation,
        Some("ontology_transfer") => AgentState::OntologyTransfer,
        Some("service_api") => AgentState::ServiceAPI,
        Some("composing") => AgentState::Composing,
        Some("verifying") => AgentState::Verifying {
            verification_round: 0,
        },
        Some("publishing") => AgentState::Publishing {
            publish_attempt: 0,
            last_error: None,
        },
        _ => AgentState::SemanticAnalysis,
    };

    let resume = app_agent::state::ResumeConfig {
        target_state: target,
        preserve_ontology: true,
        preserve_flow_plan: true,
        preserve_scratch: false,
        preserve_yaml_ops: false,
    };

    if let Err(e) = AppAgent::reset_to_checkpoint(&mut ctx, &resume) {
        return HttpResponse::InternalServerError().json(error_response("RESET_ERROR", e));
    }
    let _ = save_agent_context(pool.get_ref(), session_id, &ctx).await;
    let _ = update_session_status(pool.get_ref(), session_id, "active").await;

    HttpResponse::Ok().json(ApiResponse::success(json!({
        "status": "reset",
        "state": app_agent::state_name(&ctx.state)
    })))
}

fn status_to_progress(status: &str) -> u8 {
    match status {
        "active" | "idle" => 25,
        "generating" => 50,
        "completed" => 100,
        "abandoned" => 100,
        "interrupted" => 75,
        _ => 10,
    }
}

pub async fn progress_handler(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<i64>,
) -> HttpResponse {
    let _user = match require_auth(&req) {
        Ok(u) => u,
        Err(r) => return r,
    };
    let session_id = path.into_inner();

    let (tx, rx) = progress_channel(4);

    let pool = pool.get_ref().clone();
    tokio::spawn(async move {
        match get_session(&pool, session_id).await {
            Ok(Some(session)) => {
                let progress = status_to_progress(&session.status);
                let _ = tx
                    .send(ProgressEvent::info(
                        &session.status,
                        "Agent state",
                        &format!("Session {} is in state {}", session_id, session.status),
                        progress,
                    ))
                    .await;

                if session.status == "completed" || session.status == "abandoned" {
                    let _ = tx
                        .send(ProgressEvent::success(
                            "done",
                            "Complete",
                            "Session reached terminal state",
                            100,
                        ))
                        .await;
                }
            }
            Ok(None) => {
                let _ = tx
                    .send(ProgressEvent::error(
                        "not_found",
                        "Session not found",
                        &format!("No session with id {}", session_id),
                        0,
                    ))
                    .await;
            }
            Err(e) => {
                let _ = tx
                    .send(ProgressEvent::error(
                        "db_error",
                        "Database error",
                        &format!("Failed to load session: {}", e),
                        0,
                    ))
                    .await;
            }
        }
        // Channel closes when tx drops at end of scope.
    });

    sse_response(rx)
}

#[derive(Debug, Deserialize)]
pub struct CreateAppRequest {
    pub name: String,
    pub description: String,
}

pub async fn create_app_handler(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<CreateAppRequest>,
) -> HttpResponse {
    let user = match crate::handlers::require_auth(&req) {
        Ok(u) => u,
        Err(r) => return r,
    };
    let namespace = crate::middleware::extract_namespace(&req);
    if namespace.is_empty() {
        return HttpResponse::BadRequest().json(error_response(
            "MISSING_NAMESPACE",
            "namespace not available from auth context",
        ));
    }
    let name = body.name.trim();
    let description = body.description.trim();
    if name.is_empty() || description.is_empty() {
        return HttpResponse::BadRequest().json(error_response(
            "MISSING_FIELDS",
            "name and description are required",
        ));
    }

    match create_session(pool.get_ref(), name, None, &namespace).await {
        Ok(row) => {
            let mut ctx =
                ConversationContext::new(row.id, user.username.clone(), namespace.to_string());
            ctx.user_description = description.to_string();
            let _ = save_agent_context(pool.get_ref(), row.id, &ctx).await;
            let _ = add_message(pool.get_ref(), row.id, "user", description).await;
            let _ = update_session_status(pool.get_ref(), row.id, "app_creating").await;

            log::info!(
                "App creation started: session={} app='{}' ns={}",
                row.id,
                name,
                namespace
            );
            HttpResponse::Created().json(ApiResponse::success(serde_json::json!({
                "session": row_to_session(row, vec![]),
                "app_name": name,
            })))
        }
        Err(e) => {
            log::error!("create_app failed: {}", e);
            HttpResponse::InternalServerError().json(error_response("DB_ERROR", e.to_string()))
        }
    }
}
/// 解析 prototype.html 路径（防路径穿越）。
///
/// 坐标来自 LLM 产物（ConversationContext.namespace/app_name），
/// 必须限制为单段目录名，拒绝 `..`、`/`、`\` 及隐藏段。
/// 根目录优先取 `APPCREATOR_PROJECT_ROOT` 环境变量，否则回退到进程 CWD。
fn resolve_prototype_path(ctx: &ConversationContext) -> Result<std::path::PathBuf, HttpResponse> {
    let (namespace, app_name) = match (ctx.namespace.as_deref(), ctx.app_name.as_deref()) {
        (Some(ns), Some(name)) if !ns.is_empty() && !name.is_empty() => (ns, name),
        _ => {
            return Err(HttpResponse::NotFound().json(error_response(
                "PROTOTYPE_NOT_READY",
                "App has not been composed yet",
            )))
        }
    };

    let is_safe_segment = |s: &str| {
        !s.is_empty()
            && !s.contains("..")
            && !s.contains('/')
            && !s.contains('\\')
            && !s.starts_with('.')
    };
    if !is_safe_segment(namespace) || !is_safe_segment(app_name) {
        return Err(HttpResponse::BadRequest().json(error_response(
            "INVALID_APP_COORDS",
            "Invalid namespace/app_name in session context",
        )));
    }

    let root = std::env::var("APPCREATOR_PROJECT_ROOT")
        .ok()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    Ok(root
        .join("Pre-Proc")
        .join(namespace)
        .join("Apps")
        .join(app_name)
        .join("prototype.html"))
}

/// 预览原型产物：GET /sessions/{id}/prototype
///
/// 从 ConversationContext 解析 app 坐标（namespace + app_name），
/// 返回 Pre-Proc/{namespace}/Apps/{app_name}/prototype.html（text/html）。
pub async fn prototype_handler(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<i64>,
) -> HttpResponse {
    let _user = match require_auth(&req) {
        Ok(u) => u,
        Err(r) => return r,
    };
    let session_id = path.into_inner();

    let ctx = match load_agent_context(pool.get_ref(), session_id).await {
        Ok(Some(ctx)) => ctx,
        Ok(None) => {
            return HttpResponse::NotFound().json(error_response("NOT_FOUND", "Session not found"))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(error_response("DB_ERROR", e.to_string()))
        }
    };

    let proto_path = match resolve_prototype_path(&ctx) {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    match std::fs::read_to_string(&proto_path) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(_) => HttpResponse::NotFound().json(error_response(
            "PROTOTYPE_NOT_FOUND",
            format!(
                "prototype.html not found for app '{}/{}'",
                ctx.namespace.as_deref().unwrap_or("?"),
                ctx.app_name.as_deref().unwrap_or("?")
            ),
        )),
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sessions")
            .route("", web::get().to(list_sessions_handler))
            .route("", web::post().to(create_session_handler))
            .route("/{id}", web::get().to(get_session_handler))
            .route("/{id}/messages", web::post().to(add_message_handler))
            .route(
                "/{id}/generate-response",
                web::post().to(generate_response_handler),
            )
            .route("/{id}/progress", web::get().to(progress_handler))
            .route("/{id}/prototype", web::get().to(prototype_handler))
            .route("/{id}/interrupt", web::post().to(interrupt_handler))
            .route("/{id}/resume", web::post().to(resume_handler))
            .route("/{id}/reset-state", web::post().to(reset_state_handler)),
    );
    cfg.service(web::scope("/apps").route("", web::post().to(create_app_handler)));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_with(ns: Option<&str>, app: Option<&str>) -> ConversationContext {
        let mut ctx = ConversationContext::new(1, "test".to_string(), "Ns".to_string());
        ctx.namespace = ns.map(|s| s.to_string());
        ctx.app_name = app.map(|s| s.to_string());
        ctx
    }

    #[test]
    fn prototype_path_ok_for_valid_coords() {
        let ctx = ctx_with(Some("Cosmic-Tools"), Some("I_need_a"));
        let path = resolve_prototype_path(&ctx).expect("should resolve");
        assert!(path.ends_with("Pre-Proc/Cosmic-Tools/Apps/I_need_a/prototype.html"));
    }

    #[test]
    fn prototype_path_rejects_missing_coords() {
        assert!(resolve_prototype_path(&ctx_with(None, Some("a"))).is_err());
        assert!(resolve_prototype_path(&ctx_with(Some("ns"), None)).is_err());
    }

    #[test]
    fn prototype_path_rejects_traversal() {
        for bad in ["../etc", "a/b", "a\\b", ".hidden", "x..y"] {
            let ctx = ctx_with(Some("ns"), Some(bad));
            assert!(
                resolve_prototype_path(&ctx).is_err(),
                "must reject app_name: {bad}"
            );
        }
        let ctx = ctx_with(Some("../ns"), Some("app"));
        assert!(resolve_prototype_path(&ctx).is_err());
    }
}
