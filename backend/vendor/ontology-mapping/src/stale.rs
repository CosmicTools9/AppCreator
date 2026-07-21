//! stale.rs — ontology MappingOutput 保鲜检测（1:1 移植 check-ontology-stale.py）
//!
//! 双检查：时间戳比较（原型 mtime vs ontology generated_at/mtime，1s 容差）
//! + 字段覆盖（原型 mock 字段 vs ontology db_bindings/service.json 字段集）。

use anyhow::Result;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

// ── 原型与 ontology 文件定位 ──────────────────────────

/// 在 Pre-Proc/{ns}/Prototypes/Blocks/{scene}/（回退 Modules/）找最新 v{N}.html
pub fn find_latest_prototype_html(ns: &str, scene: &str) -> Option<PathBuf> {
    let mut dir = PathBuf::from(format!("Pre-Proc/{ns}/Prototypes/Blocks/{scene}"));
    if !dir.is_dir() {
        dir = PathBuf::from(format!("Pre-Proc/{ns}/Prototypes/Modules/{scene}"));
        if !dir.is_dir() {
            return None;
        }
    }
    let v_re = regex::Regex::new(r"v\d+\.html$").unwrap();
    let mut candidates: Vec<(PathBuf, f64)> = Vec::new();
    collect_html(&dir, &v_re, &mut candidates);
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    candidates.into_iter().next().map(|(p, _)| p)
}

fn collect_html(dir: &Path, v_re: &regex::Regex, out: &mut Vec<(PathBuf, f64)>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_html(&p, v_re, out);
        } else if p.extension().is_some_and(|e| e == "html") {
            let name = p.file_name().unwrap_or_default().to_string_lossy();
            if v_re.is_match(&name) {
                let mtime = std::fs::metadata(&p)
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs_f64())
                    .unwrap_or(0.0);
                out.push((p, mtime));
            }
        }
    }
}

/// 找到 ontology 产出文件（3 候选 + service.json 回退）
pub fn find_ontology_output(ns: &str) -> Option<PathBuf> {
    let candidates = [
        PathBuf::from(format!("Pre-Proc/{ns}/Prototypes/ontology-output.json")),
        PathBuf::from(format!("Pre-Proc/tmp/ontology-mapping-{ns}.result.json")),
        PathBuf::from(format!("/tmp/ontology-mapping-{ns}.result.json")),
    ];
    for p in &candidates {
        if p.is_file() {
            return Some(p.clone());
        }
    }
    // 回退：最新 service.json（含 ontology.entities）
    let services_dir = PathBuf::from(format!("Pre-Proc/{ns}/Sources/Services"));
    let mut best: Option<(PathBuf, f64)> = None;
    collect_services(&services_dir, &mut |p, m| {
        if best.as_ref().map(|(_, bm)| m > *bm).unwrap_or(true) {
            best = Some((p.clone(), m));
        }
    });
    best.map(|(p, _)| p)
}

fn collect_services(dir: &Path, f: &mut dyn FnMut(&PathBuf, f64)) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_services(&p, f);
        } else if p.file_name().is_some_and(|n| n == "service.json") {
            let Ok(content) = std::fs::read_to_string(&p) else {
                continue;
            };
            let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) else {
                continue;
            };
            let has_entities = data
                .get("ontology")
                .and_then(|o| o.get("entities"))
                .and_then(|e| e.as_array())
                .is_some_and(|a| !a.is_empty());
            if has_entities {
                let m = std::fs::metadata(&p)
                    .and_then(|md| md.modified())
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs_f64())
                    .unwrap_or(0.0);
                f(&p, m);
            }
        }
    }
}

// ── 时间戳提取 ─────────────────────────────────────────

fn parse_ts(ts: &str) -> Option<f64> {
    // 支持: ISO 带 Z / 带毫秒 / 纯日期 / 日期时间
    let formats = [
        "%Y-%m-%dT%H:%M:%S%.fZ",
        "%Y-%m-%dT%H:%M:%SZ",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d",
    ];
    for fmt in formats {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(ts, fmt) {
            return Some(dt.and_utc().timestamp() as f64);
        }
        if let Ok(d) = chrono::NaiveDate::parse_from_str(ts, fmt) {
            return d
                .and_hms_opt(0, 0, 0)
                .map(|dt| dt.and_utc().timestamp() as f64);
        }
    }
    None
}

