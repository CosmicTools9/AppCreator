//! Markdown Documentation Generator

use crate::generator::ir::{
    GeneratorEntity, GeneratorField, GeneratorFieldType, GeneratorModel, PrimaryKeyType,
};

/// Markdown documentation generator
pub struct MarkdownGenerator;

impl MarkdownGenerator {
    /// Create a new markdown generator
    pub fn new() -> Self {
        Self
    }

    /// Generate API documentation
    pub fn generate_api_documentation(&self, model: &GeneratorModel) -> String {
        let mut lines = vec![
            "# API Reference".to_string(),
            "".to_string(),
            "## Base URL".to_string(),
            "".to_string(),
            "```".to_string(),
            "/api".to_string(),
            "```".to_string(),
            "".to_string(),
            "## Endpoints".to_string(),
            "".to_string(),
        ];

        for entity in &model.entities {
            lines.push(format!("### {}", entity.name.pascal));
            lines.push("".to_string());

            let base_path = format!("/api/{}", entity.name.plural_kebab);

            lines.push("| Method | Endpoint | Description |".to_string());
            lines.push("|--------|----------|-------------|".to_string());
            lines.push(format!(
                "| GET | `{}` | List all {}",
                base_path, entity.name.plural_snake
            ));
            lines.push(format!(
                "| POST | `{}` | Create {}",
                base_path, entity.name.snake
            ));
            lines.push(format!(
                "| GET | `{}/{{id}}` | Get {} by ID",
                base_path, entity.name.snake
            ));
            lines.push(format!(
                "| PUT | `{}/{{id}}` | Update {}",
                base_path, entity.name.snake
            ));
            lines.push(format!(
                "| DELETE | `{}/{{id}}` | Delete {}",
                base_path, entity.name.snake
            ));
            lines.push("".to_string());

            // Request/Response schema
            lines.push("#### Schema".to_string());
            lines.push("".to_string());
            lines.push("```typescript".to_string());
            lines.push(format!("interface {} {{", entity.name.pascal));
            lines.push(format!(
                "  id: {};",
                match entity.primary_key_type {
                    PrimaryKeyType::BigInt => "number",
                    PrimaryKeyType::Uuid => "string",
                }
            ));

            for field in &entity.fields {
                let ts_type = self.field_to_typescript(field);
                lines.push(format!("  {}: {};", field.name.camel, ts_type));
            }

            lines.push("  createdAt: Date;".to_string());
            lines.push("  updatedAt: Date;".to_string());
            lines.push("}".to_string());
            lines.push("```".to_string());
            lines.push("".to_string());
        }

        lines.join("\n")
    }

