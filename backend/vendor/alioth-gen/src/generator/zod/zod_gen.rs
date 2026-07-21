//! Zod Schema Generator
//!
//! Generates Zod validation schemas from IR-2 models.
//!
//! **架构**: 两阶段生成
//! 1. IR-2 → TypeScript AST (`generator::ast::transform::entity_to_ts_zod_ast`)
//! 2. AST → String (`TypeScriptEmitter`)

use crate::generator::ast::transform::entity_to_ts_zod_ast;
use crate::generator::ast::ts::TypeScriptEmitter;
use crate::generator::ast::AstEmitter;
use crate::generator::ir::{GeneratorEntity, GeneratorFieldType, GeneratorModel, PrimaryKeyType};
use crate::generator::{
    GenerateError, GeneratedFile, GeneratedOutput, GenerationMetadata, Generator,
};

use super::mappings::ZodTypeMapper;

/// Zod schema generator for a single entity
pub struct ZodGenerator<'a> {
    entity: &'a GeneratorEntity,
    type_mapper: ZodTypeMapper,
}

impl<'a> ZodGenerator<'a> {
    /// Create a new Zod generator for an entity
    pub fn new(entity: &'a GeneratorEntity) -> Self {
        Self {
            entity,
            type_mapper: ZodTypeMapper::new(),
        }
    }

    /// Generate the complete schema file content via AST intermediate
    pub fn generate_schema_file(&self) -> String {
        // 阶段 1: IR-2 → AST
        let ast = entity_to_ts_zod_ast(self.entity);
        // 阶段 2: AST → String
        TypeScriptEmitter.emit(&ast).unwrap_or_default()
    }

    /// Generate import statements (legacy direct-string API, kept for compatibility)
    #[allow(dead_code)]
    fn generate_imports(&self) -> String {
        "import { z } from 'zod';".to_string()
    }

    /// Generate enum type definitions (legacy direct-string API)
    #[allow(dead_code)]
    fn generate_enum_types(&self) -> Vec<String> {
        self.entity
            .fields
            .iter()
            .filter_map(|f| match &f.field_type {
                GeneratorFieldType::Enum(name) => Some(format!(
                    "export enum {} {{\n  // Define enum values here\n}}",
                    name
                )),
                _ => None,
            })
            .collect()
    }

    /// Generate Zod schema for the entity (legacy direct-string API)
    #[allow(dead_code)]
    fn generate_zod_schema(&self) -> String {
        let entity_name = &self.entity.name.pascal;
        let mut field_schemas = Vec::new();

        // ID field
        let id_schema = match self.entity.primary_key_type {
            PrimaryKeyType::BigInt => "z.bigint()",
            PrimaryKeyType::Uuid => "z.string().uuid()",
        };
        field_schemas.push(format!("  id: {},", id_schema));

        // Regular fields
        for field in &self.entity.fields {
            if field.name.snake == "id" {
                continue;
            }

            let schema = self.type_mapper.map_field(field);

            field_schemas.push(format!("  {}: {}", field.name.camel, schema));
        }

        // System fields
        field_schemas.push("  // System fields".to_string());
        field_schemas.push("  createdAt: z.date(),".to_string());
        field_schemas.push("  updatedAt: z.date(),".to_string());

        let description = self
            .entity
            .description
            .as_ref()
            .map(|d| format!("/** {} */\n", d))
            .unwrap_or_default();

        format!(
            "{}export const {}Schema = z.object({{\n{}\n}});",
            description,
            entity_name,
            field_schemas.join("\n")
        )
    }

    /// Generate input schema (for create/update) (legacy direct-string API)
    #[allow(dead_code)]
    fn generate_input_schema(&self) -> String {
        let entity_name = &self.entity.name.pascal;
        let mut field_schemas = Vec::new();

        for field in &self.entity.fields {
            // Skip id and system fields
            if field.name.snake == "id"
                || field.name.snake.starts_with("created_")
                || field.name.snake.starts_with("updated_")
            {
                continue;
            }

            let schema = self.type_mapper.map_field(field);
            let description = field
                .description
                .as_ref()
                .map(|d| format!(" // {}", d))
                .unwrap_or_default();

            field_schemas.push(format!("  {}{}:{}", field.name.camel, description, schema));
        }

        format!(
            "export const {}InputSchema = z.object({{\n{}\n}});",
            entity_name,
            field_schemas.join("\n")
        )
    }

