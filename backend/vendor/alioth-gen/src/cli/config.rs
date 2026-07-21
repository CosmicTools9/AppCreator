//! CLI Configuration

use serde::{Deserialize, Serialize};
use std::path::Path;

/// CLI Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// Project name
    pub name: String,

    /// Project version
    pub version: String,

    /// Default input file
    pub input: String,

    /// Default output directory
    pub output: String,

    /// Generator configurations
    #[serde(default)]
    pub generators: GeneratorConfigs,

    /// Custom templates directory
    #[serde(default)]
    pub templates: Option<String>,

    /// Naming conventions
    #[serde(default)]
    pub naming: NamingConfig,

    /// Build targets configuration
    #[serde(default)]
    pub build: BuildConfig,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            name: "my-project".to_string(),
            version: "1.0.0".to_string(),
            input: "model.dsl".to_string(),
            output: "./generated".to_string(),
            generators: GeneratorConfigs::default(),
            templates: None,
            naming: NamingConfig::default(),
            build: BuildConfig::default(),
        }
    }
}

impl CliConfig {
    /// Load config from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path).map_err(|e| ConfigError::Io(e.to_string()))?;

        toml::from_str(&content).map_err(|e| ConfigError::Parse(e.to_string()))
    }

    /// Save config to file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let toml =
            toml::to_string_pretty(self).map_err(|e| ConfigError::Serialize(e.to_string()))?;

        std::fs::write(path, toml).map_err(|e| ConfigError::Io(e.to_string()))
    }
}

/// Generator-specific configurations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneratorConfigs {
    #[serde(default)]
    pub zod: ZodConfig,

    #[serde(default)]
    pub api: ApiConfig,

    #[serde(default)]
    pub frontend: FrontendConfig,
}

/// Zod Generator config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZodConfig {
    /// Enable strict mode
    #[serde(default = "default_true")]
    pub strict: bool,

    /// Enable type coercion
    #[serde(default = "default_true")]
    pub coerce: bool,

    /// Generate React Hook Form integration
    #[serde(default = "default_true")]
    pub react_hook_form: bool,
}

impl Default for ZodConfig {
    fn default() -> Self {
        Self {
            strict: true,
            coerce: true,
            react_hook_form: true,
        }
    }
}

/// API Generator config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// API base path
    #[serde(default = "default_api_base")]
    pub base_path: String,

    /// Generate OpenAPI spec
    #[serde(default = "default_true")]
    pub openapi: bool,

    /// Generate Rust handlers
    #[serde(default = "default_true")]
    pub rust_handlers: bool,

    /// Generate TypeScript client
    #[serde(default = "default_true")]
    pub ts_client: bool,

    /// Client type
    #[serde(default = "default_client_type")]
    pub client_type: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_path: "/api".to_string(),
            openapi: true,
            rust_handlers: true,
            ts_client: true,
            client_type: "fetch".to_string(),
        }
    }
}

/// Frontend Generator config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendConfig {
    /// UI library
    #[serde(default = "default_ui_lib")]
    pub ui_library: String,

    /// Generate forms
    #[serde(default = "default_true")]
    pub forms: bool,

    /// Generate tables
    #[serde(default = "default_true")]
    pub tables: bool,

    /// Generate charts
    #[serde(default = "default_true")]
    pub charts: bool,
}

impl Default for FrontendConfig {
    fn default() -> Self {
        Self {
            ui_library: "shadcn".to_string(),
            forms: true,
            tables: true,
            charts: true,
        }
    }
}

/// Build targets configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Backend targets: (directory, package-name)
    #[serde(default = "default_backends")]
    pub backends: Vec<(String, String)>,

    /// Frontend targets: (directory, pnpm-filter)
    #[serde(default = "default_frontends")]
    pub frontends: Vec<(String, String)>,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            backends: default_backends(),
            frontends: default_frontends(),
        }
    }
}

fn default_backends() -> Vec<(String, String)> {
    vec![
        ("Meta/backend".to_string(), "meta-backend".to_string()),
        ("Gateway/backend".to_string(), "alioth-gateway".to_string()),
        ("SSO/backend".to_string(), "gateway-sso".to_string()),
    ]
}

fn default_frontends() -> Vec<(String, String)> {
    vec![
        (
            "Meta/frontend".to_string(),
            "@aliothstudio/meta-admin".to_string(),
        ),
        (
            "Gateway/frontend".to_string(),
            "@aliothstudio/gateway-frontend".to_string(),
        ),
    ]
}

/// Naming convention config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingConfig {
    /// Table naming style
    #[serde(default)]
    pub tables: NamingStyle,

    /// Column naming style
    #[serde(default)]
    pub columns: NamingStyle,

    /// Type naming style
    #[serde(default)]
    pub types: NamingStyle,
}

impl Default for NamingConfig {
    fn default() -> Self {
        Self {
            tables: NamingStyle::SnakeCase,
            columns: NamingStyle::SnakeCase,
            types: NamingStyle::PascalCase,
        }
    }
}

/// Naming style options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[derive(Default)]
pub enum NamingStyle {
    #[default]
    SnakeCase,
    CamelCase,
    PascalCase,
    KebabCase,
    ScreamingSnake,
}

/// Config errors
#[derive(Debug)]
pub enum ConfigError {
    Io(String),
    Parse(String),
    Serialize(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(s) => write!(f, "IO Error: {}", s),
            ConfigError::Parse(s) => write!(f, "Parse Error: {}", s),
            ConfigError::Serialize(s) => write!(f, "Serialize Error: {}", s),
        }
    }
}

impl std::error::Error for ConfigError {}

// Default value helpers
fn default_api_base() -> String {
    "/api".to_string()
}
fn default_ui_lib() -> String {
    "shadcn".to_string()
}
fn default_client_type() -> String {
    "fetch".to_string()
}
fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CliConfig::default();
        assert_eq!(config.input, "model.dsl");
        assert_eq!(config.output, "./generated");
    }

    #[test]
    fn test_config_serde() {
        let config = CliConfig::default();
        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("name = \"my-project\""));
    }
}
