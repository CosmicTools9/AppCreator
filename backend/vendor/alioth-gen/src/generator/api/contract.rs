//! API Contract intermediate representation.
//!
//! `ApiContract` is the internal seam inside `OpenApiGenerator`. It owns the API
//! contract shape — endpoints, operations, DTO references, and pagination — and
//! delegates serialization of the raw OpenAPI 3.0 types to `paths.rs` and
//! `schemas.rs`. `openapi.rs` is the thin adapter that builds the contract,
//! serializes it to `OpenApiSpec`, and emits JSON.

use crate::generator::api::paths;
use crate::generator::api::schemas;
use crate::generator::api::types::{Components, Info, OpenApiSpec, Operation, PathItem, Schema};
use crate::generator::ir::{GeneratorEntity, GeneratorFieldType, GeneratorModel};
use crate::generator::ValidationError;
use std::collections::HashMap;
use std::collections::HashSet;

/// Configuration passed to the contract builder.
#[derive(Debug, Clone)]
pub struct GenerationContext {
    pub api_title: String,
    pub api_version: String,
    pub base_path: String,
    pub include_examples: bool,
}

impl Default for GenerationContext {
    fn default() -> Self {
        Self {
            api_title: "AliothStudio API".to_string(),
            api_version: "1.0.0".to_string(),
            base_path: "/api".to_string(),
            include_examples: false,
        }
    }
}

/// The full API contract for a `GeneratorModel`.
#[derive(Debug, Clone)]
pub struct ApiContract {
    pub spec: OpenApiSpec,
    pub endpoints: Vec<EntityEndpoint>,
    pub context: GenerationContext,
}

/// Contract view of a single entity's API surface.
#[derive(Debug, Clone)]
pub struct EntityEndpoint {
    pub entity: GeneratorEntity,
    pub base_path: String,
    pub operations: Vec<CrudOperation>,
    pub schema: Schema,
    pub input_schema: Schema,
    pub dto_refs: Vec<DtoRef>,
    pub pagination: PaginationShape,
    pub examples: ExampleBundle,
}

/// A single CRUD operation on an entity endpoint.
#[derive(Debug, Clone)]
pub struct CrudOperation {
    pub method: HttpMethod,
    pub path: String,
    pub operation_id: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub operation: Operation,
}

/// HTTP method used by an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

/// Classification of a DTO reference field in the entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DtoRefKind {
    ForeignKey,
    Scalar,
    Unit,
    Category,
    Tag,
    Level,
}

/// A DTO reference discovered on an entity field.
#[derive(Debug, Clone)]
pub struct DtoRef {
    pub field_name: String,
    pub kind: DtoRefKind,
}

/// Shape of the list response for an entity.
#[derive(Debug, Clone)]
pub struct PaginationShape {
    pub item_schema_ref: String,
}

/// Examples associated with an entity endpoint.
#[derive(Debug, Clone)]
pub struct ExampleBundle {
    pub entity: Option<serde_json::Value>,
    pub input: Option<serde_json::Value>,
    pub array: Option<serde_json::Value>,
}

impl ApiContract {
    /// Build an API contract from an IR-2 model.
    pub fn from_model(
        model: &GeneratorModel,
        context: GenerationContext,
    ) -> Result<Self, ValidationError> {
        let mut paths = HashMap::new();
        let mut schemas = HashMap::new();
        let mut endpoints = Vec::with_capacity(model.entities.len());

        for entity in &model.entities {
            let entity_paths =
                paths::generate_entity_paths(entity, &context.base_path, context.include_examples);
            paths.extend(entity_paths);

            let (entity_schema, input_schema) =
                schemas::generate_entity_schemas(entity, context.include_examples);
            schemas.insert(entity.name.pascal.clone(), entity_schema.clone());
            schemas.insert(format!("{}Input", entity.name.pascal), input_schema.clone());

            let endpoint = Self::build_entity_endpoint(
                entity,
                &context.base_path,
                &entity_schema,
                &input_schema,
                &paths,
            );
            endpoints.push(endpoint);
        }

        let spec = OpenApiSpec {
            openapi: "3.0.3".to_string(),
            info: Info {
                title: context.api_title.clone(),
                version: context.api_version.clone(),
                description: Some(format!(
                    "Generated API for {} entities",
                    model.entities.len()
                )),
            },
            paths,
            components: Some(Components {
                schemas: Some(schemas),
            }),
        };

        let contract = Self {
            spec,
            endpoints,
            context,
        };
        contract.validate()?;
        Ok(contract)
    }

