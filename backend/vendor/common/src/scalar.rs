//! Alioth 标量引用转换服务
//!
//! 提供 `qk_*` 标量引用 ID 与实际值之间的查找/创建/转换能力。
//!
//! ## 设计原则
//!
//! - 数据层存储标量引用 ID（`bigint`），不存储实际值
//! - DTO 可接收实际值（`Decimal`/`String`），由服务层通过 `ScalarService` 转换为标量 ID
//! - 标量实体采用「查找存在则复用，不存在则创建」的 UPSERT 策略
//!
//! ## 标量表映射
//!
//! | 业务含义 | 标量表 | 实际值字段 | 类型 |
//! |---------|--------|-----------|------|
//! | 日期 | `zc_id_scal-date` | `date` | `timestamptz` |
//! | 金额 | `zc_id_scal-amount` | `mark` | `numeric(30,10)` |
//! | 价格 | `zc_id_scal-price` | `mark` | `numeric(30,10)` |
//! | 通用数量 | `zc_id_scal-common` | `mark` | `numeric(30,10)` |
//! | 其他刻度 | `zc_id_scale` | `mark` | `numeric(30,10)` |

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::{AssertSqlSafe, PgPool, Postgres, Transaction};

use crate::error::AliothError;

// ---------------------------------------------------------------------------
// 标量值对象类型 — DTO 层统一使用，由 ScalarService 转换为标量引用 ID
// 所有模块从此处导入，禁止本地重复定义
// ---------------------------------------------------------------------------

/// 标量日期值对象（前端传 "YYYY-MM-DD" 字符串）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScalarDateValue {
    pub value: String,
}

/// 标量通用数值对象（前端传 { value: 123.45 }）。
/// 对应 `zc_id_scal-common`，用于数量、容量等通用数值标量。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScalarCommonValue {
    pub value: Decimal,
}

/// `ScalarQtyValue` 是 `ScalarCommonValue` 的别名，语义等价。
pub type ScalarQtyValue = ScalarCommonValue;

/// 标量价格值对象（前端传 { value: 999.99 }）。
/// 对应 `zc_id_scal-price`。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScalarPriceValue {
    pub value: Decimal,
}

/// 标量金额值对象（前端传 { value: 999.99 }）。
/// 对应 `zc_id_scal-amount`。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScalarAmountValue {
    pub value: Decimal,
}
/// 标量转换服务
///
/// 持有一个数据库连接池，提供标量查找/创建/查询方法。
#[derive(Clone)]
pub struct ScalarService {
    pool: PgPool,
}

