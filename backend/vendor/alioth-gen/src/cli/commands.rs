//! CLI Commands and Arguments

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// MetaModel CLI
#[derive(Parser, Debug)]
#[command(name = "alioth")]
#[command(about = "Alioth code generation CLI")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate code from DSL
    Generate(GenerateArgs),

    /// Validate DSL syntax
    Validate(ValidateArgs),

    /// Initialize new project
    Init(InitArgs),

    /// Export model to DSL
    Export(ExportArgs),

    /// Preview code generation without writing to disk
    Preview(PreviewArgs),

    /// Batch generate for multiple models
    Batch(BatchArgs),

    /// List generation history
    History(HistoryArgs),

    /// Rollback to a previous generation
    Rollback(RollbackArgs),

    /// Publish a module version to the registry
    Publish(PublishArgs),

    /// Install a module from the registry
    Install(InstallArgs),

    /// Initialize a new Alioth module
    InitModule(InitModuleArgs),

    /// Build all or selected components
    Build(BuildArgs),
}

/// Generate command arguments
#[derive(Parser, Debug, Clone)]
pub struct GenerateArgs {
    /// Input DSL file
    #[arg(short, long, value_name = "FILE")]
    pub input: Option<PathBuf>,

    /// Output directory
    #[arg(short, long, value_name = "DIR", default_value = "Pre-Proc/Apps")]
    pub output: PathBuf,

    /// App name for Pre-Proc output subdirectory
    #[arg(short, long, value_name = "NAME")]
    pub app_name: Option<String>,

    /// Config file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Generators to run (comma-separated: zod,api,frontend,all)
    #[arg(short, long, value_name = "LIST")]
    pub generators: Option<String>,

    /// Watch mode - regenerate on file changes
    #[arg(long)]
    pub watch: bool,

    /// Dry run - don't write to disk
    #[arg(long)]
    pub dry_run: bool,

    /// Model ID for history tracking
    #[arg(long)]
    pub fk_model: Option<i64>,

    /// Generate from ontology ID (fetches model from database)
    #[arg(long, value_name = "ID")]
    pub from_ontology: Option<i64>,
}

impl GenerateArgs {
    /// Parse generators string into Vec
    pub fn generators(&self) -> Option<Vec<String>> {
        self.generators
            .as_ref()
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
    }
}

/// Validate command arguments
#[derive(Parser, Debug, Clone)]
pub struct ValidateArgs {
    /// Input DSL file
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,
}

/// Init command arguments
#[derive(Parser, Debug, Clone)]
pub struct InitArgs {
    /// Project directory
    #[arg(short, long, value_name = "DIR", default_value = ".")]
    pub dir: PathBuf,

    /// Project name
    #[arg(short, long)]
    pub name: Option<String>,
}

/// Export command arguments
#[derive(Parser, Debug, Clone)]
pub struct ExportArgs {
    /// Input file (DSL or JSON)
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,

    /// Output DSL file
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Input format (auto-detect if not specified)
    #[arg(long, value_enum)]
    pub format: Option<ExportFormat>,

    /// Include patterns (comma-separated)
    #[arg(long, value_name = "PATTERNS")]
    pub include: Option<String>,

    /// Exclude patterns (comma-separated)
    #[arg(long, value_name = "PATTERNS")]
    pub exclude: Option<String>,

    /// Schema filter
    #[arg(long, value_name = "SCHEMA")]
    pub schema: Option<String>,

    /// Pretty print output
    #[arg(long, default_value = "true")]
    pub pretty: bool,
}

impl ExportArgs {
    /// Parse include patterns
    pub fn include_patterns(&self) -> Vec<String> {
        self.include
            .as_ref()
            .map(|s| s.split(',').map(|p| p.trim().to_string()).collect())
            .unwrap_or_default()
    }

    /// Parse exclude patterns
    pub fn exclude_patterns(&self) -> Vec<String> {
        self.exclude
            .as_ref()
            .map(|s| s.split(',').map(|p| p.trim().to_string()).collect())
            .unwrap_or_default()
    }
}

/// Preview command arguments
#[derive(Parser, Debug, Clone)]
pub struct PreviewArgs {
    /// Input DSL file or Model ID
    #[arg(short, long, value_name = "FILE_OR_ID")]
    pub input: String,

    /// Output directory to compare against
    #[arg(short, long, value_name = "DIR", default_value = "./generated")]
    pub output: PathBuf,

    /// Print unified diff to terminal
    #[arg(long)]
    pub diff: bool,

    /// Save preview to local directory
    #[arg(long, value_name = "DIR")]
    pub save: Option<PathBuf>,

    /// Include patterns (glob, comma-separated)
    #[arg(long, value_name = "PATTERNS")]
    pub include: Option<String>,