    /// Validate that the contract is well-formed.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.endpoints.is_empty() {
            return Ok(());
        }

        let mut operation_ids = HashSet::new();
        for endpoint in &self.endpoints {
            if endpoint.operations.is_empty() {
                return Err(ValidationError::InvalidField {
                    entity: endpoint.entity.name.pascal.clone(),
                    field: "operations".to_string(),
                    reason: "entity endpoint has no operations".to_string(),
                });
            }
            for op in &endpoint.operations {
                if !operation_ids.insert(op.operation_id.clone()) {
                    return Err(ValidationError::InvalidName {
                        name: op.operation_id.clone(),
                        reason: "duplicate operation id".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    fn build_entity_endpoint(
        entity: &GeneratorEntity,
        base_path: &str,
        entity_schema: &Schema,
        input_schema: &Schema,
        paths: &HashMap<String, PathItem>,
    ) -> EntityEndpoint {
        let base = format!("{}/{}", base_path, entity.name.plural_kebab);
        let single = format!("{}/{{id}}", base);

        let mut operations = Vec::with_capacity(5);
        if let Some(item) = paths.get(&base) {
            if let Some(op) = &item.get {
                operations.push(CrudOperation {
                    method: HttpMethod::Get,
                    path: base.clone(),
                    operation_id: op.operation_id.clone(),
                    summary: op.summary.clone(),
                    description: op.description.clone(),
                    operation: op.clone(),
                });
            }
            if let Some(op) = &item.post {
                operations.push(CrudOperation {
                    method: HttpMethod::Post,
                    path: base.clone(),
                    operation_id: op.operation_id.clone(),
                    summary: op.summary.clone(),
                    description: op.description.clone(),
                    operation: op.clone(),
                });
            }
        }
        if let Some(item) = paths.get(&single) {
            if let Some(op) = &item.get {
                operations.push(CrudOperation {
                    method: HttpMethod::Get,
                    path: single.clone(),
                    operation_id: op.operation_id.clone(),
                    summary: op.summary.clone(),
                    description: op.description.clone(),
                    operation: op.clone(),
                });
            }
            if let Some(op) = &item.put {
                operations.push(CrudOperation {
                    method: HttpMethod::Put,
                    path: single.clone(),
                    operation_id: op.operation_id.clone(),
                    summary: op.summary.clone(),
                    description: op.description.clone(),
                    operation: op.clone(),
                });
            }
            if let Some(op) = &item.patch {
                operations.push(CrudOperation {
                    method: HttpMethod::Patch,
                    path: single.clone(),
                    operation_id: op.operation_id.clone(),
                    summary: op.summary.clone(),
                    description: op.description.clone(),
                    operation: op.clone(),
                });
            }
            if let Some(op) = &item.delete {
                operations.push(CrudOperation {
                    method: HttpMethod::Delete,
                    path: single.clone(),
                    operation_id: op.operation_id.clone(),
                    summary: op.summary.clone(),
                    description: op.description.clone(),
                    operation: op.clone(),
                });
            }
        }

        let dto_refs = entity
            .fields
            .iter()
            .filter_map(dto_ref_from_field)
            .collect();

        let examples = ExampleBundle {
            entity: entity_schema.example.clone(),
            input: input_schema.example.clone(),
            array: None,
        };

        EntityEndpoint {
            entity: entity.clone(),
            base_path: base,
            operations,
            schema: entity_schema.clone(),
            input_schema: input_schema.clone(),
            dto_refs,
            pagination: PaginationShape {
                item_schema_ref: format!("#/components/schemas/{}", entity.name.pascal),
            },
            examples,
        }
    }
}

fn dto_ref_from_field(field: &crate::generator::ir::GeneratorField) -> Option<DtoRef> {
    let prefix = if field.name.snake.starts_with("fk_") {
        Some(DtoRefKind::ForeignKey)
    } else if field.name.snake.starts_with("qk_") {
        Some(DtoRefKind::Scalar)
    } else if field.name.snake.starts_with("sk_") {
        Some(DtoRefKind::Unit)
    } else if field.name.snake.starts_with("ck_") {
        Some(DtoRefKind::Category)
    } else if field.name.snake.starts_with("tk_") {
        Some(DtoRefKind::Tag)
    } else if field.name.snake.starts_with("lk_") {
        Some(DtoRefKind::Level)
    } else if matches!(field.field_type, GeneratorFieldType::Reference(_)) {
        Some(DtoRefKind::ForeignKey)
    } else {
        None
    };

    prefix.map(|kind| DtoRef {
        field_name: field.name.snake.clone(),
        kind,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::{
        EntityName, FieldName, GeneratorField, GeneratorFieldType, ModelMetadata, PrimaryKeyType,
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
            fields: vec![
                GeneratorField {
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
                },
                GeneratorField {
                    name: FieldName {
                        raw: "fk_role".to_string(),
                        snake: "fk_role".to_string(),
                        camel: "fk_role".to_string(),
                        pascal: "FkRole".to_string(),
                    },
                    field_type: GeneratorFieldType::Reference("Role".to_string()),
                    description: None,
                    nullable: true,
                    unique: false,
                    indexed: false,
                    default_value: None,
                    validations: vec![],
                    annotations: vec![],
                    ..Default::default()
                },
            ],
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
    fn test_from_model_creates_endpoints() {
        let model = create_test_model();
        let contract = ApiContract::from_model(&model, GenerationContext::default()).unwrap();

        assert_eq!(contract.endpoints.len(), 1);
        let endpoint = &contract.endpoints[0];
        assert_eq!(endpoint.base_path, "/api/users");
        assert_eq!(endpoint.operations.len(), 6);
        let op_ids: std::collections::HashSet<_> = endpoint
            .operations
            .iter()
            .map(|o| o.operation_id.clone())
            .collect();
        assert!(op_ids.contains("list_user"));
        assert!(op_ids.contains("get_user"));
        assert!(op_ids.contains("create_user"));
        assert!(op_ids.contains("update_user"));
        assert!(op_ids.contains("patch_user"));
        assert!(op_ids.contains("delete_user"));
    }

    #[test]
    fn test_dto_refs_resolved() {
        let model = create_test_model();
        let contract = ApiContract::from_model(&model, GenerationContext::default()).unwrap();
        let endpoint = &contract.endpoints[0];

        assert_eq!(endpoint.dto_refs.len(), 1);
        assert_eq!(endpoint.dto_refs[0].field_name, "fk_role");
        assert_eq!(endpoint.dto_refs[0].kind, DtoRefKind::ForeignKey);
    }

    #[test]
    fn test_validate_catches_duplicate_operation_ids() {
        let mut contract =
            ApiContract::from_model(&create_test_model(), GenerationContext::default()).unwrap();
        // Force a duplicate by cloning the first operation
        let first_op = contract.endpoints[0].operations[0].clone();
        contract.endpoints[0].operations.push(first_op);

        let result = contract.validate();
        assert!(result.is_err());
    }
}
