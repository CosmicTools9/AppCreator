use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};

const SYSTEM_COLUMNS: &[&str] = &[
    "id",
    "created_at",
    "updated_at",
    "deleted_at",
    "created_by_id",
    "updated_by_id",
    "deleted_by_id",
    "o_number",
];

const TRIGGER_COLUMNS: &[&str] = &[
    "projection",
    "ak_benefit_user",
    "ak_permit_user",
    "ak_access_user",
    "ak_source",
    "tpl_id",
];

const DK_COLUMNS: &[&str] = &["dk_scene", "dk_factor", "dk_function"];
const LIFECYCLE_INHERITED: &[&str] = &["_f_", "_t_"];

fn excluded_columns() -> HashSet<&'static str> {
    let mut s = HashSet::new();
    for c in SYSTEM_COLUMNS {
        s.insert(*c);
    }
    for c in TRIGGER_COLUMNS {
        s.insert(*c);
    }
    for c in DK_COLUMNS {
        s.insert(*c);
    }
    for c in LIFECYCLE_INHERITED {
        s.insert(*c);
    }
    s
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceEntity {
    pub name: Option<String>,
    pub table: Option<String>,
    #[serde(default)]
    pub field_mappings: Vec<FieldMapping>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldMapping {
    pub column: Option<String>,
    pub json_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceJson {
    pub entities: Option<Vec<ServiceEntity>>,
    #[serde(flatten)]
    pub other: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct SyncIssue {
    pub entity: String,
    pub severity: String,
    pub message: String,
}

pub async fn query_table_columns(pool: &PgPool, table_name: &str) -> Result<Vec<String>> {
    let tbl = table_name
        .split('.')
        .next_back()
        .unwrap_or(table_name)
        .trim_matches('"');
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT column_name FROM information_schema.columns \
         WHERE table_schema = 'isahl' AND table_name = $1 \
         ORDER BY ordinal_position",
    )
    .bind(tbl)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

fn is_business_column(col: &str) -> bool {
    !excluded_columns().contains(col)
}

pub fn compare_entity(
    entity_name: &str,
    db_cols: &[String],
    field_mappings: &[FieldMapping],
) -> Vec<SyncIssue> {
    let mut issues = Vec::new();
    let excluded = excluded_columns();
    let db_biz_cols: HashSet<&str> = db_cols
        .iter()
        .map(|s| s.as_str())
        .filter(|c| is_business_column(c))
        .collect();
    let mapped_cols: HashSet<&str> = field_mappings
        .iter()
        .filter_map(|fm| fm.column.as_deref())
        .collect();

    for col in &db_biz_cols {
        if !mapped_cols.contains(col) {
            issues.push(SyncIssue {
                entity: entity_name.to_string(),
                severity: "error".into(),
                message: format!("DB 列 '{}' 未在 field_mappings 中（缺少映射）", col),
            });
        }
    }

    for col in &mapped_cols {
        if !db_cols.iter().any(|c| c == col) {
            issues.push(SyncIssue {
                entity: entity_name.to_string(),
                severity: "error".into(),
                message: format!("field_mappings 列 '{}' 在 DB 中不存在", col),
            });
        }
    }

    for col in &mapped_cols {
        if excluded.contains(col) {
            issues.push(SyncIssue {
                entity: entity_name.to_string(),
                severity: "error".into(),
                message: format!("系统列 '{}' 不应出现在 field_mappings 中", col),
            });
        }
    }
    let mut jp_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for fm in field_mappings {
        if let Some(ref jp) = fm.json_path {
            *jp_counts.entry(jp.as_str()).or_insert(0) += 1;
        }
    }
    for (jp, count) in &jp_counts {
        if *count > 1 {
            issues.push(SyncIssue {
                entity: entity_name.to_string(),
                severity: "error".into(),
                message: format!("json_path '{}' 重复出现 {} 次", jp, count),
            });
        }
    }

    issues
}

/// Collect entities from service.json files.
/// Supports two formats:
///   - legacy: { "entities": [...] }
///   - new:    { "ontology": { "entities": [...] } }
pub fn collect_entities(services_dir: &Path) -> Result<Vec<(String, ServiceEntity)>> {
    let mut entities = Vec::new();
    if !services_dir.exists() {
        return Ok(entities);
    }

    for entry in std::fs::read_dir(services_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let service_json_path = path.join("service.json");
        if !service_json_path.exists() {
            continue;
        }

        let content = std::fs::read_to_string(&service_json_path)?;
        let svc: ServiceJson = serde_json::from_str(&content)?;

        if let Some(ents) = svc.entities {
            for ent in ents {
                if let Some(ref name) = ent.name {
                    entities.push((name.clone(), ent));
                }
            }
        } else if let Some(ont) = svc
            .other
            .get("ontology")
            .and_then(|o| o.get("entities"))
            .and_then(|e| e.as_array())
        {
            for val in ont {
                if let Ok(ent) = serde_json::from_value::<ServiceEntity>(val.clone()) {
                    if let Some(ref name) = ent.name {
                        entities.push((name.clone(), ent));
                    }
                }
            }
        }
    }
    Ok(entities)
}

/// Collect entities from service.json files, returning (name, entity, service_json_path).
/// Supports both legacy `entities` and current `ontology.entities` formats.
pub fn collect_entities_with_paths(
    services_dir: &Path,
) -> Result<Vec<(String, ServiceEntity, PathBuf)>> {
    let mut entities = Vec::new();
    if !services_dir.exists() {
        return Ok(entities);
    }
    for entry in std::fs::read_dir(services_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let svc_path = path.join("service.json");
        if !svc_path.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&svc_path)?;
        let svc: ServiceJson = serde_json::from_str(&content)?;

        if let Some(ents) = svc.entities {
            for ent in ents {
                if let Some(ref name) = ent.name {
                    entities.push((name.clone(), ent, svc_path.clone()));
                }
            }
        } else if let Some(ont) = svc
            .other
            .get("ontology")
            .and_then(|o| o.get("entities"))
            .and_then(|e| e.as_array())
        {
            for val in ont {
                if let Ok(ent) = serde_json::from_value::<ServiceEntity>(val.clone()) {
                    if let Some(ref name) = ent.name {
                        entities.push((name.clone(), ent, svc_path.clone()));
                    }
                }
            }
        }
    }
    Ok(entities)
}

pub fn infer_json_path(column: &str) -> String {
    match column {
        "notice" => return "name".into(),
        "code" => return "code".into(),
        "comments" => return "comments".into(),
        _ => {}
    }
    for prefix in &["fk_", "qk_", "ck_", "sk_", "tk_", "lk_"] {
        if let Some(stripped) = column.strip_prefix(prefix) {
            return stripped.to_string();
        }
    }
    column.to_string()
}

pub fn build_correct_mappings(
    db_cols: &[String],
    existing_map: &std::collections::HashMap<String, String>,
) -> Vec<FieldMapping> {
    let excluded = excluded_columns();
    let mut mappings = Vec::new();
    let mut used_paths = HashSet::new();

    for col in db_cols {
        if excluded.contains(col.as_str()) {
            continue;
        }
        let json_path = existing_map
            .get(col)
            .cloned()
            .unwrap_or_else(|| infer_json_path(col));

        let final_path = if used_paths.contains(&json_path) {
            let inferred = infer_json_path(col);
            if used_paths.contains(&inferred) {
                format!("{inferred}_{col}")
            } else {
                inferred
            }
        } else {
            json_path
        };

        used_paths.insert(final_path.clone());
        mappings.push(FieldMapping {
            column: Some(col.clone()),
            json_path: Some(final_path),
        });
    }
    mappings
}

pub fn fix_service_json(
    path: &Path,
    fix_map: &std::collections::HashMap<String, Vec<FieldMapping>>,
) -> Result<bool> {
    let content = std::fs::read_to_string(path)?;
    let mut root: serde_json::Value = serde_json::from_str(&content)?;

    // Locate entities array: prefer /ontology/entities, fall back to /entities
    fn patch_entities(
        arr: &mut [serde_json::Value],
        fix_map: &std::collections::HashMap<String, Vec<FieldMapping>>,
    ) -> bool {
        let mut changed = false;
        for ent in arr.iter_mut() {
            let name = ent["name"].as_str().unwrap_or("");
            if let Some(fix_mappings) = fix_map.get(name) {
                let mappings_val: Vec<serde_json::Value> = fix_mappings
                    .iter()
                    .map(|fm| {
                        serde_json::json!({
                            "json_path": fm.json_path.as_deref().unwrap_or(""),
                            "column": fm.column.as_deref().unwrap_or(""),
                        })
                    })
                    .collect();
                ent["field_mappings"] = serde_json::Value::Array(mappings_val);
                changed = true;
            }
        }
        changed
    }

    let changed = if let Some(arr) = root
        .pointer_mut("/ontology/entities")
        .and_then(|v| v.as_array_mut())
    {
        patch_entities(arr, fix_map)
    } else if let Some(arr) = root.get_mut("entities").and_then(|v| v.as_array_mut()) {
        patch_entities(arr, fix_map)
    } else {
        return Ok(false);
    };

    if !changed {
        return Ok(false);
    }

    // Atomic write: temp file then rename
    let tmp = path.with_extension("json.tmp");
    let mut tmp_file = std::fs::File::create(&tmp)?;
    tmp_file.write_all(serde_json::to_string_pretty(&root)?.as_bytes())?;
    tmp_file.write_all(b"\n")?;
    tmp_file.flush()?;
    std::mem::drop(tmp_file);
    std::fs::rename(&tmp, path)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_compare_entity_missing_biz_col() {
        let issues = compare_entity("E", &["id".into(), "name".into()], &[]);
        assert!(issues.iter().any(|i| i.message.contains("name")));
    }

    #[test]
    fn test_compare_entity_system_col_skipped() {
        let issues = compare_entity("E", &["id".into(), "created_at".into()], &[]);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_compare_entity_extra_col() {
        let issues = compare_entity(
            "E",
            &["name".into()],
            &[FieldMapping {
                column: Some("ghost".into()),
                json_path: Some("p".into()),
            }],
        );
        assert!(issues.iter().any(|i| i.message.contains("ghost")));
    }

    #[test]
    fn test_compare_entity_system_in_mappings() {
        let issues = compare_entity(
            "E",
            &["name".into()],
            &[FieldMapping {
                column: Some("id".into()),
                json_path: Some("id".into()),
            }],
        );
        assert!(issues.iter().any(|i| i.message.contains("系统列")));
    }

    #[test]
    fn test_compare_entity_dup_json_path() {
        let issues = compare_entity(
            "E",
            &["a".into(), "b".into()],
            &[
                FieldMapping {
                    column: Some("a".into()),
                    json_path: Some("x".into()),
                },
                FieldMapping {
                    column: Some("b".into()),
                    json_path: Some("x".into()),
                },
            ],
        );
        assert!(issues.iter().any(|i| i.message.contains("重复")));
    }

    #[test]
    fn test_collect_entities_legacy_format() {
        let dir = std::env::temp_dir().join("sync-test-legacy");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("svc1")).unwrap();
        std::fs::write(
            dir.join("svc1/service.json"),
            r#"{"entities":[{"name":"E1","table":"isahl.zc_id_test","field_mappings":[{"column":"name","json_path":"name"}]}]}"#,
        ).unwrap();
        let ents = collect_entities(&dir).unwrap();
        assert_eq!(ents.len(), 1);
        assert_eq!(ents[0].0, "E1");
    }

    #[test]
    fn test_collect_entities_ontology_format() {
        let dir = std::env::temp_dir().join("sync-test-ont");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("svc1")).unwrap();
        std::fs::write(
            dir.join("svc1/service.json"),
            r#"{"ontology":{"entities":[{"name":"E1","table":"isahl.zc_id_test","field_mappings":[{"column":"name","json_path":"name"}]}]}}"#,
        ).unwrap();
        let ents = collect_entities(&dir).unwrap();
        assert_eq!(ents.len(), 1);
        assert_eq!(ents[0].0, "E1");
    }

    #[test]
    fn test_collect_entities_missing_dir() {
        let dir = std::env::temp_dir().join("sync-test-nonexistent");
        let _ = std::fs::remove_dir_all(&dir);
        let ents = collect_entities(&dir).unwrap();
        assert!(ents.is_empty());
    }
}

#[test]
fn test_build_correct_mappings_adds_missing() {
    let db_cols = vec!["name".into(), "status".into()];
    let existing = std::collections::HashMap::new();
    let mappings = build_correct_mappings(&db_cols, &existing);
    assert_eq!(mappings.len(), 2);
    assert!(mappings.iter().any(|m| m.column == Some("name".into())));
    assert!(mappings.iter().any(|m| m.column == Some("status".into())));
}

#[test]
fn test_build_correct_mappings_preserves_custom_json_path() {
    let db_cols = vec!["fk_user".into()];
    let mut existing = std::collections::HashMap::new();
    existing.insert("fk_user".into(), "customUser".into());
    let mappings = build_correct_mappings(&db_cols, &existing);
    assert_eq!(mappings.len(), 1);
    assert_eq!(mappings[0].json_path, Some("customUser".into()));
}

#[test]
fn test_build_correct_mappings_removes_extra() {
    let db_cols = vec!["name".into()];
    let existing = std::collections::HashMap::new();
    let mappings = build_correct_mappings(&db_cols, &existing);
    assert_eq!(mappings.len(), 1);
    assert_eq!(mappings[0].column, Some("name".into()));
}

#[test]
fn test_fix_service_json_creates_file() {
    use std::io::Write;
    let dir = std::env::temp_dir().join("fix-test-json");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let svc_path = dir.join("service.json");
    let original = r#"{"ontology":{"entities":[{"name":"E1","table":"isahl.zc_id_test","field_mappings":[{"column":"old","json_path":"old"}]},{"name":"E2","table":"isahl.zc_id_test","field_mappings":[]}]}}"#;
    let mut f = std::fs::File::create(&svc_path).unwrap();
    f.write_all(original.as_bytes()).unwrap();
    f.flush().unwrap();

    let mut fix_map = std::collections::HashMap::new();
    fix_map.insert(
        "E1".into(),
        vec![FieldMapping {
            column: Some("name".into()),
            json_path: Some("name".into()),
        }],
    );
    fix_map.insert(
        "E2".into(),
        vec![FieldMapping {
            column: Some("status".into()),
            json_path: Some("status".into()),
        }],
    );

    let result = fix_service_json(&svc_path, &fix_map).unwrap();
    assert!(result, "should have modified file");

    // Verify both entities fixed
    let content = std::fs::read_to_string(&svc_path).unwrap();
    let root: serde_json::Value = serde_json::from_str(&content).unwrap();
    let entities = root["ontology"]["entities"].as_array().unwrap();
    assert_eq!(entities.len(), 2);
    assert_eq!(entities[0]["field_mappings"][0]["column"], "name");
    assert_eq!(entities[1]["field_mappings"][0]["column"], "status");

    let _ = std::fs::remove_dir_all(&dir);
}
