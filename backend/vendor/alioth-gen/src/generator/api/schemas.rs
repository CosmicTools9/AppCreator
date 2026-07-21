//! Schema generation for the API contract.
//!
//! Converts IR-2 entities and fields into OpenAPI 3.0 Schema objects.

use crate::generator::api::example_values;
use crate::generator::api::types::{Schema, SchemaRef};
use crate::generator::ir::{GeneratorEntity, GeneratorField, GeneratorFieldType};
use serde_json::Value;
use std::collections::HashMap;

/// Generate the entity and input schemas for an IR-2 entity.
pub fn generate_entity_schemas(
    entity: &GeneratorEntity,
    include_examples: bool,
) -> (Schema, Schema) {
    let mut properties = HashMap::new();
    let mut input_properties = HashMap::new();
    let mut required = vec!["id".to_string()];
    let mut input_required = vec![];

    // ID field
    properties.insert(
        "id".to_string(),
        SchemaRef::Schema(Box::new(id_schema(include_examples))),
    );

    // Regular fields
    for field in &entity.fields {
        if field.name.snake == "id" {
            continue;
        }

        let schema = field_to_schema(field, include_examples);
        let is_required = !field.nullable;

        properties.insert(
            field.name.camel.clone(),
            SchemaRef::Schema(Box::new(schema.clone())),
        );

        // For input schema, skip system fields
        if !field.name.snake.starts_with("created_") && !field.name.snake.starts_with("updated_") {
            input_properties.insert(
                field.name.camel.clone(),
                SchemaRef::Schema(Box::new(schema)),
            );
            if is_required {
                input_required.push(field.name.camel.clone());
            }
        }
    }

    // System fields
    let timestamp_example = if include_examples {
        Some(Value::String("2026-04-10T12:00:00Z".to_string()))
    } else {
        None
    };

    properties.insert(
        "createdAt".to_string(),
        SchemaRef::Schema(Box::new(Schema {
            r#type: Some("string".to_string()),
            format: Some("date-time".to_string()),
            description: Some("Creation timestamp".to_string()),
            nullable: None,
            items: None,
            properties: None,
            required: None,
            example: timestamp_example.clone(),
            extensions: None,
        })),
    );
    properties.insert(
        "updatedAt".to_string(),
        SchemaRef::Schema(Box::new(Schema {
            r#type: Some("string".to_string()),
            format: Some("date-time".to_string()),
            description: Some("Last update timestamp".to_string()),
            nullable: None,
            items: None,
            properties: None,
            required: None,
            example: timestamp_example,
            extensions: None,
        })),
    );
    required.push("createdAt".to_string());
    required.push("updatedAt".to_string());

    let mut entity_extensions = HashMap::new();
    entity_extensions.insert(
        "x-alioth-entity".to_string(),
        Value::String(entity.name.pascal.clone()),
    );

    let entity_schema = Schema {
        r#type: Some("object".to_string()),
        format: None,
        description: entity.description.clone(),
        nullable: None,
        items: None,
        properties: Some(properties),
        required: Some(required),
        example: None,
        extensions: Some(entity_extensions),
    };

    let mut input_extensions = HashMap::new();
    input_extensions.insert(
        "x-alioth-entity".to_string(),
        Value::String(format!("{}Input", entity.name.pascal)),
    );
    input_extensions.insert(
        "x-alioth-input-type".to_string(),
        Value::String("create-update".to_string()),
    );

    let input_schema = Schema {
        r#type: Some("object".to_string()),
        format: None,
        description: Some(format!(
            "Input for creating/updating {}",
            entity.name.pascal
        )),
        nullable: None,
        items: None,
        properties: Some(input_properties),
        required: if input_required.is_empty() {
            None
        } else {
            Some(input_required)
        },
        example: None,
        extensions: Some(input_extensions),
    };

    (entity_schema, input_schema)
}

/// Convert a single field to an OpenAPI schema.
pub fn field_to_schema(field: &GeneratorField, include_examples: bool) -> Schema {
    let (r#type, format) = match &field.field_type {
        GeneratorFieldType::Text => ("string", None),
        GeneratorFieldType::Integer => ("integer", Some("int32")),
        GeneratorFieldType::BigInt => ("integer", Some("int64")),
        GeneratorFieldType::Decimal => ("number", Some("double")),
        GeneratorFieldType::Boolean => ("boolean", None),
        GeneratorFieldType::DateTime => ("string", Some("date-time")),
        GeneratorFieldType::Uuid => ("string", Some("uuid")),
        GeneratorFieldType::Json => ("object", None),
        GeneratorFieldType::Enum(_) => ("string", None),
        GeneratorFieldType::Reference(_) => ("integer", Some("int64")),
    };

    let example = if include_examples {
        Some(example_values::generate_for_field(field))
    } else {
        None
    };

    Schema {
        r#type: Some(r#type.to_string()),
        format: format.map(|s| s.to_string()),
        description: field.description.clone(),
        nullable: if field.nullable { Some(true) } else { None },
        items: None,
        properties: None,
        required: None,
        example,
        extensions: None,
    }
}

/// Schema reference for the ID parameter.
pub fn id_schema_ref(include_examples: bool) -> SchemaRef {
    SchemaRef::Schema(Box::new(id_schema(include_examples)))
}

/// Schema for the ID parameter.
pub fn id_schema(include_examples: bool) -> Schema {
    Schema {
        r#type: Some("integer".to_string()),
        format: Some("int64".to_string()),
        description: Some("Entity ID".to_string()),
        nullable: None,
        items: None,
        properties: None,
        required: None,
        example: if include_examples {
            Some(Value::Number(1000000000001i64.into()))
        } else {
            None
        },
        extensions: None,
    }
}