impl ScalarService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ------------------------------------------------------------------
    // 查找或创建：日期标量
    // ------------------------------------------------------------------

    /// 根据日期文本查找或创建 `zc_id_scal-date` 记录，返回标量 ID。
    ///
    /// `date_text` 格式应为 `YYYY-MM-DD`，内部解析为 `timestamptz`（当天 00:00:00 UTC）。
    pub async fn find_or_create_date(&self, date_text: &str) -> Result<i64, AliothError> {
        let naive = NaiveDate::parse_from_str(date_text, "%Y-%m-%d").map_err(|e| {
            AliothError::BadRequest(format!("Invalid date format '{}': {}", date_text, e))
        })?;
        let date =
            DateTime::<Utc>::from_naive_utc_and_offset(naive.and_hms_opt(0, 0, 0).unwrap(), Utc);

        // 先尝试查找已存在的记录
        if let Some(id) = self.find_date_id(date).await? {
            return Ok(id);
        }

        // 不存在则创建
        let id: i64 = sqlx::query_scalar(
            r#"INSERT INTO isahl."zc_id_scal-date" (notice, date, created_by_id)
               VALUES ($1, $2, 1)
               RETURNING id"#,
        )
        .bind(date_text)
        .bind(date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AliothError::Database(format!("Failed to create scalar date: {}", e)))?;

        Ok(id)
    }

    /// 根据 `DateTime<Utc>` 查找日期标量 ID。
    async fn find_date_id(&self, date: DateTime<Utc>) -> Result<Option<i64>, AliothError> {
        let id: Option<i64> =
            sqlx::query_scalar(r#"SELECT id FROM isahl."zc_id_scal-date" WHERE date = $1 LIMIT 1"#)
                .bind(date)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AliothError::Database(format!("Failed to find scalar date: {}", e)))?;
        Ok(id)
    }

    /// 通过日期标量 ID 查询实际日期值。
    pub async fn get_date(&self, scale_id: i64) -> Result<Option<DateTime<Utc>>, AliothError> {
        let date: Option<DateTime<Utc>> =
            sqlx::query_scalar(r#"SELECT date FROM isahl."zc_id_scal-date" WHERE id = $1"#)
                .bind(scale_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AliothError::Database(format!("Failed to get scalar date: {}", e)))?;
        Ok(date)
    }

    // ------------------------------------------------------------------
    // 查找或创建：金额标量
    // ------------------------------------------------------------------

    /// 根据金额值查找或创建 `zc_id_scal-amount` 记录，返回标量 ID。
    pub async fn find_or_create_amount(&self, mark: Decimal) -> Result<i64, AliothError> {
        self.find_or_create_scalar_mark(mark, r#"isahl."zc_id_scal-amount""#, "amount")
            .await
    }

    /// 通过金额标量 ID 查询实际金额值。
    pub async fn get_amount(&self, scale_id: i64) -> Result<Option<Decimal>, AliothError> {
        self.get_mark(scale_id, r#"isahl."zc_id_scal-amount""#)
            .await
    }

    // ------------------------------------------------------------------
    // 查找或创建：价格标量
    // ------------------------------------------------------------------

    /// 根据价格值查找或创建 `zc_id_scal-price` 记录，返回标量 ID。
    pub async fn find_or_create_price(&self, mark: Decimal) -> Result<i64, AliothError> {
        self.find_or_create_scalar_mark(mark, r#"isahl."zc_id_scal-price""#, "price")
            .await
    }

    /// 通过价格标量 ID 查询实际价格值。
    pub async fn get_price(&self, scale_id: i64) -> Result<Option<Decimal>, AliothError> {
        self.get_mark(scale_id, r#"isahl."zc_id_scal-price""#).await
    }

    // ------------------------------------------------------------------
    // 查找或创建：通用数量标量
    // ------------------------------------------------------------------

    /// 根据通用数值查找或创建 `zc_id_scal-common` 记录，返回标量 ID。
    pub async fn find_or_create_common(&self, mark: Decimal) -> Result<i64, AliothError> {
        self.find_or_create_scalar_mark(mark, r#"isahl."zc_id_scal-common""#, "common")
            .await
    }

    /// 通过通用标量 ID 查询实际数值。
    pub async fn get_common(&self, scale_id: i64) -> Result<Option<Decimal>, AliothError> {
        self.get_mark(scale_id, r#"isahl."zc_id_scal-common""#)
            .await
    }

    // ------------------------------------------------------------------
    // 通用标量操作（基于 mark）
    // ------------------------------------------------------------------

    /// 在任意标量表中根据 `mark` 查找或创建记录。
    ///
    /// `table` 应为完全限定表名（如 `isahl."zc_id_scal-amount"`）。
    /// `notice_prefix` 用于生成默认 notice（如 `"amount: 100.50"`）。
    pub async fn find_or_create_scalar_mark(
        &self,
        mark: Decimal,
        table: &str,
        notice_prefix: &str,
    ) -> Result<i64, AliothError> {
        // 先查找
        if let Some(id) = self.find_mark_id(mark, table).await? {
            return Ok(id);
        }

        // 不存在则创建（使用动态 SQL，表名已做基本校验）
        let notice = format!("{}: {}", notice_prefix, mark);
        let sql = format!(
            r#"INSERT INTO {} (notice, mark, created_by_id) VALUES ($1, $2, 1) RETURNING id"#,
            table
        );
        let id: i64 = sqlx::query_scalar(AssertSqlSafe(sql.as_str()))
            .bind(&notice)
            .bind(mark)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                AliothError::Database(format!("Failed to create scalar in {}: {}", table, e))
            })?;

        Ok(id)
    }

    /// 在指定标量表中根据 `mark` 查找 ID。
    async fn find_mark_id(&self, mark: Decimal, table: &str) -> Result<Option<i64>, AliothError> {
        let sql = format!(r#"SELECT id FROM {} WHERE mark = $1 LIMIT 1"#, table);
        let id: Option<i64> = sqlx::query_scalar(AssertSqlSafe(sql.as_str()))
            .bind(mark)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                AliothError::Database(format!("Failed to find scalar in {}: {}", table, e))
            })?;
        Ok(id)
    }

    /// 通过标量 ID 在任意标量表中查询 `mark` 值。
    pub async fn get_mark(
        &self,
        scale_id: i64,
        table: &str,
    ) -> Result<Option<Decimal>, AliothError> {
        let sql = format!(r#"SELECT mark FROM {} WHERE id = $1"#, table);
        let mark: Option<Decimal> = sqlx::query_scalar(AssertSqlSafe(sql.as_str()))
            .bind(scale_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                AliothError::Database(format!("Failed to get scalar mark from {}: {}", table, e))
            })?;
        Ok(mark)
    }

    // ------------------------------------------------------------------
    // 事务安全版本
    // ------------------------------------------------------------------

    /// 在事务内查找或创建日期标量。
    pub async fn find_or_create_date_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        date_text: &str,
    ) -> Result<i64, AliothError> {
        let naive = NaiveDate::parse_from_str(date_text, "%Y-%m-%d").map_err(|e| {
            AliothError::BadRequest(format!("Invalid date format '{}': {}", date_text, e))
        })?;
        let date =
            DateTime::<Utc>::from_naive_utc_and_offset(naive.and_hms_opt(0, 0, 0).unwrap(), Utc);

        let id: Option<i64> =
            sqlx::query_scalar(r#"SELECT id FROM isahl."zc_id_scal-date" WHERE date = $1 LIMIT 1"#)
                .bind(date)
                .fetch_optional(&mut **tx)
                .await
                .map_err(|e| AliothError::Database(format!("Failed to find scalar date: {}", e)))?;

        if let Some(id) = id {
            return Ok(id);
        }

        let id: i64 = sqlx::query_scalar(
            r#"INSERT INTO isahl."zc_id_scal-date" (notice, date, created_by_id)
               VALUES ($1, $2, 1) RETURNING id"#,
        )
        .bind(date_text)
        .bind(date)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AliothError::Database(format!("Failed to create scalar date: {}", e)))?;

        Ok(id)
    }

    /// 在事务内查找或创建 mark 标量。
    pub async fn find_or_create_mark_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        mark: Decimal,
        table: &str,
        notice_prefix: &str,
    ) -> Result<i64, AliothError> {
        let find_sql = format!(r#"SELECT id FROM {} WHERE mark = $1 LIMIT 1"#, table);
        let id: Option<i64> = sqlx::query_scalar(AssertSqlSafe(find_sql.as_str()))
            .bind(mark)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| {
                AliothError::Database(format!("Failed to find scalar in {}: {}", table, e))
            })?;

        if let Some(id) = id {
            return Ok(id);
        }

        let notice = format!("{}: {}", notice_prefix, mark);
        let insert_sql = format!(
            r#"INSERT INTO {} (notice, mark, created_by_id) VALUES ($1, $2, 1) RETURNING id"#,
            table
        );
        let id: i64 = sqlx::query_scalar(AssertSqlSafe(insert_sql.as_str()))
            .bind(&notice)
            .bind(mark)
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| {
                AliothError::Database(format!("Failed to create scalar in {}: {}", table, e))
            })?;

        Ok(id)
    }

    /// 在事务内查找或创建金额标量。
    pub async fn find_or_create_amount_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        mark: Decimal,
    ) -> Result<i64, AliothError> {
        self.find_or_create_mark_tx(tx, mark, r#"isahl."zc_id_scal-amount""#, "amount")
            .await
    }

    /// 在事务内查找或创建价格标量。
    pub async fn find_or_create_price_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        mark: Decimal,
    ) -> Result<i64, AliothError> {
        self.find_or_create_mark_tx(tx, mark, r#"isahl."zc_id_scal-price""#, "price")
            .await
    }

    /// 在事务内查找或创建通用数量标量。
    pub async fn find_or_create_common_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        mark: Decimal,
    ) -> Result<i64, AliothError> {
        self.find_or_create_mark_tx(tx, mark, r#"isahl."zc_id_scal-common""#, "common")
            .await
    }
}
