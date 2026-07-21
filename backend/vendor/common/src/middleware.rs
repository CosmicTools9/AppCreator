//! Alioth 中间件基础设施
//!
//! 提供跨应用共享的中间件组件，消除 Meta/Gateway/SSO 中的重复模式。

use std::sync::Arc;

// ── 公开路由匹配器 ──────────────────────────────────────────────────────────

/// 可配置的公开路由匹配器
///
/// 替代中间件中硬编码的公开路由列表，支持前缀匹配和精确匹配。
///
/// # 示例
///
/// ```rust,ignore
/// let matcher = PublicRouteMatcher::new()
///     .prefix("/api/meta/auth/")
///     .prefix("/health")
///     .exact("/api/meta/mise/config")
///     .predicate(|path, method| path.starts_with("/api/meta/mise/") && method == "GET");
///
/// assert!(matcher.is_public("/api/meta/auth/login", "POST"));
/// assert!(!matcher.is_public("/api/meta/users", "GET"));
/// ```
#[derive(Clone, Default)]
pub struct PublicRouteMatcher {
    prefixes: Arc<Vec<String>>,
    exact: Arc<Vec<String>>,
}

impl PublicRouteMatcher {
    /// 创建空匹配器（默认所有路由都不公开）
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加前缀匹配规则
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        Arc::make_mut(&mut self.prefixes).push(prefix.into());
        self
    }

    /// 添加精确匹配规则
    pub fn exact(mut self, path: impl Into<String>) -> Self {
        Arc::make_mut(&mut self.exact).push(path.into());
        self
    }

    /// 从多个前缀批量创建
    pub fn from_prefixes(prefixes: &[&str]) -> Self {
        Self {
            prefixes: Arc::new(prefixes.iter().map(|s| s.to_string()).collect()),
            exact: Arc::new(Vec::new()),
        }
    }

    /// 判断给定路径和方法是否为公开路由
    pub fn is_public(&self, path: &str, _method: &str) -> bool {
        // 精确匹配
        if self.exact.iter().any(|e| e == path) {
            return true;
        }
        // 前缀匹配
        if self.prefixes.iter().any(|p| path.starts_with(p)) {
            return true;
        }
        false
    }
}

// ── 认证上下文 trait ────────────────────────────────────────────────────────

/// 标准化认证上下文
///
/// 由认证中间件设置，供 handler 和下游中间件读取。
/// 实现此 trait 的类型应通过 actix-web 的 `req.extensions().insert(ctx)` 注入。
pub trait AuthContext: Clone + Send + Sync + 'static {
    /// 用户主键（ZUID）
    fn user_id(&self) -> i64;
    /// 用户邮箱
    fn email(&self) -> Option<String>;
    /// 用户名
    fn username(&self) -> Option<String>;
    /// 是否超级管理员
    fn is_superuser(&self) -> bool;
}

// ── NGAC PEP JWT 中间件 ─────────────────────────────────────────────────────

/// NGAC PEP Middleware — JWT authentication only.
///
/// Authorization is enforced centrally by Gateway NgacEnforcer.
/// This middleware is used when the module runs standalone during development.
use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use std::rc::Rc;

use crate::context::RequestContext;

/// NGAC PEP Middleware
///
/// Performs JWT verification and inserts RequestContext into extensions.
/// Permission checks are handled by Gateway in production.
#[derive(Clone)]
pub struct NgacPepMiddleware {
    jwt_secret: String,
}

impl NgacPepMiddleware {
    /// Create middleware with an explicit secret
    pub fn new(jwt_secret: impl Into<String>) -> Self {
        Self {
            jwt_secret: jwt_secret.into(),
        }
    }

    /// Create middleware from environment variables
    pub fn from_env() -> Self {
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "dev-secret-key-change-in-production-min-32-chars".to_string());
        Self::new(jwt_secret)
    }
}

impl<S, B> Transform<S, ServiceRequest> for NgacPepMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = NgacPepMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(NgacPepMiddlewareService {
            service: Rc::new(service),
            jwt_secret: self.jwt_secret.clone(),
        }))
    }
}

pub struct NgacPepMiddlewareService<S> {
    service: Rc<S>,
    jwt_secret: String,
}

impl<S, B> Service<ServiceRequest> for NgacPepMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let jwt_secret = self.jwt_secret.clone();

        Box::pin(async move {
            let path = req.path();

            // Allow health checks without auth
            if path == "/health" || path.starts_with("/health/") {
                let res = service.call(req).await?;
                return Ok(res.map_into_left_body());
            }

            // Extract JWT token
            let token = match extract_token(req.request()) {
                Some(t) => t,
                None => {
                    let response = HttpResponse::Unauthorized()
                        .json(error_body("MISSING_AUTH", "Authentication required"));
                    return Ok(req.into_response(response).map_into_right_body());
                }
            };

            // Verify token
            let claims = match verify_token(&token, &jwt_secret) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("JWT verification failed: {}", e);
                    let response = HttpResponse::Unauthorized().json(error_body(
                        "INVALID_TOKEN",
                        &format!("Invalid or expired token: {}", e),
                    ));
                    return Ok(req.into_response(response).map_into_right_body());
                }
            };

            // Build request context
            let user_id = claims.sub.parse().unwrap_or(0);
            let context =
                RequestContext::with_username(user_id, claims.username.clone(), claims.username);

            req.extensions_mut().insert(context);
            req.extensions_mut().insert(user_id);
            let res = service.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    username: String,
    #[serde(default)]
    exp: i64,
}

fn verify_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    use jsonwebtoken::{decode, DecodingKey, Validation};
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

