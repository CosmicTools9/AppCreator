//! E2E 测试框架 — Ontology IR 全链路集成测试
//!
//! 提供从 OntologyModel 到代码生成的完整测试基础设施

use alioth_gen::generator::ir::ontology::{
    DomainKind, DomainOntology, OntologyMetadata, OntologyModel, OntologyProperty, PropertyType,
};
use alioth_gen::generator::ir::ontology_transformer::OntologyTransformer;
use alioth_gen::generator::ir::GeneratorModel;
use std::time::{Duration, Instant};

/// E2E 测试上下文
pub struct E2eTestContext {
    pub name: String,
    pub start_time: Instant,
    pub ontology: OntologyModel,
    pub model_result: Option<GeneratorModel>,
    pub timing: TestTiming,
}

/// 测试计时信息
#[derive(Debug, Default)]
pub struct TestTiming {
    pub transform_duration: Duration,
}

impl E2eTestContext {
    pub fn new(name: &str, ontology: OntologyModel) -> Self {
        Self {
            name: name.to_string(),
            start_time: Instant::now(),
            ontology,
            model_result: None,
            timing: TestTiming::default(),
        }
    }

    pub fn run_transform(&mut self) -> Result<&GeneratorModel, String> {
        let start = Instant::now();
        let model = OntologyTransformer::transform(&self.ontology, vec![]);
        self.timing.transform_duration = start.elapsed();
        self.model_result = Some(model);
        Ok(self.model_result.as_ref().unwrap())
    }

    pub fn assert_has_entity(&self, name: &str) {
        let model = self.model_result.as_ref().expect("Model not generated");
        assert!(
            model.entities.iter().any(|e| e.name.raw == name),
            "Expected entity '{}' not found",
            name
        );
    }
}

#[test]
fn test_e2e_basic_entity_transform() {
    let ontology = OntologyModel {
        id: "test".to_string(),
        name: "Test".to_string(),
        description: None,
        version: "1.0".to_string(),
        domains: vec![DomainOntology {
            id: "user".to_string(),
            name: "User".to_string(),
            description: Some("A user entity".to_string()),
            kind: DomainKind::Entity,
            parent_ids: vec![],
            equivalent_ids: vec![],
            disjoint_ids: vec![],
            properties: vec![OntologyProperty {
                id: "email".to_string(),
                name: "email".to_string(),
                property_type: PropertyType::DataProperty,
                required: true,
                cardinality: alioth_gen::generator::ir::ontology::Cardinality {
                    min: None,
                    max: None,
                    exact: None,
                },
                domain: "User".to_string(),
                range: "String".to_string(),
                is_functional: true,
                is_transitive: false,
                is_symmetric: false,
                constraints: vec![],
                semantic_description: Some("Email address".to_string()),
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

    let mut ctx = E2eTestContext::new("basic_entity", ontology);
    ctx.run_transform().unwrap();
    ctx.assert_has_entity("User");
}
