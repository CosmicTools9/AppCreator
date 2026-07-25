//! CLI Install Command - Module Installation from Registry
//!
//! Provides functionality to install modules from the AliothStudio registry,
//! including dependency resolution and lifecycle hooks.

use crate::cli::CliError;
use crate::cli::InstallArgs;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

/// Module information from registry
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ModuleResponse {
    #[serde(with = "common::serde_zuid")]
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub latest_version: Option<String>,
    pub is_public: bool,
    pub metadata: Option<serde_json::Value>,
}

/// Version information from registry
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct VersionResponse {
    #[serde(with = "common::serde_zuid")]
    pub id: i64,
    pub module_id: i64,
    pub version: String,
pub : Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub is_stable: bool,
    pub downloads: i64,
}

/// Dependency specification
#[derive(Debug, Clone, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
}

/// Lifecycle hooks configuration
#[derive(Debug, Clone, Deserialize)]
pub struct LifecycleHooks {
    pub pre_install: Option<String>,
    pub post_install: Option<String>,
}

/// Resolved module with its version
#[derive(Debug, Clone)]
pub struct ResolvedModule {
    pub module: ModuleResponse,
    pub version: VersionResponse,
}

/// Check if a version satisfies a semver constraint
fn satisfies_constraint(version: &str, constraint: &str) -> bool {
    // Parse version: major.minor.patch
    let version_re = Regex::new(r"^(\d+)\.(\d+)\.(\d+)").unwrap();
    let version_caps = match version_re.captures(version) {
        Some(c) => c,
        None => return false,
    };

    let v_major: u32 = version_caps.get(1).unwrap().as_str().parse().unwrap_or(0);
    let v_minor: u32 = version_caps.get(2).unwrap().as_str().parse().unwrap_or(0);
    let v_patch: u32 = version_caps.get(3).unwrap().as_str().parse().unwrap_or(0);

    // Parse constraint types
    if let Some(constraint_version) = constraint.strip_prefix(">=") {
        // Greater than or equal
        if let Some(caps) = version_re.captures(constraint_version) {
            let c_major: u32 = caps.get(1).unwrap().as_str().parse().unwrap_or(0);
            let c_minor: u32 = caps.get(2).unwrap().as_str().parse().unwrap_or(0);
            let c_patch: u32 = caps.get(3).unwrap().as_str().parse().unwrap_or(0);

            if v_major > c_major {
                return true;
            } else if v_major == c_major {
                if v_minor > c_minor {
                    return true;
                } else if v_minor == c_minor {
                    return v_patch >= c_patch;
                }
            }
            return false;
        }
    } else if let Some(constraint_version) = constraint.strip_prefix("^") {
        // Compatible version (2.x.x for ^2.0.0)
        if let Some(caps) = version_re.captures(constraint_version) {
            let c_major: u32 = caps.get(1).unwrap().as_str().parse().unwrap_or(0);
            let c_minor: u32 = caps.get(2).unwrap().as_str().parse().unwrap_or(0);
            let c_patch: u32 = caps.get(3).unwrap().as_str().parse().unwrap_or(0);

            // ^ means compatible - same major, minor can be >=, patch can be >=
            if v_major == c_major {
                if v_minor > c_minor {
                    return true;
                } else if v_minor == c_minor {
                    return v_patch >= c_patch;
                }
            }
            return false;
        }
    } else if let Some(constraint_version) = constraint.strip_prefix("~") {
        // Patch compatible (1.x.x for ~1.0.0)
        if let Some(caps) = version_re.captures(constraint_version) {
            let c_major: u32 = caps.get(1).unwrap().as_str().parse().unwrap_or(0);
            let c_minor: u32 = caps.get(2).unwrap().as_str().parse().unwrap_or(0);
            let c_patch: u32 = caps.get(3).unwrap().as_str().parse().unwrap_or(0);

            // ~ means patch-compatible - same major.minor, patch can be >=
            if v_major == c_major && v_minor == c_minor {
                return v_patch >= c_patch;
            }
            return false;
        }
    } else if let Some(rest) = constraint.strip_prefix("=") {
        // Exact version (skip the =)
        return satisfies_constraint(version, rest.trim());
    } else {
        // Assume exact version match
        return version == constraint;
    }

    false
}

