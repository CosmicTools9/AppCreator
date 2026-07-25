//! CLI Tool for MetaModel Code Generation
//!
//! Provides command-line interface for generating code from DSL files.
//!
//! ## Usage
//!
//! ```bash
//! # Generate all artifacts
//! alioth generate --input model.json --output ./generated
//!
//! # Generate specific generators only
//! alioth generate --input model.json --generators zod,api,frontend
//!
//! # Preview changes
//! alioth preview --input model.json --diff
//!
//! # Batch generation
//! alioth batch --models 1,2,3 --parallel
//!
//! # List history
//! alioth history --model 1 --limit 20
//!
//! # Rollback
//! alioth rollback --history 123
//!
//! # Using config file
//! alioth generate --config meta-model.toml
//! ```

mod commands;
mod config;
mod install;

#[cfg(test)]
mod tests;

pub use commands::{
    BatchArgs, BuildArgs, Cli, Commands, ExportArgs, ExportFormat, GenerateArgs, GeneratorType,
    HistoryArgs, HistoryOutputFormat, InitArgs, InitModuleArgs, InstallArgs, PreviewArgs,
    PublishArgs, RollbackArgs, ValidateArgs,
};
pub use config::CliConfig;

use crate::api::generate::preview::{PreviewRequest, PreviewService};
use crate::generator::ir::GeneratorModel;
use crate::generator::{ApiGenerator, FrontendComponentGenerator, Generator, ZodSchemaGenerator};
use crate::metrics::{init_metrics, GenerationTimer, GenerationType};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// CLI runner
pub struct CliRunner;

impl CliRunner {
    /// Run CLI with args
    pub fn run() -> Result<(), CliError> {
        use clap::Parser;

        let cli = Cli::parse();

        match cli.command {
            Commands::Generate(args) => Self::run_generate(args),
            Commands::Validate(args) => Self::run_validate(args),
            Commands::Init(args) => Self::run_init(args),
            Commands::Export(args) => Self::run_export(args),
            Commands::Preview(args) => Self::run_preview(args),
            Commands::Batch(args) => Self::run_batch(args),
            Commands::History(args) => Self::run_history(args),
            Commands::Rollback(args) => Self::run_rollback(args),
            Commands::Publish(args) => Self::run_publish(args),
            Commands::Install(args) => Self::run_install(args),
            Commands::InitModule(args) => Self::run_init_module(args),
            Commands::Build(args) => Self::run_build(args),
        }
    }

