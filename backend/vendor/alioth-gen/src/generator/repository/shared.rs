//! Shared repository generator utilities
//!
//! Extracts common template helpers between [`SqlxRepositoryGenerator`] and
//! [`TriggerAwareRepositoryGenerator`], eliminating ~60 % of duplication.

use crate::generator::ir::{GeneratorEntity, GeneratorModel, PrimaryKeyType};
use crate::generator::{GenerateError, GeneratedFile, GeneratedOutput, GenerationMetadata};

/// Per-entity naming and type context extracted from [`GeneratorEntity`].
pub struct EntityContext<'a> {
    pub entity_name: &'a str,
    pub entity_snake: &'a str,
    pub entity_plural: &'a str,
    pub id_type: &'static str,
}

impl<'a> EntityContext<'a> {
    pub fn from(entity: &'a GeneratorEntity) -> Self {
        Self {
            entity_name: &entity.name.pascal,
            entity_snake: &entity.name.snake,
            entity_plural: &entity.name.plural_snake,
            id_type: match entity.primary_key_type {
                PrimaryKeyType::BigInt => "i64",
                PrimaryKeyType::Uuid => "Uuid",
            },
        }
    }
}

/// Shared file-generation loop: produces one `.rs` per entity plus `mod.rs`.
pub fn generate_repository_files(
    model: &GeneratorModel,
    generator_name: &str,
    entity_generator: impl Fn(&GeneratorEntity) -> String,
) -> Result<GeneratedOutput, GenerateError> {
    let mut files = Vec::new();

    for entity in &model.entities {
        let repo_code = entity_generator(entity);
        files.push(GeneratedFile {
            path: format!("repositories/{}.rs", entity.name.snake).into(),
            content: repo_code,
            checksum: String::new(),
        });
    }

    let mod_content = generate_mod_rs(model);
    files.push(GeneratedFile {
        path: "repositories/mod.rs".into(),
        content: mod_content,
        checksum: String::new(),
    });

    let c_file_count = files.len();

    Ok(GeneratedOutput {
        files,
        metadata: GenerationMetadata {
            generator_name: generator_name.to_string(),
            entity_count: model.entities.len(),
            c_file_count,
        },
    })
}

/// Shared `repositories/mod.rs` generator.
pub fn generate_mod_rs(model: &GeneratorModel) -> String {
    let mut modules = Vec::new();
    let mut exports = Vec::new();

    for entity in &model.entities {
        modules.push(format!("pub mod {};", entity.name.snake));
        exports.push(format!(
            "pub use {}::{{{}Repository, {}Error}};",
            entity.name.snake, entity.name.pascal, entity.name.pascal,
        ));
    }

    format!(
        "//! Auto-generated Repositories\n\n{}\n\n{}",
        modules.join("\n"),
        exports.join("\n")
    )
}

