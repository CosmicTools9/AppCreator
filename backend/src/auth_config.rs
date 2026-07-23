//! AuthConfig — SSO/Standalone 双模式认证配置（共享受限初始化的全局状态）
//!
//! - `init_auth_config()` 在 `main()` 中调用一次
//! - `auth_config()` 返回 `&'static AuthConfig`，供 middleware 与 handlers 共享

use std::sync::OnceLock;

use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum AuthMode {
    /// ES256 token from SSO (SSO_JWT_PUBLIC_KEY 配置)
    Sso,
    /// ES256 token 本地签发（SSO_JWT_PUBLIC_KEY 未配置，使用 APP_CREATOR_JWT_PRIVATE_KEY / 嵌入式 dev key）
    Standalone,
}

#[derive(Debug)]
pub struct AuthConfig {
    pub mode: AuthMode,
    /// 验证用公钥（SSO 模式为 SSO_JWT_PUBLIC_KEY；Standalone 模式从 APP_CREATOR_JWT_PRIVATE_KEY 或嵌入式 dev key 派生）
    pub decoding_key: DecodingKey,
    /// 签名用私钥（仅 Standalone 模式需要；SSO 模式为 None）
    pub encoding_key: Option<EncodingKey>,
}

/// Standalone JWT claims（与 AppClaims 同形，namespace 供 middleware 注入 RequestContext）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandaloneClaims {
    pub sub: String,
    pub email: String,
    pub exp: i64,
    pub iat: i64,
    pub username: String,
    pub namespace: String,
    #[serde(default)]
    pub iss: Option<String>,
    #[serde(default)]
    pub sid: String,
}

static CONFIG: OnceLock<AuthConfig> = OnceLock::new();

/// 全局 auth 配置初始化（main 启动时调用一次）
pub fn init_auth_config() {
    let sso_pem = std::env::var("SSO_JWT_PUBLIC_KEY")
        .ok()
        .filter(|k| !k.is_empty() && !k.starts_with("enc:"));

    match sso_pem {
        Some(pem) => {
            let decoding_key = DecodingKey::from_ec_pem(pem.as_bytes())
                .expect("SSO_JWT_PUBLIC_KEY is not a valid EC P-256 PEM");
            let _ = CONFIG.set(AuthConfig {
                mode: AuthMode::Sso,
                decoding_key,
                encoding_key: None,
            });
            log::info!("Auth mode: SSO (SSO_JWT_PUBLIC_KEY configured)");
        }
        None => {
            let (decoding_key, encoding_key) = load_or_generate_standalone_keys();
            let _ = CONFIG.set(AuthConfig {
                mode: AuthMode::Standalone,
                decoding_key,
                encoding_key: Some(encoding_key),
            });
            log::warn!("Auth mode: Standalone (no SSO_JWT_PUBLIC_KEY)");
            log::warn!("Standalone mode is for self-hosted/development only — DO NOT expose to public internet");
        }
    }
}

pub fn auth_config() -> &'static AuthConfig {
    CONFIG
        .get()
        .expect("AuthConfig not initialized — call init_auth_config() in main")
}

/// 载入或使用嵌入式开发密钥（生产环境必须配置 APP_CREATOR_JWT_PRIVATE_KEY）
fn load_or_generate_standalone_keys() -> (DecodingKey, EncodingKey) {
    if let Ok(pem) = std::env::var("APP_CREATOR_JWT_PRIVATE_KEY") {
        if !pem.is_empty() && !pem.starts_with("enc:") {
            let encoding_key = EncodingKey::from_ec_pem(pem.as_bytes())
                .expect("APP_CREATOR_JWT_PRIVATE_KEY is not a valid EC P-256 private key PEM");
            let decoding_key = DecodingKey::from_ec_pem(pem.as_bytes())
                .expect("APP_CREATOR_JWT_PRIVATE_KEY cannot derive public key");
            log::info!("Loaded standalone ES256 key from APP_CREATOR_JWT_PRIVATE_KEY");
            return (decoding_key, encoding_key);
        }
    }

    // 无持久密钥时使用嵌入式开发密钥
    // 已知不保密，dev 环境不需要生产级安全——生产必须配 APP_CREATOR_JWT_PRIVATE_KEY
    log::warn!("APP_CREATOR_JWT_PRIVATE_KEY not configured — using embedded development key");
    log::warn!("Set APP_CREATOR_JWT_PRIVATE_KEY for production use.");

    let encoding_key = EncodingKey::from_ec_pem(DEV_PRIVATE_KEY.as_bytes())
        .expect("Embedded DEV_PRIVATE_KEY is invalid");
    let decoding_key = DecodingKey::from_ec_pem(DEV_PRIVATE_KEY.as_bytes())
        .expect("Embedded DEV_PRIVATE_KEY cannot derive public key");
    (decoding_key, encoding_key)
}

// 嵌入式开发 ES256 私钥（PKCS#8 PEM）——永不上线、仅供本地开发
// 生产环境必须配置 APP_CREATOR_JWT_PRIVATE_KEY
// 生成：openssl genpkey -algorithm EC -pkeyopt ec_paramgen_curve:prime256v1
const DEV_PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgvTkNZwK8WqNH/aEn
rUkSD5+lYAesakhvTFcWpKteHbOhRANCAASmyJF5MqiJ0MkA77TZJkGAdqiqhv26
IVcpjkHR5sxTZhZ5eH/SSSV/ddphVgahp0cRM9H4HSgzNMIkDNv5dJuN
-----END PRIVATE KEY-----";
