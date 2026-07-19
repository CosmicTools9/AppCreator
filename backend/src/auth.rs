//! SSO JWT 认证中间件
//!
//! 验证 SSO 签发的 RS256 JWT token。
//! 公钥从环境变量 `SSO_JWT_PUBLIC_KEY` 读取。

use actix_web::{HttpMessage, HttpRequest};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

/// SSO JWT 声明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoClaims {
    pub sub: String,
    pub username: String,
    pub email: String,
    #[serde(default)]
    pub is_superuser: bool,
    pub iat: i64,
    pub exp: i64,
    pub iss: Option<String>,
}

/// 认证用户上下文
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: i64,
    pub username: String,
    pub email: String,
    pub is_superuser: bool,
}

/// 加载 SSO RSA 公钥
pub fn load_decoding_key() -> Result<DecodingKey, String> {
    let pem = std::env::var("SSO_JWT_PUBLIC_KEY")
        .map_err(|_| "SSO_JWT_PUBLIC_KEY not set".to_string())?;
    DecodingKey::from_rsa_pem(pem.as_bytes())
        .map_err(|e| format!("Invalid RSA public key PEM: {}", e))
}

/// 验证 Bearer token，返回用户信息
pub fn verify_token(token: &str, key: &DecodingKey) -> Result<SsoClaims, String> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&["app-creator", "alioth-sso"]);
    let token_data = decode::<SsoClaims>(token, key, &validation)
        .map_err(|e| format!("JWT verification failed: {}", e))?;
    Ok(token_data.claims)
}

/// 从请求中验证 JWT 并注入用户上下文
/// 公开路径跳过认证：/health, /api/creator/status
pub fn authenticate(req: &HttpRequest, key: &DecodingKey) -> Result<AuthUser, String> {
    let path = req.path();
    if path == "/health" || path == "/api/creator/status" {
        return Ok(AuthUser {
            user_id: 0,
            username: "system".to_string(),
            email: String::new(),
            is_superuser: true,
        });
    }

    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| "Missing or invalid Authorization header".to_string())?;

    let claims = verify_token(token, key)?;

    let user = AuthUser {
        user_id: claims.sub.parse().unwrap_or(0),
        username: claims.username,
        email: claims.email,
        is_superuser: claims.is_superuser,
    };
    req.extensions_mut().insert(user.clone());
    Ok(user)
}
