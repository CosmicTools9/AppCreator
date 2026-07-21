//! AppCreator API handlers — full implementation with in-memory store.
//! Each handler: auth check → business logic → JSON response.
//! Swappable to sqlx Pool when DB migration is ready.

use actix_web::{web, HttpRequest, HttpResponse};
use jsonwebtoken::DecodingKey;

use crate::auth;
use crate::models::*;
use crate::store::AppStore;

/// Load decoding key once at startup.
pub fn load_key() -> Option<DecodingKey> {
    match auth::load_decoding_key() {
        Ok(k) => Some(k),
        Err(e) => {
            eprintln!("WARNING: SSO_JWT_PUBLIC_KEY not set — protected endpoints reject with 401. {}", e);
            None
        }
    }
}

/// Require valid JWT or 401.
fn with_auth(req: &HttpRequest, key: &Option<DecodingKey>) -> Result<auth::AuthUser, HttpResponse> {
    let key = key.as_ref().ok_or_else(|| {
        HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "auth_not_configured",
            "message": "SSO_JWT_PUBLIC_KEY not configured on server"
        }))
    })?;
    auth::authenticate(req, key).map_err(|e| {
        HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "unauthorized", "message": e
        }))
    })
}

// ── P0: Projects CRUD ────────────────────────────────

pub async fn list_projects(
    req: HttpRequest,
    key: web::Data<Option<DecodingKey>>,
    store: web::Data<AppStore>,
    query: web::Query<PaginationParams>,
) -> HttpResponse {
    let user = match with_auth(&req, key.get_ref()) { Ok(u) => u, Err(r) => return r };
    let (projects, total) = store.list_projects(&user.username, &query).await;
    HttpResponse::Ok().json(serde_json::json!({ "projects": projects, "total": total }))
}

pub async fn create_project(
    req: HttpRequest,
    key: web::Data<Option<DecodingKey>>,
    store: web::Data<AppStore>,
    body: web::Json<CreateProjectRequest>,
) -> HttpResponse {
    let user = match with_auth(&req, key.get_ref()) { Ok(u) => u, Err(r) => return r };
    let project = store.create_project(body.into_inner(), user.user_id).await;
    HttpResponse::Created().json(serde_json::json!({ "project": project }))
}

pub async fn get_project(
    req: HttpRequest,
    key: web::Data<Option<DecodingKey>>,
    store: web::Data<AppStore>,
    path: web::Path<i64>,
) -> HttpResponse {
    let _user = match with_auth(&req, key.get_ref()) { Ok(u) => u, Err(r) => return r };
    match store.get_project(path.into_inner()).await {
        Some(p) => HttpResponse::Ok().json(serde_json::json!({ "project": p })),
        None => HttpResponse::NotFound().json(serde_json::json!({ "error": "not_found", "message": "Project not found" })),
    }
}

pub async fn update_project(
    req: HttpRequest,
    key: web::Data<Option<DecodingKey>>,
    store: web::Data<AppStore>,
    path: web::Path<i64>,
    body: web::Json<UpdateProjectRequest>,
) -> HttpResponse {
    let _user = match with_auth(&req, key.get_ref()) { Ok(u) => u, Err(r) => return r };
    match store.update_project(path.into_inner(), body.into_inner()).await {
        Some(p) => HttpResponse::Ok().json(serde_json::json!({ "project": p })),
        None => HttpResponse::NotFound().json(serde_json::json!({ "error": "not_found", "message": "Project not found" })),
    }
}

