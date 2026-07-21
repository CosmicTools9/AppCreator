//! Chat session management + AppAgent integration.
//!
//! Stores sessions/messages in `isahl_meta.meta_chat_sessions` / `meta_chat_messages`
//! and drives `app_agent::AppAgent` for LLM-based app creation.

use actix_web::{web, HttpRequest, HttpResponse};
use app_agent::llm::{GenerationOverrides, LlmError as AgentLlmError, LlmService as AgentLlmService};
use app_agent::{AgentState, AppAgent, ConversationContext};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::progress::{ProgressEvent, progress_channel, sse_response};
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
            .map_err(|e| AgentLlmError { message: e.to_string() })
    }

    async fn generate_with_system(
        &self,
        system: &str,
        prompt: &str,
    ) -> Result<String, AgentLlmError> {
        self.inner
            .generate_with_system_preamble(
                system,
                prompt,
                None,
                None,
                None,
                None,
                None,
            )
            .await
            .map_err(|e| AgentLlmError { message: e.to_string() })
    }

    async fn generate_with_params(
        &self,
        system: &str,
        prompt: &str,
        _overrides: GenerationOverrides,
    ) -> Result<String, AgentLlmError> {
        self.inner
            .generate_with_system_preamble(
                system,
                prompt,
                None,
                None,
                None,
                None,
                None,
            )
            .await
            .map_err(|e| AgentLlmError { message: e.to_string() })
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

pub async fn list_messages(pool: &PgPool, session_id: i64) -> Result<Vec<ChatMessageRow>, sqlx::Error> {
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

pub async fn create_session_handler(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<CreateSessionRequest>,
) -> HttpResponse {
    let user = match require_auth(&req) {
        Ok(u) => u,
        Err(r) => return r,
    };

    let namespace = body.namespace.trim();
    if namespace.is_empty() {
        return HttpResponse::BadRequest().json(error_response("MISSING_NAMESPACE", "namespace is required"));
    }

    let title = body.title.as_deref().unwrap_or("New session").trim();
    let title = if title.is_empty() { "New session" } else { title };

    match create_session(pool.get_ref(), title, body.app_instance_id, namespace).await {
        Ok(row) => {
            let mut ctx = ConversationContext::new(
                row.id,
                user.username.clone(),
                namespace.to_string(),
            );
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
            Err(e) => HttpResponse::InternalServerError().json(error_response("DB_ERROR", e.to_string())),
        },
        Ok(None) => HttpResponse::NotFound().json(error_response("NOT_FOUND", "Session not found")),
        Err(e) => HttpResponse::InternalServerError().json(error_response("DB_ERROR", e.to_string())),
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
        return HttpResponse::BadRequest().json(error_response("EMPTY_MESSAGE", "content is required"));
    }

    match add_message(pool.get_ref(), session_id, "user", &content).await {
        Ok(msg) => HttpResponse::Ok().json(ApiResponse::success(row_to_message(msg))),
        Err(e) => HttpResponse::InternalServerError().json(error_response("DB_ERROR", e.to_string())),
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
        Ok(None) => return HttpResponse::NotFound().json(error_response("NOT_FOUND", "Session not found")),
        Err(e) => return HttpResponse::InternalServerError().json(error_response("DB_ERROR", e.to_string())),
    };

    let adapter = EnvLlmAdapter::new(llm);
    let agent = AppAgent::new(pool.clone(), Box::new(adapter));

    match agent.run_single_step(&mut ctx, None::<&fn(app_agent::AgentProgress)>).await {
        Ok(result) => {
            let _ = save_agent_context(pool.as_ref(), session_id, &ctx).await;

            let assistant_text = if result.is_terminal {
                result.message.clone()
            } else {
                format!("[{}] {}", app_agent::state_name(&ctx.state), result.message)
            };

            let _ = add_message(pool.as_ref(), session_id, "assistant", &assistant_text).await;

            let status = if result.is_terminal { "completed" } else { "active" };
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
        Ok(None) => return HttpResponse::NotFound().json(error_response("NOT_FOUND", "Session not found")),
        Err(e) => return HttpResponse::InternalServerError().json(error_response("DB_ERROR", e.to_string())),
    };

    AppAgent::request_interrupt(&mut ctx);
    let _ = save_agent_context(pool.get_ref(), session_id, &ctx).await;
    let _ = update_session_status(pool.get_ref(), session_id, "active").await;

    HttpResponse::Ok().json(ApiResponse::success(json!({"status": "interrupt_requested"})))
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
        Ok(None) => return HttpResponse::NotFound().json(error_response("NOT_FOUND", "Session not found")),
        Err(e) => return HttpResponse::InternalServerError().json(error_response("DB_ERROR", e.to_string())),
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
        Some("verifying") => AgentState::Verifying { verification_round: 0 },
        Some("publishing") => AgentState::Publishing { publish_attempt: 0, last_error: None },
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
    let _user = match require_auth(&req) { Ok(u) => u, Err(r) => return r };
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
                        &format!("Session {} is in state {}",
                            session_id, session.status
                        ),
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
    pub namespace: String,
}

pub async fn create_app_handler(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<CreateAppRequest>,
) -> HttpResponse {
    let user = match require_auth(&req) { Ok(u) => u, Err(r) => return r };

    let name = body.name.trim();
    let namespace = body.namespace.trim();
    if name.is_empty() || namespace.is_empty() {
        return HttpResponse::BadRequest().json(error_response(
            "INVALID_INPUT",
            "Both name and namespace are required",
        ));
    }

    match create_session(pool.get_ref(), name, None, namespace).await {
        Ok(row) => {
            let mut ctx = ConversationContext::new(
                row.id,
                user.username.clone(),
                namespace.to_string(),
            );
            ctx.user_description = name.to_string();
            let _ = save_agent_context(pool.get_ref(), row.id, &ctx).await;
            let _ = update_session_status(pool.get_ref(), row.id, "app_creating").await;

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
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sessions")
            .route("", web::post().to(create_session_handler))
            .route("/{id}", web::get().to(get_session_handler))
            .route("/{id}/messages", web::post().to(add_message_handler))
            .route("/{id}/generate-response", web::post().to(generate_response_handler))
            .route("/{id}/progress", web::get().to(progress_handler))
            .route("/{id}/interrupt", web::post().to(interrupt_handler))
            .route("/{id}/resume", web::post().to(resume_handler))
            .route("/{id}/reset-state", web::post().to(reset_state_handler)),
    );
    cfg.service(
        web::scope("/apps")
            .route("", web::post().to(create_app_handler)),
    );
}