/// Fetch module by name from registry
fn fetch_module_by_name(
    client: &reqwest::blocking::Client,
    registry_url: &str,
    module_name: &str,
) -> Result<ModuleResponse, CliError> {
    let url = format!("{}/module/name/{}", registry_url, module_name);

    let response = client
        .get(&url)
        .timeout(Duration::from_secs(30))
        .send()
        .map_err(|e| CliError::Registry(format!("Failed to fetch module: {}", e)))?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(CliError::Registry(format!(
            "Module '{}' not found in registry",
            module_name
        )));
    }

    if !response.status().is_success() {
        return Err(CliError::Registry(format!(
            "Failed to fetch module (status {}): {}",
            response.status(),
            response.text().unwrap_or_default()
        )));
    }

    response
        .json()
        .map_err(|e| CliError::Registry(format!("Failed to parse module response: {}", e)))
}

/// Fetch a specific version of a module
fn fetch_version_by_string(
    client: &reqwest::blocking::Client,
    registry_url: &str,
    module_name: &str,
    version_str: &str,
) -> Result<VersionResponse, CliError> {
    let url = format!(
        "{}/version/by-name/{}/{}",
        registry_url, module_name, version_str
    );

    let response = client
        .get(&url)
        .timeout(Duration::from_secs(30))
        .send()
        .map_err(|e| CliError::Registry(format!("Failed to fetch version: {}", e)))?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(CliError::Registry(format!(
            "Version '{}' not found for module '{}'",
            version_str, module_name
        )));
    }

    if !response.status().is_success() {
        return Err(CliError::Registry(format!(
            "Failed to fetch version (status {}): {}",
            response.status(),
            response.text().unwrap_or_default()
        )));
    }

    response
        .json()
        .map_err(|e| CliError::Registry(format!("Failed to parse version response: {}", e)))
}

/// List versions for a module
fn fetch_versions(
    client: &reqwest::blocking::Client,
    registry_url: &str,
    module_id: i64,
) -> Result<Vec<VersionResponse>, CliError> {
    let url = format!("{}/version/{}", registry_url, module_id);

    #[derive(Deserialize)]
    struct VersionListResponse {
        pub items: Vec<VersionResponse>,
    }

    let response = client
        .get(&url)
        .timeout(Duration::from_secs(30))
        .send()
        .map_err(|e| CliError::Registry(format!("Failed to fetch versions: {}", e)))?;

    if !response.status().is_success() {
        return Err(CliError::Registry(format!(
            "Failed to fetch versions (status {}): {}",
            response.status(),
            response.text().unwrap_or_default()
        )));
    }

    let list_response: VersionListResponse = response
        .json()
        .map_err(|e| CliError::Registry(format!("Failed to parse versions response: {}", e)))?;

    Ok(list_response.items)
}

/// Get latest version for a module
fn get_latest_version(
    client: &reqwest::blocking::Client,
    registry_url: &str,
    module: &ModuleResponse,
) -> Result<VersionResponse, CliError> {
    // First try to use latest_version field if available
    if let Some(ref latest) = module.latest_version {
        if let Ok(version) = fetch_version_by_string(client, registry_url, &module.name, latest) {
            return Ok(version);
        }
    }

    // Otherwise fetch all versions and find the latest
    let versions = fetch_versions(client, registry_url, module.id)?;

    versions
        .into_iter()
        .max_by(|a, b| {
            // Compare versions semantically
            let a_parts = parse_version(&a.version);
            let b_parts = parse_version(&b.version);
            a_parts.cmp(&b_parts)
        })
        .ok_or_else(|| CliError::Registry("No versions found for module".to_string()))
}

fn parse_version(version: &str) -> (u32, u32, u32) {
    let re = Regex::new(r"^(\d+)\.(\d+)\.(\d+)").unwrap();
    if let Some(caps) = re.captures(version) {
        (
            caps.get(1).unwrap().as_str().parse().unwrap_or(0),
            caps.get(2).unwrap().as_str().parse().unwrap_or(0),
            caps.get(3).unwrap().as_str().parse().unwrap_or(0),
        )
    } else {
        (0, 0, 0)
    }
}

/// Extract dependencies from version metadata
fn extract_dependencies(metadata: &Option<serde_json::Value>) -> Vec<Dependency> {
    if let Some(serde_json::Value::Object(map)) = metadata {
        if let Some(deps) = map.get("dependencies") {
            if let Ok(deps_array) = serde_json::from_value::<Vec<Dependency>>(deps.clone()) {
                return deps_array;
            }
        }
    }
    Vec::new()
}

