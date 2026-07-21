//! AppCreator API handlers — full implementation with in-memory store.

use actix_web::{web, HttpRequest, HttpResponse};
use common::context::{RequestContext, RequestContextExt};

use crate::models::*;
use crate::store::AppStore;

/// Require a valid request context or return 401.
pub fn require_auth(req: &HttpRequest) -> Result<RequestContext, HttpResponse> {
    req.context().ok_or_else(|| {
        HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "unauthorized",
            "message": "Valid SSO authentication is required"
        }))
    })
}

// ── P0: Projects CRUD ────────────────────────────────

pub async fn list_projects(
    req: HttpRequest,
    store: web::Data<AppStore>, query: web::Query<PaginationParams>,
) -> HttpResponse {
    let user = match require_auth(&req) { Ok(u) => u, Err(r) => return r };
    let (projects, total) = store.list_projects(&user.username, &query).await;
    HttpResponse::Ok().json(serde_json::json!({ "projects": projects, "total": total }))
}

pub async fn create_project(
    req: HttpRequest,
    store: web::Data<AppStore>, body: web::Json<CreateProjectRequest>,
) -> HttpResponse {
    let user = match require_auth(&req) { Ok(u) => u, Err(r) => return r };
    let project = store.create_project(body.into_inner(), user.user_id).await;
    HttpResponse::Created().json(serde_json::json!({ "project": project }))
}

pub async fn get_project(
    req: HttpRequest,
    store: web::Data<AppStore>, path: web::Path<i64>,
) -> HttpResponse {
    let _user = match require_auth(&req) { Ok(u) => u, Err(r) => return r };
    match store.get_project(path.into_inner()).await {
        Some(p) => HttpResponse::Ok().json(serde_json::json!({ "project": p })),
        None => HttpResponse::NotFound().json(serde_json::json!({"error":"not_found"})),
    }
}

pub async fn update_project(
    req: HttpRequest,
    store: web::Data<AppStore>, path: web::Path<i64>, body: web::Json<UpdateProjectRequest>,
) -> HttpResponse {
    let _user = match require_auth(&req) { Ok(u) => u, Err(r) => return r };
    match store.update_project(path.into_inner(), body.into_inner()).await {
        Some(p) => HttpResponse::Ok().json(serde_json::json!({ "project": p })),
        None => HttpResponse::NotFound().json(serde_json::json!({"error":"not_found"})),
    }
}

pub async fn delete_project(
    req: HttpRequest,
    store: web::Data<AppStore>, path: web::Path<i64>,
) -> HttpResponse {
    let _user = match require_auth(&req) { Ok(u) => u, Err(r) => return r };
    if store.delete_project(path.into_inner()).await {
        HttpResponse::Ok().json(serde_json::json!({"status":"deleted"}))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({"error":"not_found"}))
    }
}

// ── P1: Templates ────────────────────────────────────

pub async fn list_templates(
    req: HttpRequest,
    store: web::Data<AppStore>,
    pool: web::Data<sqlx::PgPool>,
) -> HttpResponse {
    let _user = match require_auth(&req) { Ok(u) => u, Err(r) => return r };

    match crate::meta_reader::load_templates(pool.get_ref()).await {
        Ok(collections) => {
            let templates: Vec<serde_json::Value> = collections.iter().map(|c| {
                serde_json::json!({
                    "id": c.table_name,
                    "name": c.name,
                    "description": c.biz_description,
                    "category": c.r#type,
                    "source": "isahl_meta",
                })
            }).collect();
            return HttpResponse::Ok().json(serde_json::json!({ "templates": templates }));
        }
        Err(e) => log::warn!("isahl_meta query failed, falling back to in-memory: {}", e),
    }

    // Fallback — in-memory store
    HttpResponse::Ok().json(serde_json::json!({ "templates": store.list_templates().await }))
}

pub async fn get_template(
    req: HttpRequest,
    store: web::Data<AppStore>, path: web::Path<i64>,
) -> HttpResponse {
    let _user = match require_auth(&req) { Ok(u) => u, Err(r) => return r };
    match store.get_template(path.into_inner()).await {
        Some(t) => HttpResponse::Ok().json(serde_json::json!({ "template": t })),
        None => HttpResponse::NotFound().json(serde_json::json!({"error":"not_found"})),
    }
}