    /// Exclude patterns (glob, comma-separated)
    #[arg(long, value_name = "PATTERNS")]
    pub exclude: Option<String>,

    /// Generators to run (comma-separated: zod,api,frontend,all)
    #[arg(short, long, value_name = "LIST")]
    pub generators: Option<String>,

    /// Check protected region conflicts
    #[arg(long, default_value = "true")]
    pub check_conflicts: bool,
}

impl PreviewArgs {
    /// Parse generators string into Vec
    pub fn generators(&self) -> Option<Vec<String>> {
        self.generators
            .as_ref()
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
    }

    /// Parse include glob patterns
    pub fn include_patterns(&self) -> Vec<String> {
        self.include
            .as_ref()
            .map(|s| s.split(',').map(|p| p.trim().to_string()).collect())
            .unwrap_or_default()
    }

    /// Parse exclude glob patterns
    pub fn exclude_patterns(&self) -> Vec<String> {
        self.exclude
            .as_ref()
            .map(|s| s.split(',').map(|p| p.trim().to_string()).collect())
            .unwrap_or_default()
    }
}

/// Batch command arguments
#[derive(Parser, Debug, Clone)]
pub struct BatchArgs {
    /// Model IDs to generate (comma-separated)
    #[arg(short, long, value_name = "IDS")]
    pub models: String,

    /// Output directory
    #[arg(short, long, value_name = "DIR", default_value = "./generated")]
    pub output: PathBuf,

    /// Generators to run (comma-separated: zod,api,frontend,all)
    #[arg(short, long, value_name = "LIST")]
    pub generators: Option<String>,

    /// Run in parallel
    #[arg(long)]
    pub parallel: bool,

    /// Max concurrent jobs (for parallel mode)
    #[arg(long, default_value = "4")]
    pub max_concurrent: usize,

    /// Continue on error
    #[arg(long)]
    pub continue_on_error: bool,

    /// Dry run
    #[arg(long)]
    pub dry_run: bool,
}

impl BatchArgs {
    /// Parse model IDs
    pub fn model_ids(&self) -> Vec<i64> {
        self.models
            .split(',')
            .filter_map(|s| s.trim().parse::<i64>().ok())
            .collect()
    }

    /// Parse generators string into Vec
    pub fn generators(&self) -> Option<Vec<String>> {
        self.generators
            .as_ref()
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
    }
}

/// History command arguments
#[derive(Parser, Debug, Clone)]
pub struct HistoryArgs {
    /// Model ID to filter by
    #[arg(short, long, value_name = "ID")]
    pub model: Option<i64>,

    /// Limit number of records
    #[arg(short, long, default_value = "10")]
    pub limit: i64,

    /// Offset for pagination
    #[arg(long, default_value = "0")]
    pub offset: i64,

    /// Generator type filter
    #[arg(long, value_name = "TYPE")]
    pub generator: Option<String>,

    /// Output format
    #[arg(long, value_enum, default_value = "table")]
    pub format: HistoryOutputFormat,

    /// Show detailed information including files
    #[arg(long)]
    pub detailed: bool,

    /// History ID to show details for
    #[arg(long)]
    pub fk_history: Option<i64>,
}

/// Rollback command arguments
#[derive(Parser, Debug, Clone)]
pub struct RollbackArgs {
    /// History ID to rollback to
    #[arg(short, long, value_name = "ID")]
    pub history: i64,

    /// Output directory
    #[arg(short, long, value_name = "DIR", default_value = "./generated")]
    pub output: PathBuf,

    /// Force rollback without confirmation
    #[arg(long)]
    pub force: bool,

    /// Preview changes without applying
    #[arg(long)]
    pub preview: bool,

    /// Skip protected region check
    #[arg(long)]
    pub skip_protected_check: bool,
}

/// Publish command arguments
#[derive(Parser, Debug, Clone)]
pub struct PublishArgs {
    /// Name of the module to publish
    #[arg(short, long, value_name = "NAME")]
    pub module_name: String,

    /// Semantic version string (e.g., 1.0.0, 2.1.0-beta.1)
    #[arg(short, long, value_name = "VERSION")]
    pub version: String,

    #[arg(short, long, value_name = "TEXT")]
pub : Option<String>,

    /// Path to the module artifact (tarball or zip)
    #[arg(short, long, value_name = "FILE")]
    pub artifact_path: PathBuf,

    /// Registry API base URL
    #[arg(
        short,
        long,
        value_name = "URL",
        default_value = "http://localhost:8091"
    )]
    pub registry_url: String,

    /// Mark as stable release
    #[arg(short, long, default_value = "false")]
    pub stable: bool,

    /// Additional metadata as JSON string
    #[arg(long, value_name = "JSON")]
    pub metadata: Option<String>,
}

