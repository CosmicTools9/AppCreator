//! Example Value Generator for OpenAPI
//!
//! Generates realistic example values for API requests and responses
//! based on field types and constraints.

use crate::generator::ir::{
    GeneratorEntity, GeneratorField, GeneratorFieldType, GeneratorValidationType,
};
use serde_json::{Map, Value};

/// Generate example value for a field
pub fn generate_for_field(field: &GeneratorField) -> Value {
    if field.nullable {
        // For nullable fields, we could return null, but let's provide a real example
        // Swagger UI will show the non-null example
    }

    match &field.field_type {
        GeneratorFieldType::Text => generate_text_example(field),
        GeneratorFieldType::Integer => generate_integer_example(field),
        GeneratorFieldType::BigInt => generate_bigint_example(field),
        GeneratorFieldType::Decimal => generate_decimal_example(field),
        GeneratorFieldType::Boolean => Value::Bool(true),
        GeneratorFieldType::DateTime => Value::String("2026-04-10T12:00:00Z".to_string()),
        GeneratorFieldType::Uuid => {
            Value::String("550e8400-e29b-41d4-a716-446655440000".to_string())
        }
        GeneratorFieldType::Json => generate_json_example(),
        GeneratorFieldType::Enum(enum_name) => generate_enum_example(enum_name),
        GeneratorFieldType::Reference(target) => generate_reference_example(target),
    }
}

/// Generate example for Text fields
fn generate_text_example(field: &GeneratorField) -> Value {
    // Check for format hints in validations
    let is_email = field
        .validations
        .iter()
        .any(|v| matches!(v.validation_type, GeneratorValidationType::Email));

    if is_email {
        return Value::String("user@example.com".to_string());
    }

    // Check field name for hints
    let field_name_lower = field.name.snake.to_lowercase();

    if field_name_lower.contains("email") {
        Value::String("user@example.com".to_string())
    } else if field_name_lower.contains("phone") || field_name_lower.contains("tel") {
        Value::String("+1-555-123-4567".to_string())
    } else if field_name_lower.contains("url") || field_name_lower.contains("link") {
        Value::String("https://example.com/resource".to_string())
    } else if field_name_lower.contains("color") {
        Value::String("#3b82f6".to_string())
    } else if field_name_lower.contains("description") || field_name_lower.contains("content") {
        Value::String("Lorem ipsum dolor sit amet, consectetur adipiscing elit.".to_string())
    } else if field_name_lower.contains("name") {
        Value::String(format!("Sample {}", capitalize(&field.name.raw)))
    } else if field_name_lower.contains("code") || field_name_lower.contains("sku") {
        Value::String("ABC-12345".to_string())
    } else if field_name_lower.contains("status") {
        Value::String("active".to_string())
    } else {
        Value::String(format!("Example {}", capitalize(&field.name.raw)))
    }
}

/// Generate example for Integer fields
fn generate_integer_example(field: &GeneratorField) -> Value {
    let field_name_lower = field.name.snake.to_lowercase();

    if field_name_lower.contains("age") {
        Value::Number(30.into())
    } else if field_name_lower.contains("year") {
        Value::Number(2026.into())
    } else if field_name_lower.contains("month") {
        Value::Number(4.into())
    } else if field_name_lower.contains("day") {
        Value::Number(15.into())
    } else if field_name_lower.contains("quantity") || field_name_lower.contains("count") {
        Value::Number(10.into())
    } else if field_name_lower.contains("order") || field_name_lower.contains("sequence") {
        Value::Number(1.into())
    } else {
        Value::Number(42.into())
    }
}

/// Generate example for BigInt fields (IDs)
fn generate_bigint_example(field: &GeneratorField) -> Value {
    let field_name_lower = field.name.snake.to_lowercase();

    if field_name_lower == "id" || field_name_lower.ends_with("_id") {
        // ZUID format (big integers)
        Value::Number(1000000000001i64.into())
    } else {
        Value::Number(9999999999999i64.into())
    }
}

/// Generate example for Decimal fields
fn generate_decimal_example(_field: &GeneratorField) -> Value {
    // Common monetary value
    Value::Number(serde_json::Number::from_f64(99.99).unwrap())
}

/// Generate example for JSON fields
fn generate_json_example() -> Value {
    let mut map = Map::new();
    map.insert("key".to_string(), Value::String("value".to_string()));
    map.insert("count".to_string(), Value::Number(42.into()));
    Value::Object(map)
}

/// Generate example for Enum fields
fn generate_enum_example(_enum_name: &str) -> Value {
    // Return first variant placeholder - actual value resolved at runtime
    Value::String("Active".to_string())
}

/// Generate example for Reference fields
fn generate_reference_example(_target: &str) -> Value {
    // Foreign key - return a sample ID
    Value::Number(1000000000001i64.into())
}