fn file_mtime(path: &Path) -> f64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// 从 ontology 产出文件提取生成时间戳（meta.generated_at → db_bindings verified_at → mtime）
pub fn extract_mtime_or_generated_at(path: &Path) -> f64 {
    if path.extension().is_some_and(|e| e == "json") {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(meta) = data.get("meta") {
                    for key in ["generated_at", "generatedAt"] {
                        if let Some(ts) = meta.get(key).and_then(|v| v.as_str()) {
                            if let Some(t) = parse_ts(ts) {
                                return t;
                            }
                        }
                    }
                }
                if let Some(bindings) = data.get("db_bindings").and_then(|v| v.as_array()) {
                    let mut verified: Vec<f64> = bindings
                        .iter()
                        .filter_map(|b| b.get("verified_at").and_then(|v| v.as_str()))
                        .filter_map(parse_ts)
                        .collect();
                    verified.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
                    if let Some(t) = verified.first() {
                        return *t;
                    }
                }
            }
        }
    }
    file_mtime(path)
}

// ── 字段集提取 ─────────────────────────────────────────

pub type FieldSets = BTreeMap<String, BTreeSet<String>>;

/// 从 ontology-output.json 的 db_bindings[].fields[] 提取实体→物理列集合
pub fn ontology_fields_from_mapping(ns: &str) -> FieldSets {
    let mut result = FieldSets::new();
    let path = PathBuf::from(format!("Pre-Proc/{ns}/local/ontology-output.json"));
    let Ok(content) = std::fs::read_to_string(path) else {
        return result;
    };
    let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) else {
        return result;
    };
    if let Some(bindings) = data.get("db_bindings").and_then(|v| v.as_array()) {
        for b in bindings {
            let Some(ename) = b.get("model_entity").and_then(|v| v.as_str()) else {
                continue;
            };
            let fields: BTreeSet<String> = b
                .get("fields")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|f| f.get("column").and_then(|c| c.as_str()).map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            if !fields.is_empty() {
                result.insert(ename.to_string(), fields);
            }
        }
    }
    result
}

/// 从 service.json 的 ontology 块提取实体→json_path 集合（备用回退）
pub fn ontology_fields_from_service(ns: &str) -> FieldSets {
    let mut result = FieldSets::new();
    let dir = PathBuf::from(format!("Pre-Proc/{ns}/Sources/Services"));
    collect_services(&dir, &mut |p, _| {
        let Ok(content) = std::fs::read_to_string(p) else {
            return;
        };
        let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) else {
            return;
        };
        if let Some(entities) = data
            .get("ontology")
            .and_then(|o| o.get("entities"))
            .and_then(|e| e.as_array())
        {
            for e in entities {
                let Some(name) = e.get("name").and_then(|v| v.as_str()) else {
                    continue;
                };
                let fields: BTreeSet<String> = e
                    .get("field_mappings")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|f| {
                                f.get("json_path")
                                    .and_then(|c| c.as_str())
                                    .map(String::from)
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                result.insert(name.to_string(), fields);
            }
        }
    });
    result
}

/// 递归收集 JSON 对象字段名（与 Python `_collect_fields` 一致）
fn collect_fields(obj: &serde_json::Value, prefix: &str, result: &mut FieldSets) {
    if let Some(map) = obj.as_object() {
        result
            .entry(if prefix.is_empty() {
                "_root".into()
            } else {
                prefix.into()
            })
            .or_default()
            .extend(map.keys().cloned());
        for (key, val) in map {
            let sub = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{prefix}.{key}")
            };
            if val.is_object() {
                collect_fields(val, &sub, result);
            } else if let Some(arr) = val.as_array() {
                if let Some(first) = arr.first() {
                    if first.is_object() {
                        if let Some(fm) = first.as_object() {
                            result
                                .entry(sub.clone())
                                .or_default()
                                .extend(fm.keys().cloned());
                        }
                        collect_fields(first, &sub, result);
                    }
                }
            }
        }
    }
}

