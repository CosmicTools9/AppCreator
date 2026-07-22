//! ontology-gen-bridge CLI
//!
//! Reads MappingOutput JSON, generates Service backend code.
//! Uses the same `generate_service` library function as AppAgent.

use ontology_gen_bridge::generate_service;
use std::fs;
use std::path::{Component, Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let input = parse(&args, "--input").ok_or("Missing --input <path>")?;
    let output = parse(&args, "--output").unwrap_or_else(|| "./backend".to_string());
    let name = parse(&args, "--name").unwrap_or_else(|| "generated".to_string());

    // 1. Read MappingOutput JSON
    let json = fs::read_to_string(&input)?;
    let mapping: ontology_mapping::output::MappingOutput =
        serde_json::from_str(&json)?;

    // 2. Write generated files
    let out_dir = PathBuf::from(&output);
    let module = ontology_gen_bridge::adapter::mapping_output_to_meta_module(&mapping, &name)?;
    let generator = alioth_gen::generator::module::ModuleApiGenerator::new();
    let mut generated = generator.generate(&module)?;

    // 3. Fix Cargo.toml dependency paths relative to output location
    if let Some(cargo_idx) = generated.files.iter().position(|f| f.path == Path::new("Cargo.toml")) {
        if let Some(ws) = find_workspace_root(&out_dir) {
            let abs_out = out_dir.canonicalize().ok().filter(|p| p.is_absolute()).unwrap_or_else(|| out_dir.clone());
            let rel = relative_path(&abs_out, &ws);
            generated.files[cargo_idx].content = generated.files[cargo_idx]
                .content
                .replace("../../../Framework", &format!("{}/Framework", rel));
        }
    }

    // 4. Validate paths + write
    for file in &generated.files {
        let target = safe_join(&out_dir, &file.path)?;
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&target, &file.content)?;
        println!("  wrote {}", target.display());
    }

    println!("Generated {} files to {}", generated.files.len(), output);
    Ok(())
}

fn parse(args: &[String], flag: &str) -> Option<String> {
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1).cloned())
}

fn find_workspace_root(from: &Path) -> Option<PathBuf> {
    let mut cur = from.to_path_buf();
    loop {
        let cargo = cur.join("Cargo.toml");
        if cargo.exists() {
            if let Ok(c) = fs::read_to_string(&cargo) { if c.contains("[workspace]") { return Some(cur); } }
        }
        if !cur.pop() { return None; }
    }
}

fn relative_path(from: &Path, to: &Path) -> String {
    let f: Vec<_> = from.components().collect();
    let t: Vec<_> = to.components().collect();
    let common = f.iter().zip(&t).take_while(|(a, b)| a == b).count();
    let mut r = PathBuf::new();
    for _ in common..f.len() { r.push(".."); }
    for c in t.iter().skip(common) { r.push(c.as_os_str()); }
    let s = r.to_string_lossy().to_string();
    if s.is_empty() { ".".into() } else { s }
}

fn safe_join(base: &Path, rel: &Path) -> Result<PathBuf, String> {
    for c in rel.components() {
        match c {
            Component::Normal(_) | Component::CurDir => {}
            _ => return Err(format!("unsafe component {:?} in {}", c, rel.display())),
        }
    }
    Ok(base.join(rel))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_relative_simple() {
        assert_eq!(relative_path(Path::new("/a/b/c"), Path::new("/a/b/d")), "../d");
    }
    #[test] fn test_safe_join_ok() {
        assert!(safe_join(Path::new("/tmp"), Path::new("src/lib.rs")).is_ok());
    }
    #[test] fn test_safe_join_traversal() {
        assert!(safe_join(Path::new("/tmp"), Path::new("../esc")).is_err());
    }
}
