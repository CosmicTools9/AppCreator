//! MetaModel → GeneratorModel Transformer
//!
//! 将数据库 `meta_collections` / `meta_fields` 的原始查询结果统一转换为
//! `GeneratorModel` (IR-2)。所有从数据库 schema 出发的生成器均通过该
//! transformer 消费 IR-2，避免在 HTTP handler 中硬编码字段映射。

use super::{
    naming, GeneratorEntity, GeneratorField, GeneratorModel, ModelMetadata, PrimaryKeyType,
};

/// 系统自动维护的字段名称列表，生成代码时应排除
/// 对应 `ontology_spec.md` §4.4 的 🚫 全排除 / 🔒 全排除 类别
const SYSTEM_EXCLUDED_FIELDS: &[&str] = &[
    // 框架系统
    "id",
    "created_at",
    "updated_at",
    "deleted_at",
    "created_by_id",
    "updated_by_id",
    "deleted_by_id",
    // 维度派生
    "o_number",
    "number",
    "domain_",
    "_f_",
    "_t_",
    "majority",
    "sprint",
    "model",
    "p_number",
    // 应用绑定
    "dk_scene",
    "dk_factor",
    "dk_function",
    // 触发器维护
    "d_count",
    "ref_count",
    "ak_dimensions",
    "ak_components",
    "paths",
    "projection",
];

/// 从 `meta_collections` 表查询得到的原始 Collection 信息。
///
/// 与数据库 schema 解耦的纯 Rust 结构体，便于单元测试与复用。
#[derive(Debug, Clone)]
pub struct MetaCollection {
    pub name: String,
    pub description: Option<String>,
    pub inherits: Vec<String>,
}

/// 从 `meta_fields` 表查询得到的原始 Field 信息。
///
/// `config` 中的 `description`、`unique`、`indexed` 等属性应在构造本结构体时
/// 提前解析并扁平化，保持 transformer 内部无 `serde_json::Value` 依赖。
#[derive(Debug, Clone)]
pub struct MetaField {
    pub name: String,
    pub data_type: String,
    pub is_required: bool,
    pub default_value: Option<String>,
    pub description: Option<String>,
    pub unique: bool,
    pub indexed: bool,
}

/// 统一转换器：将数据库层面的 Meta 数据转换为代码生成器模型 (IR-2)。
pub struct MetaModelTransformer;

impl MetaModelTransformer {
    /// 将单个 `MetaCollection` 及其字段列表转换为 `GeneratorModel`。
    ///
    /// 当前实现为单实体模型；未来若需支持多 Collection 可扩展为
    /// `transform_many(collections: &[(MetaCollection, Vec<MetaField>)])`。
    pub fn transform(collection: &MetaCollection, fields: &[MetaField]) -> GeneratorModel {
        let entity = Self::collection_to_entity(collection, fields);

        GeneratorModel {
            i18n_config: None,
            entities: vec![entity],
            enums: vec![],
            metadata: ModelMetadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                generator_version: crate::ALIOTH_STUDIO_VERSION.clone(),
            },
            exceptions: vec![],
            exception_handlers: vec![],
            external_dependencies: vec![],
        }
    }

    // ── Internal helpers ───────────────────────────────────────────────────

    fn collection_to_entity(collection: &MetaCollection, fields: &[MetaField]) -> GeneratorEntity {
        let entity_name = naming::build_entity_name(&collection.name);
        let generator_fields: Vec<GeneratorField> = fields
            .iter()
            .filter(|f| !SYSTEM_EXCLUDED_FIELDS.contains(&f.name.as_str()))
            .map(Self::field_to_generator_field)
            .collect();

        GeneratorEntity {
            name: entity_name,
            description: collection.description.clone(),
            fields: generator_fields,
            relations: vec![],
            annotations: vec![],
            primary_key_type: PrimaryKeyType::BigInt,
            parent_classes: vec![],
            equivalent_classes: vec![],
            disjoint_classes: vec![],
            is_abstract: false,
            inheritance_depth: 0,
            state_machine: Default::default(),
            transitions: vec![],
            lifecycle_hooks: vec![],
            business_rules: vec![],
            swrl_rules: vec![],
            constraints: vec![],
            quality_rules: vec![],
            quality_config: Default::default(),
            parent_tables: collection.inherits.clone(),
        }
    }

    fn field_to_generator_field(field: &MetaField) -> GeneratorField {
        let field_name = naming::build_field_name(&field.name);
        let field_type = naming::map_field_type(&field.data_type);

        GeneratorField {
            name: field_name,
            field_type,
            description: field.description.clone(),
            nullable: !field.is_required,
            unique: field.unique,
            indexed: field.indexed,
            default_value: field.default_value.clone(),
            validations: vec![],
            annotations: vec![],
            domain: None,
            range: None,
            min_cardinality: None,
            max_cardinality: None,
            is_functional: false,
            constraints: vec![],
            throws_clauses: vec![],
            quality_rules: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::GeneratorFieldType;

    #[test]
    fn test_transform_single_collection() {
        let collection = MetaCollection {
            name: "Product".to_string(),
            description: Some("Product catalog".to_string()),
            inherits: vec![],
        };
        let fields = vec![
            MetaField {
                name: "sku".to_string(),
                data_type: "text".to_string(),
                is_required: true,
                default_value: None,
                description: Some("Stock keeping unit".to_string()),
                unique: true,
                indexed: false,
            },
            MetaField {
                name: "price".to_string(),
                data_type: "decimal".to_string(),
                is_required: true,
                default_value: Some("0.00".to_string()),
                description: None,
                unique: false,
                indexed: false,
            },
        ];

        let model = MetaModelTransformer::transform(&collection, &fields);
        assert_eq!(model.entities.len(), 1);

        let entity = &model.entities[0];
        assert_eq!(entity.name.raw, "Product");
        assert_eq!(entity.name.snake, "product");
        assert_eq!(entity.description, Some("Product catalog".to_string()));

        assert_eq!(entity.fields.len(), 2);
        assert_eq!(entity.fields[0].name.raw, "sku");
        assert!(matches!(
            entity.fields[0].field_type,
            GeneratorFieldType::Text
        ));
        assert!(!entity.fields[0].nullable);
        assert!(entity.fields[0].unique);

        assert_eq!(entity.fields[1].name.raw, "price");
        assert!(matches!(
            entity.fields[1].field_type,
            GeneratorFieldType::Decimal
        ));
        assert_eq!(entity.fields[1].default_value, Some("0.00".to_string()));
    }

    #[test]
    fn test_transform_unknown_field_type_defaults_to_text() {
        let collection = MetaCollection {
            name: "Log".to_string(),
            description: None,
            inherits: vec![],
        };
        let fields = vec![MetaField {
            name: "payload".to_string(),
            data_type: "unknown_custom_type".to_string(),
            is_required: false,
            default_value: None,
            description: None,
            unique: false,
            indexed: false,
        }];

        let model = MetaModelTransformer::transform(&collection, &fields);
        assert!(matches!(
            model.entities[0].fields[0].field_type,
            GeneratorFieldType::Text
        ));
    }

    #[test]
    fn test_transform_empty_fields() {
        let collection = MetaCollection {
            name: "Audit".to_string(),
            description: None,
            inherits: vec![],
        };
        let model = MetaModelTransformer::transform(&collection, &[]);
        assert_eq!(model.entities[0].fields.len(), 0);
        assert_eq!(model.entities[0].name.plural_snake, "audits");
    }
}