/// 从原型提取 mock 字段：首选 llm-tsx/mock.json，回退 inline MOCK（parser-utils）
pub fn extract_prototype_mock_fields(html_path: &Path) -> FieldSets {
    let mut result = FieldSets::new();
    // 1. llm-tsx/mock.json
    let mock_json = html_path
        .parent()
        .map(|p| p.join("llm-tsx").join("mock.json"));
    if let Some(mp) = mock_json {
        if mp.exists() {
            if let Ok(content) = std::fs::read_to_string(&mp) {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                    collect_fields(&data, "", &mut result);
                    return result;
                }
            }
        }
    }
    // 2. inline MOCK via parser-utils extract-all-scripts
    if let Some(parser) = super::prototype_check::find_parser_utils_pub(html_path) {
        let out = std::process::Command::new("bun")
            .arg(&parser)
            .arg("extract-all-scripts")
            .arg(html_path)
            .output();
        if let Ok(o) = out {
            if o.status.success() {
                if let Ok(scripts) = serde_json::from_slice::<Vec<String>>(&o.stdout) {
                    let prefixes = [
                        "var MOCK = ",
                        "const MOCK = ",
                        "let MOCK = ",
                        "var mockData = ",
                        "var mock = ",
                        "window.__MOCK__ = ",
                    ];
                    for script in scripts {
                        let text = script.trim();
                        for prefix in prefixes {
                            if let Some(payload) = text.strip_prefix(prefix) {
                                let payload = payload.trim_end_matches(';').trim();
                                if let Ok(data) = serde_json::from_str::<serde_json::Value>(payload)
                                {
                                    collect_fields(&data, "", &mut result);
                                    return result;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    result
}

// ── 检查逻辑 ──────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct StaleReport {
    pub ontology_path: String,
    pub prototype_path: Option<String>,
    pub issues: Vec<String>,
    pub stale: bool,
}

pub fn check_stale(
    ns: &str,
    scene: &str,
    ontology_output: Option<&str>,
    skip_field_check: bool,
) -> Result<(StaleReport, i32)> {
    let ontology_path = match ontology_output {
        Some(p) => PathBuf::from(p),
        None => {
            find_ontology_output(ns).ok_or_else(|| anyhow::anyhow!("未找到 ontology 产出文件"))?
        }
    };
    if !ontology_path.is_file() {
        // 与 Python 一致：无 ontology 文件 → 不是过时问题，exit 0
        return Ok((
            StaleReport {
                ontology_path: ontology_path.display().to_string(),
                prototype_path: None,
                issues: vec![],
                stale: false,
            },
            0,
        ));
    }

    let proto_path = find_latest_prototype_html(ns, scene);
    let effective_proto = proto_path.clone().unwrap_or_else(|| ontology_path.clone());
    let proto_mtime = file_mtime(&effective_proto);
    let onto_mtime = extract_mtime_or_generated_at(&ontology_path);

    let mut issues = Vec::new();
    if proto_mtime > onto_mtime + 1.0 {
        let fmt = |t: f64| {
            chrono::DateTime::from_timestamp(t as i64, 0)
                .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default()
        };
        issues.push(format!(
            "原型文件较新 ({}) UTC vs ontology ({}) UTC",
            fmt(proto_mtime),
            fmt(onto_mtime)
        ));
    }

    if !skip_field_check {
        let proto_fields = extract_prototype_mock_fields(&effective_proto);
        let mut ontology_fields = ontology_fields_from_mapping(ns);
        if ontology_fields.is_empty() {
            ontology_fields = ontology_fields_from_service(ns);
        }
        if !proto_fields.is_empty() && !ontology_fields.is_empty() {
            for (entity, pfields) in &proto_fields {
                let Some(ofields) = ontology_fields.get(entity) else {
                    continue;
                };
                let missing: Vec<&String> = pfields.difference(ofields).collect();
                if !missing.is_empty() {
                    issues.push(format!(
                        "实体「{entity}」原型含 ontology 缺少的字段: {missing:?}"
                    ));
                }
            }
        }
    }

    let stale = !issues.is_empty();
    Ok((
        StaleReport {
            ontology_path: ontology_path.display().to_string(),
            prototype_path: proto_path.map(|p| p.display().to_string()),
            issues,
            stale,
        },
        if stale { 1 } else { 0 },
    ))
}
