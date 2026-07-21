//! 生成器集成测试 (简化版)
//!
//! DSL 支持已移除，新的测试将在 Phase 2 中基于 IR 模型实现

// 使用内部路径导入 ModelMetadata
use alioth_gen::{
    EntityName, FieldName, GeneratorEntity, GeneratorEnum, GeneratorField, GeneratorFieldType,
    GeneratorModel, PrimaryKeyType,
};

// 从 IR 模块导入 ModelMetadata
use alioth_gen::generator::ir::ModelMetadata;

/// 创建测试用的 IR 模型
fn create_test_model() -> GeneratorModel {
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

#[test]
fn test_ir_model_creation() {
    let model = create_test_model();

    assert_eq!(model.entities.len(), 1);
    assert_eq!(model.enums.len(), 1);
    assert_eq!(model.entities[0].name.snake, "customers");
    assert_eq!(model.entities[0].fields.len(), 2);
}

#[test]
fn test_primary_key_type() {
    let model = create_test_model();

    assert!(matches!(
        model.entities[0].primary_key_type,
        PrimaryKeyType::BigInt
    ));
}
