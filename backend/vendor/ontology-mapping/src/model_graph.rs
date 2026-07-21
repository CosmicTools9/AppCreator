//! model_graph.rs — isahl_meta 模型语义图（1:1 移植 model_graph.py）
//!
//! 从 `isahl_meta.meta_collections` + `meta_fields` 实时构建模型语义图：
//! definitions（含 category 分类）/ semantic_edges（继承边）/ model_constraints。
//! 替代 Python 版经 `mise run schema-info` 的子进程链路，改为 sqlx 直连。

use anyhow::Result;
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct CollectionRow {
    pub name: String,
    pub table_name: String,
    pub r#type: String,
    pub schema: String,
    pub data_source: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub inherits: Option<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct FieldRow {
    pub name: String,
    pub fk_collection: String,
    pub data_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EntityDefinition {
    pub entity: String,
    pub table: String,
    pub category: String,
    pub role: String,
    pub derived_by: Option<String>,
    pub inherits: Option<Vec<String>>,
    pub properties: serde_json::Value,
    pub key_fields: Vec<String>,
    pub field_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SemanticEdge {
    pub from_entity: String,
    pub to_entity: String,
    pub edge_type: String,
    pub cardinality: String,
    pub condition: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelConstraint {
    pub id: String,
    pub rule: String,
    pub derived_entities: Vec<String>,
    pub reasoning: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelGraph {
    pub meta: serde_json::Value,
    pub definitions: Vec<EntityDefinition>,
    pub semantic_edges: Vec<SemanticEdge>,
    pub model_constraints: Vec<ModelConstraint>,
    pub _entity_index: HashMap<String, usize>,
    pub _fields_index: HashMap<String, Vec<FieldRow>>,
}

/// 分类实体（与 Python `_classify_category` 一致的判定顺序）
fn classify_category(table: &str, fields: &[FieldRow]) -> &'static str {
    let t = table.to_lowercase();

    if ["scal-", "unit", "rate", "amount", "qty", "price", "date"]
        .iter()
        .any(|k| t.contains(k))
    {
        return "scalar";
    }
    if t.contains("status") || t.contains("stus-") {
        return "model_base";
    }
    if t.contains("version") || t.contains("snapshot") {
        return "model_base";
    }
    if t == "zc_id_lifecycle" {
        return "model_base";
    }
    if matches!(
        t.as_str(),
        "zc_id_scene" | "zc_id_factor" | "zc_id_function"
    ) {
        return "dimension";
    }
    if matches!(t.as_str(), "zc_id_category" | "zc_id_tags" | "zc_id_level") {
        return "dimension";
    }
    if t.starts_with("zc_id_prot-")
        || t.starts_with("zc_id_prod-")
        || t.starts_with("zc_id_empl-")
        || t.starts_with("zc_id_subj-")
    {
        return "extension";
    }
    if t.contains("lifecycle") || fields.iter().any(|f| f.name == "fk_lifecycle") {
        return "model_base";
    }
    "extension"
}

fn parse_inherits(raw: &Option<String>) -> Option<Vec<String>> {
    let raw = raw.as_ref()?.trim();
    if raw.is_empty() {
        return None;
    }
    if let Ok(v) = serde_json::from_str::<Vec<String>>(raw) {
        return Some(v);
    }
    Some(raw.split(',').map(|s| s.trim().to_string()).collect())
}

/// 加载模型语义图（替代 Python `load_graph`）。
pub async fn load_graph(pool: &PgPool) -> Result<ModelGraph> {
    let collections: Vec<CollectionRow> = sqlx::query_as(
        r#"SELECT mc.name, mc.table_name, mc.type::text AS "type", mc.schema,
                  mc.data_source,
                  mc.config->>'des' AS description,
                  mc.config->>'category' AS category,
                  mc.config->>'inherits' AS inherits
           FROM isahl_meta.meta_collections mc
           WHERE mc.type::text IN ('table', 'view')
           ORDER BY mc.table_name"#,
    )
    .fetch_all(pool)
    .await?;

    anyhow::ensure!(
        !collections.is_empty(),
        "isahl_meta.meta_collections returned 0 rows. Run Meta model publish first."
    );

    let all_fields: Vec<FieldRow> = sqlx::query_as(
        r#"SELECT mf.name, mf.fk_collection, mf.data_type::text AS data_type,
                  mf.title AS description
           FROM isahl_meta.meta_fields mf
           ORDER BY mf.fk_collection, mf.name"#,
    )
    .fetch_all(pool)
    .await?;

    let mut fields_by_collection: HashMap<String, Vec<FieldRow>> = HashMap::new();
    for f in all_fields.iter() {
        fields_by_collection
            .entry(f.fk_collection.clone())
            .or_default()
            .push(f.clone());
    }

    let mut definitions = Vec::new();
    for col in &collections {
        let fields = fields_by_collection
            .get(&col.name)
            .cloned()
            .unwrap_or_default();
        let field_names: Vec<String> = fields.iter().map(|f| f.name.clone()).collect();
        definitions.push(EntityDefinition {
            entity: col.name.clone(),
            table: format!("{}.{}", col.schema, col.table_name),
            category: classify_category(&col.table_name, &fields).to_string(),
            role: col.description.clone().unwrap_or_default(),
            derived_by: None,
            inherits: parse_inherits(&col.inherits),
            properties: serde_json::json!({
                "type": col.r#type,
                "data_source": col.data_source,
            }),
            key_fields: field_names.iter().take(10).cloned().collect(),
            field_count: field_names.len(),
        });
    }

    let mut semantic_edges = Vec::new();
    for col in &collections {
        if let Some(parents) = parse_inherits(&col.inherits) {
            for parent in parents {
                let parent = parent.trim().to_string();
                semantic_edges.push(SemanticEdge {
                    condition: format!("{} inherits from {}", col.name, parent),
                    description: format!("继承关系：{} → {}", col.name, parent),
                    from_entity: parent,
                    to_entity: col.name.clone(),
                    edge_type: "inherits".into(),
                    cardinality: "一般化".into(),
                });
            }
        }
    }

    let by_cat = |cat: &str| -> Vec<String> {
        definitions
            .iter()
            .filter(|d| d.category == cat)
            .map(|d| d.entity.clone())
            .collect()
    };
    let model_constraints = vec![
        ModelConstraint {
            id: "lifecycle-for-all".into(),
            rule: "所有业务实体必须继承 isahl.zc_id_lifecycle".into(),
            derived_entities: by_cat("model_base"),
            reasoning: "4D 元数中的 Status 维要求实体有生命周期".into(),
            source: "ALIOTH_THEORY.md §2.1".into(),
        },
        ModelConstraint {
            id: "scalar-independence".into(),
            rule: "可度量值必须独立为标量表，不嵌入实体表".into(),
            derived_entities: by_cat("scalar"),
            reasoning: "度量值变化不应改变实体生命周期；通过 qk_* 标量引用".into(),
            source: "ALIOTH_ONTOLOGY_SPEC.md §5".into(),
        },
        ModelConstraint {
            id: "status-separation".into(),
            rule: "状态必须为独立的关系表引用，不嵌入实体字段".into(),
            derived_entities: definitions
                .iter()
                .filter(|d| d.entity.to_lowercase().contains("status"))
                .map(|d| d.entity.clone())
                .collect(),
            reasoning: "状态有独立字段；状态变迁需审计；状态可跨实体复用".into(),
            source: "ALIOTH_ONTOLOGY_SPEC.md §4.2".into(),
        },
        ModelConstraint {
            id: "version-audit".into(),
            rule: "实体生命周期变更必须可追溯".into(),
            derived_entities: definitions
                .iter()
                .filter(|d| {
                    let e = d.entity.to_lowercase();
                    e.contains("version") || e.contains("snapshot")
                })
                .map(|d| d.entity.clone())
                .collect(),
            reasoning: "模型可追溯性要求：每次实体变更自动创建版本记录".into(),
            source: "ALIOTH_THEORY.md §2.1".into(),
        },
    ];

    let now: (String,) = sqlx::query_as("SELECT now()::text").fetch_one(pool).await?;

    Ok(ModelGraph {
        meta: serde_json::json!({
            "title": "Alioth 模型语义图（isahl_meta 实时生成）",
            "model_version": "v10.0.0",
            "source": "isahl_meta.meta_collections + meta_fields",
            "generated_at": now.0,
            "collection_count": definitions.len(),
            "field_count": all_fields.len(),
        }),
        _entity_index: definitions
            .iter()
            .enumerate()
            .map(|(i, d)| (d.entity.clone(), i))
            .collect(),
        definitions,
        semantic_edges,
        model_constraints,
        _fields_index: fields_by_collection,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fld(name: &str) -> FieldRow {
        FieldRow {
            name: name.into(),
            fk_collection: "c".into(),
            data_type: "text".into(),
            description: None,
        }
    }

    #[test]
    fn test_classify_scalar() {
        assert_eq!(classify_category("zc_id_scal-price", &[]), "scalar");
        assert_eq!(classify_category("zc_id_unit", &[]), "scalar");
    }

    #[test]
    fn test_classify_dimension() {
        assert_eq!(classify_category("zc_id_scene", &[]), "dimension");
        assert_eq!(classify_category("zc_id_tags", &[]), "dimension");
    }

    #[test]
    fn test_classify_model_base() {
        assert_eq!(classify_category("zc_id_lifecycle", &[]), "model_base");
        assert_eq!(classify_category("zc_id_status", &[]), "model_base");
        assert_eq!(classify_category("zc_id_version", &[]), "model_base");
        assert_eq!(
            classify_category("zc_id_custom", &[fld("fk_lifecycle")]),
            "model_base"
        );
    }

    #[test]
    fn test_classify_extension() {
        assert_eq!(classify_category("zc_id_prot-env_config", &[]), "extension");
        assert_eq!(classify_category("zc_id_orde-land", &[]), "extension");
    }

    #[test]
    fn test_parse_inherits_json_and_csv() {
        assert_eq!(
            parse_inherits(&Some("[\"zc_id_lifecycle\"]".into())),
            Some(vec!["zc_id_lifecycle".to_string()])
        );
        assert_eq!(
            parse_inherits(&Some("zc_id_lifecycle, zc_ad_object".into())),
            Some(vec![
                "zc_id_lifecycle".to_string(),
                "zc_ad_object".to_string()
            ])
        );
        assert_eq!(parse_inherits(&None), None);
        assert_eq!(parse_inherits(&Some("".into())), None);
    }
}
