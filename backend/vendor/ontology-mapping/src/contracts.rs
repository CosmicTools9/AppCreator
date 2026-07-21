//! contracts.rs — 跨模块契约检测与校准（1:1 移植 apply-ontology-contracts.py）
//!
//! 数据源优先级：MappingOutput JSON（`Pre-Proc/{ns}/local/ontology-output.json`）
//! 覆盖权威表（`docs/specs/DTO_DESIGN_SPEC.md` §6 markdown 表）。

use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// ── MappingOutput JSON 读取 ───────────────────────────

fn load_ontology_mapping(ns: &str) -> Option<serde_json::Value> {
    let path = PathBuf::from(format!("Pre-Proc/{ns}/local/ontology-output.json"));
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// 从 MappingOutput JSON 的 db_bindings[].fields[] 提取 {列名: 可写性}
pub fn extract_column_writability(ns: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    if let Some(data) = load_ontology_mapping(ns) {
        if let Some(bindings) = data.get("db_bindings").and_then(|v| v.as_array()) {
            for b in bindings {
                if let Some(fields) = b.get("fields").and_then(|v| v.as_array()) {
                    for f in fields {
                        if let (Some(col), Some(w)) = (
                            f.get("column").and_then(|v| v.as_str()),
                            f.get("writability").and_then(|v| v.as_str()),
                        ) {
                            map.insert(col.to_string(), w.to_string());
                        }
                    }
                }
            }
        }
    }
    if map.is_empty() {
        return load_canonical_writability();
    }
    map
}

/// 从 MappingOutput JSON 的 system_exclusions[] 提取 {列名: 原因}
pub fn extract_system_exclusions(ns: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    if let Some(data) = load_ontology_mapping(ns) {
        if let Some(ex) = data.get("system_exclusions").and_then(|v| v.as_array()) {
            for item in ex {
                if let Some(col) = item.get("column").and_then(|v| v.as_str()) {
                    let reason = item.get("reason").and_then(|v| v.as_str()).unwrap_or("🚫");
                    map.insert(col.to_string(), reason.to_string());
                }
            }
        }
    }
    map
}

/// 合并 canonical + JSON 映射（JSON 优先）
pub fn get_all_field_mappings(ns: &str) -> BTreeMap<String, String> {
    let json_map = extract_column_writability(ns);
    let canonical = load_canonical_writability();
    let mut merged = canonical;
    merged.extend(json_map);
    merged
}

// ── DTO_DESIGN_SPEC.md §6 权威表解析 ──────────────────

/// 从 DTO_DESIGN_SPEC.md §6（`六` 与 `七` 标题之间）加载 {列名: writability emoji}
pub fn load_canonical_writability() -> BTreeMap<String, String> {
    load_canonical_writability_from(Path::new("docs/specs/DTO_DESIGN_SPEC.md"))
}

pub fn load_canonical_writability_from(path: &Path) -> BTreeMap<String, String> {
    let mut mapping = BTreeMap::new();
    let Ok(content) = std::fs::read_to_string(path) else {
        return mapping;
    };
    let head_re = regex::Regex::new(r"^#+\s*(六|七)").unwrap();
    let col_re = regex::Regex::new(r"^\|\s*`([^`]+)`\s*\|").unwrap();
    let emoji_re = regex::Regex::new(r"[✅🔒⚠️🚫]").unwrap();

    let mut in_s6 = false;
    for line in content.lines() {
        if head_re.is_match(line) {
            if line.contains('六') {
                in_s6 = true;
                continue;
            } else if in_s6 {
                break;
            }
        }
        if !in_s6 {
            continue;
        }
        if let Some(cap) = col_re.captures(line) {
            let col = cap[1].split('/').next().unwrap_or("").trim().to_string();
            if let Some(e) = emoji_re.find(line) {
                mapping.insert(col, e.as_str().to_string());
            }
        }
    }
    mapping
}

// ── 模块发现 ──────────────────────────────────────────

/// 列出所有 docs/specs/{module}/ONTOLOGY_SPEC.md
pub fn find_all_specs() -> Vec<(String, PathBuf)> {
    glob_specs()
        .into_iter()
        .filter_map(|p| {
            let module = p.parent()?.file_name()?.to_str()?.to_string();
            Some((module, p))
        })
        .collect()
}

fn glob_specs() -> Vec<PathBuf> {
    let Ok(rd) = std::fs::read_dir("docs/specs") else {
        return vec![];
    };
    let mut out = vec![];
    for entry in rd.flatten() {
        let spec = entry.path().join("ONTOLOGY_SPEC.md");
        if spec.is_file() {
            out.push(spec);
        }
    }
    out.sort();
    out
}

/// 找到所有引用某张表的 ONTOLOGY_SPEC.md 及其模块名
pub fn find_specs_for_table(target_table: &str) -> Vec<(String, PathBuf)> {
    let clean = target_table.replace('"', "").replace("isahl.", "");
    find_all_specs()
        .into_iter()
        .filter(|(_, p)| {
            std::fs::read_to_string(p)
                .map(|c| c.contains(&clean))
                .unwrap_or(false)
        })
        .collect()
}

// ── 一致性矩阵 ─────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct MatrixRow {
    pub column: String,
    pub canonical: String,
    pub target: String,
    pub action: String,
}

