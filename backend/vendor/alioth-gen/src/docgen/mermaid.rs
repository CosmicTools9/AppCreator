//! Mermaid Diagram Generator

use crate::generator::ir::{GeneratorFieldType, GeneratorModel, GeneratorRelationType};

/// Diagram type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagramType {
    ER,
    Class,
    Relationship,
}

/// Mermaid diagram generator
pub struct MermaidDiagramGenerator;

impl MermaidDiagramGenerator {
    /// Create a new diagram generator
    pub fn new() -> Self {
        Self
    }

    /// Generate ER diagram
    pub fn generate_er_diagram(&self, model: &GeneratorModel) -> String {
        let mut result = String::new();

        result.push_str("# Entity-Relationship Diagram\n\n");
        result.push_str("```mermaid\n");
        result.push_str("erDiagram\n");

        for entity in &model.entities {
            result.push_str(&format!("    {} {{\n", entity.name.pascal));

            // ID field
            result.push_str("        bigint id PK\n");

            // Fields
            for field in &entity.fields {
                let pk_marker = if field.name.snake == "id" { " PK" } else { "" };
                let fk_marker = if matches!(&field.field_type, GeneratorFieldType::Reference(_)) {
                    " FK"
                } else {
                    ""
                };
                result.push_str(&format!(
                    "        {}{}{}{}\n",
                    self.field_type_to_er(&field.field_type),
                    field.name.snake,
                    pk_marker,
                    fk_marker
                ));
            }

            result.push_str("    }\n");
        }

        // Relationships
        for entity in &model.entities {
            for rel in &entity.relations {
                let cardinality = match rel.relation_type {
                    GeneratorRelationType::OneToOne => "||--||",
                    GeneratorRelationType::OneToMany => "||--o{",
                    GeneratorRelationType::ManyToOne => "}o--||",
                    GeneratorRelationType::ManyToMany => "}o--o{",
                    GeneratorRelationType::ManyHasMany => "}o--[{",
                };

                result.push_str(&format!(
                    "    {} {} {} : {}\n",
                    entity.name.pascal, cardinality, rel.target_entity, rel.name
                ));
            }
        }

        result.push_str("```\n");
        result
    }

    /// Generate relationship diagram
    pub fn generate_relationship_diagram(&self, model: &GeneratorModel) -> String {
        let mut result = String::new();

        result.push_str("# Entity Relationships\n\n");
        result.push_str("```mermaid\n");
        result.push_str("graph TD\n");

        // Collect all entities and relationships
        let mut nodes = vec![];
        let mut edges = vec![];

        for entity in &model.entities {
            let node_id = entity.name.pascal.clone();
            nodes.push((node_id.clone(), entity.name.pascal.clone()));

            for rel in &entity.relations {
                let target_id = rel.target_entity.clone();
                let label = format!("{}:{:?}", rel.name, rel.relation_type);

                edges.push((node_id.clone(), target_id, label));
            }
        }

        // Output nodes
        for (id, label) in &nodes {
            result.push_str(&format!("    {}[{}]\n", id, label));
        }

        result.push('\n');

        // Output edges
        for (from, to, label) in &edges {
            result.push_str(&format!("    {} -->|{}| {}\n", from, label, to));
        }

        result.push_str("```\n");
        result
    }

    /// Generate class diagram
    pub fn generate_class_diagram(&self, model: &GeneratorModel) -> String {
        let mut result = String::new();

        result.push_str("# Class Diagram\n\n");
        result.push_str("```mermaid\n");
        result.push_str("classDiagram\n");

        for entity in &model.entities {
            result.push_str(&format!("    class {} {{\n", entity.name.pascal));

            // Fields as attributes
            for field in &entity.fields {
                let visibility = if field.nullable { "~" } else { "+" };
                let field_type = self.field_type_to_typescript(&field.field_type);
                result.push_str(&format!(
                    "        {}{} {}\n",
                    visibility, field_type, field.name.camel
                ));
            }

            // Methods
            result.push_str("        +validate() boolean\n");
            result.push_str(&format!(
                "        +save() Promise~{}~\n",
                entity.name.pascal
            ));

            result.push_str("    }\n");
        }

        // Relationships
        for entity in &model.entities {
            for rel in &entity.relations {
                let arrow = match rel.relation_type {
                    GeneratorRelationType::OneToOne => "<|-->",
                    GeneratorRelationType::OneToMany => "<|--*",
                    GeneratorRelationType::ManyToOne => "*--|>",
                    GeneratorRelationType::ManyToMany => "*--*",
                    GeneratorRelationType::ManyHasMany => "*--[*",
                };

                result.push_str(&format!(
                    "    {} {} {}\n",
                    entity.name.pascal, arrow, rel.target_entity
                ));
            }
        }

        result.push_str("```\n");
        result
    }

