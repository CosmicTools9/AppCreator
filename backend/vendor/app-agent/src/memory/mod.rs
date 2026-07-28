//! Memory — 跨会话持久化语义记忆
//!
//! 提供语义检索接口，让 AppAgent 在不同 session 之间复用知识。
//! 当前只有 trait 定义 + 基于 PostgreSQL JSONB 的简单实现，
//! 未来可接入 pgvector 进行向量检索。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// 记忆条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// 唯一标识
    pub id: String,
    /// namespace 隔离域
    pub namespace: String,
    /// 记忆类型（如 "entity_schema", "api_decision", "error_fix"）
    pub kind: String,
    /// 检索关键词
    pub keywords: Vec<String>,
    /// 记忆内容（结构化 JSON）
    pub content: serde_json::Value,
    /// 摘要文本（用于语义匹配）
    pub summary: String,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 记忆存储 trait
#[async_trait]
pub trait MemoryStore: Send + Sync {
    /// 存储一条记忆
    async fn store(&self, entry: MemoryEntry) -> Result<(), String>;

    /// 按关键词搜索记忆
    async fn search(
        &self,
        namespace: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, String>;

    /// 按类型搜索记忆
    async fn search_by_kind(
        &self,
        namespace: &str,
        kind: &str,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, String>;

    /// 删除过期记忆
    async fn prune(&self, before: chrono::DateTime<chrono::Utc>) -> Result<usize, String>;
}

/// 基于 PostgreSQL JSONB 的记忆存储（简易实现）
pub struct PgMemoryStore {
    pool: PgPool,
}

impl PgMemoryStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 确保表存在
    pub async fn ensure_table(&self) -> Result<(), String> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS isahl_meta.agent_memory (
                id TEXT PRIMARY KEY,
                namespace TEXT NOT NULL DEFAULT '',
                kind TEXT NOT NULL DEFAULT 'general',
                keywords TEXT[] NOT NULL DEFAULT '{}',
                content JSONB NOT NULL DEFAULT '{}',
                summary TEXT NOT NULL DEFAULT '',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // 索引
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_agent_memory_namespace_kind
            ON isahl_meta.agent_memory(namespace, kind)
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_agent_memory_keywords
            ON isahl_meta.agent_memory USING GIN(keywords)
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }
}

#[async_trait]
impl MemoryStore for PgMemoryStore {
    async fn store(&self, entry: MemoryEntry) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO isahl_meta.agent_memory (id, namespace, kind, keywords, content, summary, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                keywords = EXCLUDED.keywords,
                content = EXCLUDED.content,
                summary = EXCLUDED.summary
            "#,
        )
        .bind(&entry.id)
        .bind(&entry.namespace)
        .bind(&entry.kind)
        .bind(&entry.keywords)
        .bind(&entry.content)
        .bind(&entry.summary)
        .bind(entry.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn search(
        &self,
        namespace: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, String> {
        let pattern = format!("%{}%", query);
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                Vec<String>,
                serde_json::Value,
                String,
                chrono::DateTime<chrono::Utc>,
            ),
        >(
            r#"
            SELECT id, namespace, kind, keywords, content, summary, created_at
            FROM isahl_meta.agent_memory
            WHERE namespace = $1
              AND (summary ILIKE $2 OR content::text ILIKE $2 OR $3 = ANY(keywords))
            ORDER BY created_at DESC
            LIMIT $4
            "#,
        )
        .bind(namespace)
        .bind(&pattern)
        .bind(query)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(rows
            .into_iter()
            .map(
                |(id, namespace, kind, keywords, content, summary, created_at)| MemoryEntry {
                    id,
                    namespace,
                    kind,
                    keywords,
                    content,
                    summary,
                    created_at,
                },
            )
            .collect())
    }

    async fn search_by_kind(
        &self,
        namespace: &str,
        kind: &str,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, String> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                Vec<String>,
                serde_json::Value,
                String,
                chrono::DateTime<chrono::Utc>,
            ),
        >(
            r#"
            SELECT id, namespace, kind, keywords, content, summary, created_at
            FROM isahl_meta.agent_memory
            WHERE namespace = $1 AND kind = $2
            ORDER BY created_at DESC
            LIMIT $3
            "#,
        )
        .bind(namespace)
        .bind(kind)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(rows
            .into_iter()
            .map(
                |(id, namespace, kind, keywords, content, summary, created_at)| MemoryEntry {
                    id,
                    namespace,
                    kind,
                    keywords,
                    content,
                    summary,
                    created_at,
                },
            )
            .collect())
    }

    async fn prune(&self, before: chrono::DateTime<chrono::Utc>) -> Result<usize, String> {
        let result = sqlx::query("DELETE FROM isahl_meta.agent_memory WHERE created_at < $1")
            .bind(before)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(result.rows_affected() as usize)
    }
}
