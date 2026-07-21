use ontology_mapping::{EntityInput, FieldInput, MappingInput, OntologyMapper};
use std::path::Path;

fn fixture_path() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/services"
    ))
}

fn rules_path() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/rules.yaml"))
}

#[test]
fn test_full_mapping_order_entity() {
    let mapper = OntologyMapper::load(rules_path(), fixture_path()).unwrap();

    let input = MappingInput {
        scene_code: "FE".into(),
        factor_ids: vec!["FJA".into()],
        entities: vec![EntityInput {
            name: "Order".into(),
            table: None,
            fields: vec![
                FieldInput {
                    name: "name".into(),
                    field_type: "string".into(),
                    format: None,
                    r#enum: vec![],
                },
                FieldInput {
                    name: "amount".into(),
                    field_type: "number".into(),
                    format: None,
                    r#enum: vec![],
                },
                FieldInput {
                    name: "type".into(),
                    field_type: "string".into(),
                    format: None,
                    r#enum: vec![],
                },
            ],
            nested: vec![],
        }],
    };

    let output = mapper.map(&input);

    assert_eq!(output.entities.len(), 1);
    let entity = &output.entities[0];
    assert_eq!(entity.name, "Order");
    assert_eq!(entity.coordinates.function.value, "↓_GD");

    // name → notice (exact) — safe
    let name_field = entity
        .fields
        .iter()
        .find(|f| f.json_path == "name")
        .unwrap();
    assert_eq!(name_field.column.as_deref(), Some("notice"));
    assert_eq!(name_field.tier, ontology_mapping::output::Tier::Safe);

    // amount → qk_amount (scalar) — safe
    let amt_field = entity
        .fields
        .iter()
        .find(|f| f.json_path == "amount")
        .unwrap();
    assert_eq!(amt_field.column.as_deref(), Some("qk_amount"));
    assert!(amt_field.scalar_table.is_some());

    // type → suggest or unclear (contextual)
    let type_field = entity
        .fields
        .iter()
        .find(|f| f.json_path == "type")
        .unwrap();
    assert!(type_field.alternatives.len() >= 1);
}

#[test]
fn test_full_mapping_with_nested() {
    let mapper = OntologyMapper::load(rules_path(), fixture_path()).unwrap();

    let input = MappingInput {
        scene_code: "FE".into(),
        factor_ids: vec!["FJA".into()],
        entities: vec![EntityInput {
            name: "Order".into(),
            table: None,
            fields: vec![FieldInput {
                name: "name".into(),
                field_type: "string".into(),
                format: None,
                r#enum: vec![],
            }],
            nested: vec![ontology_mapping::NestedInput {
                name: "items".into(),
                nested_type: "array".into(),
                items: ontology_mapping::NestedEntityInput {
                    name: "LineItem".into(),
                    fields: vec![
                        FieldInput {
                            name: "qty".into(),
                            field_type: "number".into(),
                            format: None,
                            r#enum: vec![],
                        },
                        FieldInput {
                            name: "price".into(),
                            field_type: "number".into(),
                            format: None,
                            r#enum: vec![],
                        },
                        FieldInput {
                            name: "product".into(),
                            field_type: "string".into(),
                            format: None,
                            r#enum: vec![],
                        },
                    ],
                },
            }],
        }],
    };

    let output = mapper.map(&input);
    assert_eq!(output.entities.len(), 1);
    assert_eq!(output.entities[0].relationships.len(), 1);
    assert_eq!(output.entities[0].relationships[0].rel_type, "hasMany");
}

#[test]
fn test_known_entity_gets_table_name() {
    let mapper = OntologyMapper::load(rules_path(), fixture_path()).unwrap();

    let input = MappingInput {
        scene_code: "FE".into(),
        factor_ids: vec!["FJA".into()],
        entities: vec![EntityInput {
            name: "Order".into(),
            table: None,
            fields: vec![
                FieldInput {
                    name: "name".into(),
                    field_type: "string".into(),
                    format: None,
                    r#enum: vec![],
                },
                FieldInput {
                    name: "amount".into(),
                    field_type: "number".into(),
                    format: None,
                    r#enum: vec![],
                },
            ],
            nested: vec![],
        }],
    };

    let output = mapper.map(&input);
    assert_eq!(output.entities.len(), 1);
    let entity = &output.entities[0];
    assert_eq!(entity.name, "Order");
    // Known entity should get table from service.json
    assert!(
        !entity.mapping.table.is_empty(),
        "entity table should not be empty for known entities"
    );
    assert_eq!(entity.mapping.table, "isahl.zc_id_lifecycle_order");
    assert_eq!(entity.mapping.inherits.as_deref(), Some("zc_id_lifecycle"));
    assert_eq!(entity.mapping.source, "factor_match");
    assert_eq!(entity.mapping.tier, ontology_mapping::output::Tier::Safe);
    assert_eq!(entity.mapping.confidence, 1.0);
}
