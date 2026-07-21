//! gap.rs — Block 原型语义与本体规约差异检测（1:1 移植 check-ontology-gap.py）
//!
//! JSX/HTML 提取委托项目规范解析器 `scripts/parser-utils.mjs`（bun，Node 工具链，
//! 遵循 NO_REGEX_FOR_PARSING 的 parser 集中原则）；比较与报告逻辑为纯 Rust。

use anyhow::{anyhow, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};

// ── parser-utils.mjs 调用 ─────────────────────────────

fn find_parser_utils(start: &Path) -> Option<PathBuf> {
    let mut cur = Some(start.to_path_buf());
    while let Some(p) = cur {
        let candidate = p.join("scripts").join("parser-utils.mjs");
        if candidate.exists() {
            return Some(candidate);
        }
        cur = p.parent().map(|x| x.to_path_buf());
    }
    None
}

fn run_parser(cmd: &str, file: &Path) -> Result<serde_json::Value> {
    let parser = find_parser_utils(file).ok_or_else(|| anyhow!("Cannot find parser-utils.mjs"))?;
    let tmp = std::env::temp_dir().join(format!("onto-gap-{}.json", uuid_v4()));
    let out_file = std::fs::File::create(&tmp)?;
    let status = std::process::Command::new("bun")
        .arg(&parser)
        .arg(cmd)
        .arg(file)
        .stdout(out_file)
        .stderr(std::process::Stdio::piped())
        .status()?;
    if !status.success() {
        let _ = std::fs::remove_file(&tmp);
        return Err(anyhow!("parser-utils {cmd} failed"));
    }
    let content = std::fs::read_to_string(&tmp)?;
    let _ = std::fs::remove_file(&tmp);
    Ok(serde_json::from_str(&content)?)
}

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}")
}

// ── 提取逻辑 ──────────────────────────────────────────

/// 从 mock 数组字面量源码提取字段名（与 Python `_fields_from_code` 一致：
/// 取首个对象字面量的键）。
fn fields_from_code(snippet: &str) -> Vec<String> {
    let obj_re = regex::Regex::new(r"\{([^}]+)\}").unwrap();
    let key_re = regex::Regex::new(r"(\w+)\s*:").unwrap();
    let Some(cap) = obj_re.captures(snippet) else {
        return vec![];
    };
    key_re
        .captures_iter(&cap[1])
        .map(|c| c[1].to_string())
        .collect()
}

fn extract_mock_fields(file: &Path) -> Vec<(String, Vec<String>)> {
    let Ok(parsed) = run_parser("find-mock-data", file) else {
        return vec![];
    };
    let Some(list) = parsed.as_array() else {
        return vec![];
    };
    list.iter()
        .filter_map(|m| {
            let name = m.get("name")?.as_str()?.to_string();
            let code = m.get("code")?.as_str()?;
            Some((name, fields_from_code(code)))
        })
        .collect()
}

/// 提取 serviceApis（纯字符串查找，与 Python `_extract_scene_serviceapis` 一致）。
pub fn extract_scene_serviceapis(jsx: &str) -> Vec<String> {
    let Some(fp) = jsx.find("serviceApis") else {
        return vec![];
    };
    let Some(ob) = jsx[fp..].find('[').map(|i| fp + i) else {
        return vec![];
    };
    let Some(cb) = jsx[ob..].find(']').map(|i| ob + i) else {
        return vec![];
    };
    jsx[ob + 1..cb]
        .split(',')
        .map(|a| a.trim().trim_matches(['\'', '"']).to_string())
        .filter(|a| !a.is_empty())
        .collect()
}

fn extract_page_fields(file: &Path) -> Vec<String> {
    let Ok(parsed) = run_parser("extract-form-fields", file) else {
        return vec![];
    };
    let Some(list) = parsed.as_array() else {
        return vec![];
    };
    let keys: Vec<String> = list
        .iter()
        .filter_map(|item| {
            ["name", "field", "label"]
                .iter()
                .find_map(|k| item.get(k).and_then(|v| v.as_str()).map(|s| s.to_string()))
        })
        .take(20)
        .collect();
    keys
}

// ── 规约读取（ontology-output.json v3）────────────────

#[derive(Debug, Default, Serialize)]
pub struct SpecData {
    pub entities: Vec<String>,
    pub factor_ids: Vec<String>,
}

