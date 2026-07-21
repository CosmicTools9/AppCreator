//! Zod Validation Generator
//!
//! Generates TypeScript validation schemas using Zod from IR-2 models.
//!
//! ## Features
//!
//! - **Type-safe validation**: Full TypeScript type inference from Zod schemas
//! - **React Hook Form integration**: Compatible with react-hook-form + zodResolver
//! - **Rich validation rules**: Length, pattern, range, email, URL validations
//! - **Coercion support**: Automatic type coercion for form inputs
//! - **Custom error messages**: i18n-ready error messages
//!
//! ## Generated Files
//!
//! For each entity, the following files are generated:
//! - `{entity}.schema.ts` - Zod schema and TypeScript types
//! - `hooks/use{Entity}Form.ts` - React Hook Form integration
//! - `hooks/use{Entity}Api.ts` - TanStack Query hooks
//!
//! ## Example Usage
//!
//! ```typescript
//! import { UserSchema, UserInput } from './User.schema';
//! import { useUserForm } from './hooks/useUserForm';
//!
//! // Type inference
//! type User = z.infer<typeof UserSchema>;
//!
//! // Form usage
//! const { register, handleSubmit, formState: { errors } } = useUserForm({
//!   onSubmit: async (data) => {
//!     await api.createUser(data);
//!   }
//! });
//! ```

mod hooks;
mod mappings;
mod typescript;
mod zod_gen;

pub use hooks::HookGenerator;
pub use hooks::ReactHookFormGenerator;
pub use mappings::ZodTypeMapper;
pub use typescript::TypeScriptGenerator;
pub use zod_gen::FullZodGenerator;
pub use zod_gen::ZodGenerator;

use crate::generator::ir::GeneratorModel;
use crate::generator::{
    GenerateError, GeneratedFile, GeneratedOutput, GenerationMetadata, Generator,
};

/// Combined Zod schema generator
pub struct ZodSchemaGenerator;

impl ZodSchemaGenerator {
    /// Generate all schema files for a model
    pub fn generate_all(&self, model: &GeneratorModel) -> Result<GeneratedOutput, GenerateError> {
        let mut files = Vec::new();

        // Generate schema file for each entity
        for entity in &model.entities {
            let zod_gen = ZodGenerator::new(entity);
            let schema_content = zod_gen.generate_schema_file();

            files.push(GeneratedFile {
                path: format!("schemas/{}.schema.ts", entity.name.kebab).into(),
                content: schema_content,
                checksum: String::new(),
            });
        }

        // Generate shared types file (once for all entities)
        if !model.entities.is_empty() {
            let types_content = TypeScriptGenerator::generate_types_file(model);
            files.push(GeneratedFile {
                path: "schemas/types.ts".into(),
                content: types_content,
                checksum: String::new(),
            });
        }

        // Index file
        let exports: Vec<_> = model
            .entities
            .iter()
            .map(|e| format!("export * from './{}.schema';", e.name.kebab))
            .collect();

        files.push(GeneratedFile {
            path: "schemas/index.ts".into(),
            content: exports.join("\n"),
            checksum: String::new(),
        });

        let c_file_count = files.len();

        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "zod_schema".to_string(),
                entity_count: model.entities.len(),
                c_file_count,
            },
        })
    }
}

impl Default for ZodSchemaGenerator {
    fn default() -> Self {
        Self
    }
}

impl Generator for ZodSchemaGenerator {
    fn name(&self) -> &'static str {
        "zod_schema"
    }

    fn generate(&self, model: &GeneratorModel) -> Result<GeneratedOutput, GenerateError> {
        self.generate_all(model)
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
    use crate::generator::ir::{
        EntityName, FieldName, GeneratorEntity, GeneratorField, GeneratorFieldType, PrimaryKeyType,
    };
    use std::path::PathBuf;

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
                fields: vec![GeneratorField {
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
                }],
                relations: vec![],
                annotations: vec![],
                primary_key_type: PrimaryKeyType::BigInt,
                ..Default::default()
            }],
            enums: vec![],
            metadata: crate::generator::ir::ModelMetadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                generator_version: "test".to_string(),
            },
            exceptions: vec![],
            exception_handlers: vec![],
            external_dependencies: vec![],
        }
    }

    #[test]
    fn test_zod_schema_generator() {
        let model = create_test_model();
        let generator = ZodSchemaGenerator;
        let output = generator.generate(&model).unwrap();

        assert_eq!(output.files.len(), 3); // schema, types, index
        assert!(output
            .files
            .iter()
            .any(|f| f.path == PathBuf::from("schemas/user.schema.ts")));
        assert!(output
            .files
            .iter()
            .any(|f| f.path == PathBuf::from("schemas/types.ts")));
        assert!(output
            .files
            .iter()
            .any(|f| f.path == PathBuf::from("schemas/index.ts")));
    }
}
