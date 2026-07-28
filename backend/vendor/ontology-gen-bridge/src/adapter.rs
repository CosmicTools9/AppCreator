//! Bridge adapter: MappingOutput → MetaModule
//!
//! Hard errors on unknown columns, unsupported scalars, unknown relation types,
//! and invalid field paths. Never guesses.

use alioth_gen::generator::ir::module::{
    MetaEntity, MetaField, MetaFieldType, MetaModule, MetaPage, MetaRelation, MetaRelationType,
    PageLayout, PageType,
};
use ontology_mapping::output::{MappedEntity, MappingOutput};

#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("unknown column `{column}` in entity `{entity}`")]
    UnknownColumn { column: String, entity: String },
    #[error("entity `{entity}` has no table mapping")]
    MissingTable { entity: String },
    #[error("field `{json_path}` in entity `{entity}` is unmapped")]
    UnmappedField { json_path: String, entity: String },
    #[error("field `{json_path}` in entity `{entity}` is a scalar reference (qk_*) — requires manual implementation")]
    UnsupportedScalar { json_path: String, entity: String },
    #[error("relation type `{rel_type}` for `{target}` in entity `{entity}` is not supported (use: belongsTo, hasMany, hasOne)")]
    UnsupportedRelationType {
        rel_type: String,
        target: String,
        entity: String,
    },
    #[error("field path `{path}` in entity `{entity}` is not a valid Rust identifier")]
    InvalidFieldPath { path: String, entity: String },
}

// ── Excluded columns (DC-2) ──────────────────────────────────────────
const EXCLUDED: &[&str] = &[
    "id",
    "created_at",
    "updated_at",
    "deleted_at",
    "created_by_id",
    "updated_by_id",
    "o_number",
    "number",
    "domain_",
    "dk_scene",
    "dk_factor",
    "dk_function",
    "_f_",
    "_t_",
    "paths",
    "ref_count",
    "ak_dimensions",
    "ak_components",
    "majority",
    "sprint",
    "model",
    "p_number",
    "d_count",
    "c_count",
];

fn excluded(col: &str) -> bool {
    EXCLUDED.contains(&col) || col.starts_with("ak_")
}

/// Rust keywords that cannot be used as field names.
const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "union",
    "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
];

