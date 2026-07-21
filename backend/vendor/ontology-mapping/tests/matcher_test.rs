use ontology_mapping::matcher::FieldMatcher;
use ontology_mapping::output::Tier;
use ontology_mapping::rules::RuleSet;

fn load_rules() -> RuleSet {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/rules.yaml");
    RuleSet::load(path).unwrap()
}

#[test]
fn test_exact_match_name() {
    let rules = load_rules();
    let matcher = FieldMatcher::new(&rules.field_patterns);
    let result = matcher.match_field("name", &[], None);
    assert!(result.is_some());
    let m = result.unwrap();
    assert_eq!(m.column.unwrap(), "notice");
    assert_eq!(m.tier, Tier::Safe);
    assert!(m.confidence >= 0.90);
}

#[test]
fn test_exact_match_code() {
    let rules = load_rules();
    let matcher = FieldMatcher::new(&rules.field_patterns);
    let result = matcher.match_field("code", &[], None);
    assert!(result.is_some());
    let m = result.unwrap();
    assert_eq!(m.column.unwrap(), "code");
}

#[test]
fn test_semantic_group_desc() {
    let rules = load_rules();
    let matcher = FieldMatcher::new(&rules.field_patterns);
    let result = matcher.match_field("description", &[], None);
    assert!(result.is_some());
    let m = result.unwrap();
    assert_eq!(m.column.unwrap(), "notice");
    assert_eq!(m.tier, Tier::Suggest);
}

#[test]
fn test_contextual_type_with__f_() {
    let rules = load_rules();
    let matcher = FieldMatcher::new(&rules.field_patterns);
    let result = matcher.match_field("type", &["_f_", "number"], None);
    assert!(result.is_some());
    let m = result.unwrap();
    assert_eq!(m.column.unwrap(), "_t_");
    assert_eq!(m.tier, Tier::Suggest);
}

#[test]
fn test_no_match_unknown_field() {
    let rules = load_rules();
    let matcher = FieldMatcher::new(&rules.field_patterns);
    let result = matcher.match_field("xyzzy_unknown", &[], None);
    assert!(result.is_none());
}
use ontology_mapping::matcher::ScalarMatcher;

#[test]
fn test_scalar_amount() {
    let rules = load_rules();
    let matcher = ScalarMatcher::new(&rules.scalar_inference);
    let result = matcher.match_scalar("amount");
    assert!(result.is_some());
    let m = result.unwrap();
    assert_eq!(m.scalar_table.unwrap(), "zc_id_scal-amount");
    assert_eq!(m.column.unwrap(), "qk_amount");
    assert_eq!(m.tier, Tier::Safe);
}

#[test]
fn test_scalar_price() {
    let rules = load_rules();
    let matcher = ScalarMatcher::new(&rules.scalar_inference);
    let result = matcher.match_scalar("unit_price");
    assert!(result.is_some());
    let m = result.unwrap();
    assert_eq!(m.scalar_table.unwrap(), "zc_id_scal-price");
}

#[test]
fn test_scalar_no_match() {
    let rules = load_rules();
    let matcher = ScalarMatcher::new(&rules.scalar_inference);
    let result = matcher.match_scalar("username");
    assert!(result.is_none());
}

#[test]
fn test_scalar_compound_amount() {
    let rules = load_rules();
    let matcher = ScalarMatcher::new(&rules.scalar_inference);
    let result = matcher.match_scalar("total_amount");
    assert!(result.is_some());
    let m = result.unwrap();
    assert_eq!(m.column.unwrap(), "qk_amount");
}

#[test]
fn test_scalar_effective_date() {
    let rules = load_rules();
    let matcher = ScalarMatcher::new(&rules.scalar_inference);
    let result = matcher.match_scalar("effective_date");
    assert!(result.is_some());
    let m = result.unwrap();
    assert_eq!(m.column.unwrap(), "qk_date");
}

#[test]
fn test_scalar_due_date() {
    let rules = load_rules();
    let matcher = ScalarMatcher::new(&rules.scalar_inference);
    let result = matcher.match_scalar("due_date");
    assert!(result.is_some());
    let m = result.unwrap();
    assert_eq!(m.column.unwrap(), "qk_date");
}

use ontology_mapping::matcher::NestingMatcher;
use ontology_mapping::output::{FieldInput, MappingInput, NestedEntityInput, NestedInput};

#[test]
fn test_nesting_array_to_independent_entity() {
    let rules = load_rules();
    let matcher = NestingMatcher::new(&rules.nesting_rules);
    let nested = NestedInput {
        name: "items".into(),
        nested_type: "array".into(),
        items: NestedEntityInput {
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
    };
    let result = matcher.decide_nesting(&nested, false, false);
    assert!(result.is_some());
    let m = result.unwrap();
    assert_eq!(m.rel_type, "hasMany");
    assert_eq!(m.tier, Tier::Safe);
}

#[test]
fn test_nesting_small_object_flatten() {
    let rules = load_rules();
    let matcher = NestingMatcher::new(&rules.nesting_rules);
    let nested = NestedInput {
        name: "address".into(),
        nested_type: "object".into(),
        items: NestedEntityInput {
            name: "Address".into(),
            fields: vec![
                FieldInput {
                    name: "city".into(),
                    field_type: "string".into(),
                    format: None,
                    r#enum: vec![],
                },
                FieldInput {
                    name: "zip".into(),
                    field_type: "string".into(),
                    format: None,
                    r#enum: vec![],
                },
            ],
        },
    };
    let result = matcher.decide_nesting(&nested, false, false);
    assert!(result.is_some());
    let m = result.unwrap();
    assert_eq!(m.rel_type, "flatten");
}

#[test]
fn test_prefix_match_fk() {
    let rules = load_rules();
    let matcher = FieldMatcher::new(&rules.field_patterns);
    let result = matcher.match_field("fk_order_id", &[], None);
    assert!(result.is_some());
    let m = result.unwrap();
    assert_eq!(m.column.unwrap(), "fk_order_id");
    assert_eq!(m.tier, Tier::Safe);
}

#[test]
fn test_contextual_type_with_entity_type() {
    let rules = load_rules();
    let matcher = FieldMatcher::new(&rules.field_patterns);
    // Without entity_type, should fall back to default (_t_, Suggest)
    let result = matcher.match_field("type", &[], None);
    assert!(result.is_some());
    let m = result.unwrap();
    assert_eq!(m.column.unwrap(), "_t_");
}

#[test]
fn test_coordinate_inference_order() {
    let rules = load_rules();
    let input = MappingInput {
        scene_code: "FE".into(),
        factor_ids: vec!["FJA".into()],
        entities: vec![],
    };
    let inferrer = ontology_mapping::inferrer::CoordinateInferrer::new(&rules.coordinate_inference);
    let coords = inferrer.infer("Order", &input);
    assert_eq!(coords.scene.value, "FE");
    assert_eq!(coords.scene.tier, Tier::Safe);
    assert_eq!(coords.factor.value, "FJA");
    assert_eq!(coords.function.value, "↓_GD");
}

#[test]
fn test_coordinate_inference_agreement_order() {
    let rules = load_rules();
    let input = MappingInput {
        scene_code: "FE".into(),
        factor_ids: vec!["FJA".into()],
        entities: vec![],
    };
    let inferrer = ontology_mapping::inferrer::CoordinateInferrer::new(&rules.coordinate_inference);
    let coords = inferrer.infer("Agreement", &input);
    assert_eq!(coords.function.value, "↓_GD");
}