#[derive(Debug, Serialize)]
pub struct ConsistencyMatrix {
    pub rows: Vec<MatrixRow>,
    pub consistent: usize,
    pub auto_fix: usize,
    pub conflicts: usize,
}

/// 构建模块 × 字段 一致性矩阵（与 Python 的 action 判定语义一致）
pub fn build_consistency_matrix(
    target_module: &str,
    target_ns: &str,
    only_linked: Option<&str>,
) -> ConsistencyMatrix {
    let modules: Vec<(String, PathBuf)> = match only_linked {
        Some(table) => find_specs_for_table(table),
        None => find_all_specs(),
    };
    let peers: Vec<(String, BTreeMap<String, String>)> = modules
        .iter()
        .filter(|(m, _)| m != target_module)
        .map(|(m, _)| (m.clone(), get_all_field_mappings(m)))
        .collect();
    let target_mapping = get_all_field_mappings(target_ns);

    // 所有出现过的列
    let mut all_columns: Vec<String> = vec![];
    for (_, m) in &peers {
        for col in m.keys() {
            if !all_columns.contains(col) {
                all_columns.push(col.clone());
            }
        }
    }
    for col in target_mapping.keys() {
        if !all_columns.contains(col) {
            all_columns.push(col.clone());
        }
    }
    all_columns.sort();

    let canonical = load_canonical_writability();
    let mut rows = Vec::new();
    let (mut consistent, mut auto_fix, mut conflicts) = (0, 0, 0);

    for col in &all_columns {
        let peer_values: Vec<&String> = peers.iter().filter_map(|(_, m)| m.get(col)).collect();
        let target_val = target_mapping
            .get(col)
            .cloned()
            .unwrap_or_else(|| "—（待填）".into());

        let action = if peer_values.is_empty() {
            "仅此模块引用，无对比".to_string()
        } else {
            let mut unique = peer_values.clone();
            unique.sort();
            unique.dedup();
            if unique.len() == 1 {
                let consensus = unique[0].as_str();
                if target_val == consensus {
                    consistent += 1;
                    "✅ 一致".to_string()
                } else if target_val == "—（待填）" || target_val == "—" {
                    auto_fix += 1;
                    format!("→ 自动订正为 {consensus}")
                } else {
                    conflicts += 1;
                    format!("❌ 冲突：已有共识 {consensus}，当前标记 {target_val}")
                }
            } else {
                conflicts += 1;
                let vals: Vec<&str> = unique.iter().map(|s| s.as_str()).collect();
                format!("⚠️ 冲突：已有模块不一致 {vals:?}")
            }
        };

        // 残留检测（覆盖 action）
        let action = if peer_values
            .iter()
            .any(|v| v.contains("移除") || v.contains("废弃"))
            && !matches!(target_val.as_str(), "—" | "—（待填）" | "—（已移除）")
        {
            conflicts += 1;
            "❌ 残留：此列已从 DDL 移除，应从 DTO 中删除".to_string()
        } else {
            action
        };

        rows.push(MatrixRow {
            column: col.clone(),
            canonical: canonical.get(col).cloned().unwrap_or_else(|| "—".into()),
            target: target_val,
            action,
        });
    }

    ConsistencyMatrix {
        rows,
        consistent,
        auto_fix,
        conflicts,
    }
}

