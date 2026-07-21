//! Shared types for Meta Backend
//!
//! 统一 Meta 后端 API 响应格式与工具函数。
//! 响应类型已由 alioth-common 提供，本 crate 仅重新导出并补充 Meta 专用逻辑。

pub mod serde_zuid;

pub use common::{AliothError, ApiResponse, ErrorResponse, JsonResponse, ListQuery};

use actix_web::{HttpMessage, HttpRequest};

/// 从请求中提取用户ID（用于审计字段）
///
/// 优先从 common::context::RequestContext 读取，回退到裸 i64
pub fn extract_user_id(req: &HttpRequest) -> Option<i64> {
    common::context::extract_user_id(req).or_else(|| req.extensions().get::<i64>().copied())
}