    /// Generate entity documentation
    pub fn generate_entity_documentation(&self, entity: &GeneratorEntity) -> String {
        let mut lines = vec![format!("# {}", entity.name.pascal), "".to_string()];

        if let Some(desc) = &entity.description {
            lines.push(desc.clone());
            lines.push("".to_string());
        }

        // Entity info
        lines.push("## Information".to_string());
        lines.push("".to_string());
        lines.push("| Property | Value |".to_string());
        lines.push("|----------|-------|".to_string());
        lines.push(format!("| Table Name | `{}` |", entity.name.snake));
        lines.push(format!(
            "| Primary Key | `{}` |",
            match entity.primary_key_type {
                PrimaryKeyType::BigInt => "BIGINT",
                PrimaryKeyType::Uuid => "UUID",
            }
        ));
        lines.push(format!("| Fields | {} |", entity.fields.len()));
        lines.push(format!("| Relations | {} |", entity.relations.len()));
        lines.push("".to_string());

        // Fields table
        if !entity.fields.is_empty() {
            lines.push("## Fields".to_string());
            lines.push("".to_string());
            lines.push("| Name | Type | Required | Unique | Default |".to_string());
            lines.push("|------|------|----------|--------|---------|".to_string());

            for field in &entity.fields {
                lines.push(format!(
                    "| `{}` | {} | {} | {} | {} |",
                    field.name.snake,
                    self.field_type_to_string(&field.field_type),
                    if field.nullable { "No" } else { "Yes" },
                    if field.unique { "Yes" } else { "No" },
                    field.default_value.as_deref().unwrap_or("-"),
                ));
            }
            lines.push("".to_string());
        }

        // Relations
        if !entity.relations.is_empty() {
            lines.push("## Relations".to_string());
            lines.push("".to_string());
            lines.push("| Name | Type | Target | Nullable |".to_string());
            lines.push("|------|------|--------|----------|".to_string());

            for rel in &entity.relations {
                lines.push(format!(
                    "| `{}` | {:?} | `{}` | {} |",
                    rel.name,
                    rel.relation_type,
                    rel.target_entity,
                    if rel.nullable { "Yes" } else { "No" },
                ));
            }
            lines.push("".to_string());
        }

        // TypeScript Interface
        lines.push("## TypeScript Interface".to_string());
        lines.push("".to_string());
        lines.push("```typescript".to_string());
        lines.push(format!("interface {} {{", entity.name.pascal));

        let id_type = match entity.primary_key_type {
            PrimaryKeyType::BigInt => "number",
            PrimaryKeyType::Uuid => "string",
        };
        lines.push(format!("  id: {};", id_type));

        for field in &entity.fields {
            let ts_type = self.field_to_typescript(field);
            let optional = if field.nullable { "?" } else { "" };
            lines.push(format!("  {}{}: {};", field.name.camel, optional, ts_type));
        }

        lines.push("  createdAt: Date;".to_string());
        lines.push("  updatedAt: Date;".to_string());
        lines.push("}".to_string());
        lines.push("```".to_string());

        lines.join("\n")
    }

    /// Convert field type to string for documentation display
    fn field_type_to_string(&self, ft: &GeneratorFieldType) -> String {
        match ft {
            GeneratorFieldType::Text => "TEXT".to_string(),
            GeneratorFieldType::Integer => "INTEGER".to_string(),
            GeneratorFieldType::BigInt => "BIGINT".to_string(),
            GeneratorFieldType::Decimal => "DECIMAL".to_string(),
            GeneratorFieldType::Boolean => "BOOLEAN".to_string(),
            GeneratorFieldType::DateTime => "TIMESTAMPTZ".to_string(),
            GeneratorFieldType::Uuid => "UUID".to_string(),
            GeneratorFieldType::Json => "JSONB".to_string(),
            GeneratorFieldType::Enum(name) => format!("ENUM({})", name),
            GeneratorFieldType::Reference(target) => format!("→ {}", target),
        }
    }

    /// Convert field to TypeScript type
    fn field_to_typescript(&self, field: &GeneratorField) -> String {
        let base = match &field.field_type {
            GeneratorFieldType::Text => "string",
            GeneratorFieldType::Integer | GeneratorFieldType::BigInt => "number",
            GeneratorFieldType::Decimal => "number",
            GeneratorFieldType::Boolean => "boolean",
            GeneratorFieldType::DateTime => "Date",
            GeneratorFieldType::Uuid => "string",
            GeneratorFieldType::Json => "Record<string, any>",
            GeneratorFieldType::Enum(_) => "string",
            GeneratorFieldType::Reference(_) => "number",
        };

        if field.nullable {
            format!("{} | null", base)
        } else {
            base.to_string()
        }
    }
}

impl Default for MarkdownGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::{EntityName, FieldName, GeneratorField, GeneratorFieldType};

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
            description: Some("User account".to_string()),
            fields: vec![GeneratorField {
                name: FieldName {
                    raw: "email".to_string(),
                    snake: "email".to_string(),
                    camel: "email".to_string(),
                    pascal: "Email".to_string(),
                },
                field_type: GeneratorFieldType::Text,
                description: None,
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

    #[test]
    fn test_generate_entity_documentation() {
        let gen = MarkdownGenerator::new();
        let entity = create_test_entity();
        let doc = gen.generate_entity_documentation(&entity);

        assert!(doc.contains("# User"));
    }

    #[test]
    fn test_field_to_typescript() {
        let gen = MarkdownGenerator::new();

        let field = GeneratorField {
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
        };

        assert_eq!(gen.field_to_typescript(&field), "number | null");
    }
}