pub async fn delete_project(
    req: HttpRequest,
    key: web::Data<Option<DecodingKey>>,
    store: web::Data<AppStore>,
    path: web::Path<i64>,
) -> HttpResponse {
    let _user = match with_auth(&req, key.get_ref()) { Ok(u) => u, Err(r) => return r };
    if store.delete_project(path.into_inner()).await {
        HttpResponse::Ok().json(serde_json::json!({ "status": "deleted" }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({ "error": "not_found", "message": "Project not found" }))
    }
}

// ── P1: Sessions + Templates ─────────────────────────

pub async fn create_session(
    req: HttpRequest,
    key: web::Data<Option<DecodingKey>>,
    store: web::Data<AppStore>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let user = match with_auth(&req, key.get_ref()) { Ok(u) => u, Err(r) => return r };
    let project_id = body.get("project_id").and_then(|v| v.as_i64()).unwrap_or(0);
    let session = store.create_session(project_id, user.user_id).await;
    HttpResponse::Created().json(serde_json::json!({ "session": session }))
}

pub async fn get_session(
    req: HttpRequest,
    key: web::Data<Option<DecodingKey>>,
    store: web::Data<AppStore>,
    path: web::Path<i64>,
) -> HttpResponse {
    let _user = match with_auth(&req, key.get_ref()) { Ok(u) => u, Err(r) => return r };
    match store.get_session(path.into_inner()).await {
        Some(s) => {
            let messages = store.list_messages(s.id).await;
            HttpResponse::Ok().json(serde_json::json!({ "session": s, "messages": messages }))
        }
        None => HttpResponse::NotFound().json(serde_json::json!({ "error": "not_found" })),
    }
}

pub async fn send_message(
    req: HttpRequest,
    key: web::Data<Option<DecodingKey>>,
    store: web::Data<AppStore>,
    path: web::Path<i64>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let _user = match with_auth(&req, key.get_ref()) { Ok(u) => u, Err(r) => return r };
    let session_id = path.into_inner();
    let content = body.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let msg = store.add_message(session_id, "user".into(), content).await;
    // TODO: integrate AI response generation
    HttpResponse::Ok().json(serde_json::json!({ "message": msg }))
}

pub async fn list_templates(
    req: HttpRequest,
    key: web::Data<Option<DecodingKey>>,
    store: web::Data<AppStore>,
) -> HttpResponse {
    let _user = match with_auth(&req, key.get_ref()) { Ok(u) => u, Err(r) => return r };
    let templates = store.list_templates().await;
    HttpResponse::Ok().json(serde_json::json!({ "templates": templates }))
}

pub async fn get_template(
    req: HttpRequest,
    key: web::Data<Option<DecodingKey>>,
    store: web::Data<AppStore>,
    path: web::Path<i64>,
) -> HttpResponse {
    let _user = match with_auth(&req, key.get_ref()) { Ok(u) => u, Err(r) => return r };
    match store.get_template(path.into_inner()).await {
        Some(t) => HttpResponse::Ok().json(serde_json::json!({ "template": t })),
        None => HttpResponse::NotFound().json(serde_json::json!({ "error": "not_found" })),
    }
}

// ── P2: Builds + Deployments ─────────────────────────

pub async fn trigger_build(
    req: HttpRequest,
    key: web::Data<Option<DecodingKey>>,
    store: web::Data<AppStore>,
    path: web::Path<i64>,
) -> HttpResponse {
    let _user = match with_auth(&req, key.get_ref()) { Ok(u) => u, Err(r) => return r };
    let project_id = path.into_inner();
    if store.get_project(project_id).await.is_none() {
        return HttpResponse::NotFound().json(serde_json::json!({ "error": "not_found" }));
    }
    let build = store.create_build(project_id).await;
    HttpResponse::Created().json(serde_json::json!({ "build": build }))
}

pub async fn list_builds(
    req: HttpRequest,
    key: web::Data<Option<DecodingKey>>,
    store: web::Data<AppStore>,
    path: web::Path<i64>,
) -> HttpResponse {
    let _user = match with_auth(&req, key.get_ref()) { Ok(u) => u, Err(r) => return r };
    let builds = store.list_builds(path.into_inner()).await;
    HttpResponse::Ok().json(serde_json::json!({ "builds": builds }))
}

pub async fn get_build(
    req: HttpRequest,
    key: web::Data<Option<DecodingKey>>,
    store: web::Data<AppStore>,
    path: web::Path<(i64, i64)>,
) -> HttpResponse {
    let _user = match with_auth(&req, key.get_ref()) { Ok(u) => u, Err(r) => return r };
    let (_project_id, build_id) = path.into_inner();
    match store.get_build(build_id).await {
        Some(b) => HttpResponse::Ok().json(serde_json::json!({ "build": b })),
        None => HttpResponse::NotFound().json(serde_json::json!({ "error": "not_found" })),
    }
}

pub async fn trigger_deploy(
    req: HttpRequest,
    key: web::Data<Option<DecodingKey>>,
    store: web::Data<AppStore>,
    path: web::Path<i64>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let _user = match with_auth(&req, key.get_ref()) { Ok(u) => u, Err(r) => return r };
    let project_id = path.into_inner();
    let build_id = body.get("build_id").and_then(|v| v.as_i64()).unwrap_or(0);
    let target = body.get("target").and_then(|v| v.as_str()).unwrap_or("staging").to_string();
    let dep = store.create_deployment(project_id, build_id, target).await;
    HttpResponse::Created().json(serde_json::json!({ "deployment": dep }))
}

pub async fn list_deployments(
    req: HttpRequest,
    key: web::Data<Option<DecodingKey>>,
    store: web::Data<AppStore>,
    path: web::Path<i64>,
) -> HttpResponse {
    let _user = match with_auth(&req, key.get_ref()) { Ok(u) => u, Err(r) => return r };
    let deployments = store.list_deployments(path.into_inner()).await;
    HttpResponse::Ok().json(serde_json::json!({ "deployments": deployments }))
}

// ── P3: User ─────────────────────────────────────────

pub async fn get_current_user(
    req: HttpRequest,
    key: web::Data<Option<DecodingKey>>,
) -> HttpResponse {
    let user = match with_auth(&req, key.get_ref()) { Ok(u) => u, Err(r) => return r };
    HttpResponse::Ok().json(serde_json::json!({
        "user": {
            "id": user.user_id,
            "username": user.username,
            "email": user.email,
            "is_superuser": user.is_superuser,
        }
    }))
}
