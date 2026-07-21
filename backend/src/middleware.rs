//! SSO JWT authentication middleware.
//!
//! This middleware verifies ES256 (EC P-256) Bearer tokens issued by SSO.
//! It is a local implementation because the current SSO token shape does not yet
//! include the `namespace`/`aud`/`iss` claims required by `common::auth::verify_sso_token`.
//! Once SSO adopts the namespace-aware contract, this should be replaced by the
//! shared `common::middleware::NgacPepMiddleware`.

use actix_web::{
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header::AUTHORIZATION,
    HttpMessage, HttpResponse,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

use common::context::{RequestContext, RequestContextExt};

/// SSO JWT claims issued by the current identity service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppClaims {
    pub sub: String,
    pub email: String,
    #[serde(default)]
    pub is_superuser: bool,
    pub exp: i64,
    pub iat: i64,
    #[serde(default)]
    pub iss: Option<String>,
}

impl AppClaims {
    pub fn user_id(&self) -> i64 {
        self.sub.parse().unwrap_or(0)
    }
}

/// Load the SSO EC P-256 public key from the environment.
fn load_public_key() -> Option<DecodingKey> {
    let pem = std::env::var("SSO_JWT_PUBLIC_KEY").ok()?;
    DecodingKey::from_ec_pem(pem.as_bytes()).ok()
}

/// Extract a Bearer token from the request headers.
fn extract_token(req: &ServiceRequest) -> Option<String> {
    req.headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|v| v.to_string())
}

/// Verify an ES256 token and return the claims.
fn verify_token(token: &str, key: &DecodingKey) -> Result<AppClaims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(Algorithm::ES256);
    validation.set_issuer(&["app-creator", "alioth-sso"]);
    decode::<AppClaims>(token, key, &validation).map(|data| data.claims)
}

#[derive(Clone, Default)]
pub struct SsoAuthMiddleware;

impl SsoAuthMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for SsoAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = SsoAuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let key = load_public_key();
        ready(Ok(SsoAuthMiddlewareService {
            service: Rc::new(service),
            key,
        }))
    }
}

pub struct SsoAuthMiddlewareService<S> {
    service: Rc<S>,
    key: Option<DecodingKey>,
}

impl<S, B> Service<ServiceRequest> for SsoAuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let key = self.key.clone();

        Box::pin(async move {
            let path = req.path();

            // Public paths bypass authentication but still receive a system context.
            if path == "/health" || path == "/api/creator/status" {
                let ctx = RequestContext::with_username(0, "system@appcreator.local", "system");
                req.extensions_mut().insert(ctx);
                let res = service.call(req).await?;
                return Ok(res.map_into_left_body());
            }

            let key = match key {
                Some(k) => k,
                None => {
                    let response = HttpResponse::Unauthorized().json(serde_json::json!({
                        "error": "auth_not_configured",
                        "message": "SSO_JWT_PUBLIC_KEY not configured on server"
                    }));
                    return Ok(req.into_response(response).map_into_right_body());
                }
            };

            let token = match extract_token(&req) {
                Some(t) => t,
                None => {
                    let response = HttpResponse::Unauthorized().json(serde_json::json!({
                        "error": "missing_auth",
                        "message": "Authorization header is required"
                    }));
                    return Ok(req.into_response(response).map_into_right_body());
                }
            };

            match verify_token(&token, &key) {
                Ok(claims) => {
                    let username = claims
                        .email
                        .split('@')
                        .next()
                        .unwrap_or(&claims.email)
                        .to_string();
                    let ctx = RequestContext::with_username_and_admin(
                        claims.user_id(),
                        claims.email.clone(),
                        username,
                        claims.is_superuser,
                    );
                    req.extensions_mut().insert(ctx);
                    req.extensions_mut().insert(claims);
                    let res = service.call(req).await?;
                    Ok(res.map_into_left_body())
                }
                Err(e) => {
                    log::warn!("JWT verification failed: {}", e);
                    let response = HttpResponse::Unauthorized().json(serde_json::json!({
                        "error": "invalid_token",
                        "message": format!("Invalid or expired token: {}", e)
                    }));
                    Ok(req.into_response(response).map_into_right_body())
                }
            }
        })
    }
}

/// Extract the authenticated user context from a request.
pub fn extract_user(req: &actix_web::HttpRequest) -> Option<RequestContext> {
    req.context()
}

/// Extract the authenticated user ID from a request.
pub fn extract_user_id(req: &actix_web::HttpRequest) -> i64 {
    common::context::extract_user_id(req).unwrap_or(0)
}