fn find_ontology_output(proto_path: &Path) -> Option<PathBuf> {
    // Pre-Proc/{ns}/Prototypes/Blocks/{block}/v{N}.html → Pre-Proc/{ns}/local/ontology-output.json
    for ancestor in proto_path.ancestors().take(6) {
        if ancestor.file_name()?.to_str()? == "Prototypes" {
            let candidate = ancestor
                .parent()?
                .join("local")
                .join("ontology-output.json");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    None
}

fn read_spec_data(proto_path: &Path) -> SpecData {
    let Some(json_path) = find_ontology_output(proto_path) else {
        return SpecData::default();
    };
    let Ok(content) = std::fs::read_to_string(&json_path) else {
        return SpecData::default();
    };
    let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) else {
        return SpecData::default();
    };
    let mut entities = std::collections::BTreeSet::new();
    let mut factors = std::collections::BTreeSet::new();
    if let Some(alignments) = data.get("alignment_matrix").and_then(|v| v.as_array()) {
        for a in alignments {
            if let Some(e) = a
                .get("layer1_model_binding")
                .and_then(|b| b.get("entity"))
                .and_then(|v| v.as_str())
            {
                entities.insert(e.to_string());
            }
            if let Some(f) = a
                .get("layer2_coordinate_mapping")
                .and_then(|b| b.get("factor_code"))
                .and_then(|v| v.as_str())
            {
                if !f.is_empty() {
                    factors.insert(f.to_string());
                }
            }
        }
    }
    SpecData {
        entities: entities.into_iter().collect(),
        factor_ids: factors.into_iter().collect(),
    }
}

// ── 语义形状推断与差异分析 ────────────────────────────

#[derive(Debug, Serialize)]
pub struct SemanticShape {
    pub entity: String,
    pub mock_var: String,
    pub fields: Vec<String>,
    pub field_count: usize,
    pub requires_scalar_resolution: bool,
}

fn infer_semantic_shapes(mocks: &[(String, Vec<String>)]) -> Vec<SemanticShape> {
    mocks
        .iter()
        .map(|(mock_name, fields)| {
            let base = mock_name.strip_prefix("mock").unwrap_or(mock_name);
            let deplural = base.strip_suffix('s').unwrap_or(base);
            let entity = if deplural.is_empty() {
                "Entity".to_string()
            } else {
                let mut c = deplural.chars();
                c.next().unwrap().to_uppercase().collect::<String>() + c.as_str()
            };
            SemanticShape {
                requires_scalar_resolution: fields
                    .iter()
                    .any(|f| f.starts_with("qk_") || f.starts_with("sk_") || f.starts_with("ck_")),
                field_count: fields.len(),
                fields: fields.clone(),
                mock_var: mock_name.clone(),
                entity,
            }
        })
        .collect()
}

#[derive(Debug, Serialize)]
pub struct GapItem {
    pub r#type: String,
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub entities: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct GapReport {
    pub scene: String,
    pub source: String,
    pub service_apis: Vec<String>,
    pub mock_data: serde_json::Value,
    pub page_fields: Vec<String>,
    pub semantic_shapes: Vec<SemanticShape>,
    pub spec_entities: Vec<String>,
    pub gaps: Vec<GapItem>,
}

/// 主分析（替代 Python `analyze`）。
pub fn analyze(proto_path: &Path) -> Result<GapReport> {
    let mocks = extract_mock_fields(proto_path);
    let content = std::fs::read_to_string(proto_path)?;
    let service_apis = extract_scene_serviceapis(&content);
    let page_fields = extract_page_fields(proto_path);

    let scene_id = proto_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    let spec_data = read_spec_data(proto_path);
    let shapes = infer_semantic_shapes(&mocks);

    let mut gaps = Vec::new();
    if !spec_data.entities.is_empty() {
        let spec_set: std::collections::HashSet<&String> = spec_data.entities.iter().collect();
        let missing: Vec<String> = shapes
            .iter()
            .filter(|s| !spec_set.contains(&s.entity))
            .map(|s| s.entity.clone())
            .collect();
        if !missing.is_empty() {
            gaps.push(GapItem {
                r#type: "entity_missing".into(),
                message: "实体在 ONTOLOGY_SPEC 中缺失".into(),
                entities: missing,
            });
        }
    }
    let missing_scalars: Vec<String> = shapes
        .iter()
        .filter(|s| s.requires_scalar_resolution)
        .map(|s| s.entity.clone())
        .collect();
    if !missing_scalars.is_empty() {
        gaps.push(GapItem {
            r#type: "scalar_unresolved".into(),
            message: "mock 数据中的标量引用字段需要在 DTO 中解析为实际值".into(),
            entities: missing_scalars,
        });
    }
    if spec_data.entities.is_empty() && spec_data.factor_ids.is_empty() {
        gaps.push(GapItem {
            r#type: "no_ontology_spec".into(),
            message: format!("找不到 {scene_id} 的 ontology-output.json，需要先生成"),
            entities: vec![],
        });
    }

    let mock_json: serde_json::Map<String, serde_json::Value> = mocks
        .iter()
        .take(10)
        .map(|(k, v)| (k.clone(), serde_json::json!(v)))
        .collect();

    Ok(GapReport {
        scene: scene_id,
        source: proto_path.display().to_string(),
        service_apis,
        mock_data: serde_json::Value::Object(mock_json),
        page_fields,
        semantic_shapes: shapes,
        spec_entities: spec_data.entities,
        gaps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fields_from_code() {
        let code = r#"[{ id: 1, name: "千克", symbol: "kg" }, { id: 2 }]"#;
        assert_eq!(fields_from_code(code), vec!["id", "name", "symbol"]);
    }

    #[test]
    fn test_extract_serviceapis() {
        let jsx =
            r#"window.aliothBlockComponents = { serviceApis: ['unit', 'rate'], component: X }"#;
        assert_eq!(extract_scene_serviceapis(jsx), vec!["unit", "rate"]);
        assert!(extract_scene_serviceapis("nothing here").is_empty());
    }

    #[test]
    fn test_infer_shapes() {
        let mocks = vec![
            (
                "mockUnits".to_string(),
                vec!["name".into(), "qk_qty".into()],
            ),
            ("mockProducts".to_string(), vec!["name".into()]),
        ];
        let shapes = infer_semantic_shapes(&mocks);
        assert_eq!(shapes[0].entity, "Unit");
        assert!(shapes[0].requires_scalar_resolution);
        assert_eq!(shapes[1].entity, "Product");
        assert!(!shapes[1].requires_scalar_resolution);
    }
}
