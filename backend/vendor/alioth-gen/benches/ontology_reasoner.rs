use alioth_gen::generator::ir::ontology::{
    DomainKind, DomainOntology, OntologyMetadata, OntologyModel, RelationOntology, RelationType,
};
use alioth_gen::generator::ir::OntologyReasoner;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::collections::HashMap;

fn create_chain_model(n: usize) -> OntologyModel {
    let mut model = OntologyModel {
        id: "perf".to_string(),
        name: "Perf".to_string(),
        description: None,
        version: "1.0".to_string(),
        domains: Vec::with_capacity(n),
        transaction_lifecycle: None,
        relations: Vec::with_capacity(n.saturating_sub(1)),
        constraints: vec![],
        computations: vec![],
        namespaces: HashMap::new(),
        metadata: OntologyMetadata::default(),
    };

    for i in 0..n {
        model.domains.push(DomainOntology {
            id: format!("c{}", i),
            name: format!("C{}", i),
            description: None,
            kind: DomainKind::Entity,
            parent_ids: vec![],
            equivalent_ids: vec![],
            disjoint_ids: vec![],
            properties: vec![],
            prefab_contract: None,
        });
    }

    for i in 0..n.saturating_sub(1) {
        model.relations.push(RelationOntology {
            id: format!("r{}", i),
            name: format!("R{}", i),
            relation_type: RelationType::Association,
            source_ontology: format!("c{}", i),
            target_ontology: format!("c{}", i + 1),
            is_bidirectional: false,
            properties: vec![],
            constraints: vec![],
            semantic_description: None,
        });
    }

    model
}

fn bench_reasoning_100_classes(c: &mut Criterion) {
    let model = create_chain_model(100);
    let mut group = c.benchmark_group("ontology_reasoning");
    group.throughput(Throughput::Elements(100));
    group.bench_function("reason_100_classes", |b| {
        b.iter(|| {
            black_box(OntologyReasoner::reason(black_box(&model)));
        });
    });
    group.finish();
}

fn bench_reasoning_500_classes(c: &mut Criterion) {
    let model = create_chain_model(500);
    let mut group = c.benchmark_group("ontology_reasoning");
    group.throughput(Throughput::Elements(500));
    group.bench_function("reason_500_classes", |b| {
        b.iter(|| {
            black_box(OntologyReasoner::reason(black_box(&model)));
        });
    });
    group.finish();
}

fn bench_reasoning_1000_classes(c: &mut Criterion) {
    let model = create_chain_model(1000);
    let mut group = c.benchmark_group("ontology_reasoning");
    group.throughput(Throughput::Elements(1000));
    group.bench_function("reason_1000_classes", |b| {
        b.iter(|| {
            black_box(OntologyReasoner::reason(black_box(&model)));
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_reasoning_100_classes,
    bench_reasoning_500_classes,
    bench_reasoning_1000_classes
);
criterion_main!(benches);
