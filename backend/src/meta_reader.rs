//! AppCreator Meta reader — reads `isahl_meta` schema directly via sqlx.
//!
//! Shares the same database as Meta. No HTTP API calls to Meta service.
//! The AppCreator DB user must have SELECT on `isahl_meta` schema.

use serde::Serialize;
use sqlx::PgPool;

/// A collection (entity/app definition) from Meta.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MetaCollection {
    pub table_name: String,
    pub name: String,
    #[allow(dead_code)]
    pub r#type: Option<String>,
    pub biz_description: Option<String>,
}

/// A field definition from Meta.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MetaField {
    pub name: String,
    pub title: Option<String>,
    pub data_type: Option<String>,
    pub is_required: bool,
}

impl MetaField {
    pub fn data_type_display(&self) -> &str {
        match self.data_type.as_deref() {
            Some("varchar") | Some("text") => "文本",
            Some("int4") | Some("int8") | Some("bigint") => "数字",
            Some("bool") => "开关",
            Some("date") | Some("timestamp") | Some("timestamptz") => "日期",
            Some("numeric") | Some("float8") | Some("decimal") => "金额",
            _ => "文本",
        }
    }
}


/// Load fields for a specific collection.
pub async fn load_fields(pool: &PgPool, collection: &str) -> Result<Vec<MetaField>, sqlx::Error> {
    let rows = sqlx::query_as::<_, MetaField>(
        r#"
        SELECT name, title, data_type::text as "data_type", is_required
        FROM isahl_meta.meta_fields
        WHERE fk_collection = $1
        ORDER BY created_at
        "#
    )
    .bind(collection)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Generate a prototype description from a collection's metadata.
pub async fn describe_collection(pool: &PgPool, table_name: &str) -> Result<String, sqlx::Error> {
    let col = sqlx::query_as::<_, MetaCollection>(
        "SELECT table_name, name, type::text as \"type\", biz_description FROM isahl_meta.meta_collections WHERE table_name = $1"
    )
    .bind(table_name)
    .fetch_optional(pool)
    .await?;

    let col = match col {
        Some(c) => c,
        None => return Ok(format!("未找到表 `{}` 的元数据", table_name)),
    };

    let fields = load_fields(pool, table_name).await?;

    let mut desc = format!(
        "## {}\n\n{}\n\n**字段列表：**\n\n",
        col.name,
        col.biz_description.as_deref().unwrap_or("")
    );

    for f in &fields {
        let required = if f.is_required { "（必填）" } else { "" };
        desc.push_str(&format!(
            "- **{}**：{} {} {}\n",
            f.name,
            f.title.as_deref().unwrap_or(""),
            f.data_type_display(),
            required,
        ));
    }

    Ok(desc)
}
