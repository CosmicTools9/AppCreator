use ontology_mapping::output::*;

#[test]
fn test_mapping_output_roundtrip() {
    let output = MappingOutput {
        meta: OutputMeta {
            tool_version: "0.1.0".into(),
            alioth_model: "10.0.0".into(),
        },
        entities: vec![MappedEntity {
            name: "Order".into(),
            mapping: EntityMapping {
                table: "isahl.zc_id_lifecycle_order".into(),
                inherits: Some("zc_id_lifecycle".into()),
                source: "factor_match".into(),
                tier: Tier::Safe,
                confidence: 0.95,
            },
            coordinates: Coordinates {
                scene: TieredValue {
                    value: "FE".into(),
                    tier: Tier::Safe,
                    confidence: 1.0,
                    source: "input".into(),
                },
                factor: TieredValue {
                    value: "FJA".into(),
                    tier: Tier::Safe,
                    confidence: 0.95,
                    source: "factor_match".into(),
                },
                function: TieredValue {
                    value: "↓_GD".into(),
                    tier: Tier::Safe,
                    confidence: 0.85,
                    source: "semantic_inference".into(),
                },
            },
            fields: vec![FieldMapping {
                json_path: "name".into(),
                column: Some("notice".into()),
                scalar_table: None,
                ref_table: None,
                tier: Tier::Safe,
                confidence: 0.95,
                source: "exact_pattern".into(),
                alternatives: vec![],
            }],
            relationships: vec![RelationshipMapping {
                target: "LineItem".into(),
                rel_type: "hasMany".into(),
                via: None,
                tier: Tier::Safe,
                confidence: 0.90,
                source: "nesting_rule".into(),
            }],
        }],
        summary: TierSummary {
            safe: 1,
            suggest: 0,
            unclear: 0,
        },
    };

    let json = serde_json::to_string_pretty(&output).unwrap();
    let parsed: MappingOutput = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.entities.len(), 1);
    assert_eq!(parsed.entities[0].name, "Order");
    assert_eq!(parsed.meta.tool_version, "0.1.0");
}
#[test]
fn test_mapping_input_parse() {
    let json = r#"{
        "scene_code": "FE",
        "factor_ids": ["FJA"],
        "entities": [{
            "name": "Order",
            "fields": [
                { "name": "id", "type": "integer" },
                { "name": "name", "type": "string" }
            ],
            "nested": [{
                "name": "items",
                "type": "array",
                "items": { "name": "LineItem", "fields": [
                    { "name": "qty", "type": "number" }
                ]}
            }]
        }]
    }"#;

    let input: MappingInput = serde_json::from_str(json).unwrap();
    assert_eq!(input.scene_code, "FE");
    assert_eq!(input.factor_ids, vec!["FJA"]);
    assert_eq!(input.entities.len(), 1);
    assert_eq!(input.entities[0].name, "Order");
    assert_eq!(input.entities[0].fields.len(), 2);
    assert_eq!(input.entities[0].nested[0].items.name, "LineItem");
}