/// Extract lifecycle hooks from version metadata
fn extract_hooks(metadata: &Option<serde_json::Value>) -> LifecycleHooks {
    if let Some(serde_json::Value::Object(map)) = metadata {
        if let Some(hooks_val) = map.get("hooks") {
            if let Ok(hooks) = serde_json::from_value::<LifecycleHooks>(hooks_val.clone()) {
                return hooks;
            }
        }
    }
    LifecycleHooks {
        pre_install: None,
        post_install: None,
    }
}

/// Execute a lifecycle hook
fn execute_hook(hook_name: &str, command: &str, module_dir: &PathBuf) -> Result<(), CliError> {
    println!("   Running {} hook...", hook_name);

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(module_dir)
        .output()
        .map_err(|e| CliError::Io(format!("Failed to execute {} hook: {}", hook_name, e)))?;

    let log_path = module_dir.join("hooks.log");
    let log_entry = format!(
        "[{}] {}: exit={}\nstdout:\n{}\nstderr:\n{}\n",
        hook_name,
        command,
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .and_then(|mut f| f.write_all(log_entry.as_bytes()))
        .map_err(|e| CliError::Io(format!("Failed to write hook log: {}", e)))?;

    if !output.status.success() {
        println!(
            "   ⚠️  {} hook exited with non-zero status (may be expected)",
            hook_name
        );
    }

    Ok(())
}

/// Resolve dependencies for a module
fn resolve_dependencies(
    client: &reqwest::blocking::Client,
    registry_url: &str,
    module: &ModuleResponse,
    version: &VersionResponse,
    visited: &mut HashSet<String>,
    all_modules: &mut Vec<ResolvedModule>,
) -> Result<(), CliError> {
    let key = format!("{}:{}", module.name, version.version);
    if visited.contains(&key) {
        return Ok(()); // Already resolved
    }

    let dependencies = extract_dependencies(&version.metadata);
    if dependencies.is_empty() {
        visited.insert(key);
        all_modules.push(ResolvedModule {
            module: module.clone(),
            version: version.clone(),
        });
        return Ok(());
    }

    // Resolve each dependency
    for dep in dependencies {
        if visited.contains(&format!("{}:{}", dep.name, dep.version)) {
            continue;
        }

        println!("   Resolving dependency: {} ({})", dep.name, dep.version);

        // Fetch dependency module
        let dep_module = fetch_module_by_name(client, registry_url, &dep.name)?;

        // Find version that satisfies constraint
        let versions = fetch_versions(client, registry_url, dep_module.id)?;
        let dep_version = versions
            .into_iter()
            .find(|v| satisfies_constraint(&v.version, &dep.version))
            .ok_or_else(|| {
                CliError::Registry(format!(
                    "No version of '{}' satisfies constraint '{}'",
                    dep.name, dep.version
                ))
            })?;

        // Recursively resolve
        resolve_dependencies(
            client,
            registry_url,
            &dep_module,
            &dep_version,
            visited,
            all_modules,
        )?;
    }

    visited.insert(key);
    all_modules.push(ResolvedModule {
        module: module.clone(),
        version: version.clone(),
    });

    Ok(())
}

/// Get the local cache directory for installed modules
fn get_cache_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("alioth")
        .join("registry")
        .join("cache")
}

/// Check if a module is already installed
fn is_module_installed(module_name: &str, version: &str) -> bool {
    let cache_dir = get_cache_dir();
    let module_dir = cache_dir.join(module_name).join(version);
    module_dir.join("module.json").exists()
}