// ── module.json 校准 ──────────────────────────────────

/// 更新 module.json 的 ontology.calibration 字段
pub fn update_calibration(module: &str, status: &str, note: &str) -> Result<Option<PathBuf>> {
    let module_json = find_module_json(module);
    let Some(path) = module_json else {
        return Ok(None);
    };
    let content = std::fs::read_to_string(&path)?;
    let mut config: serde_json::Value = serde_json::from_str(&content)?;
    let obj = config.as_object_mut().unwrap();
    let ont = obj
        .entry("ontology")
        .or_insert_with(|| serde_json::json!({}));
    ont.as_object_mut().unwrap().insert(
        "calibration".into(),
        serde_json::json!({
            "status": status,
            "lastAutoFix": "2026-07-17",
            "note": note,
        }),
    );
    std::fs::write(
        &path,
        format!("{}\n", serde_json::to_string_pretty(&config)?),
    )?;
    Ok(Some(path))
}

fn find_module_json(module: &str) -> Option<PathBuf> {
    let Ok(rd) = std::fs::read_dir("Pre-Proc") else {
        return None;
    };
    for ns in rd.flatten() {
        let candidate = ns
            .path()
            .join("Sources")
            .join("Modules")
            .join(module)
            .join("module.json");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// 为所有有 ONTOLOGY_SPEC.md 但无 calibration 的模块初始化
pub fn init_calibration_all() -> Vec<(String, Option<PathBuf>)> {
    let mut out = vec![];
    for (module, _) in find_all_specs() {
        let Some(path) = find_module_json(&module) else {
            continue;
        };
        let needs_init = std::fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
            .map(|cfg| {
                cfg.get("ontology")
                    .and_then(|o| o.get("calibration"))
                    .is_none()
            })
            .unwrap_or(false);
        if needs_init {
            let written = update_calibration(&module, "uncalibrated", "初始状态，尚未校准")
                .ok()
                .flatten();
            out.push((module, written));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_load_canonical_writability_from() {
        let dir = std::env::temp_dir().join(format!("contracts-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let spec = dir.join("DTO_DESIGN_SPEC.md");
        let mut f = std::fs::File::create(&spec).unwrap();
        writeln!(f, "## 五、其他").unwrap();
        writeln!(f, "## 六、字段映射可写性速查表").unwrap();
        writeln!(f, "| 列 | 可写性 | 说明 |").unwrap();
        writeln!(f, "|----|--------|------|").unwrap();
        writeln!(f, "| `notice` | ✅ | 用户可写 |").unwrap();
        writeln!(f, "| `id` | 🔒 | 系统生成 |").unwrap();
        writeln!(f, "| `qk_amount/qk_total` | ⚠️ | 标量引用 |").unwrap();
        writeln!(f, "## 七、后续章节").unwrap();
        drop(f);

        let map = load_canonical_writability_from(&spec);
        assert_eq!(map.get("notice").map(|s| s.as_str()), Some("✅"));
        assert_eq!(map.get("id").map(|s| s.as_str()), Some("🔒"));
        assert_eq!(map.get("qk_amount").map(|s| s.as_str()), Some("⚠")); // 与 Python 一致：仅基础码点，不含 FE0F
        assert!(map.get("other").is_none());
        std::fs::remove_dir_all(&dir).ok();
    }
}
