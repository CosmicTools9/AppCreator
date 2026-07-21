//! Trigger-Aware Repository Generator
//!
//! Generates thin SQLx repository adapters with trigger support.
//! create/update/hard_delete delegate to `crud::trigger::*_with_triggers`.

use crate::generator::ir::{GeneratorEntity, GeneratorModel};
use crate::generator::repository::shared::{
    error_enum, generate_repository_files, repository_header, shared_methods, EntityContext,
};
use crate::generator::{GenerateError, GeneratedOutput};
use crate::impl_repository_generator_defaults;

/// Trigger-aware SQLx repository generator
pub struct TriggerAwareRepositoryGenerator;

impl TriggerAwareRepositoryGenerator {
    /// Generate repository modules for all entities with trigger support
    pub fn generate_repositories(
        &self,
        model: &GeneratorModel,
    ) -> Result<GeneratedOutput, GenerateError> {
        generate_repository_files(model, "trigger_aware_repositories", |e| {
            self.generate_entity_repository(e)
        })
    }

    /// Generate thin adapter repository for a single entity with trigger support
    fn generate_entity_repository(&self, entity: &GeneratorEntity) -> String {
        let ctx = EntityContext::from(entity);

        let extra_imports =
            "use crud::trigger;\nuse serde_json::Value;\nuse std::collections::HashMap;\n";
        let header = repository_header(&ctx, extra_imports);
        let methods = shared_methods(&ctx);
        let error = error_enum(&ctx);

        let custom_methods = format!(
            r##"
    /// Create a new {entity_name} with trigger execution
    pub async fn create(
        &self,
        input: {entity_name}Input,
        fk_user: Option<i64>,
    ) -> Result<{entity_name}, {entity_name}Error> {{
        let mut record: HashMap<String, Value> = serde_json::from_value(
            serde_json::to_value(&input).map_err(|e| {entity_name}Error::Alioth(e.to_string()))?
        ).map_err(|e| {entity_name}Error::Alioth(e.to_string()))?;
        record.insert("created_by_id".to_string(), serde_json::json!(fk_user));
        record.insert("updated_by_id".to_string(), serde_json::json!(fk_user));

        let result_map = trigger::insert_with_triggers(
            self.generic.pool(),
            "{entity_plural}",
            record,
            fk_user,
        ).await.map_err(|e| {entity_name}Error::Alioth(e.to_string()))?;

        serde_json::from_value(serde_json::to_value(&result_map).unwrap_or_default())
            .map_err(|e| {entity_name}Error::Alioth(e.to_string()))
    }}

    /// Update an existing {entity_name} with trigger execution
    pub async fn update(
        &self,
        id: i64,
        input: {entity_name}Input,
        fk_user: Option<i64>,
    ) -> Result<Option<{entity_name}>, {entity_name}Error> {{
        let old = self.get_by_id(id).await?;
        let old_record = match old {{
            Some(ref e) => trigger::to_record(e).map_err(|e| {entity_name}Error::Alioth(e.to_string()))?,
            None => return Ok(None),
        }};

        let mut record: HashMap<String, Value> = serde_json::from_value(
            serde_json::to_value(&input).map_err(|e| {entity_name}Error::Alioth(e.to_string()))?
        ).map_err(|e| {entity_name}Error::Alioth(e.to_string()))?;
        record.insert("updated_by_id".to_string(), serde_json::json!(fk_user));

        let result_map = trigger::update_with_triggers(
            self.generic.pool(),
            "{entity_plural}",
            id,
            record,
            &old_record,
            fk_user,
        ).await.map_err(|e| {entity_name}Error::Alioth(e.to_string()))?;

        serde_json::from_value(serde_json::to_value(&result_map).unwrap_or_default())
            .map_err(|e| {entity_name}Error::Alioth(e.to_string()))
    }}

    /// Hard delete an {entity_name} with trigger execution
    pub async fn hard_delete(&self, id: i64) -> Result<bool, {entity_name}Error> {{
        trigger::delete_with_triggers(
            self.generic.pool(),
            "{entity_plural}",
            id,
            None,
        ).await.map_err(|e| {entity_name}Error::Alioth(e.to_string()))
    }}
"##,
            entity_name = ctx.entity_name,
            entity_plural = ctx.entity_plural,
        );

        format!("{header}{custom_methods}{methods}\n{error}")
    }
}

impl_repository_generator_defaults!(
    TriggerAwareRepositoryGenerator,
    "trigger_aware_repositories"
);

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
    fn test_generate_trigger_aware_repository() {
        let gen = TriggerAwareRepositoryGenerator;
        let entity = create_test_entity();
        let code = gen.generate_entity_repository(&entity);

        assert!(code.contains("pub struct OrderRepository"));
        assert!(code.contains("generic: GenericRepository<Order>"));
        assert!(code.contains("self.generic.list(&query)"));
        assert!(code.contains("self.generic.get(id)"));
        assert!(code.contains("trigger::insert_with_triggers"));
        assert!(code.contains("trigger::update_with_triggers"));
        assert!(code.contains("trigger::delete_with_triggers"));
        assert!(code.contains("trigger::to_record"));
        assert!(code.contains("self.generic.pool()"));
    }
}
