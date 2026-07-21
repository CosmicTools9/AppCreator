//! API Generator (Phase 5)
//!
//! Generates OpenAPI specifications, Actix-web handlers, and frontend API clients.
//!
//! ## Features
//!
//! - **OpenAPI 3.0**: Full API specification generation
//! - **Actix-web handlers**: Rust HTTP handlers with SQLx integration
//! - **Frontend clients**: TypeScript clients (Fetch/Axios) with React Query hooks
//!
//! ## Generated Files
//!
//! ### Backend (Rust)
//! ```text
//! src/
//! ├── handlers/
//! │   ├── mod.rs
//! │   └── {entity}.rs       # CRUD handlers
//! └── routes/
//!     └── {entity}.rs       # Route configuration
//! ```
//!
//! ### Frontend (TypeScript)
//! ```text
//! api/
//! ├── index.ts              # Export barrel
//! ├── types.ts              # Shared types
//! ├── {entity}.client.ts    # API client class
//! └── {entity}.hooks.ts     # React Query hooks
//! ```
//!
//! ### Documentation
//! ```text
//! openapi.json              # OpenAPI 3.0 specification
//! ```

mod client;
mod contract;
mod example_values;
mod handler;
mod openapi;
mod paths;
mod schemas;
mod types;

pub use client::{ClientGeneratorOptions, ClientType, FrontendClientGenerator};
pub use contract::{ApiContract, GenerationContext};
pub use handler::ActixHandlerGenerator;
pub use openapi::OpenApiGenerator;
pub use types::OpenApiSpec;

use crate::generator::ir::GeneratorModel;
use crate::generator::{
    GenerateError, GeneratedFile, GeneratedOutput, GenerationMetadata, Generator,
};

/// Combined API generator
pub struct ApiGenerator {
    openapi_gen: OpenApiGenerator,
    handler_gen: ActixHandlerGenerator,
    client_gen: FrontendClientGenerator,
}

impl ApiGenerator {
    /// Create a new API generator with default settings
    pub fn new() -> Self {
        Self {
            openapi_gen: OpenApiGenerator::default(),
            handler_gen: ActixHandlerGenerator,
            client_gen: FrontendClientGenerator::default(),
        }
    }

    /// Configure OpenAPI generator
    pub fn with_openapi(mut self, gen: OpenApiGenerator) -> Self {
        self.openapi_gen = gen;
        self
    }

    /// Configure handler generator
    pub fn with_handlers(mut self, gen: ActixHandlerGenerator) -> Self {
        self.handler_gen = gen;
        self
    }

    /// Configure client generator
    pub fn with_client(mut self, gen: FrontendClientGenerator) -> Self {
        self.client_gen = gen;
        self
    }

    /// Generate all API artifacts
    pub fn generate_all(
        &self,
        model: &GeneratorModel,
    ) -> Result<ApiGeneratedOutput, GenerateError> {
        // Generate OpenAPI spec
        let openapi_spec = self.openapi_gen.generate(model);
        let openapi_json = serde_json::to_string_pretty(&openapi_spec)
            .map_err(|e| GenerateError::Template(e.to_string()))?;

        // Generate backend handlers
        let handlers = self.handler_gen.generate_handlers(model)?;

        // Generate frontend client
        let client = self.client_gen.generate(model)?;

        Ok(ApiGeneratedOutput {
            openapi: openapi_json,
            handlers,
            client,
        })
    }
}

impl Default for ApiGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator for ApiGenerator {
    fn name(&self) -> &'static str {
        "api"
    }

    fn generate(&self, model: &GeneratorModel) -> Result<GeneratedOutput, GenerateError> {
        let all = self.generate_all(model)?;

        // Combine all files
        let mut files = Vec::new();

        // Add OpenAPI spec
        files.push(GeneratedFile {
            path: "openapi.json".into(),
            content: all.openapi,
            checksum: String::new(),
        });

        // Add handler files
        files.extend(all.handlers.files);

        // Add client files
        files.extend(all.client.files);

        let c_file_count = files.len();

        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: self.name().to_string(),
                entity_count: model.entities.len(),
                c_file_count,
            },
        })
    }

    fn validate(&self, _model: &GeneratorModel) -> Result<(), crate::generator::ValidationError> {
        Ok(())
    }

    fn supports_incremental(&self) -> bool {
        false
    }

    fn file_extensions(&self) -> Vec<&'static str> {
        vec!["rs", "ts", "json"]
    }
}

/// Output from API generation
pub struct ApiGeneratedOutput {
    /// OpenAPI JSON specification
    pub openapi: String,
    /// Backend handlers
    pub handlers: GeneratedOutput,
    /// Frontend client
    pub client: GeneratedOutput,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::{EntityName, GeneratorEntity, ModelMetadata, PrimaryKeyType};

    fn create_test_model() -> GeneratorModel {
        GeneratorModel {
            i18n_config: None,
            entities: vec![GeneratorEntity {
                name: EntityName {
                    raw: "Order".to_string(),
                    snake: "order".to_string(),
                    camel: "order".to_string(),
                    pascal: "Order".to_string(),
                    kebab: "order".to_string(),
                    screaming_snake: "ORDER".to_string(),
                    plural_snake: "orders".to_string(),
                    plural_pascal: "Orders".to_string(),
                    plural_kebab: "orders".to_string(),
                },
                description: Some("Order entity".to_string()),
                fields: vec![],
                relations: vec![],
                annotations: vec![],
                primary_key_type: PrimaryKeyType::BigInt,
                ..Default::default()
            }],
            enums: vec![],
            metadata: ModelMetadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                generator_version: "test".to_string(),
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_api_generator() {
        let model = create_test_model();
        let gen = ApiGenerator::new();
        let output = gen.generate_all(&model).unwrap();

        assert!(!output.openapi.is_empty());
        assert!(output.openapi.contains("3.0.3"));
        assert_eq!(output.handlers.metadata.entity_count, 1);
        assert_eq!(output.client.metadata.entity_count, 1);
    }

    #[test]
    fn test_combined_generator() {
        let model = create_test_model();
        let gen = ApiGenerator::new();
        let output = gen.generate(&model).unwrap();

        // Should have: openapi.json + handler files + client files
        assert!(output.files.len() >= 5);
        assert!(output
            .files
            .iter()
            .any(|f| f.path.to_string_lossy() == "openapi.json"));
    }
}
