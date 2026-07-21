//! Actix-web Handler Generator
//!
//! Generates Actix-web route handlers from IR-2 models with audit support.
//!
//! **架构**: 两阶段生成
//! 1. IR-2 → Rust AST (`generator::ast::transform`)
//! 2. AST → String (`RustEmitter`)

use crate::generator::ast::rust::RustEmitter;
use crate::generator::ast::transform::{
    entity_to_rust_handler_ast, entity_to_rust_routes_ast, model_to_rust_handlers_mod_ast,
};
use crate::generator::ast::AstEmitter;
use crate::generator::ir::{GeneratorEntity, GeneratorModel};
use crate::generator::{
    DiffReport, ExistingFile, FileChange, GenerateError, GeneratedFile, GeneratedOutput,
    GenerationMetadata, Generator, IncrementalGenerator, MergeConflict,
};
use rayon::prelude::*;
use similar::{ChangeTag, TextDiff};
use std::collections::HashMap;
use std::path::Path;

/// Actix-web handler generator with audit support
pub struct ActixHandlerGenerator;

impl ActixHandlerGenerator {
    /// Generate handler module for the entire model
    pub fn generate_handlers(
        &self,
        model: &GeneratorModel,
    ) -> Result<GeneratedOutput, GenerateError> {
        // Parallel generation of entity handlers
        let mut entity_files: Vec<GeneratedFile> = model
            .entities
            .par_iter()
            .map(|entity| {
                let handler_code = self.generate_entity_handlers(entity);
                let routes_code = self.generate_route_config(entity);

                vec![
                    GeneratedFile {
                        path: format!("handlers/{}.rs", entity.name.snake).into(),
                        content: handler_code,
                        checksum: String::new(),
                    },
                    GeneratedFile {
                        path: format!("routes/{}.rs", entity.name.snake).into(),
                        content: routes_code,
                        checksum: String::new(),
                    },
                ]
            })
            .flatten()
            .collect();

        let mut files = Vec::with_capacity(entity_files.len() + 1);
        files.append(&mut entity_files);

        // Generate mod.rs
        let mod_content = self.generate_mod_rs(model);
        files.push(GeneratedFile {
            path: "handlers/mod.rs".into(),
            content: mod_content,
            checksum: String::new(),
        });

        let c_file_count = files.len();

        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "actix_handlers".to_string(),
                entity_count: model.entities.len(),
                c_file_count,
            },
        })
    }

    /// Generate handlers for a single entity with audit support via AST
    fn generate_entity_handlers(&self, entity: &GeneratorEntity) -> String {
        // 阶段 1: IR-2 → Rust AST
        let ast = entity_to_rust_handler_ast(entity);
        // 阶段 2: AST → String
        RustEmitter.emit(&ast).unwrap_or_default()
    }

    /// Generate route configuration via AST
    fn generate_route_config(&self, entity: &GeneratorEntity) -> String {
        // 阶段 1: IR-2 → Rust AST
        let ast = entity_to_rust_routes_ast(entity);
        // 阶段 2: AST → String
        RustEmitter.emit(&ast).unwrap_or_default()
    }

    /// Generate mod.rs for handlers via AST
    fn generate_mod_rs(&self, model: &GeneratorModel) -> String {
        // 阶段 1: IR-2 → Rust AST
        let ast = model_to_rust_handlers_mod_ast(model);
        // 阶段 2: AST → String
        RustEmitter.emit(&ast).unwrap_or_default()
    }
}

impl Default for ActixHandlerGenerator {
    fn default() -> Self {
        Self
    }
}

impl Generator for ActixHandlerGenerator {
    fn name(&self) -> &'static str {
        "actix_handlers"
    }

    fn generate(&self, model: &GeneratorModel) -> Result<GeneratedOutput, GenerateError> {
        self.generate_handlers(model)
    }

    fn validate(&self, _model: &GeneratorModel) -> Result<(), crate::generator::ValidationError> {
        Ok(())
    }

    fn supports_incremental(&self) -> bool {
        true
    }

    fn file_extensions(&self) -> Vec<&'static str> {
        vec!["rs"]
    }
}

impl IncrementalGenerator for ActixHandlerGenerator {
    fn generate_diff(
        &self,
        model: &GeneratorModel,
        existing_files: &[ExistingFile],
    ) -> Result<DiffReport, GenerateError> {
        let generated = self.generate_handlers(model)?;
        let existing_map: HashMap<_, _> =
            existing_files.iter().map(|f| (f.path.clone(), f)).collect();

        let mut files_changed = Vec::new();
        let mut files_added = Vec::new();

        for file in generated.files.iter() {
            if let Some(existing) = existing_map.get(&file.path) {
                if existing.content != file.content {
                    files_changed.push(FileChange {
                        path: file.path.clone(),
                        old_content: existing.content.clone(),
                        new_content: file.content.clone(),
                        diff: generate_unified_diff(&file.path, &existing.content, &file.content),
                    });
                }
            } else {
                files_added.push(file.clone());
            }
        }

        Ok(DiffReport {
            files_changed,
            files_added,
            files_removed: Vec::new(),
        })
    }

