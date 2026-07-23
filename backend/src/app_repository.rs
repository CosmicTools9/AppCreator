use serde_json::Value;
use std::path::PathBuf;

pub fn apps_dir(namespace: &str) -> PathBuf {
    let root = std::env::var("APPCREATOR_PROJECT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
    root.join("Pre-Proc").join(namespace).join("Apps")
}

pub fn find_app_by_code(
    apps_root: &PathBuf,
    namespace: &str,
    code: &str,
) -> Option<(PathBuf, Value)> {
    let dirs = std::fs::read_dir(apps_root).ok()?;
    for e in dirs.flatten() {
        let p = e.path();
        if !p.is_dir() {
            continue;
        }
        let j = p.join("app.json");
        if !j.exists() {
            continue;
        }
        let content = match std::fs::read_to_string(&j) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let val: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if val.get("code").and_then(|c| c.as_str()) != Some(code) {
            continue;
        }
        if let Some(ns_f) = val.get("namespace").and_then(|n| n.as_str()) {
            if !ns_f.is_empty() && ns_f != namespace {
                continue;
            }
        }
        let canonical = p.canonicalize().ok()?;
        let root_canon = apps_root.canonicalize().ok()?;
        if canonical.starts_with(&root_canon) {
            return Some((canonical, val));
        }
    }
    None
}

pub fn app_id_from_json(val: &Value) -> Option<i64> {
    val.get("id").and_then(|v| match v {
        Value::String(s) => s.parse::<i64>().ok(),
        Value::Number(n) => n.as_i64(),
        _ => None,
    })
}

pub fn list_apps(namespace: &str) -> Vec<Value> {
    list_apps_from_dir(&apps_dir(namespace), namespace)
}

pub fn list_apps_from_dir(dir: &PathBuf, namespace: &str) -> Vec<Value> {
    if !dir.exists() {
        return vec![];
    }
    let mut apps = vec![];
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            if !e.path().is_dir() {
                continue;
            }
            let j = e.path().join("app.json");
            if let Ok(c) = std::fs::read_to_string(&j) {
                if let Ok(v) = serde_json::from_str::<Value>(&c) {
                    if let Some(ns_f) = v.get("namespace").and_then(|n| n.as_str()) {
                        if !ns_f.is_empty() && ns_f != namespace {
                            continue;
                        }
                    }
                    apps.push(v);
                }
            }
        }
    }
    apps
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    fn d() -> PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("_ar_{ts}"))
    }
    #[test]
    fn bad_json_skipped() {
        let t = d();
        let _ = fs::remove_dir_all(&t);
        fs::create_dir_all(t.join("00-bad")).unwrap();
        fs::create_dir_all(t.join("10-good")).unwrap();
        fs::write(t.join("00-bad/app.json"), "not-json").unwrap();
        fs::write(
            t.join("10-good/app.json"),
            r#"{"code":"x","namespace":"ns"}"#,
        )
        .unwrap();
        assert!(find_app_by_code(&t, "ns", "x").is_some());
        let _ = fs::remove_dir_all(&t);
    }
    #[test]
    fn ns_mismatch_rejected() {
        let t = d();
        let _ = fs::remove_dir_all(&t);
        fs::create_dir_all(t.join("a")).unwrap();
        fs::write(t.join("a/app.json"), r#"{"code":"x","namespace":"ns1"}"#).unwrap();
        assert!(find_app_by_code(&t, "ns2", "x").is_none());
        let _ = fs::remove_dir_all(&t);
    }
    #[test]
    fn traversal_rejected() {
        let t = d();
        let _ = fs::remove_dir_all(&t);
        fs::create_dir_all(t.join("safe")).unwrap();
        fs::write(t.join("safe/app.json"), r#"{"code":"s","namespace":"ns"}"#).unwrap();
        assert!(find_app_by_code(&t, "ns", "../x").is_none());
        assert!(find_app_by_code(&t, "ns", "s").is_some());
        let _ = fs::remove_dir_all(&t);
    }
    #[test]
    fn list_filters_ns() {
        let t = d();
        let _ = fs::remove_dir_all(&t);
        fs::create_dir_all(t.join("a")).unwrap();
        fs::create_dir_all(t.join("b")).unwrap();
        fs::write(t.join("a/app.json"), r#"{"code":"a","namespace":"NS-A"}"#).unwrap();
        fs::write(t.join("b/app.json"), r#"{"code":"b","namespace":"NS-B"}"#).unwrap();
        assert_eq!(list_apps_from_dir(&t, "NS-A").len(), 1);
        let _ = fs::remove_dir_all(&t);
    }
    #[test]
    fn id_from_string() {
        let v: Value = serde_json::from_str(r#"{"id":"9876543210"}"#).unwrap();
        assert_eq!(app_id_from_json(&v), Some(9876543210));
    }
    #[test]
    fn id_from_number() {
        let v: Value = serde_json::from_str(r#"{"id":42}"#).unwrap();
        assert_eq!(app_id_from_json(&v), Some(42));
    }
    #[test]
    fn id_absent() {
        let v: Value = serde_json::from_str(r#"{"code":"x"}"#).unwrap();
        assert_eq!(app_id_from_json(&v), None);
    }
}
