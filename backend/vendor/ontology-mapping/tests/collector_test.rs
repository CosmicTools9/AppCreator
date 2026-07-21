use ontology_mapping::collector;

#[test]
fn test_collect_from_fixtures() {
    let fixtures = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/services");
    let mappings = collector::collect_service_mappings(fixtures).unwrap();

    assert!(mappings.len() >= 1);
    let product = mappings.iter().find(|m| m.name == "TestProduct").unwrap();
    assert_eq!(product.table, "isahl.zc_id_lifecycle_test_product");
    let coords = product.coordinates.as_ref().expect("coordinates 应存在");
    assert_eq!(coords.scene, "FE");
    assert_eq!(product.field_mappings.len(), 2);
    assert_eq!(product.relationships.len(), 1);
}
