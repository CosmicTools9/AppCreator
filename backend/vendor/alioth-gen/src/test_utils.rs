//! 测试工具函数
//!
//! 统一提供测试用的工厂函数

use crate::generator::ir::ModelMetadata;
use crate::{
    EntityName, FieldName, GeneratorEntity, GeneratorEnum, GeneratorField, GeneratorFieldType,
    GeneratorModel, GeneratorStateMachine, PrimaryKeyType,
};

/// 创建测试用的 IR-2 实体
pub fn create_test_entity(
    name: &str,
    parent_classes: Vec<&str>,
    fields: Vec<GeneratorField>,
) -> GeneratorEntity {
    GeneratorEntity {
        name: EntityName {
            raw: name.to_string(),
            snake: name.to_lowercase(),
            camel: name.to_lowercase(),
            pascal: name.to_string(),
            kebab: name.to_lowercase(),
            screaming_snake: name.to_uppercase(),
            plural_snake: format!("{}s", name.to_lowercase()),
            plural_pascal: format!("{}s", name),
            plural_kebab: format!("{}s", name.to_lowercase()),
        },
        description: None,
        fields,
        relations: vec![],
        annotations: vec![],
        primary_key_type: Default::default(),
        parent_classes: parent_classes
            .into_iter()
            .map(|p| EntityName {
                raw: p.to_string(),
                snake: p.to_lowercase(),
                camel: p.to_lowercase(),
                pascal: p.to_string(),
                kebab: p.to_lowercase(),
                screaming_snake: p.to_uppercase(),
                plural_snake: format!("{}s", p.to_lowercase()),
                plural_pascal: format!("{}s", p),
                plural_kebab: format!("{}s", p.to_lowercase()),
            })
            .collect(),
        equivalent_classes: vec![],
        disjoint_classes: vec![],
        is_abstract: false,
        inheritance_depth: 0,
        state_machine: GeneratorStateMachine::default(),
        transitions: vec![],
        lifecycle_hooks: vec![],
        business_rules: vec![],
        swrl_rules: vec![],
        constraints: vec![],
        quality_rules: vec![],
        quality_config: Default::default(),
        parent_tables: vec![],
    }
}

/// 创建测试用的 IR-2 字段
pub fn create_test_field(name: &str, nullable: bool) -> GeneratorField {
    GeneratorField {
        name: FieldName {
            raw: name.to_string(),
            snake: name.to_lowercase(),
            camel: name.to_lowercase(),
            pascal: name.to_string(),
        },
        field_type: GeneratorFieldType::Text,
        description: None,
        nullable,
        unique: false,
        indexed: false,
        default_value: None,
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

/// 创建测试用的 IR 模型
pub fn create_test_model() -> GeneratorModel {
    GeneratorModel {
        i18n_config: None,
        entities: vec![GeneratorEntity {
            name: EntityName {
                raw: "Customer".to_string(),
                snake: "customers".to_string(),
                camel: "customers".to_string(),
                pascal: "Customers".to_string(),
                kebab: "customers".to_string(),
                screaming_snake: "CUSTOMERS".to_string(),
                plural_snake: "customers".to_string(),
                plural_pascal: "Customers".to_string(),
                plural_kebab: "customers".to_string(),
            },
            description: Some("客户信息表".to_string()),
            fields: vec![
                GeneratorField {
                    name: FieldName {
                        raw: "name".to_string(),
                        snake: "name".to_string(),
                        camel: "name".to_string(),
                        pascal: "Name".to_string(),
                    },
                    field_type: GeneratorFieldType::Text,
                    description: Some("客户名称".to_string()),
                    nullable: false,
                    unique: false,
                    indexed: false,
                    default_value: None,
                    validations: vec![],
                    annotations: vec![],
                    ..Default::default()
                },
                GeneratorField {
                    name: FieldName {
                        raw: "email".to_string(),
                        snake: "email".to_string(),
                        camel: "email".to_string(),
                        pascal: "Email".to_string(),
                    },
                    field_type: GeneratorFieldType::Text,
                    description: Some("邮箱".to_string()),
                    nullable: true,
                    unique: true,
                    indexed: true,
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
        }],
        enums: vec![GeneratorEnum {
            name: "customer_status".to_string(),
            values: vec!["active".to_string(), "inactive".to_string()],
        }],
        metadata: ModelMetadata {
            generated_at: "2024-01-01T00:00:00Z".to_string(),
            generator_version: "1.0.0".to_string(),
        },
        ..Default::default()
    }
}
