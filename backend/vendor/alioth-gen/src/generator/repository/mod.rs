//! Repository Generator
//!
//! Generates SQLx repository implementations from IR-2 models with audit and trigger support.

mod shared;
mod trigger_aware;

pub use trigger_aware::TriggerAwareRepositoryGenerator;

use crate::generator::ir::{GeneratorEntity, GeneratorModel};
use crate::generator::repository::shared::{
    error_enum, generate_repository_files, repository_header, shared_methods, EntityContext,
};
use crate::generator::{GenerateError, GeneratedOutput};
use crate::impl_repository_generator_defaults;

/// SQLx repository generator with audit support
pub struct SqlxRepositoryGenerator;

impl SqlxRepositoryGenerator {
    /// Generate repository modules for all entities
    pub fn generate_repositories(
        &self,
        model: &GeneratorModel,
    ) -> Result<GeneratedOutput, GenerateError> {
        generate_repository_files(model, "sqlx_repositories", |e| {
            self.generate_entity_repository(e)
        })
    }

    /// Generate thin adapter repository for a single entity
    fn generate_entity_repository(&self, entity: &GeneratorEntity) -> String {
        let ctx = EntityContext::from(entity);

        let header = repository_header(&ctx, "");
        let methods = shared_methods(&ctx);
        let error = error_enum(&ctx);

        let custom_methods = format!(
            r##"
    /// Create a new {entity_name}
    pub async fn create(
        &self,
        input: {entity_name}Input,
        fk_user: Option<i64>,
    ) -> Result<{entity_name}, {entity_name}Error> {{
        let now = chrono::Utc::now();
        sqlx::query_as::<_, {entity_name}>(
            "INSERT INTO {entity_plural} (created_at, updated_at, created_by_id, updated_by_id)
             VALUES ($1, $2, $3, $4)
             RETURNING *"
        )
        .bind(now)
        .bind(now)
        .bind(fk_user)
        .bind(fk_user)
        .fetch_one(self.generic.pool())
        .await
        .map_err({entity_name}Error::Database)
    }}

    /// Update an existing {entity_name}
    pub async fn update(
        &self,
        id: i64,
        input: {entity_name}Input,
        fk_user: Option<i64>,
    ) -> Result<Option<{entity_name}>, {entity_name}Error> {{
        sqlx::query_as::<_, {entity_name}>(
            "UPDATE {entity_plural}
             SET updated_at = NOW(), updated_by_id = $1
             WHERE id = $2 AND deleted_at IS NULL
             RETURNING *"
        )
        .bind(fk_user)
        .bind(id)
        .fetch_optional(self.generic.pool())
        .await
        .map_err({entity_name}Error::Database)
    }}

    /// Hard delete an {entity_name} (permanent removal)
    pub async fn hard_delete(&self, id: i64) -> Result<bool, {entity_name}Error> {{
        let result = sqlx::query(
            "DELETE FROM {entity_plural} WHERE id = $1"
        )
        .bind(id)
        .execute(self.generic.pool())
        .await
        .map_err({entity_name}Error::Database)?;

        Ok(result.rows_affected() > 0)
    }}
"##,
            entity_name = ctx.entity_name,
            entity_plural = ctx.entity_plural,
        );

        format!("{header}{custom_methods}{methods}\n{error}")
    }
}

impl_repository_generator_defaults!(SqlxRepositoryGenerator, "sqlx_repositories");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::{EntityName, GeneratorField, GeneratorFieldType, PrimaryKeyType};

    fn create_test_entity() -> GeneratorEntity {
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
            description: Some("Customer order".to_string()),
            fields: vec![GeneratorField {
                name: crate::generator::ir::FieldName {
                    raw: "total".to_string(),
                    snake: "total".to_string(),
                    camel: "total".to_string(),
                    pascal: "Total".to_string(),
                },
                field_type: GeneratorFieldType::Decimal,
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
    fn test_generate_repository_with_audit() {
        let gen = SqlxRepositoryGenerator;
        let entity = create_test_entity();
        let code = gen.generate_entity_repository(&entity);

        assert!(code.contains("pub struct OrderRepository"));
        assert!(code.contains("generic: GenericRepository<Order>"));
        assert!(code.contains("self.generic.list(&query)"));
        assert!(code.contains("self.generic.get(id)"));
        assert!(code.contains("self.generic.pool()"));
        assert!(code.contains("fk_user: Option<i64>"));
        assert!(code.contains("created_by_id"));
        assert!(code.contains("updated_by_id"));
        assert!(code.contains("deleted_by_id"));
        assert!(code.contains("soft_delete"));
    }
}