/// Generate complete example object for an entity
pub fn generate_entity_example(entity: &GeneratorEntity, include_id: bool) -> Value {
    let mut map = Map::new();

    for field in &entity.fields {
        // Skip ID for input schemas unless explicitly requested
        if field.name.snake == "id" && !include_id {
            continue;
        }

        // Skip internal fields
        if is_internal_field(&field.name.snake) {
            continue;
        }

        let example = generate_for_field(field);
        map.insert(field.name.snake.clone(), example);
    }

    Value::Object(map)
}

/// Generate array example with sample items
pub fn generate_array_example(entity: &GeneratorEntity) -> Value {
    let item = generate_entity_example(entity, true);
    Value::Array(vec![item])
}

/// Check if field is internal/system field
fn is_internal_field(name: &str) -> bool {
    matches!(
        name,
        "created_at" | "updated_at" | "deleted_at" | "created_by_id" | "updated_by_id" | "version"
    )
}

/// Capitalize first letter
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::{FieldName, GeneratorValidation};

    fn create_test_field(name: &str, field_type: GeneratorFieldType) -> GeneratorField {
        GeneratorField {
            name: FieldName {
                raw: name.to_string(),
                snake: name.to_string(),
                camel: name.to_string(),
                pascal: capitalize(name),
            },
            field_type,
            description: None,
            nullable: false,
            unique: false,
            indexed: false,
            default_value: None,
            validations: vec![],
            annotations: vec![],
            domain: None,
            range: None,
            min_cardinality: None,
            max_cardinality: None,
            is_functional: false,
            constraints: vec![],
            throws_clauses: vec![],
            quality_rules: vec![],
        }
    }

    #[test]
    fn test_text_example() {
        let field = create_test_field("name", GeneratorFieldType::Text);
        let example = generate_for_field(&field);
        assert!(example.as_str().unwrap().contains("Sample"));
    }

    #[test]
    fn test_email_example() {
        let mut field = create_test_field("email", GeneratorFieldType::Text);
        field.validations.push(GeneratorValidation {
            validation_type: GeneratorValidationType::Email,
            params: Default::default(),
        });
        let example = generate_for_field(&field);
        assert_eq!(example.as_str().unwrap(), "user@example.com");
    }

    #[test]
    fn test_integer_example() {
        let field = create_test_field("age", GeneratorFieldType::Integer);
        let example = generate_for_field(&field);
        assert_eq!(example.as_i64(), Some(30));
    }

    #[test]
    fn test_bigint_id_example() {
        let field = create_test_field("id", GeneratorFieldType::BigInt);
        let example = generate_for_field(&field);
        assert_eq!(example.as_i64(), Some(1000000000001));
    }

    #[test]
    fn test_decimal_example() {
        let field = create_test_field("price", GeneratorFieldType::Decimal);
        let example = generate_for_field(&field);
        assert!(example.as_f64().unwrap() > 0.0);
    }

    #[test]
    fn test_boolean_example() {
        let field = create_test_field("is_active", GeneratorFieldType::Boolean);
        let example = generate_for_field(&field);
        assert_eq!(example.as_bool(), Some(true));
    }

    #[test]
    fn test_datetime_example() {
        let field = create_test_field("created_at", GeneratorFieldType::DateTime);
        let example = generate_for_field(&field);
        assert!(example.as_str().unwrap().contains("T"));
    }

    #[test]
    fn test_entity_example() {
        let entity = GeneratorEntity {
            name: crate::generator::ir::EntityName {
                raw: "Product".to_string(),
                snake: "product".to_string(),
                camel: "product".to_string(),
                pascal: "Product".to_string(),
                kebab: "product".to_string(),
                screaming_snake: "PRODUCT".to_string(),
                plural_snake: "products".to_string(),
                plural_pascal: "Products".to_string(),
                plural_kebab: "products".to_string(),
            },
            description: None,
            fields: vec![
                create_test_field("id", GeneratorFieldType::BigInt),
                create_test_field("name", GeneratorFieldType::Text),
                create_test_field("price", GeneratorFieldType::Decimal),
            ],
            relations: vec![],
            annotations: vec![],
            primary_key_type: crate::generator::ir::PrimaryKeyType::BigInt,
            parent_classes: vec![],
            equivalent_classes: vec![],
            disjoint_classes: vec![],
            is_abstract: false,
            inheritance_depth: 0,
            state_machine: Default::default(),
            transitions: vec![],
            lifecycle_hooks: vec![],
            business_rules: vec![],
            swrl_rules: vec![],
            constraints: vec![],
            quality_rules: vec![],
            quality_config: Default::default(),
            parent_tables: vec![],
        };

        let example = generate_entity_example(&entity, true);
        let obj = example.as_object().unwrap();
        assert!(obj.contains_key("id"));
        assert!(obj.contains_key("name"));
        assert!(obj.contains_key("price"));
    }
}
