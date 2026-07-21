//! 测试基础设施：共享的测试数据库连接与清理函数。
//!
//! WZ、Alioth 等 namespace 的集成测试通过 `helpers::setup()` 模式使用此模块，
//! 而非在每个 service 的 `tests/` 下复制一份连接逻辑。
//! 这符合项目「一次定义，到处使用」的框架规约。
#![doc(hidden)]

use sqlx::PgPool;

/// 连接测试数据库。
///
/// 优先读取 `DATABASE_URL` 环境变量，fallback 到 `postgres://localhost:5432/aliothstudio_test`。
pub async fn connect_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost:5432/aliothstudio_test".to_string());
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// 清理测试数据。
///
/// 当前实现为空操作。测试应通过独立的种子/回滚策略管理各自的数据生命周期。
/// 此函数作为扩展点预留，后续可接入 schema-level 清理逻辑。
pub async fn cleanup_test_db(_pool: &PgPool) {
    // no-op: 测试自己管理数据的创建与清理
}

/// 开始测试事务。事务结束时自动回滚，不影响其他测试。
pub async fn begin_test_tx(pool: &PgPool) -> sqlx::Transaction<'static, sqlx::Postgres> {
    pool.begin().await.expect("Failed to begin test transaction")
}

/// 测试 schema 守门：断言当前连接是测试库。
///
/// 仅检查库名含 `_test`，不执行 TRUNCATE。
/// Alioth namespace 可通过自己的测试 helper 实现 TRUNCATE。
pub async fn setup_test_schema_light(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let db_name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(pool)
        .await?;
    if !db_name.contains("_test") {
        return Err(format!(
            "test db required (got '{}'); set DATABASE_URL=postgres://localhost:5432/aliothstudio_test",
            db_name
        ).into());
    }
    Ok(())
}
