//! 维度/刻度表注册表（Dimension Table Registry）
//!
//! 提供元数据驱动的维度表查询基础设施。
//! 因子通过声明 `DimTable` 静态常量 + `DimRegistry::for_name()` 即可获得
//! 通用列表/详情 handler，无需为每张结构一致的表重复编写 handler + SQL。
//!
//! ## 使用场景
//!
//! 适用于结构简单（id + notice + code + 若干可选字段）的维度/刻度表，
//! 如：zc_id_scene、zc_id_unit、zc_id_status 等标量表。
//! 对于有复杂关联查询、标量引用或业务逻辑的实体，请使用 `crud` 框架的
//! `AliothRepository` + `crud_routes`。

use serde_json::json;
use sqlx::{AssertSqlSafe, PgPool, Row};

// ──────────── DimTable ─── 维度表元数据 ───────────────────────────────────────

/// 单张维度/刻度表的元数据
pub struct DimTable {
    /// SELECT 子句列名（不含 SELECT 关键字，用于传给 sqlx::query 后逐列提取）
    pub select_cols: &'static [&'static str],
    /// FROM 子句（含表名、别名、可选的 JOIN）
    pub from_clause: &'static str,
    /// ILIKE 搜索的目标列（带表别名前缀，如 `"notice"` 或 `"s.notice"`）
    pub search_cols: &'static [&'static str],
    /// 主键列（带别名前缀，如 `"id"` 或 `"s.id"`）
    pub pk_col: &'static str,
}

// ──────────── SQL 辅助函数 ─────────────────────────────────────────────────────

/// 构建 ILIKE 搜索条件的 OR 子句
pub fn build_search_conditions(cfg: &DimTable) -> String {
    cfg.search_cols
        .iter()
        .map(|c| format!("{}::text ILIKE $1", c))
        .collect::<Vec<_>>()
        .join(" OR ")
}

/// 将 `PgRow` 按 `DimTable` 的 `select_cols` 转换为 JSON
pub fn row_to_json(row: &sqlx::postgres::PgRow, cfg: &DimTable) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (i, col_name) in cfg.select_cols.iter().enumerate() {
        let name = col_name.split('.').next_back().unwrap_or(col_name);
        let val: serde_json::Value = match row.try_get::<Option<serde_json::Value>, _>(i) {
            Ok(Some(serde_json::Value::Number(n))) => {
                // Serialize id / qk_* / fk_* / ref_* columns as strings to avoid JS precision loss
                if name == "id" || name.starts_with("qk_") || name.starts_with("fk_") || name.starts_with("ref_") {
                    if let Some(n) = n.as_i64() {
                        json!(n.to_string())
                    } else {
                        json!(n)
                    }
                } else {
                    json!(n)
                }
            }
            Ok(Some(v)) => v,
            _ => {
                match row.try_get::<Option<String>, _>(i) {
                    Ok(Some(s)) => json!(s),
                    _ => serde_json::Value::Null,
                }
            }
        };
        map.insert(name.to_string(), val);
    }
    serde_json::Value::Object(map)
}

/// 批量转换
pub fn rows_to_json(rows: Vec<sqlx::postgres::PgRow>, cfg: &DimTable) -> Vec<serde_json::Value> {
    rows.iter().map(|r| row_to_json(r, cfg)).collect()
}

/// 通用维度 COUNT（支持关键词搜索）
pub async fn count_dimension(
    pool: &PgPool,
    cfg: &DimTable,
    keyword: &str,
) -> Result<i64, sqlx::Error> {
    if keyword.is_empty() {
        let sql = format!(
            "SELECT COUNT(*) FROM {} WHERE deleted_at IS NULL",
            cfg.from_clause
        );
        sqlx::query_scalar(AssertSqlSafe(sql.as_str()))
            .fetch_one(pool)
            .await
    } else {
        let conditions = build_search_conditions(cfg);
        let sql = format!(
            "SELECT COUNT(*) FROM {} WHERE deleted_at IS NULL AND ({})",
            cfg.from_clause, conditions,
        );
        sqlx::query_scalar(AssertSqlSafe(sql.as_str()))
            .bind(format!("%{}%", keyword))
            .fetch_one(pool)
            .await
    }
}

/// 通用维度分页列表查询
pub async fn list_dimension_rows(
    pool: &PgPool,
    cfg: &DimTable,
    keyword: &str,
    page_size: i64,
    offset: i64,
) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
    if keyword.is_empty() {
        let select_sql = cfg.select_cols.join(", ");
        let sql = format!(
            "SELECT {} FROM {} WHERE deleted_at IS NULL ORDER BY {} \
             LIMIT {} OFFSET {}",
            select_sql, cfg.from_clause, cfg.pk_col, page_size, offset,
        );
        sqlx::query(AssertSqlSafe(sql.as_str()))
            .fetch_all(pool)
            .await
    } else {
        let select_sql = cfg.select_cols.join(", ");
        let conditions = build_search_conditions(cfg);
        let sql = format!(
            "SELECT {} FROM {} WHERE deleted_at IS NULL AND ({}) \
             ORDER BY {} LIMIT {} OFFSET {}",
            select_sql, cfg.from_clause, conditions, cfg.pk_col, page_size, offset,
        );
        sqlx::query(AssertSqlSafe(sql.as_str()))
            .bind(format!("%{}%", keyword))
            .fetch_all(pool)
            .await
    }
}
