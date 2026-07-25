//! Dual-mode authentication middleware.
//!
//! Detects auth mode at startup via `auth_config::init_auth_config()`:
//! - SSO_JWT_PUBLIC_KEY configured → SSO mode (ES256, issuer "app-creator"|"alioth-sso")
//! - Missing → Standalone mode (ES256, issuer "app-creator-standalone")
//!
//! Both modes inject RequestContext + namespace (as String extension).
//! Downstream handlers are auth-mode-agnostic.

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

use crate::auth_config;

/// JWT claims for both SSO and Standalone modes.
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
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub namespace: String,
    #[serde(default)]
    pub sid: String,
}

impl AppClaims {
    pub fn user_id(&self) -> i64 {
        self.sub.parse().unwrap_or(0)
    }
}

fn extract_token(req: &ServiceRequest) -> Option<String> {
    req.headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|v| v.to_string())
}

fn verify_token(
    token: &str,
    key: &DecodingKey,
    mode: &auth_config::AuthMode,
) -> Result<AppClaims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(Algorithm::ES256);
    match mode {
        auth_config::AuthMode::Sso => {
            validation.set_issuer(&["app-creator", "alioth-sso"]);
        }
        auth_config::AuthMode::Standalone => {
            validation.set_issuer(&["app-creator-standalone"]);
        }
    }
    decode::<AppClaims>(token, key, &validation).map(|data| data.claims)
}

#[derive(Clone, Default)]
pub struct SsoAuthMiddleware;

impl SsoAuthMiddleware {
    pub fn new() -> Self {
        Self
    }
}

pub struct SsoAuthMiddlewareService<S> {
    service: Rc<S>,
    mode: auth_config::AuthMode,
    key: DecodingKey,
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
        let cfg = auth_config::auth_config();
        let mode = cfg.mode.clone();
        let key = cfg.decoding_key.clone();
        ready(Ok(SsoAuthMiddlewareService {
            service: Rc::new(service),
            mode,
            key,
        }))
    }
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
        let mode = self.mode.clone();
        let key = self.key.clone();

        Box::pin(async move {
            let path = req.path();

            // Public paths — no auth required
            if path == "/health"
                || path == "/api/creator/status"
                || path == "/api/creator/auth/login"
            {
                let ctx = RequestContext::with_username(0, "system@appcreator.local", "system");
                req.extensions_mut().insert(ctx);
                req.extensions_mut().insert(String::new());
                let res = service.call(req).await?;
                return Ok(res.map_into_left_body());
            }

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

            match verify_token(&token, &key, &mode) {
                Ok(claims) => {
                    // SSO mode: reject if namespace claim is missing or empty
                    if matches!(&mode, auth_config::AuthMode::Sso) && claims.namespace.trim().is_empty() {
                        let response = HttpResponse::Forbidden().json(serde_json::json!({
                            "error": "no_namespace",
                            "message": "SSO JWT must include a namespace claim"
                        }));
                        return Ok(req.into_response(response).map_into_right_body());
                    }

                    let (ctx, ns) = match &mode {
                        auth_config::AuthMode::Sso => {
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
                            (ctx, claims.namespace.trim().to_owned())
                        }
                        auth_config::AuthMode::Standalone => {
                            let ctx = RequestContext::with_username(
                                claims.user_id(),
                                claims.email.clone(),
                                claims.username.clone(),
                            );
                            (ctx, claims.namespace.clone())
                        }
                    };
                    req.extensions_mut().insert(ctx);
                    req.extensions_mut().insert(ns);
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

pub fn extract_user(req: &actix_web::HttpRequest) -> Option<RequestContext> {
    req.context()
}

pub fn extract_user_id(req: &actix_web::HttpRequest) -> i64 {
    common::context::extract_user_id(req).unwrap_or(0)
}

/// Extract the namespace injected by middleware (standalone mode).
/// Returns empty string in SSO mode or for public paths.
pub fn extract_namespace(req: &actix_web::HttpRequest) -> String {
    req.extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_default()
}
