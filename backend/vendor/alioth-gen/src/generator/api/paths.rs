//! Path and operation generation for the API contract.
//!
//! Converts IR-2 entities into OpenAPI 3.0 PathItem and Operation objects.

use crate::generator::api::example_values;
use crate::generator::api::schemas::id_schema_ref;
use crate::generator::api::types::{
    MediaType, Operation, Parameter, PathItem, RequestBody, Response, Schema, SchemaRef,
};
use crate::generator::ir::GeneratorEntity;
use serde_json::Value;
use std::collections::HashMap;

/// Generate the path items for a single entity.
pub fn generate_entity_paths(
    entity: &GeneratorEntity,
    base_path: &str,
    include_examples: bool,
) -> HashMap<String, PathItem> {
    let mut paths = HashMap::new();
    let entity_base_path = format!("{}/{}", base_path, entity.name.plural_kebab);
    let single_path = format!("{}/{{id}}", entity_base_path);

    // Collection operations
    paths.insert(
        entity_base_path.clone(),
        PathItem {
            get: Some(generate_list_operation(entity, include_examples)),
            post: Some(generate_create_operation(entity, include_examples)),
            put: None,
            patch: None,
            delete: None,
        },
    );

    // Single-entity operations
    paths.insert(
        single_path,
        PathItem {
            get: Some(generate_get_operation(entity, include_examples)),
            post: None,
            put: Some(generate_update_operation(entity, include_examples)),
            patch: Some(generate_patch_operation(entity, include_examples)),
            delete: Some(generate_delete_operation(entity, include_examples)),
        },
    );

    paths
}

/// Generate the list operation.
pub fn generate_list_operation(entity: &GeneratorEntity, include_examples: bool) -> Operation {
    let mut responses = HashMap::new();

    let list_schema = Schema {
        r#type: Some("array".to_string()),
        format: None,
        description: None,
        nullable: None,
        items: Some(Box::new(SchemaRef::Ref {
            r#ref: format!("#/components/schemas/{}", entity.name.pascal),
        })),
        properties: None,
        required: None,
        example: None,
        extensions: None,
    };

    let example = if include_examples {
        Some(example_values::generate_array_example(entity))
    } else {
        None
    };

    responses.insert(
        "200".to_string(),
        Response {
            description: "List of entities".to_string(),
            content: Some({
                let mut content = HashMap::new();
                content.insert(
                    "application/json".to_string(),
                    MediaType {
                        schema: SchemaRef::Schema(Box::new(list_schema)),
                        example,
                    },
                );
                content
            }),
        },
    );

    Operation {
        operation_id: format!("list_{}", entity.name.snake),
        summary: Some(format!("List all {}", entity.name.plural_snake)),
        description: entity.description.clone(),
        parameters: None,
        request_body: None,
        responses,
    }
}

/// Generate the get-by-id operation.
pub fn generate_get_operation(entity: &GeneratorEntity, include_examples: bool) -> Operation {
    let mut responses = HashMap::new();

    let example = if include_examples {
        Some(example_values::generate_entity_example(entity, true))
    } else {
        None
    };

    responses.insert(
        "200".to_string(),
        Response {
            description: "Entity found".to_string(),
            content: Some({
                let mut content = HashMap::new();
                content.insert(
                    "application/json".to_string(),
                    MediaType {
                        schema: SchemaRef::Ref {
                            r#ref: format!("#/components/schemas/{}", entity.name.pascal),
                        },
                        example,
                    },
                );
                content
            }),
        },
    );
    responses.insert(
        "404".to_string(),
        Response {
            description: "Entity not found".to_string(),
            content: None,
        },
    );

    let id_example = if include_examples {
        Some(Value::Number(1000000000001i64.into()))
    } else {
        None
    };

    Operation {
        operation_id: format!("get_{}", entity.name.snake),
        summary: Some(format!("Get {} by ID", entity.name.snake)),
        description: None,
        parameters: Some(vec![Parameter {
            name: "id".to_string(),
            r#in: "path".to_string(),
            required: true,
            description: Some(format!("{} ID", entity.name.pascal)),
            schema: id_schema_ref(include_examples),
            example: id_example,
        }]),
        request_body: None,
        responses,
    }
}

/// Generate the create operation.
pub fn generate_create_operation(entity: &GeneratorEntity, include_examples: bool) -> Operation {
    let mut responses = HashMap::new();

    let response_example = if include_examples {
        Some(example_values::generate_entity_example(entity, true))
    } else {
        None
    };

    responses.insert(
        "201".to_string(),
        Response {
            description: "Entity created".to_string(),
            content: Some({
                let mut content = HashMap::new();
                content.insert(
                    "application/json".to_string(),
                    MediaType {
                        schema: SchemaRef::Ref {
                            r#ref: format!("#/components/schemas/{}", entity.name.pascal),
                        },
                        example: response_example,
                    },
                );
                content
            }),
        },
    );

    let request_example = if include_examples {
        Some(example_values::generate_entity_example(entity, false))
    } else {
        None
    };

    Operation {
        operation_id: format!("create_{}", entity.name.snake),
        summary: Some(format!("Create a new {}", entity.name.snake)),
        description: None,
        parameters: None,
        request_body: Some(RequestBody {
            required: true,
            content: {
                let mut content = HashMap::new();
                content.insert(
                    "application/json".to_string(),
                    MediaType {
                        schema: SchemaRef::Ref {
                            r#ref: format!("#/components/schemas/{}Input", entity.name.pascal),
                        },
                        example: request_example,
                    },
                );
                content
            },
        }),
        responses,
    }
}