/// Run the install command
pub fn run_install(args: InstallArgs) -> Result<(), CliError> {
    println!("📦 Installing module: {}", args.module_name);
    if let Some(ref version) = args.version {
        println!("   Version: {}", version);
    }
    println!("   Registry: {}", args.registry_url());

    // Create HTTP client
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| CliError::Io(format!("Failed to create HTTP client: {}", e)))?;

    // Fetch module
    println!("\n🔍 Looking up module '{}'...", args.module_name);
    let module = fetch_module_by_name(&client, &args.registry_url(), &args.module_name)?;
    println!("   Found: {} (ID: {})", module.name, module.id);

    // Determine version to install
    let version = if let Some(ref version_str) = args.version {
        fetch_version_by_string(
            &client,
            &args.registry_url(),
            &args.module_name,
            version_str,
        )?
    } else {
        get_latest_version(&client, &args.registry_url(), &module)?
    };
    println!(
        "   Version: {} ({} downloads)",
        version.version, version.downloads
    );

    // Resolve dependencies if not skipped
    let mut resolved_modules = Vec::new();
    if !args.no_deps {
        println!("\n📋 Resolving dependencies...");
        let mut visited = HashSet::new();
        resolve_dependencies(
            &client,
            &args.registry_url(),
            &module,
            &version,
            &mut visited,
            &mut resolved_modules,
        )?;
        println!("   Total modules to install: {}", resolved_modules.len());
    } else {
        resolved_modules.push(ResolvedModule { module, version });
    }

    // Show dependency tree in dry-run mode
    if args.dry_run {
        println!("\n📦 Would install:");
        for (i, rm) in resolved_modules.iter().enumerate() {
            let deps = extract_dependencies(&rm.version.metadata);
            if deps.is_empty() {
                println!("   {}. {}@{}", i + 1, rm.module.name, rm.version.version);
            } else {
                println!(
                    "   {}. {}@{} (depends on {})",
                    i + 1,
                    rm.module.name,
                    rm.version.version,
                    deps.iter()
                        .map(|d| format!("{} ({})", d.name, d.version))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
        return Ok(());
    }

    // Create cache directory
    let cache_dir = get_cache_dir();
    fs::create_dir_all(&cache_dir)
        .map_err(|e| CliError::Io(format!("Failed to create cache directory: {}", e)))?;

    // Install each module (leaf-first order from resolve_dependencies)
    println!("\n📥 Installing modules...");
    for rm in &resolved_modules {
        let is_already_installed = is_module_installed(&rm.module.name, &rm.version.version);

        if is_already_installed && !args.force {
            println!(
                "   ✓ {}@{} (already installed, skipping)",
                rm.module.name, rm.version.version
            );
            continue;
        }

        let module_dir = cache_dir.join(&rm.module.name).join(&rm.version.version);

        if is_already_installed && args.force {
            println!(
                "   🔄 {}@{} (reinstalling...)",
                rm.module.name, rm.version.version
            );
        } else {
            println!("   📦 {}@{}", rm.module.name, rm.version.version);
        }

        // Create module directory
        fs::create_dir_all(&module_dir)
            .map_err(|e| CliError::Io(format!("Failed to create module directory: {}", e)))?;

        // Extract hooks
        let hooks = extract_hooks(&rm.version.metadata);

        // Execute pre_install hook
        if let Some(ref pre_hook) = hooks.pre_install {
            execute_hook("pre_install", pre_hook, &module_dir)?;
        }

        // Write module metadata
        let metadata_path = module_dir.join("module.json");
        let metadata_json = serde_json::json!({
            "id": rm.module.id,
            "name": rm.module.name,
            "version": rm.version.version,
            "description": rm.module.description,
            "metadata": rm.version.metadata,
            "installed_at": chrono::Utc::now().to_rfc3339(),
        });
        fs::write(
            &metadata_path,
            serde_json::to_string_pretty(&metadata_json).unwrap(),
        )
        .map_err(|e| CliError::Io(format!("Failed to write module metadata: {}", e)))?;

        // Execute post_install hook
        if let Some(ref post_hook) = hooks.post_install {
            execute_hook("post_install", post_hook, &module_dir)?;
        }

        println!("   ✓ {}@{} installed", rm.module.name, rm.version.version);
    }

    println!("\n✅ Installation complete!");
    println!("   Installed {} module(s)", resolved_modules.len());
    println!("   Cache location: {}", cache_dir.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_satisfies_constraint_exact() {
        assert!(satisfies_constraint("1.0.0", "1.0.0"));
        assert!(!satisfies_constraint("1.0.0", "2.0.0"));
    }

    #[test]
    fn test_satisfies_constraint_gte() {
        assert!(satisfies_constraint("1.0.0", ">=1.0.0"));
        assert!(satisfies_constraint("2.0.0", ">=1.0.0"));
        assert!(!satisfies_constraint("0.9.0", ">=1.0.0"));
    }

    #[test]
    fn test_satisfies_constraint_caret() {
        assert!(satisfies_constraint("1.0.0", "^1.0.0"));
        assert!(satisfies_constraint("1.9.9", "^1.0.0"));
        assert!(!satisfies_constraint("2.0.0", "^1.0.0"));
    }

    #[test]
    fn test_satisfies_constraint_tilde() {
        assert!(satisfies_constraint("1.0.0", "~1.0.0"));
        assert!(satisfies_constraint("1.0.5", "~1.0.0"));
        assert!(!satisfies_constraint("1.1.0", "~1.0.0"));
    }

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("1.2.3"), (1, 2, 3));
        assert_eq!(parse_version("10.20.30"), (10, 20, 30));
        assert_eq!(parse_version("invalid"), (0, 0, 0));
    }
}