/// Header shared by every thin adapter:
///
/// * `use` statements
/// * `{Entity}Repository` struct wrapping `GenericRepository<{Entity}>`
/// * `new(pool)` constructor
/// * `From<PgPool>` impl
///
/// `extra_imports` is appended right after the standard `use` block.
pub fn repository_header(ctx: &EntityContext, extra_imports: &str) -> String {
    let EntityContext {
        entity_name,
        entity_snake,
        entity_plural,
        ..
    } = ctx;

    format!(
        r##"//! Repository for {entity_name}

use common::AliothError as ApiError;
use crud::{{GenericRepository, ListQuery, PaginatedResponse}};
{extra_imports}use crate::models::{entity_snake}::{{{entity_name}, {entity_name}Input}};
use sqlx::{{AssertSqlSafe, PgPool}};

#[derive(Clone, Debug)]
pub struct {entity_name}Repository {{
    generic: GenericRepository<{entity_name}>,
}}

impl {entity_name}Repository {{
    /// Create a new repository instance
    pub fn new(pool: PgPool) -> Self {{
        Self {{
            generic: GenericRepository::new(pool),
        }}
    }}
}}

impl From<PgPool> for {entity_name}Repository {{
    fn from(pool: PgPool) -> Self {{
        Self::new(pool)
    }}
}}

impl {entity_name}Repository {{
    /// List all {entity_plural} (excluding soft deleted)
    pub async fn list(&self) -> Result<Vec<{entity_name}>, {entity_name}Error> {{
        let query = ListQuery {{
            page: 1,
            page_size: 10_000,
            filter_field: None,
            filter_op: None,
            filter_value: None,
            sort_field: None,
            sort_order: None,
        }};
        let resp = self.generic.list(&query).await.map_err({entity_name}Error::from)?;
        Ok(resp.items)
    }}

    /// Get {entity_name} by ID (excluding soft deleted)
    pub async fn get_by_id(&self, id: i64) -> Result<Option<{entity_name}>, {entity_name}Error> {{
        self.generic.get(id).await.map_err({entity_name}Error::from)
    }}
"##,
        entity_name = entity_name,
        entity_snake = entity_snake,
        entity_plural = entity_plural,
        extra_imports = extra_imports,
    )
}

/// Methods that are identical in both Sqlx and Trigger-aware adapters:
/// `soft_delete`, `restore`, `list_deleted`.
pub fn shared_methods(ctx: &EntityContext) -> String {
    let EntityContext {
        entity_name,
        entity_plural,
        id_type,
        ..
    } = ctx;

    format!(
        r##"
    /// Soft delete an {entity_name}
    pub async fn soft_delete(
        &self,
        id: {id_type},
        fk_user: Option<i64>,
    ) -> Result<bool, {entity_name}Error> {{
        let result = sqlx::query(
            "UPDATE {entity_plural}
             SET deleted_at = NOW(), deleted_by_id = $1, updated_by_id = $2
             WHERE id = $3 AND deleted_at IS NULL"
        )
        .bind(fk_user)
        .bind(fk_user)
        .bind(id)
        .execute(self.generic.pool())
        .await
        .map_err({entity_name}Error::Database)?;

        Ok(result.rows_affected() > 0)
    }}

    /// Restore a soft-deleted {entity_name}
    pub async fn restore(
        &self,
        id: {id_type},
        fk_user: Option<i64>,
    ) -> Result<Option<{entity_name}>, {entity_name}Error> {{
        sqlx::query_as::<_, {entity_name}>(
            "UPDATE {entity_plural}
             SET deleted_at = NULL, deleted_by_id = NULL, updated_by_id = $1, updated_at = NOW()
             WHERE id = $2 AND deleted_at IS NOT NULL
             RETURNING *"
        )
        .bind(fk_user)
        .bind(id)
        .fetch_optional(self.generic.pool())
        .await
        .map_err({entity_name}Error::Database)
    }}

    /// List soft-deleted {entity_plural}
    pub async fn list_deleted(&self) -> Result<Vec<{entity_name}>, {entity_name}Error> {{
        sqlx::query_as::<_, {entity_name}>(
            "SELECT * FROM {entity_plural} WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC"
        )
        .fetch_all(self.generic.pool())
        .await
        .map_err({entity_name}Error::Database)
    }}
}}
"##,
        entity_name = entity_name,
        entity_plural = entity_plural,
        id_type = id_type,
    )
}

/// Error enum + `From<common::AliothError>` impl.
pub fn error_enum(ctx: &EntityContext) -> String {
    let EntityContext {
        entity_name,
        id_type,
        ..
    } = ctx;

    format!(
        r##"/// Repository errors
#[derive(Debug, thiserror::Error)]
pub enum {entity_name}Error {{
    #[error("Database error: {{0}}")]
    Database(#[from] sqlx::Error),
    #[error("Alioth error: {{0}}")]
    Alioth(String),
    #[error("Entity not found: {{0}}")]
    NotFound({id_type}),
    #[error("Validation error: {{0}}")]
    Validation(String),
}}

impl From<common::AliothError> for {entity_name}Error {{
    fn from(err: common::AliothError) -> Self {{
        {entity_name}Error::Alioth(err.to_string())
    }}
}}
"##,
        entity_name = entity_name,
        id_type = id_type,
    )
}

/// Convenience macro for `impl Default + impl Generator` boilerplate.
#[macro_export]
macro_rules! impl_repository_generator_defaults {
    ($type:ty, $name:literal) => {
        impl Default for $type {
            fn default() -> Self {
                Self
            }
        }

        impl $crate::generator::Generator for $type {
            fn name(&self) -> &'static str {
                $name
            }

            fn generate(
                &self,
                model: &$crate::generator::ir::GeneratorModel,
            ) -> Result<$crate::generator::GeneratedOutput, $crate::generator::GenerateError> {
                self.generate_repositories(model)
            }

            fn validate(
                &self,
                _model: &$crate::generator::ir::GeneratorModel,
            ) -> Result<(), $crate::generator::ValidationError> {
                Ok(())
            }

            fn supports_incremental(&self) -> bool {
                false
            }

            fn file_extensions(&self) -> Vec<&'static str> {
                vec!["rs"]
            }
        }
    };
}
