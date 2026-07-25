use actix_web::{web, HttpRequest, HttpResponse};
use common::context::{RequestContext, RequestContextExt};
use serde::Deserialize;
use crate::chat;
use serde_json::json;

pub async fn list_apps(req: HttpRequest) -> HttpResponse {
    let ns = crate::middleware::extract_namespace(&req);
    if ns.is_empty() {
        return HttpResponse::BadRequest().json(json!({"error":"no_namespace"}));
    }
    HttpResponse::Ok().json(json!({"apps": crate::app_repository::list_apps(&ns)}))
}

pub async fn get_app(req: HttpRequest, path: web::Path<String>) -> HttpResponse {
    let ns = crate::middleware::extract_namespace(&req);
    if ns.is_empty() {
        return HttpResponse::BadRequest().json(json!({"error":"no_namespace"}));
    }
    let code = path.into_inner();
    if !is_safe_path_segment(&code) {
        return HttpResponse::BadRequest().json(json!({"error":"invalid_code"}));
    }
    let dir = crate::app_repository::apps_dir(&ns);
    match crate::app_repository::find_app_by_code(&dir, &ns, &code) {
        Some((_, val)) => HttpResponse::Ok().json(json!({"app": val})),
        None => HttpResponse::NotFound().json(json!({"error":"not_found"})),
    }
}

pub async fn delete_app(req: HttpRequest, path: web::Path<String>) -> HttpResponse {
    let ns = crate::middleware::extract_namespace(&req);
    if ns.is_empty() {
        return HttpResponse::BadRequest().json(json!({"error":"no_namespace"}));
    }
    let code = path.into_inner();
    if !is_safe_path_segment(&code) {
        return HttpResponse::BadRequest().json(json!({"error":"invalid_code"}));
    }
    let dir = crate::app_repository::apps_dir(&ns);
    match crate::app_repository::find_app_by_code(&dir, &ns, &code) {
        Some((canonical, _)) => match std::fs::remove_dir_all(&canonical) {
            Ok(()) => HttpResponse::Ok().json(json!({"status":"deleted"})),
            Err(e) => HttpResponse::InternalServerError()
                .json(json!({"error":"delete_failed","message":e.to_string()})),
        },
        None => HttpResponse::NotFound().json(json!({"error":"not_found"})),
    }
}

fn is_safe_path_segment(seg: &str) -> bool {
    !seg.is_empty()
        && seg != "."
        && seg != ".."
        && !seg.contains('/')
        && !seg.contains('\\')
        && !seg.starts_with('.')
}

pub fn require_auth(req: &HttpRequest) -> Result<RequestContext, HttpResponse> {
    req.context()
        .ok_or_else(|| HttpResponse::Unauthorized().json(json!({"error":"unauthorized"})))
}

pub async fn get_current_user(req: HttpRequest) -> HttpResponse {
    let user = match require_auth(&req) {
        Ok(u) => u,
        Err(r) => return r,
    };
    let namespace = crate::middleware::extract_namespace(&req);
    HttpResponse::Ok().json(json!({"user": {
        "id": user.user_id, "username": user.username, "email": user.email,
        "is_superuser": user.is_superuser, "namespace": namespace,
    }}))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
}

