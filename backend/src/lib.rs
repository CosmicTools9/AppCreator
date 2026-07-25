//! AppCreator shared library
//!
//! 独立服务，与 Meta 仅共享 DB。
//! API 无交互，认证通过 SSO JWT (ES256)。

pub mod app_repository;
pub mod auth_config;
pub mod chat;
pub mod handlers;
pub mod middleware;