impl PublishArgs {
    /// Returns the registry URL with trailing slash stripped
    pub fn registry_url(&self) -> String {
        self.registry_url.trim_end_matches('/').to_string()
    }

    /// Parses the metadata JSON string into serde_json::Value
    pub fn parse_metadata(&self) -> Option<serde_json::Value> {
        self.metadata
            .as_ref()
            .and_then(|m| serde_json::from_str(m).ok())
    }
}

/// Install command arguments
#[derive(Parser, Debug, Clone)]
pub struct InstallArgs {
    /// Name of the module to install
    #[arg(short, long, value_name = "NAME")]
    pub module_name: String,

    /// Specific version to install (defaults to latest)
    #[arg(short, long, value_name = "VERSION")]
    pub version: Option<String>,

    /// Registry API base URL
    #[arg(
        short,
        long,
        value_name = "URL",
        default_value = "http://localhost:8091"
    )]
    pub registry_url: String,

    /// Force reinstall even if already installed
    #[arg(short, long, default_value = "false")]
    pub force: bool,

    /// Skip dependency resolution
    #[arg(long, default_value = "false")]
    pub no_deps: bool,

    /// Show what would be installed without installing
    #[arg(short, long, default_value = "false")]
    pub dry_run: bool,
}

impl InstallArgs {
    /// Returns the registry URL with trailing slash stripped
    pub fn registry_url(&self) -> String {
        self.registry_url.trim_end_matches('/').to_string()
    }
}

/// Export input formats
#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum ExportFormat {
    /// Auto-detect from file extension
    Auto,
    /// DSL format
    Dsl,
    /// JSON format
    Json,
}

/// History output format
#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum HistoryOutputFormat {
    /// Table format
    Table,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
}

/// Available generators
#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum GeneratorType {
    /// Zod TypeScript schemas
    Zod,
    /// API (OpenAPI + handlers + client)
    Api,
    /// Frontend components
    Frontend,
    /// Documentation
    Docs,
    /// All generators
    All,
}

/// Init module command arguments
#[derive(Parser, Debug, Clone)]
pub struct InitModuleArgs {
    /// Module name (kebab-case recommended)
    pub module_name: String,

    /// Output directory (default: Modules/)
    #[arg(short, long, value_name = "DIR", default_value = "Modules")]
    pub output: PathBuf,
}

/// Build command arguments
#[derive(Parser, Debug, Clone)]
pub struct BuildArgs {
    /// Build only backends
    #[arg(long)]
    pub backend_only: bool,

    /// Build only frontends
    #[arg(long)]
    pub frontend_only: bool,

    /// Release mode (default: debug)
    #[arg(long)]
    pub release: bool,

    /// Config file for build targets
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_type_variants() {
        use clap::ValueEnum;
        let variants = GeneratorType::value_variants();
        assert_eq!(variants.len(), 5);
    }

    #[test]
    fn test_generate_args_parsing() {
        let args = GenerateArgs {
            input: Some(PathBuf::from("test.dsl")),
            output: PathBuf::from("./out"),
            app_name: None,
            config: None,
            generators: Some("zod,api".to_string()),
            watch: false,
            dry_run: false,
            fk_model: None,
            from_ontology: None,
        };

        let gens = args.generators().unwrap();
        assert_eq!(gens, vec!["zod", "api"]);
    }

    #[test]
    fn test_batch_args_model_ids() {
        let args = BatchArgs {
            models: "1,2,3,4,5".to_string(),
            output: PathBuf::from("./out"),
            generators: None,
            parallel: false,
            max_concurrent: 4,
            continue_on_error: false,
            dry_run: false,
        };

        let ids = args.model_ids();
        assert_eq!(ids, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_preview_args_patterns() {
        let args = PreviewArgs {
            input: "test.dsl".to_string(),
            output: PathBuf::from("./out"),
            diff: false,
            save: None,
            include: Some("*.rs,*.ts".to_string()),
            exclude: Some("*.test.rs".to_string()),
            generators: None,
            check_conflicts: true,
        };

        let include = args.include_patterns();
        assert_eq!(include, vec!["*.rs", "*.ts"]);

        let exclude = args.exclude_patterns();
        assert_eq!(exclude, vec!["*.test.rs"]);
    }

    #[test]
    fn test_history_args_defaults() {
        let args = HistoryArgs {
            model: Some(123),
            limit: 10,
            offset: 0,
            generator: None,
            format: HistoryOutputFormat::Table,
            detailed: false,
            fk_history: None,
        };

        assert_eq!(args.model, Some(123));
        assert_eq!(args.limit, 10);
    }

    #[test]
    fn test_rollback_args() {
        let args = RollbackArgs {
            history: 456,
            output: PathBuf::from("./out"),
            force: false,
            preview: false,
            skip_protected_check: false,
        };

        assert_eq!(args.history, 456);
        assert!(!args.force);
    }
}