    /// Generate type inference helpers (legacy direct-string API)
    #[allow(dead_code)]
    fn generate_type_helpers(&self) -> String {
        let entity_name = &self.entity.name.pascal;

        format!(
            "// Type inference\n\
             export type {} = z.infer<typeof {}Schema>;\n\
             export type {}Input = z.infer<typeof {}InputSchema>;\n\n\
             // Safe parse helpers\n\
             export const parse{} = (data: unknown) => {}Schema.safeParse(data);\n\
             export const parse{}Input = (data: unknown) => {}InputSchema.safeParse(data);",
            entity_name,
            entity_name,
            entity_name,
            entity_name,
            entity_name,
            entity_name,
            entity_name,
            entity_name
        )
    }

    /// Generate default values for forms
    pub fn generate_default_values(&self) -> String {
        let defaults = self
            .type_mapper
            .generate_default_values(&self.entity.fields);

        let fields: Vec<_> = defaults
            .iter()
            .map(|(k, v)| format!("  {}: {}", k, v))
            .collect();

        format!(
            "export const {}Defaults = {{\n{}\n}};",
            self.entity.name.pascal,
            fields.join(",\n")
        )
    }
}

/// Full model Zod generator
pub struct FullZodGenerator;

impl FullZodGenerator {
    /// Generate all schema files for a model
    pub fn generate(model: &GeneratorModel) -> Result<GeneratedOutput, GenerateError> {
        let mut files = Vec::new();

        // Generate individual entity schemas
        for entity in &model.entities {
            let generator = ZodGenerator::new(entity);
            let content = generator.generate_schema_file();

            files.push(GeneratedFile {
                path: format!("schemas/{}.schema.ts", entity.name.kebab).into(),
                content,
                checksum: String::new(),
            });
        }

        // Generate index file
        let index_content = model
            .entities
            .iter()
            .map(|e| format!("export * from './{}.schema';", e.name.kebab))
            .collect::<Vec<_>>()
            .join("\n");

        files.push(GeneratedFile {
            path: "schemas/index.ts".into(),
            content: index_content,
            checksum: String::new(),
        });

        let c_file_count = files.len();

        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "zod".to_string(),
                entity_count: model.entities.len(),
                c_file_count,
            },
        })
    }
}

impl Generator for FullZodGenerator {
    fn name(&self) -> &'static str {
        "zod_full"
    }

    fn generate(&self, model: &GeneratorModel) -> Result<GeneratedOutput, GenerateError> {
        FullZodGenerator::generate(model)
    }

    fn validate(&self, _model: &GeneratorModel) -> Result<(), crate::generator::ValidationError> {
        Ok(())
    }

    fn supports_incremental(&self) -> bool {
        false
    }

    fn file_extensions(&self) -> Vec<&'static str> {
        vec!["ts"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::{EntityName, FieldName, GeneratorField};

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
            primary_key_type: PrimaryKeyType::BigInt,
            ..Default::default()
        }
    }

    #[test]
    fn test_generate_imports() {
        let entity = create_test_entity();
        let generator = ZodGenerator::new(&entity);
        let imports = generator.generate_imports();

        assert!(imports.contains("zod"));
    }

    #[test]
    fn test_generate_zod_schema() {
        let entity = create_test_entity();
        let generator = ZodGenerator::new(&entity);
        let schema = generator.generate_zod_schema();

        assert!(schema.contains("export const UserSchema"));
        assert!(schema.contains("id: z.bigint()"));
        assert!(schema.contains("email: z.string()"));
        assert!(schema.contains("createdAt: z.date()"));
        assert!(schema.contains("updatedAt: z.date()"));
    }

    #[test]
    fn test_generate_input_schema() {
        let entity = create_test_entity();
        let generator = ZodGenerator::new(&entity);
        let schema = generator.generate_input_schema();

        assert!(schema.contains("export const UserInputSchema"));
        assert!(!schema.contains("id:")); // Should not include id
        assert!(!schema.contains("createdAt:")); // Should not include system fields
    }

    #[test]
    fn test_generate_type_helpers() {
        let entity = create_test_entity();
        let generator = ZodGenerator::new(&entity);
        let helpers = generator.generate_type_helpers();

        assert!(helpers.contains("export type User"));
        assert!(helpers.contains("export type UserInput"));
        assert!(helpers.contains("parseUser"));
        assert!(helpers.contains("parseUserInput"));
    }
}
