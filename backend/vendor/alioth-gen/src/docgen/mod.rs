//! Documentation Generator
//!
//! Generates Markdown documentation and Mermaid diagrams from MetaModel.

mod markdown;
mod mermaid;

pub use markdown::MarkdownGenerator;
pub use mermaid::{DiagramType, MermaidDiagramGenerator};

use crate::generator::ir::GeneratorModel;
use crate::generator::{
    GenerateError, GeneratedFile, GeneratedOutput, GenerationMetadata, Generator,
};

/// Combined documentation generator
pub struct DocGenerator {
    markdown_gen: MarkdownGenerator,
    mermaid_gen: MermaidDiagramGenerator,
}

impl DocGenerator {
    /// Create a new documentation generator
    pub fn new() -> Self {
        Self {
            markdown_gen: MarkdownGenerator::new(),
            mermaid_gen: MermaidDiagramGenerator::new(),
        }
    }

    /// Generate all documentation
    pub fn generate(&self, model: &GeneratorModel) -> Result<GeneratedOutput, GenerateError> {
        let mut files = Vec::new();

        // Generate API documentation
        let api_doc = self.markdown_gen.generate_api_documentation(model);
        files.push(GeneratedFile {
            path: "docs/api.md".into(),
            content: api_doc,
            checksum: String::new(),
        });

        // Generate entity documentation
        for entity in &model.entities {
            let entity_doc = self.markdown_gen.generate_entity_documentation(entity);
            files.push(GeneratedFile {
                path: format!("docs/entities/{}.md", entity.name.kebab).into(),
                content: entity_doc,
                checksum: String::new(),
            });
        }

        // Generate ER diagram
        let er_diagram = self.mermaid_gen.generate_er_diagram(model);
        files.push(GeneratedFile {
            path: "docs/diagrams/er-diagram.md".into(),
            content: er_diagram,
            checksum: String::new(),
        });

        // Generate relationship diagram
        let rel_diagram = self.mermaid_gen.generate_relationship_diagram(model);
        files.push(GeneratedFile {
            path: "docs/diagrams/relationships.md".into(),
            content: rel_diagram,
            checksum: String::new(),
        });

        // Generate navigation index
        let index = self.generate_index(model, &files);
        files.push(GeneratedFile {
            path: "docs/README.md".into(),
            content: index,
            checksum: String::new(),
        });

        let c_file_count = files.len();

        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "documentation".to_string(),
                entity_count: model.entities.len(),
                c_file_count,
            },
        })
    }

    /// Generate documentation index
    fn generate_index(&self, model: &GeneratorModel, _files: &[GeneratedFile]) -> String {
        let mut lines = vec![
            "# Model Documentation".to_string(),
            "".to_string(),
            format!("Generated at: {}", chrono::Utc::now().to_rfc3339()),
            "".to_string(),
            "## Overview".to_string(),
            "".to_string(),
            format!("- **Entities**: {}", model.entities.len()),
            format!("- **Enums**: {}", model.enums.len()),
            "".to_string(),
            "## Quick Links".to_string(),
            "".to_string(),
            "- [API Reference](api.md)".to_string(),
            "- [ER Diagram](diagrams/er-diagram.md)".to_string(),
            "- [Relationships](diagrams/relationships.md)".to_string(),
            "".to_string(),
            "## Entities".to_string(),
            "".to_string(),
        ];

        for entity in &model.entities {
            lines.push(format!(
                "- [{}](entities/{}.md) - {}",
                entity.name.pascal,
                entity.name.kebab,
                entity.description.as_deref().unwrap_or("No description")
            ));
        }

        lines.join("\n")
    }
}

impl Default for DocGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator for DocGenerator {
    fn name(&self) -> &'static str {
        "documentation"
    }

    fn generate(&self, model: &GeneratorModel) -> Result<GeneratedOutput, GenerateError> {
        self.generate(model)
    }

    fn validate(&self, _model: &GeneratorModel) -> Result<(), crate::generator::ValidationError> {
        Ok(())
    }

    fn supports_incremental(&self) -> bool {
        false
    }

    fn file_extensions(&self) -> Vec<&'static str> {
        vec!["md"]
    }
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
                fields: vec![],
                relations: vec![],
                annotations: vec![],
                primary_key_type: PrimaryKeyType::BigInt,
                ..Default::default()
            }],
            enums: vec![],
            metadata: ModelMetadata::default(),
            exceptions: vec![],
            exception_handlers: vec![],
            external_dependencies: vec![],
        }
    }

    #[test]
    fn test_doc_generator() {
        use std::path::PathBuf;

        let model = create_test_model();
        let gen = DocGenerator::new();
        let output = gen.generate(&model).unwrap();

        assert!(output
            .files
            .iter()
            .any(|f| f.path == PathBuf::from("docs/README.md")));
        assert!(output
            .files
            .iter()
            .any(|f| f.path == PathBuf::from("docs/api.md")));
        assert!(output
            .files
            .iter()
            .any(|f| f.path == PathBuf::from("docs/entities/user.md")));
    }
}