    /// Convert field type to ER notation
    fn field_type_to_er(&self, ft: &GeneratorFieldType) -> String {
        match ft {
            GeneratorFieldType::Text => "string",
            GeneratorFieldType::Integer => "int",
            GeneratorFieldType::BigInt => "bigint",
            GeneratorFieldType::Decimal => "decimal",
            GeneratorFieldType::Boolean => "bool",
            GeneratorFieldType::DateTime => "timestamp",
            GeneratorFieldType::Uuid => "uuid",
            GeneratorFieldType::Json => "json",
            GeneratorFieldType::Enum(_) => "enum",
            GeneratorFieldType::Reference(_) => "bigint",
        }
        .to_string()
    }

    /// Convert field type to TypeScript notation
    fn field_type_to_typescript(&self, ft: &GeneratorFieldType) -> String {
        match ft {
            GeneratorFieldType::Text => "string",
            GeneratorFieldType::Integer | GeneratorFieldType::BigInt => "number",
            GeneratorFieldType::Decimal => "number",
            GeneratorFieldType::Boolean => "boolean",
            GeneratorFieldType::DateTime => "Date",
            GeneratorFieldType::Uuid => "string",
            GeneratorFieldType::Json => "any",
            GeneratorFieldType::Enum(name) => name,
            GeneratorFieldType::Reference(name) => name,
        }
        .to_string()
    }
}

impl Default for MermaidDiagramGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::{
        EntityName, GeneratorEntity, GeneratorRelation, ModelMetadata, PrimaryKeyType,
    };

    fn create_test_model() -> GeneratorModel {
        GeneratorModel {
            i18n_config: None,
            entities: vec![
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
                    description: None,
                    fields: vec![],
                    relations: vec![GeneratorRelation {
                        name: "orders".to_string(),
                        target_entity: "Order".to_string(),
                        relation_type: GeneratorRelationType::OneToMany,
                        nullable: false,
                    }],
                    annotations: vec![],
                    primary_key_type: PrimaryKeyType::BigInt,
                    ..Default::default()
                },
                GeneratorEntity {
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
                    description: None,
                    fields: vec![],
                    relations: vec![GeneratorRelation {
                        name: "customer".to_string(),
                        target_entity: "User".to_string(),
                        relation_type: GeneratorRelationType::ManyToOne,
                        nullable: false,
                    }],
                    annotations: vec![],
                    primary_key_type: PrimaryKeyType::BigInt,
                    ..Default::default()
                },
            ],
            enums: vec![],
            metadata: ModelMetadata::default(),
            exceptions: vec![],
            exception_handlers: vec![],
            external_dependencies: vec![],
        }
    }

    #[test]
    fn test_generate_er_diagram() {
        let gen = MermaidDiagramGenerator::new();
        let model = create_test_model();
        let diagram = gen.generate_er_diagram(&model);

        assert!(diagram.contains("erDiagram"));
        assert!(diagram.contains("User"));
        assert!(diagram.contains("Order"));
        assert!(diagram.contains("||--o{"));
    }

    #[test]
    fn test_generate_relationship_diagram() {
        let gen = MermaidDiagramGenerator::new();
        let model = create_test_model();
        let diagram = gen.generate_relationship_diagram(&model);

        assert!(diagram.contains("graph TD"));
        assert!(diagram.contains("User["));
        assert!(diagram.contains("-->"));
    }

    #[test]
    fn test_generate_class_diagram() {
        let gen = MermaidDiagramGenerator::new();
        let model = create_test_model();
        let diagram = gen.generate_class_diagram(&model);

        assert!(diagram.contains("classDiagram"));
        assert!(diagram.contains("class User"));
    }
}
