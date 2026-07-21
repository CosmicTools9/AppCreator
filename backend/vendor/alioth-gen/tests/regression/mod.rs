//! 回归测试套件
//!
//! 验证已修复的 Bug 不会再次出现

use alioth_gen::generator::ir::ontology::{
    DomainKind, DomainOntology, OntologyMetadata, OntologyModel, OntologyProperty, PropertyType,
};
use alioth_gen::generator::ir::ontology_transformer::OntologyTransformer;

// ==================== 已知问题回归测试 ====================

/// REG-001: 空实体应生成空字段列表而非崩溃
#[test]
fn regression_empty_entity_no_crash() {
    let ontology = OntologyModel {
        id: "test".to_string(),
        name: "Test".to_string(),
        description: None,
        version: "1.0".to_string(),
        domains: vec![DomainOntology {
            id: "empty".to_string(),
            name: "Empty".to_string(),
            description: None,
            kind: DomainKind::Entity,
            parent_ids: vec![],
            equivalent_ids: vec![],
            disjoint_ids: vec![],
            properties: vec![],
            prefab_contract: None,
        }],
        transaction_lifecycle: None,
        relations: vec![],
        constraints: vec![],
        computations: vec![],
        namespaces: Default::default(),
        metadata: OntologyMetadata::default(),
    };

    let model = OntologyTransformer::transform(&ontology, vec![]);
    assert_eq!(model.entities.len(), 1);
    assert!(model.entities[0].fields.is_empty());
}

/// REG-002: 数据属性应正确映射为 GeneratorField
#[test]
fn regression_data_property_to_field() {
    let ontology = OntologyModel {
        id: "test".to_string(),
        name: "Test".to_string(),
        description: None,
        version: "1.0".to_string(),
        domains: vec![DomainOntology {
            id: "product".to_string(),
            name: "Product".to_string(),
            description: None,
            kind: DomainKind::Entity,
            parent_ids: vec![],
            equivalent_ids: vec![],
            disjoint_ids: vec![],
            properties: vec![OntologyProperty {
                id: "price".to_string(),
                name: "price".to_string(),
                property_type: PropertyType::DataProperty,
                required: true,
                cardinality: alioth_gen::generator::ir::ontology::Cardinality {
                    min: None,
                    max: None,
                    exact: None,
                },
                domain: "Product".to_string(),
                range: "Decimal".to_string(),
                is_functional: true,
                is_transitive: false,
                is_symmetric: false,
                constraints: vec![],
                semantic_description: None,
            }],
            prefab_contract: None,
        }],
        transaction_lifecycle: None,
        relations: vec![],
        constraints: vec![],
        computations: vec![],
        namespaces: Default::default(),
        metadata: OntologyMetadata::default(),
    };

    let model = OntologyTransformer::transform(&ontology, vec![]);
    let product = &model.entities[0];
    assert_eq!(product.fields.len(), 1);
    assert_eq!(product.fields[0].name.raw, "price");
}