    fn detect_conflicts(
        &self,
        generated: &GeneratedOutput,
        existing: &[ExistingFile],
    ) -> Vec<MergeConflict> {
        let mut conflicts = Vec::new();

        for file in generated.files.iter() {
            if let Some(existing_file) = existing.iter().find(|e| e.path == file.path) {
                if let Some(conflict) =
                    self.detect_protected_region_conflict(&existing_file.content, &file.content)
                {
                    conflicts.push(MergeConflict {
                        path: file.path.clone(),
                        description: conflict,
                    });
                }
            }
        }

        conflicts
    }
}

impl ActixHandlerGenerator {
    fn detect_protected_region_conflict(
        &self,
        old_content: &str,
        new_content: &str,
    ) -> Option<String> {
        const PROTECTED_START: &str = "[! BEGIN PROTECTED]";
        const PROTECTED_END: &str = "[! END PROTECTED]";

        if old_content.contains(PROTECTED_START) && old_content.contains(PROTECTED_END) {
            let old_protected_regions = self.extract_protected_regions(old_content);
            let new_protected_regions = self.extract_protected_regions(new_content);

            for (region_id, old_region_content) in &old_protected_regions {
                if let Some(new_region_content) = new_protected_regions.get(region_id) {
                    if old_region_content != new_region_content {
                        return Some(format!(
                            "Protected region '{}' was modified in both old and new content",
                            region_id
                        ));
                    }
                } else {
                    return Some(format!(
                        "Protected region '{}' was removed in new content",
                        region_id
                    ));
                }
            }

            for region_id in new_protected_regions.keys() {
                if !old_protected_regions.contains_key(region_id) {
                    return Some(format!("New protected region '{}' was added", region_id));
                }
            }
        }

        None
    }

    fn extract_protected_regions(&self, content: &str) -> HashMap<String, String> {
        let mut regions = HashMap::new();
        const START: &str = "[! BEGIN PROTECTED]";
        const END: &str = "[! END PROTECTED]";

        let mut current_region_id: Option<String> = None;
        let mut current_content = String::new();

        for line in content.lines() {
            if line.contains(START) {
                if let Some(id) = line.split(START).nth(1) {
                    current_region_id = Some(id.trim().to_string());
                }
                current_content = String::new();
            } else if line.contains(END) {
                if let Some(id) = current_region_id.take() {
                    regions.insert(id, current_content.clone());
                }
            } else if current_region_id.is_some() {
                current_content.push_str(line);
                current_content.push('\n');
            }
        }

        regions
    }
}

fn generate_unified_diff(path: &Path, old_content: &str, new_content: &str) -> String {
    let diff = TextDiff::from_lines(old_content, new_content);
    let mut output = String::new();
    output.push_str(&format!(
        "--- a/{}\n+++ b/{}\n",
        path.display(),
        path.display()
    ));
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        output.push_str(&format!("{}{}", sign, change));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::{
        EntityName, FieldName, GeneratorField, GeneratorFieldType, PrimaryKeyType,
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
            description: None,
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
                unique: false,
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
    fn test_generate_handlers_with_audit() {
        let gen = ActixHandlerGenerator;
        let entity = create_test_entity();
        let code = gen.generate_entity_handlers(&entity);

        // Check that all handlers are generated
        assert!(code.contains("pub async fn list_user"));
        assert!(code.contains("pub async fn get_user"));
        assert!(code.contains("pub async fn create_user"));
        assert!(code.contains("pub async fn update_user"));
        assert!(code.contains("pub async fn delete_user"));
        assert!(code.contains("pub async fn hard_delete_user"));

        // Check audit support
        assert!(code.contains("extract_user_id"));
        assert!(code.contains("HttpRequest"));
        assert!(code.contains("fk_user"));
        assert!(code.contains("soft_delete"));
        assert!(code.contains("Repository"));
    }

    #[test]
    fn test_generate_route_config() {
        let gen = ActixHandlerGenerator;
        let entity = create_test_entity();
        let code = gen.generate_route_config(&entity);

        assert!(code.contains("web::scope(\"/users\")"));
        assert!(code.contains("hard_delete_user"));
    }

    #[test]
    fn test_handler_protected_region_detection() {
        let gen = ActixHandlerGenerator;

        let old_content = r#"//! Handlers for User

// [! BEGIN PROTECTED]
// Custom user logic
// [! END PROTECTED]
"#;

        let new_content = r#"//! Handlers for User

// [! BEGIN PROTECTED]
// Custom user logic modified
// [! END PROTECTED]
"#;

        let conflict = gen.detect_protected_region_conflict(old_content, new_content);
        assert!(conflict.is_some());
        assert!(conflict.unwrap().contains("modified"));
    }

    #[test]
    fn test_handler_protected_region_no_conflict() {
        let gen = ActixHandlerGenerator;

        let old_content = r#"// [! BEGIN PROTECTED custom_imports]
use actix_web::web;
// [! END PROTECTED custom_imports]
"#;

        let new_content = r#"// [! BEGIN PROTECTED custom_imports]
use actix_web::web;
// [! END PROTECTED custom_imports]
"#;

        let conflict = gen.detect_protected_region_conflict(old_content, new_content);
        assert!(conflict.is_none());
    }
}
