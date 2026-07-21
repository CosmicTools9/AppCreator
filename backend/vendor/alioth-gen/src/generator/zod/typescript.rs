//! TypeScript Type Generator
//!
//! Generates TypeScript interfaces and types from IR-2 models.

use crate::generator::ir::{GeneratorEntity, GeneratorField, GeneratorFieldType, GeneratorModel};

/// TypeScript generator
pub struct TypeScriptGenerator;

impl TypeScriptGenerator {
    /// Generate TypeScript interface for an entity
    pub fn generate_interface(entity: &GeneratorEntity) -> String {
        let mut fields = Vec::new();

        for field in &entity.fields {
            let ts_type = Self::map_field_to_typescript(field);
            let description = field
                .description
                .as_ref()
                .map(|d| format!("  /** {} */\n", d))
                .unwrap_or_default();

            fields.push(format!(
                "{}  {}: {};",
                description, field.name.camel, ts_type
            ));
        }

        // Add system fields
        fields.push("  /** Creation timestamp */".to_string());
        fields.push("  createdAt: Date;".to_string());
        fields.push("  /** Last update timestamp */".to_string());
        fields.push("  updatedAt: Date;".to_string());

        let description = entity
            .description
            .as_ref()
            .map(|d| format!("/** {} */\n", d))
            .unwrap_or_default();

        format!(
            "{}export interface {} {{\n{}\n}}",
            description,
            entity.name.pascal,
            fields.join("\n")
        )
    }

    /// Generate input type (for create/update)
    pub fn generate_input_type(entity: &GeneratorEntity) -> String {
        let mut fields = Vec::new();

        for field in &entity.fields {
            // Skip id and system fields for input
            if field.name.snake == "id"
                || field.name.snake.starts_with("created_")
                || field.name.snake.starts_with("updated_")
            {
                continue;
            }

            let ts_type = Self::map_field_to_typescript(field);
            let optional = if field.nullable { "?" } else { "" };

            fields.push(format!("  {}{}: {};", field.name.camel, optional, ts_type));
        }

        format!(
            "export interface {}Input {{\n{}\n}}",
            entity.name.pascal,
            fields.join("\n")
        )
    }

    /// Generate form data type (all fields optional)
    pub fn generate_form_type(entity: &GeneratorEntity) -> String {
        let mut fields = Vec::new();

        for field in &entity.fields {
            if field.name.snake == "id" {
                continue;
            }

            let ts_type = Self::map_field_to_typescript(field);

            fields.push(format!("  {}?: {} | null;", field.name.camel, ts_type));
        }

        format!(
            "export interface {}Form {{\n{}\n}}",
            entity.name.pascal,
            fields.join("\n")
        )
    }

    /// Map a field to TypeScript type
    fn map_field_to_typescript(field: &GeneratorField) -> String {
        Self::map_type(&field.field_type, field.nullable)
    }

    /// Map IR-2 type to TypeScript type
    fn map_type(field_type: &GeneratorFieldType, nullable: bool) -> String {
        let base_type = match field_type {
            GeneratorFieldType::Text => "string".to_string(),
            GeneratorFieldType::Integer => "number".to_string(),
            GeneratorFieldType::BigInt => "bigint".to_string(),
            GeneratorFieldType::Decimal => "number".to_string(),
            GeneratorFieldType::Boolean => "boolean".to_string(),
            GeneratorFieldType::DateTime => "Date".to_string(),
            GeneratorFieldType::Uuid => "string".to_string(),
            GeneratorFieldType::Json => "Record<string, any>".to_string(),
            GeneratorFieldType::Enum(name) => name.clone(),
            GeneratorFieldType::Reference(target) => format!("{}['id']", target),
        };

        if nullable {
            format!("{} | null", base_type)
        } else {
            base_type
        }
    }

    /// Generate enum type definitions
    pub fn generate_enum_types(model: &GeneratorModel) -> Vec<String> {
        model
            .enums
            .iter()
            .map(|enm| {
                let values: Vec<_> = enm
                    .values
                    .iter()
                    .map(|v| format!("  '{}' = '{}'", v, v))
                    .collect();

                format!(
                    "export enum {} {{\n{}\n}}\n\nexport const {}Values = [{}] as const;",
                    enm.name,
                    values.join(",\n"),
                    enm.name,
                    enm.values
                        .iter()
                        .map(|v| format!("'{}'", v))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .collect()
    }

    /// Generate complete types file content
    pub fn generate_types_file(model: &GeneratorModel) -> String {
        let mut sections = Vec::new();

        // Enums
        let enums = Self::generate_enum_types(model);
        if !enums.is_empty() {
            sections.push(enums.join("\n\n"));
        }

        // Interfaces
        for entity in &model.entities {
            sections.push(Self::generate_interface(entity));
            sections.push(Self::generate_input_type(entity));
            sections.push(Self::generate_form_type(entity));
        }

        sections.join("\n\n")
    }
}

impl Default for TypeScriptGenerator {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::{EntityName, FieldName};

    fn create_test_entity() -> GeneratorEntity {
        GeneratorEntity {
            name: EntityName {
                raw: "User".to_string(),
                snake: "users".to_string(),
                camel: "users".to_string(),
                pascal: "Users".to_string(),
                kebab: "users".to_string(),
                screaming_snake: "USERS".to_string(),
                plural_snake: "users".to_string(),
                plural_pascal: "Users".to_string(),
                plural_kebab: "users".to_string(),
            },
            description: Some("User accounts".to_string()),
            fields: vec![
                GeneratorField {
                    name: FieldName {
                        raw: "email".to_string(),
                        snake: "email".to_string(),
                        camel: "email".to_string(),
                        pascal: "Email".to_string(),
                    },
                    field_type: GeneratorFieldType::Text,
                    description: Some("Email address".to_string()),
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
                        raw: "age".to_string(),
                        snake: "age".to_string(),
                        camel: "age".to_string(),
                        pascal: "Age".to_string(),
                    },
                    field_type: GeneratorFieldType::Integer,
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
            primary_key_type: crate::generator::ir::PrimaryKeyType::BigInt,
            ..Default::default()
        }
    }

    #[test]
    fn test_generate_interface() {
        let entity = create_test_entity();
        let interface = TypeScriptGenerator::generate_interface(&entity);

        assert!(interface.contains("export interface Users"));
        assert!(interface.contains("email: string;"));
        assert!(interface.contains("age: number | null;"));
        assert!(interface.contains("createdAt: Date;"));
        assert!(interface.contains("User accounts")); // Description
    }

    #[test]
    fn test_generate_input_type() {
        let entity = create_test_entity();
        let input = TypeScriptGenerator::generate_input_type(&entity);

        assert!(input.contains("export interface UsersInput"));
        assert!(input.contains("email: string;"));
        assert!(!input.contains("id:")); // Should not include id
        assert!(!input.contains("createdAt:")); // Should not include system fields
    }

    #[test]
    fn test_map_type() {
        assert_eq!(
            TypeScriptGenerator::map_type(&GeneratorFieldType::Text, false),
            "string"
        );
        assert_eq!(
            TypeScriptGenerator::map_type(&GeneratorFieldType::Integer, true),
            "number | null"
        );
        assert_eq!(
            TypeScriptGenerator::map_type(&GeneratorFieldType::BigInt, false),
            "bigint"
        );
    }
}