pub async fn login_standalone(
    pool: web::Data<sqlx::PgPool>,
    body: web::Json<LoginRequest>,
) -> HttpResponse {
    let cfg = crate::auth_config::auth_config();
    if cfg.mode != crate::auth_config::AuthMode::Standalone {
        return HttpResponse::NotFound().json(json!({"error":"not_found"}));
    }
    let raw = body.username.trim();
    if raw.is_empty() {
        return HttpResponse::BadRequest()
            .json(json!({"error":"bad_request","message":"Username required"}));
    }
    let un = raw.to_lowercase();
    let existing = match sqlx::query_as::<_, (i64, String, String)>(
        "SELECT id, username, namespace FROM app_creator.users WHERE username_norm = $1",
    )
    .bind(&un)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(row) => row,
        Err(e) => {
            log::error!("login_standalone: DB lookup failed: {e}");
            return HttpResponse::InternalServerError()
                .json(json!({"error":"db_error","message":"Login check failed, ensure database is initialized (ensure-schema.sh + backend/ddl/*.sql)"}));
        }
    };
    let (user_id, username, namespace, is_new) = match existing {
        Some((id, uname, ns)) => (id, uname, ns, false),
        None => {
            let ns = match derive_namespace(raw) {
                Ok(n) => n,
                Err(e) => {
                    return HttpResponse::BadRequest()
                        .json(json!({"error":"bad_request","message":e}))
                }
            };
            match sqlx::query_as::<_, (i64,)>(
                "INSERT INTO app_creator.users (username, username_norm, namespace) VALUES ($1,$2,$3) RETURNING id",
            ).bind(raw).bind(&un).bind(&ns).fetch_optional(pool.get_ref()).await {
                Ok(Some((id,))) => (id, raw.to_string(), ns, true),
                _ => return HttpResponse::Conflict().json(json!({"error":"namespace_conflict","message":format!("Namespace '{ns}' taken")})),
            }
        }
    };
    let now = chrono::Utc::now();
    let exp = (now.timestamp() + 1800) as usize;
    let claims = crate::auth_config::StandaloneClaims {
        sub: user_id.to_string(),
        email: format!("{username}@standalone.local"),
        exp: exp as i64,
        iat: now.timestamp() as i64,
        username: username.clone(),
        namespace: namespace.clone(),
        iss: Some("app-creator-standalone".to_string()),
        sid: String::new(),
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::ES256),
        &claims,
        cfg.encoding_key.as_ref().expect("encoding_key"),
    )
    .expect("JWT signing");

    // Issue refresh token (best-effort)
    let refresh_token = chat::issue_refresh_token(pool.get_ref(), user_id)
        .await
        .unwrap_or_else(|e| {
            log::warn!("Failed to issue refresh token: {e}");
            String::new()
        });

    let st = if is_new { 201 } else { 200 };
    HttpResponse::build(actix_web::http::StatusCode::from_u16(st).unwrap())
        .json(json!({"token":token,"refresh_token":refresh_token,"user":{"id":user_id,"username":username,"namespace":namespace,"is_new":is_new}}))
}

fn derive_namespace(raw: &str) -> Result<String, &'static str> {
    let s: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    if s.is_empty() {
        return Err("No alphanumeric chars");
    }
    let pascal = s
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string() + &s[1..])
        .unwrap_or_default();
    Ok(format!("NS-{pascal}"))

}
pub async fn refresh_token(
    pool: web::Data<sqlx::PgPool>,
    body: web::Json<RefreshRequest>,
) -> HttpResponse {
    let cfg = crate::auth_config::auth_config();
    if cfg.mode != crate::auth_config::AuthMode::Standalone {
        return HttpResponse::NotFound().json(json!({"error":"not_found"}));
    }

    let user_id = match chat::consume_refresh_token(pool.get_ref(), &body.refresh_token).await {
        Ok(Some(id)) => id,
        Ok(None) => return HttpResponse::Unauthorized().json(json!({"error":"invalid_refresh_token"})),
        Err(e) => {
            log::error!("refresh_token DB error: {e}");
            return HttpResponse::InternalServerError().json(json!({"error":"db_error"}));
        }
    };

    let (username, namespace) = match sqlx::query_as::<_, (String, String)>(
        "SELECT username, namespace FROM app_creator.users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(row)) => row,
        _ => return HttpResponse::NotFound().json(json!({"error":"user_not_found"})),
    };

    let now = chrono::Utc::now();
    let claims = crate::auth_config::StandaloneClaims {
        sub: user_id.to_string(),
        email: format!("{username}@standalone.local"),
        exp: (now.timestamp() + 1800) as i64,
        iat: now.timestamp() as i64,
        username: username.clone(),
        namespace: namespace.clone(),
        iss: Some("app-creator-standalone".to_string()),
        sid: String::new(),
    };
    let access_token = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::ES256),
        &claims,
        cfg.encoding_key.as_ref().expect("encoding_key"),
    )
    .expect("JWT signing");

    let new_refresh = chat::issue_refresh_token(pool.get_ref(), user_id)
        .await
        .unwrap_or_default();

    HttpResponse::Ok().json(json!({
        "token": access_token,
        "refresh_token": new_refresh,
        "user": {"id": user_id, "username": username, "namespace": namespace}
    }))
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn derive_basic() {
        assert_eq!(derive_namespace("alice").unwrap(), "NS-Alice");
    }
    #[test]
    fn derive_special() {
        assert_eq!(derive_namespace("bob.smith!").unwrap(), "NS-Bobsmith");
    }
    #[test]
    fn derive_invalid() {
        assert!(derive_namespace("!!!").is_err());
    }
    #[test]
    fn derive_empty() {
        assert!(derive_namespace("").is_err());
    }
    #[test]
    fn safe_ok() {
        assert!(is_safe_path_segment("valid"));
    }
    #[test]
    fn safe_dotdot() {
        assert!(!is_safe_path_segment(".."));
    }
    #[test]
    fn safe_slash() {
        assert!(!is_safe_path_segment("a/b"));
    }
}