/// Generate the update operation (PUT).
pub fn generate_update_operation(entity: &GeneratorEntity, include_examples: bool) -> Operation {
    let mut responses = HashMap::new();

    let response_example = if include_examples {
        Some(example_values::generate_entity_example(entity, true))
    } else {
        None
    };

    responses.insert(
        "200".to_string(),
        Response {
            description: "Entity updated".to_string(),
            content: Some({
                let mut content = HashMap::new();
                content.insert(
                    "application/json".to_string(),
                    MediaType {
                        schema: SchemaRef::Ref {
                            r#ref: format!("#/components/schemas/{}", entity.name.pascal),
                        },
                        example: response_example,
                    },
                );
                content
            }),
        },
    );

    let id_example = if include_examples {
        Some(Value::Number(1000000000001i64.into()))
    } else {
        None
    };

    let request_example = if include_examples {
        Some(example_values::generate_entity_example(entity, false))
    } else {
        None
    };

    Operation {
        operation_id: format!("update_{}", entity.name.snake),
        summary: Some(format!("Update {}", entity.name.snake)),
        description: None,
        parameters: Some(vec![Parameter {
            name: "id".to_string(),
            r#in: "path".to_string(),
            required: true,
            description: Some(format!("{} ID", entity.name.pascal)),
            schema: id_schema_ref(include_examples),
            example: id_example,
        }]),
        request_body: Some(RequestBody {
            required: true,
            content: {
                let mut content = HashMap::new();
                content.insert(
                    "application/json".to_string(),
                    MediaType {
                        schema: SchemaRef::Ref {
                            r#ref: format!("#/components/schemas/{}Input", entity.name.pascal),
                        },
                        example: request_example,
                    },
                );
                content
            },
        }),
        responses,
    }
}

/// Generate the patch operation.
pub fn generate_patch_operation(entity: &GeneratorEntity, include_examples: bool) -> Operation {
    let mut responses = HashMap::new();

    let response_example = if include_examples {
        Some(example_values::generate_entity_example(entity, true))
    } else {
        None
    };

    responses.insert(
        "200".to_string(),
        Response {
            description: "Entity patched".to_string(),
            content: Some({
                let mut content = HashMap::new();
                content.insert(
                    "application/json".to_string(),
                    MediaType {
                        schema: SchemaRef::Ref {
                            r#ref: format!("#/components/schemas/{}", entity.name.pascal),
                        },
                        example: response_example,
                    },
                );
                content
            }),
        },
    );

    let id_example = if include_examples {
        Some(Value::Number(1000000000001i64.into()))
    } else {
        None
    };

    let request_example = if include_examples {
        Some(example_values::generate_entity_example(entity, false))
    } else {
        None
    };

    Operation {
        operation_id: format!("patch_{}", entity.name.snake),
        summary: Some(format!("Partially update {}", entity.name.snake)),
        description: None,
        parameters: Some(vec![Parameter {
            name: "id".to_string(),
            r#in: "path".to_string(),
            required: true,
            description: Some(format!("{} ID", entity.name.pascal)),
            schema: id_schema_ref(include_examples),
            example: id_example,
        }]),
        request_body: Some(RequestBody {
            required: true,
            content: {
                let mut content = HashMap::new();
                content.insert(
                    "application/json".to_string(),
                    MediaType {
                        schema: SchemaRef::Ref {
                            r#ref: format!("#/components/schemas/{}Input", entity.name.pascal),
                        },
                        example: request_example,
                    },
                );
                content
            },
        }),
        responses,
    }
}

/// Generate the delete operation.
pub fn generate_delete_operation(entity: &GeneratorEntity, include_examples: bool) -> Operation {
    let mut responses = HashMap::new();
    responses.insert(
        "204".to_string(),
        Response {
            description: "Entity deleted".to_string(),
            content: None,
        },
    );

    let id_example = if include_examples {
        Some(Value::Number(1000000000001i64.into()))
    } else {
        None
    };

    Operation {
        operation_id: format!("delete_{}", entity.name.snake),
        summary: Some(format!("Delete {}", entity.name.snake)),
        description: None,
        parameters: Some(vec![Parameter {
            name: "id".to_string(),
            r#in: "path".to_string(),
            required: true,
            description: Some(format!("{} ID", entity.name.pascal)),
            schema: id_schema_ref(include_examples),
            example: id_example,
        }]),
        request_body: None,
        responses,
    }
}
