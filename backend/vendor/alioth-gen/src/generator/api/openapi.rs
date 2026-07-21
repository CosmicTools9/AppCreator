//! OpenAPI JSON serializer.
//!
//! `OpenApiGenerator` is now a thin adapter: it builds an `ApiContract` from the
//! IR-2 model and returns the raw OpenAPI 3.0 specification. The contract shape
//! itself is owned by `contract.rs`, `paths.rs`, and `schemas.rs`.

use crate::generator::api::contract::{ApiContract, GenerationContext};
use crate::generator::api::types::OpenApiSpec;
use crate::generator::ir::GeneratorModel;
use crate::generator::ValidationError;

/// OpenAPI generator
pub struct OpenApiGenerator {
    context: GenerationContext,
}

impl OpenApiGenerator {
    /// Create a new OpenAPI generator
    pub fn new(api_title: impl Into<String>, api_version: impl Into<String>) -> Self {
        Self {
            context: GenerationContext {
                api_title: api_title.into(),
                api_version: api_version.into(),
                ..GenerationContext::default()
            },
        }
    }

    /// Set base API path
    pub fn with_base_path(mut self, path: impl Into<String>) -> Self {
        self.context.base_path = path.into();
        self
    }

    /// Enable example generation
    pub fn with_examples(mut self, include: bool) -> Self {
        self.context.include_examples = include;
        self
    }

    /// Generate OpenAPI spec from model
    pub fn generate(&self, model: &GeneratorModel) -> OpenApiSpec {
        let contract = ApiContract::from_model(model, self.context.clone())
            .unwrap_or_else(|e| panic!("OpenApiGenerator failed to build contract: {}", e));
        contract.spec
    }

    /// Validate the model against this generator.
    pub fn validate(&self, model: &GeneratorModel) -> Result<(), ValidationError> {
        let contract = ApiContract::from_model(model, self.context.clone())?;
        contract.validate()
    }
}

impl Default for OpenApiGenerator {
    fn default() -> Self {
        Self::new("AliothStudio API", "1.0.0")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::api::contract::{ApiContract, GenerationContext};
    use crate::generator::ir::{
        EntityName, FieldName, GeneratorEntity, GeneratorField, GeneratorFieldType, GeneratorModel,
        ModelMetadata, PrimaryKeyType,
    };

    fn create_test_entity() -> GeneratorEntity {
        GeneratorEntity {
            name: EntityName {
                raw: "User".to_string(),
                snake: "user".to_string(),
                camel: "user".to_string(),
                pascal: "User".to_string(),
                kebab: "user".to_string(),
                screaming_snake: "USER".to_string(),
                plural_snake: "users".to_string(),
                plural_pascal: "Users".to_string(),
                plural_kebab: "users".to_string(),
            },
            description: Some("User entity".to_string()),
            fields: vec![GeneratorField {
                name: FieldName {
                    raw: "email".to_string(),
                    snake: "email".to_string(),
                    camel: "email".to_string(),
                    pascal: "Email".to_string(),
                },
                field_type: GeneratorFieldType::Text,
                description: Some("User email".to_string()),
                nullable: false,
                unique: true,
                indexed: false,
                default_value: None,
                validations: vec![],
                annotations: vec![],
                ..Default::default()
            }],
            relations: vec![],
            annotations: vec![],
            primary_key_type: PrimaryKeyType::BigInt,
            ..Default::default()
        }
    }

    fn create_test_model() -> GeneratorModel {
        GeneratorModel {
            i18n_config: None,
            entities: vec![create_test_entity()],
            enums: vec![],
            metadata: ModelMetadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                generator_version: "test".to_string(),
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_openapi_generator() {
        let gen = OpenApiGenerator::new("Test API", "1.0.0");
        let model = create_test_model();
        let spec = gen.generate(&model);

        assert_eq!(spec.openapi, "3.0.3");
        assert_eq!(spec.info.title, "Test API");
        assert!(spec.paths.contains_key("/api/users"));
        assert!(spec.components.is_some());
    }

    #[test]
    fn test_openapi_generator_matches_contract() {
        let gen = OpenApiGenerator::new("Test API", "1.0.0");
        let model = create_test_model();
        let spec = gen.generate(&model);

        let contract = ApiContract::from_model(
            &model,
            GenerationContext {
                api_title: "Test API".to_string(),
                api_version: "1.0.0".to_string(),
                ..GenerationContext::default()
            },
        )
        .unwrap();

        assert_eq!(spec.openapi, contract.spec.openapi);
        assert_eq!(spec.info.title, contract.spec.info.title);
        assert_eq!(spec.paths.len(), contract.spec.paths.len());
    }

    #[test]
    fn test_generate_list_operation_via_contract() {
        let gen = OpenApiGenerator::default();
        let model = create_test_model();
        let spec = gen.generate(&model);

        let path_item = spec.paths.get("/api/users").unwrap();
        let list_op = path_item.get.as_ref().unwrap();
        assert_eq!(list_op.operation_id, "list_user");
        assert!(list_op.responses.contains_key("200"));
    }
}