    /// Run generate command
    fn run_generate(args: GenerateArgs) -> Result<(), CliError> {
        // Initialize metrics
        init_metrics();
        let _total_timer = GenerationTimer::new(GenerationType::Full);

        // Load config if provided
        let _config = if let Some(config_path) = &args.config {
            CliConfig::from_file(config_path)?
        } else {
            CliConfig::default()
        };

        // Determine JSON content source
        let json_content = if let Some(input_path) = &args.input {
            fs::read_to_string(input_path).map_err(|e| {
                CliError::Io(format!("Failed to read {}: {}", input_path.display(), e))
            })?
        } else if let Some(ontology_id) = args.from_ontology {
            println!("Fetching ontology {} from database...", ontology_id);
            match Self::fetch_model_from_db(ontology_id) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Database fetch failed: {}", e);
                    eprintln!("Hint: Ensure DATABASE_URL is set and the ontology ID exists.");
                    return Err(CliError::Io(format!("DB fetch failed: {}", e)));
                }
            }
        } else {
            return Err(CliError::InvalidArgs(
                "Either --input or --from-ontology must be provided".to_string(),
            ));
        };

        // Parse JSON to GeneratorModel
        let model: GeneratorModel = serde_json::from_str(&json_content)
            .map_err(|e| CliError::Parse(format!("JSON parse failed: {}", e)))?;

        // Determine which generators to run
        let generators = args.generators().unwrap_or_else(|| vec!["all".to_string()]);

        // Determine output path: Pre-Proc/Apps/{app-name}/ or fallback to default
        let (output_base, backend_dir, frontend_dir) = if let Some(app_name) = &args.app_name {
            // Try to find project root by looking for .git or AGENTS.md
            let project_root = std::env::current_dir()
                .ok()
                .and_then(|cwd| {
                    // Check if we're in Meta/backend
                    if cwd.ends_with("Meta/backend") {
                        cwd.parent().map(|p| {
                            p.parent()
                                .map(|pp| pp.to_path_buf())
                                .unwrap_or(p.to_path_buf())
                        })
                    } else {
                        Some(cwd)
                    }
                })
                .unwrap_or_else(|| PathBuf::from("."));

            let base = project_root.join("Pre-Proc/Apps");
            let app_dir = base.join(app_name);
            let backend = app_dir.join("backend");
            let frontend = app_dir.join("frontend");
            // Create backend and frontend subdirectories
            fs::create_dir_all(&backend)
                .map_err(|e| CliError::Io(format!("Failed to create backend dir: {}", e)))?;
            fs::create_dir_all(&frontend)
                .map_err(|e| CliError::Io(format!("Failed to create frontend dir: {}", e)))?;
            fs::create_dir_all(app_dir.join("config"))
                .map_err(|e| CliError::Io(format!("Failed to create config dir: {}", e)))?;
            (app_dir.clone(), backend, frontend)
        } else {
            let base = args.output.clone();
            (base.clone(), base.clone(), base.clone())
        };

        // Create output directory
        fs::create_dir_all(&output_base)
            .map_err(|e| CliError::Io(format!("Failed to create output dir: {}", e)))?;

        // Build sub-output paths for backend/frontend
        let run_all = generators.contains(&"all".to_string());

        if run_all || generators.iter().any(|g| g == "zod" || g == "typescript") {
            let timer = GenerationTimer::new(GenerationType::Full);
            let zod_gen = ZodSchemaGenerator;
            let output = zod_gen
                .generate(&model)
                .map_err(|e| CliError::Generate(format!("Zod: {}", e)))?;
            timer.finish(model.entities.len(), output.files.len(), 0);
            if !args.dry_run {
                Self::write_output(&backend_dir, "zod", &output)?;
            }
            println!("✓ Generated Zod schemas ({} files)", output.files.len());
        }

        if run_all || generators.iter().any(|g| g == "api") {
            let timer = GenerationTimer::new(GenerationType::Full);
            let api_gen = ApiGenerator::new();
            let all = api_gen
                .generate_all(&model)
                .map_err(|e| CliError::Generate(format!("API: {}", e)))?;

            timer.finish(
                model.entities.len(),
                all.handlers.files.len() + all.client.files.len(),
                0,
            );
            if !args.dry_run {
                // Write OpenAPI spec
                let openapi_path = backend_dir.join("openapi.json");
                fs::write(&openapi_path, &all.openapi)
                    .map_err(|e| CliError::Io(format!("Failed to write openapi.json: {}", e)))?;

                // Write handlers
                Self::write_output(&backend_dir, "handlers", &all.handlers)?;

                // Write client
                Self::write_output(&backend_dir, "client", &all.client)?;

                // 编译验证门禁
                if backend_dir.join("Cargo.toml").exists() {
                    println!("🔍 Running cargo check on generated backend...");
                    let status = std::process::Command::new("cargo")
                        .arg("check")
                        .current_dir(&backend_dir)
                        .status()
                        .map_err(|e| CliError::Io(format!("Failed to run cargo check: {}", e)))?;
                    if !status.success() {
                        return Err(CliError::Io(
                            "Generated code failed cargo check. Fix the generator or the model and retry.".to_string()
                        ));
                    }
                    println!("✅ cargo check passed");
                } else {
                    println!("⚠️  Skipping cargo check: no Cargo.toml found in backend dir");
                }
            }

            println!("✓ Generated API (OpenAPI + handlers + client)");
        }

        if run_all
            || generators
                .iter()
                .any(|g| g == "frontend" || g == "components")
        {
            let timer = GenerationTimer::new(GenerationType::Full);
            let frontend_gen = FrontendComponentGenerator::new();
            let output = frontend_gen
                .generate(&model)
                .map_err(|e| CliError::Generate(format!("Frontend: {}", e)))?;
            timer.finish(model.entities.len(), output.files.len(), 0);
            if !args.dry_run {
                Self::write_output(&frontend_dir, "frontend", &output)?;
            }
            println!(
                "✓ Generated frontend components ({} files)",
                output.files.len()
            );
        }

        if run_all
            || generators
                .iter()
                .any(|g| g == "docs" || g == "documentation")
        {
            let timer = GenerationTimer::new(GenerationType::Full);
            let doc_gen = crate::docgen::DocGenerator::new();
            let output = doc_gen
                .generate(&model)
                .map_err(|e| CliError::Generate(format!("Docs: {}", e)))?;
            timer.finish(model.entities.len(), output.files.len(), 0);
            if !args.dry_run {
                Self::write_output(&output_base, "docs", &output)?;
            }
            println!("✓ Generated documentation ({} files)", output.files.len());
        }

        println!(
            "\n✅ Generation complete! Output: {}",
            output_base.display()
        );
        Ok(())
    }

    /// Run validate command
    fn run_validate(args: ValidateArgs) -> Result<(), CliError> {
        let json_content = fs::read_to_string(&args.input)
            .map_err(|e| CliError::Io(format!("Failed to read {}: {}", args.input.display(), e)))?;

        match serde_json::from_str::<GeneratorModel>(&json_content) {
            Ok(_) => {
                println!("✅ JSON is valid!");
                Ok(())
            }
            Err(e) => {
                println!("❌ Validation failed: {}", e);
                Err(CliError::Parse(e.to_string()))
            }
        }
    }

    /// Run init command
    fn run_init(args: InitArgs) -> Result<(), CliError> {
        let config = CliConfig::default();
        let toml = toml::to_string_pretty(&config)
            .map_err(|e| CliError::Config(format!("Failed to serialize config: {}", e)))?;

        let config_path = args.dir.join("meta-model.toml");
        fs::write(&config_path, toml)
            .map_err(|e| CliError::Io(format!("Failed to write config: {}", e)))?;

        // Create example JSON model
        let example_json = r#"{
  "entities": [
    {
      "name": {
        "raw": "User",
        "snake": "user",
        "camel": "user",
        "pascal": "User",
        "kebab": "user",
        "screaming_snake": "USER",
        "plural_snake": "users",
        "plural_pascal": "Users",
        "plural_kebab": "users"
      },
      "description": null,
      "fields": [
        {
          "name": {
            "raw": "email",
            "snake": "email",
            "camel": "email",
            "pascal": "Email"
          },
          "field_type": "Text",
          "description": null,
          "nullable": false,
          "unique": true,
          "indexed": false,
          "default_value": null,
          "validations": [],
          "annotations": []
        },
        {
          "name": {
            "raw": "name",
            "snake": "name",
            "camel": "name",
            "pascal": "Name"
          },
          "field_type": "Text",
          "description": null,
          "nullable": false,
          "unique": false,
          "indexed": false,
          "default_value": null,
          "validations": [],
          "annotations": []
        }
      ],
      "relations": [],
      "annotations": []
    }
  ],
  "enums": [],
  "metadata": {
    "generated_at": "2024-01-01T00:00:00+00:00",
    "generator_version": "0.1.0"
  }
}"#;

        let example_path = args.dir.join("example.model.json");
        fs::write(&example_path, example_json)
            .map_err(|e| CliError::Io(format!("Failed to write example: {}", e)))?;

        println!(
            "✅ Initialized meta-model project in {}",
            args.dir.display()
        );
        println!("   Config: {}", config_path.display());
        println!("   Example: {}", example_path.display());

        Ok(())
    }

    /// Run export command
    fn run_export(args: ExportArgs) -> Result<(), CliError> {
        // Read input JSON
        let input_content = fs::read_to_string(&args.input)
            .map_err(|e| CliError::Io(format!("Failed to read {}: {}", args.input.display(), e)))?;

        // Validate and pretty-print JSON
        let model: GeneratorModel = serde_json::from_str(&input_content)
            .map_err(|e| CliError::Parse(format!("JSON parse failed: {}", e)))?;

        let output = if args.pretty {
            serde_json::to_string_pretty(&model)
        } else {
            serde_json::to_string(&model)
        }
        .map_err(|e| CliError::Generate(format!("JSON serialization failed: {}", e)))?;

        // Write output
        if let Some(output_path) = args.output {
            fs::write(&output_path, &output).map_err(|e| {
                CliError::Io(format!("Failed to write {}: {}", output_path.display(), e))
            })?;
            println!("✅ Exported to {}", output_path.display());
        } else {
            // Print to stdout
            println!("{}", output);
        }

        Ok(())
    }

    /// Run preview command
    fn run_preview(args: PreviewArgs) -> Result<(), CliError> {
        // Read input JSON
        let input_path = Path::new(&args.input);
        let json_content = fs::read_to_string(input_path)
            .map_err(|e| CliError::Io(format!("Failed to read {}: {}", args.input, e)))?;

        // Parse JSON to GeneratorModel
        let model: GeneratorModel = serde_json::from_str(&json_content)
            .map_err(|e| CliError::Parse(format!("JSON parse failed: {}", e)))?;

        // Generate output to memory
        let generators = args.generators().unwrap_or_else(|| vec!["all".to_string()]);
        let run_all = generators.contains(&"all".to_string());

        let mut all_files: Vec<crate::generator::GeneratedFile> = Vec::new();

        if run_all || generators.iter().any(|g| g == "zod" || g == "typescript") {
            let zod_gen = ZodSchemaGenerator;
            let output = zod_gen
                .generate(&model)
                .map_err(|e| CliError::Generate(format!("Zod: {}", e)))?;
            all_files.extend(output.files);
        }

        if run_all || generators.iter().any(|g| g == "api") {
            let api_gen = ApiGenerator::new();
            let all = api_gen
                .generate_all(&model)
                .map_err(|e| CliError::Generate(format!("API: {}", e)))?;
            all_files.extend(all.handlers.files);
            all_files.extend(all.client.files);
            // Add openapi.json as a generated file
            all_files.push(crate::generator::GeneratedFile {
                path: std::path::PathBuf::from("openapi.json"),
                content: all.openapi,
                checksum: String::new(),
            });
        }

        if run_all
            || generators
                .iter()
                .any(|g| g == "frontend" || g == "components")
        {
            let frontend_gen = FrontendComponentGenerator::new();
            let output = frontend_gen
                .generate(&model)
                .map_err(|e| CliError::Generate(format!("Frontend: {}", e)))?;
            all_files.extend(output.files);
        }

        if run_all
            || generators
                .iter()
                .any(|g| g == "docs" || g == "documentation")
        {
            let doc_gen = crate::docgen::DocGenerator::new();
            let output = doc_gen
                .generate(&model)
                .map_err(|e| CliError::Generate(format!("Docs: {}", e)))?;
            all_files.extend(output.files);
        }

        // Create generated output
        let generated_output = crate::generator::GeneratedOutput {
            files: all_files,
            metadata: crate::generator::GenerationMetadata {
                generator_name: "preview".to_string(),
                entity_count: model.entities.len(),
                c_file_count: 0,
            },
        };

        // Run preview service
        let preview_service = PreviewService::new();
        let request = PreviewRequest {
            output_dir: args.output.to_string_lossy().to_string(),
            include_unchanged: true,
            check_conflicts: args.check_conflicts,
        };

        let response = preview_service
            .preview(generated_output, request)
            .map_err(|e| CliError::Generate(format!("Preview failed: {}", e)))?;

        // Apply include/exclude filters if specified
        let include_patterns = args.include_patterns();
        let exclude_patterns = args.exclude_patterns();

        let mut filtered_files = response.files;

        if !include_patterns.is_empty() {
            filtered_files.retain(|f| {
                include_patterns
                    .iter()
                    .any(|pattern| Self::glob_match(pattern, &f.path))
            });
        }

        if !exclude_patterns.is_empty() {
            filtered_files.retain(|f| {
                !exclude_patterns
                    .iter()
                    .any(|pattern| Self::glob_match(pattern, &f.path))
            });
        }

        // Display results
        println!("\n📋 Preview Summary:");
        println!("   Total: {} files", filtered_files.len());
        println!("   Created: {}", response.stats.created);
        println!("   Updated: {}", response.stats.updated);
        println!("   Deleted: {}", response.stats.deleted);
        println!("   Unchanged: {}", response.stats.unchanged);

        // Show conflict report if any
        if let Some(conflict_report) = &response.conflict_report {
            println!("\n⚠️  Protected Region Conflicts Detected:");
            println!("   File: {}", conflict_report.file_path);
            for conflict in &conflict_report.conflicts {
                println!(
                    "   - {:?}: {}",
                    conflict.conflict_type, conflict.description
                );
            }
            println!("   Suggestion: {}", conflict_report.suggestion);
        }

        // Print diff if requested
        if args.diff {
            for file in &filtered_files {
                if let Some(diff) = &file.diff {
                    println!("\n{}", "─".repeat(60));
                    println!("📄 {}", file.path);
                    println!("{}", "─".repeat(60));
                    println!("{}", diff);
                }
            }
        }

        // Save preview to directory if requested
        if let Some(save_dir) = args.save {
            fs::create_dir_all(&save_dir)
                .map_err(|e| CliError::Io(format!("Failed to create save dir: {}", e)))?;

            // Save summary JSON
            let summary = serde_json::json!({
                "stats": response.stats,
                "files": filtered_files.iter().map(|f| {
                    serde_json::json!({
                        "path": f.path,
                        "change_type": f.change_type,
                        "checksum": f.checksum,
                        "has_diff": f.diff.is_some(),
                    })
                }).collect::<Vec<_>>(),
            });

            fs::write(
                save_dir.join("preview-summary.json"),
                serde_json::to_string_pretty(&summary).unwrap(),
            )
            .map_err(|e| CliError::Io(format!("Failed to write summary: {}", e)))?;

            // Save each file's content
            for file in &filtered_files {
                let file_path = save_dir.join(&file.path);
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| CliError::Io(format!("Failed to create dir: {}", e)))?;
                }
                fs::write(&file_path, &file.content).map_err(|e| {
                    CliError::Io(format!("Failed to write {}: {}", file_path.display(), e))
                })?;

                // Save diff if exists
                if let Some(diff) = &file.diff {
                    let diff_path = save_dir.join(format!("{}.diff", file.path));
                    if let Some(parent) = diff_path.parent() {
                        fs::create_dir_all(parent).ok();
                    }
                    fs::write(&diff_path, diff).ok();
                }
            }

            println!("\n💾 Preview saved to: {}", save_dir.display());
        }

        Ok(())
    }

    /// Run batch command
    fn run_batch(args: BatchArgs) -> Result<(), CliError> {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        use std::thread;
        use std::time::Instant;

        let model_ids = args.model_ids();
        if model_ids.is_empty() {
            return Err(CliError::InvalidArgs(
                "No valid model IDs provided".to_string(),
            ));
        }

        println!("🚀 Batch generation for {} models", model_ids.len());

        let start_time = Instant::now();
        let generators = args.generators().unwrap_or_else(|| vec!["all".to_string()]);

        let success_count = Arc::new(AtomicUsize::new(0));
        let failure_count = Arc::new(AtomicUsize::new(0));
        let skipped_count = Arc::new(AtomicUsize::new(0));

        if args.parallel {
            // Parallel execution
            let max_concurrent = args.max_concurrent.max(1);
            println!("⚡ Running in parallel (max {} concurrent)", max_concurrent);

            let chunk_size = model_ids.len().div_ceil(max_concurrent);
            let chunks: Vec<Vec<i64>> = model_ids.chunks(chunk_size).map(|c| c.to_vec()).collect();

            let mut handles = Vec::new();

            for chunk in chunks {
                let success = Arc::clone(&success_count);
                let failure = Arc::clone(&failure_count);
                let skipped = Arc::clone(&skipped_count);
                let gens = generators.clone();
                let output_dir = args.output.clone();
                let dry_run = args.dry_run;

                let handle = thread::spawn(move || {
                    for fk_model in chunk {
                        match Self::generate_for_model(fk_model, &gens, &output_dir, dry_run) {
                            Ok(()) => {
                                success.fetch_add(1, Ordering::SeqCst);
                                println!("  ✓ Model {} generated", fk_model);
                            }
                            Err(e) if e.to_string().contains("not found") => {
                                skipped.fetch_add(1, Ordering::SeqCst);
                                println!("  ⏭ Model {} not found, skipped", fk_model);
                            }
                            Err(e) => {
                                failure.fetch_add(1, Ordering::SeqCst);
                                println!("  ✗ Model {} failed: {}", fk_model, e);
                            }
                        }
                    }
                });

                handles.push(handle);
            }

            for handle in handles {
                handle.join().expect("Thread panicked");
            }
        } else {
            // Sequential execution
            println!("📝 Running sequentially");

            for fk_model in &model_ids {
                match Self::generate_for_model(*fk_model, &generators, &args.output, args.dry_run) {
                    Ok(()) => {
                        success_count.fetch_add(1, Ordering::SeqCst);
                        println!("  ✓ Model {} generated", fk_model);
                    }
                    Err(e) if e.to_string().contains("not found") => {
                        skipped_count.fetch_add(1, Ordering::SeqCst);
                        println!("  ⏭ Model {} not found, skipped", fk_model);
                    }
                    Err(e) => {
                        failure_count.fetch_add(1, Ordering::SeqCst);
                        println!("  ✗ Model {} failed: {}", fk_model, e);
                        if !args.continue_on_error {
                            break;
                        }
                    }
                }
            }
        }

        let duration = start_time.elapsed();
        let success = success_count.load(Ordering::SeqCst);
        let failure = failure_count.load(Ordering::SeqCst);
        let skipped = skipped_count.load(Ordering::SeqCst);

        println!("\n📊 Batch Summary:");
        println!("   Success: {}", success);
        println!("   Failed: {}", failure);
        println!("   Skipped: {}", skipped);
        println!("   Duration: {:.2}s", duration.as_secs_f64());

        if failure > 0 {
            Err(CliError::BatchFailed(format!(
                "{} models failed generation",
                failure
            )))
        } else {
            Ok(())
        }
    }

    /// Generate for a single model (placeholder for actual implementation)
    fn generate_for_model(
        _model_id: i64,
        _generators: &[String],
        _output_dir: &Path,
        _dry_run: bool,
    ) -> Result<(), CliError> {
        // This would fetch the model from database and generate
        // For now, it's a placeholder that would be implemented with database access
        // In real implementation, this would:
        // 1. Fetch model from meta_model table
        // 2. Parse fields_json and relations_json
        // 3. Generate code using the generators
        Ok(())
    }

    /// Run history command
    fn run_history(args: HistoryArgs) -> Result<(), CliError> {
        // This would query the database for history records
        // For CLI without DB access, we'll provide a mock implementation
        // that can be enhanced when DB connection is available

        println!("📜 Generation History");

        if let Some(fk_history) = args.fk_history {
            // Show detailed view for specific history
            println!("\nDetailed view for history ID: {}", fk_history);
            println!("(This would show detailed info when DB is connected)");
            return Ok(());
        }

        // Build query description
        let mut filters = Vec::new();
        if let Some(fk_model) = args.model {
            filters.push(format!("fk_model={}", fk_model));
        }
        if let Some(generator) = &args.generator {
            filters.push(format!("generator={}", generator));
        }

        if !filters.is_empty() {
            println!("   Filters: {}", filters.join(", "));
        }
        println!("   Limit: {}", args.limit);

        // Mock table output (in real implementation, this queries DB)
        println!("\n{}", "─".repeat(80));
        println!(
            "{:>8} | {:>8} | {:>12} | {:>12} | Generated At",
            "ID", "Model", "Generator", "Files"
        );
        println!("{}", "─".repeat(80));

        // Placeholder rows
        println!("(History records would be displayed here when DB is connected)");
        println!("{}", "─".repeat(80));

        if args.detailed {
            println!("\n📁 Files in each history entry:");
            println!("(File details would be shown here when DB is connected)");
        }

        Ok(())
    }

    /// Run rollback command
    fn run_rollback(args: RollbackArgs) -> Result<(), CliError> {
        println!("⏮️  Rollback to history ID: {}", args.history);
        println!("   Output directory: {}", args.output.display());

        if args.preview {
            println!("\n📋 Preview mode - no changes will be applied");
        }

        if !args.force && !args.preview {
            // Interactive confirmation
            print!("\n⚠️  Are you sure you want to rollback? This will overwrite current files. [y/N] ");
            std::io::stdout()
                .flush()
                .map_err(|e| CliError::Io(e.to_string()))?;

            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .map_err(|e| CliError::Io(e.to_string()))?;

            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Rollback cancelled.");
                return Ok(());
            }
        }

        // In real implementation with DB access:
        // 1. Fetch history entry and files from DB
        // 2. Check protected regions if needed
        // 3. Restore files from content_snapshot

        if !args.skip_protected_check {
            println!("\n🔒 Checking protected regions...");
            // This would use MergeEngine to check for conflicts
        }

        println!("\n✅ Rollback completed (placeholder - DB connection required)");

        Ok(())
    }

    /// Run publish command
    fn run_publish(args: PublishArgs) -> Result<(), CliError> {
        println!("📦 Publishing module: {}", args.module_name);
        println!("   Version: {}", args.version);
        println!("   Registry: {}", args.registry_url());

        // 1. Validate version format
        println!("\n🔍 Validating version...");
        Self::validate_version(&args.version)?;

        // 2. Check artifact exists and is readable
        let artifact_path = &args.artifact_path;
        if !artifact_path.exists() {
            return Err(CliError::InvalidArgs(format!(
                "Artifact file not found: {}",
                artifact_path.display()
            )));
        }

        // 3. Read artifact and compute checksum
        println!("📂 Reading artifact...");
        let artifact_bytes = fs::read(artifact_path)
            .map_err(|e| CliError::Io(format!("Failed to read artifact: {}", e)))?;

        let mut hasher = Sha256::new();
        hasher.update(&artifact_bytes);
        let checksum = format!("sha256:{}", hex::encode(hasher.finalize()));

        println!("   Size: {} bytes", artifact_bytes.len());
        println!("   Checksum: {}", checksum);

        // 4. Store artifact locally in registry artifacts directory
        let registry_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("alioth")
            .join("registry")
            .join("artifacts")
            .join(&args.module_name)
            .join(&args.version);

        fs::create_dir_all(&registry_dir)
            .map_err(|e| CliError::Io(format!("Failed to create registry directory: {}", e)))?;

        let artifact_file_name = artifact_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let stored_artifact_path = registry_dir.join(&artifact_file_name);

        fs::write(&stored_artifact_path, &artifact_bytes)
            .map_err(|e| CliError::Io(format!("Failed to store artifact: {}", e)))?;

        println!("   Stored at: {}", stored_artifact_path.display());

        // 5. Build metadata JSON
        let user_metadata = args.parse_metadata();
        let metadata = serde_json::json!({
            "artifact_checksum": checksum,
            "artifact_size": artifact_bytes.len(),
            "artifact_path": stored_artifact_path.to_string_lossy(),
            "published_at": chrono::Utc::now().to_rfc3339(),
            "user_metadata": user_metadata,
        });

        // 6. Look up module_id by name via registry API
        println!("\n🔗 Looking up module '{}'...", args.module_name);
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| CliError::Io(format!("Failed to create HTTP client: {}", e)))?;

        let module_url = format!("{}/module/name/{}", args.registry_url(), args.module_name);
        let module_response = client.get(&module_url).send().map_err(|e| {
            CliError::Registry(format!(
                "Failed to query module: {}. Is the registry running?",
                e
            ))
        })?;

        if module_response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(CliError::Registry(format!(
                "Module '{}' not found in registry. Please create it first.",
                args.module_name
            )));
        }

        if !module_response.status().is_success() {
            return Err(CliError::Registry(format!(
                "Failed to query module (status {}): {}",
                module_response.status(),
                module_response.text().unwrap_or_default()
            )));
        }

        let module_json: serde_json::Value = module_response
            .json()
            .map_err(|e| CliError::Registry(format!("Failed to parse module response: {}", e)))?;

        let module_id = module_json["id"]
            .as_i64()
            .ok_or_else(|| CliError::Registry("Invalid module ID in response".to_string()))?;

        println!("   Found module ID: {}", module_id);

        // 7. Create version in registry
        println!("\n🚀 Creating version in registry...");
        let version_request = serde_json::json!({
            "module_id": module_id,
            "version": args.version,
"": args.,
            "metadata": metadata,
            "is_stable": args.stable,
        });

        let version_url = format!("{}/version", args.registry_url());
        let version_response = client
            .post(&version_url)
            .json(&version_request)
            .send()
            .map_err(|e| {
                CliError::Registry(format!(
                    "Failed to create version: {}. Is the registry running?",
                    e
                ))
            })?;

        let status = version_response.status();

        if status == reqwest::StatusCode::CONFLICT {
            return Err(CliError::Registry(format!(
                "Version {} already exists for module '{}'",
                args.version, args.module_name
            )));
        }

        if !status.is_success() && status != reqwest::StatusCode::CREATED {
            let error_body = version_response.text().unwrap_or_default();
            return Err(CliError::Registry(format!(
                "Failed to create version (status {}): {}",
                status, error_body
            )));
        }

        let version_response_json: serde_json::Value = version_response
            .json()
            .map_err(|e| CliError::Registry(format!("Failed to parse version response: {}", e)))?;

        // 8. Display success
        println!(
            "\n✅ Successfully published {}@{}",
            args.module_name, args.version
        );
        println!(
            "   Version ID: {}",
            version_response_json["id"].as_i64().unwrap_or(0)
        );
        println!(
            "   Downloads: {}",
            version_response_json["downloads"].as_i64().unwrap_or(0)
        );
        println!(
            "   Stable: {}",
            version_response_json["is_stable"]
                .as_bool()
                .unwrap_or(false)
        );

        Ok(())
    }

    /// Run install command
    fn run_install(args: InstallArgs) -> Result<(), CliError> {
        install::run_install(args)
    }

    /// Validate semantic version string
    fn validate_version(version: &str) -> Result<(), CliError> {
        // Semantic versioning regex: major.minor.patch(-prerelease)?(+build)?
        let semver_regex = Regex::new(r"^\d+\.\d+\.\d+(-[a-zA-Z0-9.]+)?(\+[a-zA-Z0-9.]+)?$")
            .expect("Invalid semver regex");

        if !semver_regex.is_match(version) {
            return Err(CliError::InvalidArgs(format!(
                "Invalid version format: '{}'. Expected semantic version (e.g., 1.0.0, 2.1.0-beta.1)",
                version
            )));
        }

        Ok(())
    }

    /// Write generator output to disk
    fn write_output(
        base_path: &std::path::Path,
        subdir: &str,
        output: &crate::generator::GeneratedOutput,
    ) -> Result<(), CliError> {
        let dir = base_path.join(subdir);
        fs::create_dir_all(&dir)
            .map_err(|e| CliError::Io(format!("Failed to create {}: {}", subdir, e)))?;

        for file in &output.files {
            let file_path = dir.join(&file.path);

            // Create parent directories
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| CliError::Io(format!("Failed to create dir: {}", e)))?;
            }

            fs::write(&file_path, &file.content).map_err(|e| {
                CliError::Io(format!("Failed to write {}: {}", file_path.display(), e))
            })?;
        }

        Ok(())
    }

    /// Simple glob matching (supports * and ? wildcards)
    fn glob_match(pattern: &str, text: &str) -> bool {
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let text_chars: Vec<char> = text.chars().collect();

        let mut p_idx = 0;
        let mut t_idx = 0;
        let mut star_idx = None;
        let mut match_idx = 0;

        while t_idx < text_chars.len() {
            if p_idx < pattern_chars.len()
                && (pattern_chars[p_idx] == '?' || pattern_chars[p_idx] == text_chars[t_idx])
            {
                p_idx += 1;
                t_idx += 1;
            } else if p_idx < pattern_chars.len() && pattern_chars[p_idx] == '*' {
                star_idx = Some(p_idx);
                match_idx = t_idx;
                p_idx += 1;
            } else if let Some(star) = star_idx {
                p_idx = star + 1;
                match_idx += 1;
                t_idx = match_idx;
            } else {
                return false;
            }
        }

        while p_idx < pattern_chars.len() && pattern_chars[p_idx] == '*' {
            p_idx += 1;
        }

        p_idx == pattern_chars.len()
    }

    /// Fetch model JSON from database by ontology ID
    fn fetch_model_from_db(_ontology_id: i64) -> Result<String, CliError> {
        Err(CliError::Io(
            "Ontology-based generation not yet implemented in CLI".to_string(),
        ))
    }

    /// Run init module command
    fn run_init_module(args: InitModuleArgs) -> Result<(), CliError> {
        let module_dir = args.output.join(&args.module_name);
        let backend_dir = module_dir.join("backend");
        let frontend_dir = module_dir.join("frontend");
        let docs_dir = module_dir.join("docs");

        fs::create_dir_all(&backend_dir)
            .map_err(|e| CliError::Io(format!("Failed to create backend dir: {}", e)))?;
        fs::create_dir_all(&frontend_dir)
            .map_err(|e| CliError::Io(format!("Failed to create frontend dir: {}", e)))?;
        fs::create_dir_all(&docs_dir)
            .map_err(|e| CliError::Io(format!("Failed to create docs dir: {}", e)))?;

        // Backend Cargo.toml stub
        let cargo_toml = format!(
            r#"[package]
name = "{}-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4"
actix-cors = "0.7"
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
sqlx = {{ version = "0.8", features = ["runtime-tokio", "postgres", "macros", "chrono"] }}
chrono = {{ version = "0.4", features = ["serde"] }}
thiserror = "2"
log = "0.4"
env_logger = "0.11"
dotenvy = "0.15"
uuid = {{ version = "1", features = ["v4", "serde"] }}
alioth-common = {{ path = "../../../Framework/backend/common" }}
alioth-crud = {{ path = "../../../Framework/backend/crud" }}
"#,
            args.module_name.replace("-", "_")
        );
        fs::write(backend_dir.join("Cargo.toml"), cargo_toml)
            .map_err(|e| CliError::Io(format!("Failed to write Cargo.toml: {}", e)))?;

        // Frontend package.json stub
        let package_json = format!(
            r#"{{
  "name": "@aliothstudio/{}",
  "version": "0.1.0",
  "type": "module",
  "scripts": {{
    "dev": "vite",
    "build": "vite build",
    "lint": "tsc --noEmit"
  }},
  "dependencies": {{
    "react": "^19",
    "react-dom": "^19"
  }}
}}"#,
            args.module_name
        );
        fs::write(frontend_dir.join("package.json"), package_json)
            .map_err(|e| CliError::Io(format!("Failed to write package.json: {}", e)))?;

        println!("✅ Initialized module '{}'", args.module_name);
        println!("   Backend:  {}", backend_dir.display());
        println!("   Frontend: {}", frontend_dir.display());
        println!("   Docs:     {}", docs_dir.display());

        Ok(())
    }

    /// Run build command
    fn run_build(args: BuildArgs) -> Result<(), CliError> {
        let project_root = std::env::current_dir()
            .map_err(|e| CliError::Io(format!("Failed to get cwd: {}", e)))?;

        // TODO: 当前构建目标仍硬编码了 Gateway/SSO 目录结构，需通过配置文件解耦。
        // 支持通过 `--config alioth-gen.toml` 注入自定义 backends/frontends 列表。
        let config = if let Some(config_path) = &args.config {
            CliConfig::from_file(config_path)?.build
        } else {
            config::BuildConfig::default()
        };

        let backends: Vec<(&str, &str)> = config
            .backends
            .iter()
            .map(|(d, n)| (d.as_str(), n.as_str()))
            .collect();
        let frontends: Vec<(&str, &str)> = config
            .frontends
            .iter()
            .map(|(d, f)| (d.as_str(), f.as_str()))
            .collect();

        let mode_flag = if args.release { "--release" } else { "" };

        if !args.frontend_only {
            println!("🔨 Building backends...");
            for (dir, _name) in &backends {
                let build_path = project_root.join(dir);
                println!("   Building {} ...", dir);
                let status = Command::new("cargo")
                    .arg("build")
                    .arg(mode_flag)
                    .current_dir(&build_path)
                    .status()
                    .map_err(|e| CliError::Io(format!("Failed to run cargo: {}", e)))?;
                if !status.success() {
                    return Err(CliError::Io(format!("Backend build failed for {}", dir)));
                }
            }
        }

        if !args.backend_only {
            println!("🔨 Building frontends...");
            for (dir, filter) in &frontends {
                let _path = project_root.join(dir);
                println!("   Building {} ...", dir);
                let status = Command::new("pnpm")
                    .arg("--filter")
                    .arg(filter)
                    .arg("build")
                    .current_dir(&project_root)
                    .status()
                    .map_err(|e| CliError::Io(format!("Failed to run pnpm: {}", e)))?;
                if !status.success() {
                    return Err(CliError::Io(format!("Frontend build failed for {}", dir)));
                }
            }
        }

        println!("\n✅ Build complete!");
        Ok(())
    }
}

/// CLI errors
#[derive(Debug)]
pub enum CliError {
    Io(String),
    Parse(String),
    Generate(String),
    Config(String),
    UnknownGenerator(String),
    InvalidArgs(String),
    BatchFailed(String),
    Registry(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Io(s) => write!(f, "IO Error: {}", s),
            CliError::Parse(s) => write!(f, "Parse Error: {}", s),
            CliError::Generate(s) => write!(f, "Generate Error: {}", s),
            CliError::Config(s) => write!(f, "Config Error: {}", s),
            CliError::UnknownGenerator(s) => write!(f, "Unknown Generator: {}", s),
            CliError::InvalidArgs(s) => write!(f, "Invalid Arguments: {}", s),
            CliError::BatchFailed(s) => write!(f, "Batch Failed: {}", s),
            CliError::Registry(s) => write!(f, "Registry Error: {}", s),
        }
    }
}

impl std::error::Error for CliError {}

impl From<config::ConfigError> for CliError {
    fn from(e: config::ConfigError) -> Self {
        CliError::Config(e.to_string())
    }
}
