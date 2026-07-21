use ontology_mapping::rules::RuleSet;

#[test]
fn test_parse_rules_yaml() {
    let rules_path = concat!(env!("CARGO_MANIFEST_DIR"), "/rules.yaml");
    let rules = RuleSet::load(rules_path).unwrap();

    assert_eq!(rules.version, "0.1.0");
    assert!(!rules.field_patterns.exact.is_empty());
    assert_eq!(rules.field_patterns.exact[0].column, "notice");
    assert!(!rules.scalar_inference.rules.is_empty());
    assert_eq!(
        rules.scalar_inference.rules[0].scalar_table,
        "zc_id_scal-amount"
    );
    assert!(!rules.nesting_rules.is_empty());
    assert!(!rules.coordinate_inference.function.rules.is_empty());
}