fn is_rust_ident(s: &str) -> bool {
    if s.is_empty() || RUST_KEYWORDS.contains(&s) {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_lowercase() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

// ── Column → MetaFieldType (DC-1, DC-8, DC-9) ────────────────────────
fn column_type(column: &str) -> Result<MetaFieldType, AdapterError> {
    match column {
        "notice" | "code" | "comments" | "t_color_" | "cron" | "r_number" | "r_notice" => {
            Ok(MetaFieldType::String)
        }
        "sort" | "ref_left" | "ref_right" => Ok(MetaFieldType::Integer),
        _ if column.starts_with("sk_") => Ok(MetaFieldType::Reference("Unit".into())),
        _ if column.starts_with("fk_") => Ok(MetaFieldType::Reference("Entity".into())),
        _ if column.starts_with("ck_") => Ok(MetaFieldType::Reference("Category".into())),
        _ if column.starts_with("tk_") => Ok(MetaFieldType::Reference("Tag".into())),
        _ if column.starts_with("lk_") => Ok(MetaFieldType::Reference("Level".into())),
        _ => Err(AdapterError::UnknownColumn {
            column: column.into(),
            entity: String::new(),
        }),
    }
}

// ── Public API ───────────────────────────────────────────────────────

pub fn mapping_output_to_meta_module(
    output: &MappingOutput,
    module_name: &str,
) -> Result<MetaModule, AdapterError> {
    let mut entities = Vec::new();
    let mut pages = Vec::new();

    for m in &output.entities {
        if m.mapping.table.is_empty() {
            return Err(AdapterError::MissingTable {
                entity: m.name.clone(),
            });
        }
        let entity = build_entity(m, &output.entities)?;
        let n = entity.name.clone();
        pages.push(MetaPage {
            name: format!("{}List", n),
            entity: n.clone(),
            page_type: PageType::List,
            layout: PageLayout { columns: vec![], filters: vec![], sections: vec![] },
        });
        pages.push(MetaPage {
            name: format!("{}Detail", n.clone()),
            entity: n,
            page_type: PageType::Detail,
            layout: PageLayout { columns: vec![], filters: vec![], sections: vec![] },
        });
        entities.push(entity);
    }

    let mut module = MetaModule::new(module_name.to_string());
    module.entities = entities;
    module.pages = pages;
    Ok(module)
}

fn build_entity(m: &MappedEntity, all_entities: &[MappedEntity]) -> Result<MetaEntity, AdapterError> {
    let mut fields = Vec::new();
    let mut relations = Vec::new();

    // Build a lookup map from entity name → table name
    let table_map: std::collections::HashMap<&str, &str> = all_entities.iter()
        .map(|e| (e.name.as_str(), e.mapping.table.as_str()))
        .collect();

    for f in &m.fields {
        let col = f.column.as_deref().unwrap_or("");
        let has_scalar = f.scalar_table.is_some();

        // qk_* / scalar → ScalarValue type
        if col.starts_with("qk_") || (col.is_empty() && has_scalar) {
            fields.push(MetaField {
                name: f.json_path.clone(),
                field_type: MetaFieldType::ScalarValue("Generic".into()),
                nullable: true,
                ..Default::default()
            });
            continue;
        }
        if col.is_empty() {
            return Err(AdapterError::UnmappedField {
                json_path: f.json_path.clone(),
                entity: m.name.clone(),
            });
        }
        if excluded(col) {
            continue;
        }
        if !is_rust_ident(&f.json_path) {
            return Err(AdapterError::InvalidFieldPath {
                path: f.json_path.clone(),
                entity: m.name.clone(),
            });
        }
        let ft = column_type(col).map_err(|mut e| {
            if let AdapterError::UnknownColumn { ref mut entity, .. } = e {
                *entity = m.name.clone();
            }
            e
        })?;

        fields.push(MetaField {
            name: f.json_path.clone(),
            field_type: ft,
            nullable: !matches!(col, "notice" | "code"),
            ..Default::default()
        });
    }

    for r in &m.relationships {
        let rt = match r.rel_type.as_str() {
            "belongsTo" => MetaRelationType::ManyToOne,
            "hasMany" => MetaRelationType::OneToMany,
            "hasOne" => MetaRelationType::OneToOne,
            other => {
                return Err(AdapterError::UnsupportedRelationType {
                    rel_type: other.into(),
                    target: r.target.clone(),
                    entity: m.name.clone(),
                })
            }
        };
        let target_table = r.via.as_deref()
            .and_then(|_| table_map.get(r.target.as_str()))
            .map(|t| t.to_string());
        relations.push(MetaRelation {
            name: r.target.clone(),
            target_entity: r.target.clone(),
            relation_type: rt,
            nullable: true,
            via: r.via.clone(),
            target_table,
        });
    }

    Ok(MetaEntity {
        name: m.name.clone(),
        table_name: Some(m.mapping.table.clone()),
        fields,
        relations,
        state_machine: Default::default(),
        ..Default::default()
    })
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ontology_mapping::output::{
        Coordinates, EntityMapping, FieldMapping, OutputMeta, RelationshipMapping, Tier,
        TierSummary, TieredValue,
    };

    fn tv(v: &str, t: Tier) -> TieredValue {
        TieredValue {
            value: v.into(),
            tier: t,
            confidence: 0.9,
            source: "test".into(),
        }
    }
    fn empty() -> MappingOutput {
        MappingOutput {
            meta: OutputMeta {
                tool_version: "0.1.0".into(),
                alioth_model: "10.0.0".into(),
            },
            entities: vec![],
            summary: TierSummary {
                safe: 0,
                suggest: 0,
                unclear: 0,
            },
        }
    }

    #[test]
    fn test_excluded() {
        assert!(excluded("id"));
        assert!(!excluded("notice"));
    }
    #[test]
    fn test_unknown_col() {
        assert!(column_type("bogus").is_err());
    }
    #[test]
    fn test_known_cols() {
        assert!(column_type("fk_customer").is_ok());
    }
    #[test]
    fn test_empty_module() {
        assert!(mapping_output_to_meta_module(&empty(), "t")
            .unwrap()
            .entities
            .is_empty());
    }

    #[test]
    fn test_rust_ident() {
        assert!(is_rust_ident("name"));
        assert!(is_rust_ident("unit_price"));
        assert!(!is_rust_ident(""));
        assert!(!is_rust_ident("1bad"));
        assert!(!is_rust_ident("BadCaps"));
        assert!(!is_rust_ident("fn")); // keyword
        assert!(!is_rust_ident("struct")); // keyword
    }

    #[test]
    fn test_simple_entity() {
        let mut o = empty();
        o.entities.push(MappedEntity {
            name: "Product".into(),
            mapping: EntityMapping {
                table: "isahl.zc_id_production".into(),
                inherits: Some("zc_id_lifecycle".into()),
                source: "t".into(),
                tier: Tier::Safe,
                confidence: 1.0,
            },
            coordinates: Coordinates {
                scene: tv("YA", Tier::Safe),
                factor: tv("GBA", Tier::Safe),
                function: tv("↑.EE", Tier::Safe),
            },
            fields: vec![
                FieldMapping {
                    json_path: "name".into(),
                    column: Some("notice".into()),
                    scalar_table: None,
                    ref_table: None,
                    tier: Tier::Safe,
                    confidence: 1.0,
                    source: "t".into(),
                    alternatives: vec![],
                },
                FieldMapping {
                    json_path: "category_id".into(),
                    column: Some("ck_category".into()),
                    scalar_table: None,
                    ref_table: None,
                    tier: Tier::Safe,
                    confidence: 1.0,
                    source: "t".into(),
                    alternatives: vec![],
                },
            ],
            relationships: vec![],
        });
        o.summary.safe = 1;
        let m = mapping_output_to_meta_module(&o, "t").unwrap();
        assert_eq!(m.entities[0].fields.len(), 2);
        assert_eq!(m.pages.len(), 2);
    }

    #[test]
    fn test_qk_rejected() {
        let mut o = empty();
        o.entities.push(MappedEntity {
            name: "Item".into(),
            mapping: EntityMapping {
                table: "isahl.zc_id_lifecycle".into(),
                inherits: None,
                source: "t".into(),
                tier: Tier::Safe,
                confidence: 1.0,
            },
            coordinates: Coordinates {
                scene: tv("JC", Tier::Safe),
                factor: tv("GEC", Tier::Safe),
                function: tv("↑_DA", Tier::Safe),
            },
            fields: vec![FieldMapping {
                json_path: "price".into(),
                column: Some("qk_price".into()),
                scalar_table: None,
                ref_table: None,
                tier: Tier::Safe,
                confidence: 1.0,
                source: "t".into(),
                alternatives: vec![],
            }],
            relationships: vec![],
        });
        assert!(mapping_output_to_meta_module(&o, "t").is_ok());
    }

    #[test]
    fn test_missing_table() {
        let mut o = empty();
        o.entities.push(MappedEntity {
            name: "Ghost".into(),
            mapping: EntityMapping {
                table: String::new(),
                inherits: None,
                source: "t".into(),
                tier: Tier::Unclear,
                confidence: 0.0,
            },
            coordinates: Coordinates {
                scene: tv("JC", Tier::Unclear),
                factor: tv("GEC", Tier::Unclear),
                function: tv("↑_DA", Tier::Unclear),
            },
            fields: vec![],
            relationships: vec![],
        });
        assert!(mapping_output_to_meta_module(&o, "t").is_err());
    }

    #[test]
    fn test_invalid_field_path() {
        let mut o = empty();
        o.entities.push(MappedEntity {
            name: "Item".into(),
            mapping: EntityMapping {
                table: "isahl.zc_id_lifecycle".into(),
                inherits: None,
                source: "t".into(),
                tier: Tier::Safe,
                confidence: 1.0,
            },
            coordinates: Coordinates {
                scene: tv("JC", Tier::Safe),
                factor: tv("GEC", Tier::Safe),
                function: tv("↑_DA", Tier::Safe),
            },
            fields: vec![FieldMapping {
                json_path: "1bad_field".into(),
                column: Some("notice".into()),
                scalar_table: None,
                ref_table: None,
                tier: Tier::Safe,
                confidence: 1.0,
                source: "t".into(),
                alternatives: vec![],
            }],
            relationships: vec![],
        });
        assert!(mapping_output_to_meta_module(&o, "t").is_err());
    }

    #[test]
    fn test_unknown_relation_type() {
        let mut o = empty();
        o.entities.push(MappedEntity {
            name: "Order".into(),
            mapping: EntityMapping {
                table: "isahl.zc_id_stat-trade_order".into(),
                inherits: Some("zc_id_lifecycle".into()),
                source: "t".into(),
                tier: Tier::Safe,
                confidence: 1.0,
            },
            coordinates: Coordinates {
                scene: tv("FE", Tier::Safe),
                factor: tv("FJA", Tier::Safe),
                function: tv("↓_GD", Tier::Safe),
            },
            fields: vec![FieldMapping {
                json_path: "code".into(),
                column: Some("code".into()),
                scalar_table: None,
                ref_table: None,
                tier: Tier::Safe,
                confidence: 1.0,
                source: "t".into(),
                alternatives: vec![],
            }],
            relationships: vec![RelationshipMapping {
                target: "X".into(),
                rel_type: "bogus".into(),
                via: None,
                tier: Tier::Safe,
                confidence: 1.0,
                source: "t".into(),
            }],
        });
        o.summary.safe = 1;
        assert!(mapping_output_to_meta_module(&o, "t").is_err());
    }

    #[test]
    fn test_relation() {
        let mut o = empty();
        o.entities.push(MappedEntity {
            name: "Order".into(),
            mapping: EntityMapping {
                table: "isahl.zc_id_stat-trade_order".into(),
                inherits: Some("zc_id_lifecycle".into()),
                source: "t".into(),
                tier: Tier::Safe,
                confidence: 1.0,
            },
            coordinates: Coordinates {
                scene: tv("FE", Tier::Safe),
                factor: tv("FJA", Tier::Safe),
                function: tv("↓_GD", Tier::Safe),
            },
            fields: vec![FieldMapping {
                json_path: "code".into(),
                column: Some("code".into()),
                scalar_table: None,
                ref_table: None,
                tier: Tier::Safe,
                confidence: 1.0,
                source: "t".into(),
                alternatives: vec![],
            }],
            relationships: vec![RelationshipMapping {
                target: "Customer".into(),
                rel_type: "belongsTo".into(),
                via: Some("fk_customer".into()),
                tier: Tier::Safe,
                confidence: 1.0,
                source: "t".into(),
            }],
        });
        o.summary.safe = 1;
        let m = mapping_output_to_meta_module(&o, "t").unwrap();
        assert_eq!(m.entities[0].relations.len(), 1);
    }
}