fn extract_token(req: &actix_web::HttpRequest) -> Option<String> {
    if let Some(cookie) = req.cookie("access_token") {
        let value = cookie.value().trim();
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    req.headers()
        .get(actix_web::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
        .map(|s| s.trim().to_string())
}

fn error_body(code: &str, message: &str) -> serde_json::Value {
    serde_json::json!({
        "error": code,
        "message": message
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http, test, web, App, HttpResponse};

    async fn echo_handler(req: actix_web::HttpRequest) -> HttpResponse {
        let ctx = RequestContext::from_request(&req);
        HttpResponse::Ok().json(serde_json::json!({"ok": true, "user": ctx.map(|c| c.user_id)}))
    }

    fn test_secret() -> String {
        "test-secret-key-min-32-chars-long".to_string()
    }

    fn make_token(claims: &Claims, secret: &str) -> String {
        use jsonwebtoken::{encode, EncodingKey, Header};
        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    #[actix_rt::test]
    async fn test_public_path_allowed() {
        let app = test::init_service(
            App::new()
                .wrap(NgacPepMiddleware::new(test_secret()))
                .route("/health", web::get().to(echo_handler)),
        )
        .await;
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_missing_token_rejected() {
        let app = test::init_service(
            App::new()
                .wrap(NgacPepMiddleware::new(test_secret()))
                .route("/products", web::post().to(echo_handler)),
        )
        .await;
        let req = test::TestRequest::post().uri("/products").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
    }

    #[actix_rt::test]
    async fn test_valid_token_allowed() {
        let app = test::init_service(
            App::new()
                .wrap(NgacPepMiddleware::new(test_secret()))
                .route("/products", web::post().to(echo_handler)),
        )
        .await;
        let token = make_token(
            &Claims {
                sub: "42".to_string(),
                username: "user".to_string(),
                exp: i64::MAX,
            },
            &test_secret(),
        );
        let req = test::TestRequest::post()
            .uri("/products")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_cookie_token_extracted() {
        let app = test::init_service(
            App::new()
                .wrap(NgacPepMiddleware::new(test_secret()))
                .route("/products", web::post().to(echo_handler)),
        )
        .await;
        let token = make_token(
            &Claims {
                sub: "99".to_string(),
                username: "cookie_user".to_string(),
                exp: i64::MAX,
            },
            &test_secret(),
        );
        let req = test::TestRequest::post()
            .uri("/products")
            .cookie(actix_web::cookie::Cookie::new("access_token", token))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}

// ── Rate Limiting Middleware ────────────────────────────────────────────────

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

#[derive(Debug)]
pub struct TokenBucket {
    tokens: f64,
    capacity: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    pub fn try_consume(&mut self, n: f64) -> bool {
        self.refill();
        if self.tokens >= n {
            self.tokens -= n;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        self.last_refill = now;
    }
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    capacity: f64,
    refill_rate: f64,
}

impl RateLimiter {
    pub fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            capacity,
            refill_rate,
        }
    }

    pub fn try_consume(&self, key: &str, cost: f64) -> bool {
        let mut buckets = self.buckets.lock().unwrap();
        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(self.capacity, self.refill_rate));
        bucket.try_consume(cost)
    }
}

#[derive(Clone)]
pub struct RateLimitMiddleware {
    limiter: RateLimiter,
    key_extractor: fn(&ServiceRequest) -> String,
    cost: f64,
    path_prefixes: Vec<String>,
}

impl RateLimitMiddleware {
    pub fn per_ip(path_prefix: &str, capacity: f64, refill_rate: f64) -> Self {
        Self {
            limiter: RateLimiter::new(capacity, refill_rate),
            key_extractor: |req: &ServiceRequest| {
                req.connection_info()
                    .realip_remote_addr()
                    .unwrap_or("unknown")
                    .to_string()
            },
            cost: 1.0,
            path_prefixes: vec![path_prefix.to_string()],
        }
    }

    pub fn per_ip_any(path_prefixes: &[&str], capacity: f64, refill_rate: f64) -> Self {
        Self {
            limiter: RateLimiter::new(capacity, refill_rate),
            key_extractor: |req: &ServiceRequest| {
                req.connection_info()
                    .realip_remote_addr()
                    .unwrap_or("unknown")
                    .to_string()
            },
            cost: 1.0,
            path_prefixes: path_prefixes.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: actix_web::dev::Service<
            ServiceRequest,
            Response = ServiceResponse<B>,
            Error = actix_web::Error,
        > + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddlewareService {
            service: Rc::new(service),
            limiter: self.limiter.clone(),
            key_extractor: self.key_extractor,
            cost: self.cost,
            path_prefixes: self.path_prefixes.clone(),
        }))
    }
}

pub struct RateLimitMiddlewareService<S> {
    service: Rc<S>,
    limiter: RateLimiter,
    key_extractor: fn(&ServiceRequest) -> String,
    cost: f64,
    path_prefixes: Vec<String>,
}

impl<S, B> actix_web::dev::Service<ServiceRequest> for RateLimitMiddlewareService<S>
where
    S: actix_web::dev::Service<
            ServiceRequest,
            Response = ServiceResponse<B>,
            Error = actix_web::Error,
        > + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let limiter = self.limiter.clone();
        let key_extractor = self.key_extractor;
        let cost = self.cost;
        let path_prefixes = self.path_prefixes.clone();

        Box::pin(async move {
            let path = req.path();
            if path_prefixes.iter().any(|p| path.starts_with(p)) {
                let key = (key_extractor)(&req);
                if !limiter.try_consume(&key, cost) {
                    return Ok(req
                        .into_response(HttpResponse::TooManyRequests().json(serde_json::json!({
                            "error": "RATE_LIMITED",
                            "message": "Too many requests, please try again later"
                        })))
                        .map_into_right_body());
                }
            }
            let res = service.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}
