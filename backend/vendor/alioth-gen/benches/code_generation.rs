use alioth_gen::generator::ir::module::MetaModule;
use alioth_gen::generator::ir::module::{MetaEntity, MetaField, MetaFieldType};
use alioth_gen::generator::ir::ModelMetadata;
use alioth_gen::generator::module::ModuleApiGenerator;
use alioth_gen::generator::zod::FullZodGenerator;
use alioth_gen::{
    EntityName, FieldName, GeneratorEntity, GeneratorEnum, GeneratorField, GeneratorFieldType,
    GeneratorModel, PrimaryKeyType,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

fn create_test_model(entity_count: usize, field_count: usize) -> GeneratorModel {
    let mut entities = Vec::with_capacity(entity_count);
    for i in 0..entity_count {
        let name = format!("Entity{}", i);
        let mut fields = Vec::with_capacity(field_count);
        for f in 0..field_count {
            fields.push(GeneratorField {
                name: FieldName {
                    raw: format!("field_{}", f),
                    snake: format!("field_{}", f),
                    camel: format!("field_{}", f),
                    pascal: format!("Field{}", f),
                },
                field_type: if f % 3 == 0 {
                    GeneratorFieldType::Text
                } else if f % 3 == 1 {
                    GeneratorFieldType::Integer
                } else {
                    GeneratorFieldType::Boolean
                },
                description: Some(format!("Field {} description", f)),
                nullable: f % 2 == 0,
                unique: f % 5 == 0,
                indexed: f % 4 == 0,
                default_value: None,
                validations: vec![],
                annotations: vec![],
                ..Default::default()
            });
        }
        entities.push(GeneratorEntity {
            name: EntityName {
                raw: name.clone(),
                snake: name.to_lowercase(),
                camel: name.to_lowercase(),
                pascal: name.clone(),
                kebab: name.to_lowercase(),
                screaming_snake: name.to_uppercase(),
                plural_snake: name.to_lowercase(),
                plural_pascal: name.clone(),
                plural_kebab: name.to_lowercase(),
            },
            description: Some(format!("Entity {} description", i)),
            fields,
            relations: vec![],
            annotations: vec![],
            primary_key_type: PrimaryKeyType::BigInt,
            ..Default::default()
        });
    }

    GeneratorModel {
        i18n_config: None,
        entities,
        enums: vec![GeneratorEnum {
            name: "status".to_string(),
            values: vec!["active".to_string(), "inactive".to_string()],
        }],
        metadata: ModelMetadata {
            generated_at: "2024-01-01T00:00:00Z".to_string(),
            generator_version: "1.0.0".to_string(),
        },
        ..Default::default()
    }
}

fn create_meta_module(entity_count: usize, field_count: usize) -> MetaModule {
    let mut module = MetaModule::new("perf_module");
    for i in 0..entity_count {
        let name = format!("Entity{}", i);
        let mut fields = Vec::with_capacity(field_count);
        for f in 0..field_count {
            fields.push(MetaField {
                name: format!("field_{}", f),
                field_type: if f % 3 == 0 {
                    MetaFieldType::String
                } else if f % 3 == 1 {
                    MetaFieldType::Integer
                } else {
                    MetaFieldType::Boolean
                },
                description: Some(format!("Field {} description", f)),
                nullable: f % 2 == 0,
                unique: f % 5 == 0,
                indexed: f % 4 == 0,
                default_value: None,
                validations: vec![],
                annotations: vec![],
                ..Default::default()
            });
        }
        module.add_entity(MetaEntity {
            name: name.clone(),
            description: Some(format!("Entity {} description", i)),
            fields,
            relations: vec![],
            annotations: vec![],
            ..Default::default()
        });
    }
    module.pages = MetaModule::infer_pages(&module.entities);
    module
}

// === Zod Generation Benchmarks ===

fn bench_zod_single_entity_5_fields(c: &mut Criterion) {
    let model = create_test_model(1, 5);
    let mut group = c.benchmark_group("zod_generation");
    group.throughput(Throughput::Elements(1));
    group.bench_function("single_entity_5_fields", |b| {
        b.iter(|| {
            black_box(FullZodGenerator::generate(black_box(&model)).unwrap());
        });
    });
    group.finish();
}

fn bench_zod_single_entity_10_fields(c: &mut Criterion) {
    let model = create_test_model(1, 10);
    let mut group = c.benchmark_group("zod_generation");
    group.throughput(Throughput::Elements(1));
    group.bench_function("single_entity_10_fields", |b| {
        b.iter(|| {
            black_box(FullZodGenerator::generate(black_box(&model)).unwrap());
        });
    });
    group.finish();
}

fn bench_zod_5_entities_5_fields(c: &mut Criterion) {
    let model = create_test_model(5, 5);
    let mut group = c.benchmark_group("zod_generation");
    group.throughput(Throughput::Elements(5));
    group.bench_function("5_entities_5_fields", |b| {
        b.iter(|| {
            black_box(FullZodGenerator::generate(black_box(&model)).unwrap());
        });
    });
    group.finish();
}

fn bench_zod_10_entities_10_fields(c: &mut Criterion) {
    let model = create_test_model(10, 10);
    let mut group = c.benchmark_group("zod_generation");
    group.throughput(Throughput::Elements(10));
    group.bench_function("10_entities_10_fields", |b| {
        b.iter(|| {
            black_box(FullZodGenerator::generate(black_box(&model)).unwrap());
        });
    });
    group.finish();
}

// === Module API Generation Benchmarks ===

fn bench_module_api_5_entities_10_fields(c: &mut Criterion) {
    let module = create_meta_module(5, 10);
    let api_gen = ModuleApiGenerator::new();
    let mut group = c.benchmark_group("module_api_generation");
    group.throughput(Throughput::Elements(5));
    group.bench_function("5_entities_10_fields", |b| {
        b.iter(|| {
            black_box(api_gen.generate(black_box(&module)).unwrap());
        });
    });
    group.finish();
}

fn bench_module_api_10_entities_10_fields(c: &mut Criterion) {
    let module = create_meta_module(10, 10);
    let api_gen = ModuleApiGenerator::new();
    let mut group = c.benchmark_group("module_api_generation");
    group.throughput(Throughput::Elements(10));
    group.bench_function("10_entities_10_fields", |b| {
        b.iter(|| {
            black_box(api_gen.generate(black_box(&module)).unwrap());
        });
    });
    group.finish();
}

// === 50 Fields Benchmarks (Phase 184 target) ===

fn bench_zod_1_entity_50_fields(c: &mut Criterion) {
    let model = create_test_model(1, 50);
    let mut group = c.benchmark_group("zod_generation_50_fields");
    group.throughput(Throughput::Elements(50));
    group.bench_function("1_entity_50_fields", |b| {
        b.iter(|| {
            black_box(FullZodGenerator::generate(black_box(&model)).unwrap());
        });
    });
    group.finish();
}

fn bench_module_api_1_entity_50_fields(c: &mut Criterion) {
    let module = create_meta_module(1, 50);
    let api_gen = ModuleApiGenerator::new();
    let mut group = c.benchmark_group("module_api_generation_50_fields");
    group.throughput(Throughput::Elements(50));
    group.bench_function("1_entity_50_fields", |b| {
        b.iter(|| {
            black_box(api_gen.generate(black_box(&module)).unwrap());
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_zod_single_entity_5_fields,
    bench_zod_single_entity_10_fields,
    bench_zod_5_entities_5_fields,
    bench_zod_10_entities_10_fields,
    bench_zod_1_entity_50_fields,
    bench_module_api_5_entities_10_fields,
    bench_module_api_10_entities_10_fields,
    bench_module_api_1_entity_50_fields,
);
criterion_main!(benches);