// ── P2: Builds + Deployments ─────────────────────────

pub async fn trigger_build(
    req: HttpRequest,
    store: web::Data<AppStore>, path: web::Path<i64>,
) -> HttpResponse {
    let _user = match require_auth(&req) { Ok(u) => u, Err(r) => return r };
    let project_id = path.into_inner();
    let project = match store.get_project(project_id).await {
        Some(p) => p,
        None => return HttpResponse::NotFound().json(serde_json::json!({"error":"not_found"})),
    };

    let app_config = crate::AppConfig {
        name: project.name.clone(),
        namespace: project.namespace.clone(),
        project_root: std::env::current_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        version: "0.1.0".into(),
        alioth_model_version: "10.0.0".into(),
        port: 8080,
    };
    let gateway_tag = format!("gateway-{}", &app_config.namespace);

    match crate::docker::build(&app_config, &gateway_tag, &[49495]) {
        Ok(output) => {
            let build = store.create_build(project_id).await;
            // Write artifacts if possible
            if !project.description.is_empty() {
                // ignore write errors in dev
                let _ = std::fs::create_dir_all(
                    std::path::Path::new(&app_config.project_root)
                        .join("Pre-Proc").join(&app_config.namespace)
                        .join("Apps").join(&app_config.name)
                );
            }
            log::info!("Build completed for project {}: {} artifacts", project_id, output.artifacts.len());
            HttpResponse::Created().json(serde_json::json!({
                "build": build,
                "artifacts": output.artifacts,
                "lock": output.lock_content,
            }))
        }
        Err(e) => {
            let build = store.create_build(project_id).await;
            log::warn!("Build failed for project {}: {}", project_id, e);
            HttpResponse::Ok().json(serde_json::json!({
                "build": build,
                "warning": format!("Build skipped (dev mode): {}", e),
            }))
        }
    }
}

pub async fn list_builds(
    req: HttpRequest,
    store: web::Data<AppStore>, path: web::Path<i64>,
) -> HttpResponse {
    let _user = match require_auth(&req) { Ok(u) => u, Err(r) => return r };
    HttpResponse::Ok().json(serde_json::json!({ "builds": store.list_builds(path.into_inner()).await }))
}

pub async fn get_build(
    req: HttpRequest,
    store: web::Data<AppStore>, path: web::Path<(i64, i64)>,
) -> HttpResponse {
    let _user = match require_auth(&req) { Ok(u) => u, Err(r) => return r };
    let (_project_id, build_id) = path.into_inner();
    match store.get_build(build_id).await {
        Some(b) => HttpResponse::Ok().json(serde_json::json!({ "build": b })),
        None => HttpResponse::NotFound().json(serde_json::json!({"error":"not_found"})),
    }
}

pub async fn trigger_deploy(
    req: HttpRequest,
    store: web::Data<AppStore>, path: web::Path<i64>, body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let _user = match require_auth(&req) { Ok(u) => u, Err(r) => return r };
    let project_id = path.into_inner();
    let build_id = body.get("build_id").and_then(|v| v.as_i64()).unwrap_or(0);
    let target = body.get("target").and_then(|v| v.as_str()).unwrap_or("staging").to_string();
    HttpResponse::Created().json(serde_json::json!({ "deployment": store.create_deployment(project_id, build_id, target).await }))
}

pub async fn list_deployments(
    req: HttpRequest,
    store: web::Data<AppStore>, path: web::Path<i64>,
) -> HttpResponse {
    let _user = match require_auth(&req) { Ok(u) => u, Err(r) => return r };
    HttpResponse::Ok().json(serde_json::json!({ "deployments": store.list_deployments(path.into_inner()).await }))
}

// ── P3: User ─────────────────────────────────────────

pub async fn get_current_user(
    req: HttpRequest,
) -> HttpResponse {
    let user = match require_auth(&req) { Ok(u) => u, Err(r) => return r };
    HttpResponse::Ok().json(serde_json::json!({
        "user": { "id": user.user_id, "username": user.username, "email": user.email, "is_superuser": user.is_superuser }
    }))
}
