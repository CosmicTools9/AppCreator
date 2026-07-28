//! Service generation bridge — converts AppAgent state types to ontology-gen-bridge input.
//! Replaces the hand-written `composer::write_service_backend`.

use ontology_mapping::output::{Coordinates, EntityMapping, FieldMapping, MappingOutput, OutputMeta, Tier, TierSummary, TieredValue};
use crate::state::MappedEntity;

pub fn mapped_entities_to_mapping_output(
    entities: &[MappedEntity],
    alioth_version: &str,
) -> MappingOutput {
    let mapped: Vec<_> = entities.iter().map(|e| ontology_mapping::output::MappedEntity {
        name: e.domain_id.clone(),
        mapping: EntityMapping {
            table: e.table.clone(),
            inherits: None,
            source: "app-agent".into(),
            tier: Tier::Safe,
            confidence: e.score.min(1.0),
        },
        coordinates: Coordinates {
            scene: tiered(e.scene_code.as_deref(), "JC", Tier::Suggest),
            factor: tiered(e.factor_code.as_deref(), "GEC", Tier::Suggest),
            function: tiered(e.function_code.as_deref(), "↑_DA", Tier::Suggest),
        },
        fields: e.field_mappings.iter().map(|f| FieldMapping {
            json_path: f.json_path.clone(),
            column: f.column.clone(),
            scalar_table: f.scalar_table.clone(),
            ref_table: None,
            tier: tier_from_str(&f.tier),
            confidence: 0.8,
            source: "app-agent".into(),
            alternatives: vec![],
        }).collect(),
        relationships: vec![],
    }).collect();
    let summary = TierSummary {
        safe: mapped.iter().filter(|e| e.mapping.tier == Tier::Safe).count(),
        suggest: mapped.iter().filter(|e| e.mapping.tier == Tier::Suggest).count(),
        unclear: mapped.iter().filter(|e| e.mapping.tier == Tier::Unclear).count(),
    };
    MappingOutput {
        meta: OutputMeta {
            tool_version: "app-agent".into(),
            alioth_model: alioth_version.into(),
        },
        entities: mapped,
        summary,
    }
}

/// Full pipeline: convert AppAgent MappedEntity → MappingOutput → generate → write.
/// Replaces `composer::write_service_backend`.
pub async fn generate_service_backend(
    namespace: &str,
    service_id: &str,
    entities: &[MappedEntity],
) -> Result<usize, String> {
    if entities.is_empty() { return Ok(0); }

    let root = crate::composer::resolve_project_root();
    let out_dir = root
        .join("Pre-Proc").join(namespace).join("Sources")
        .join("Services").join(service_id).join("backend");
    std::fs::create_dir_all(&out_dir).map_err(|e| format!("mkdir: {}", e))?;

    let mapping = mapped_entities_to_mapping_output(entities, "10.0.0");
    let module = ontology_gen_bridge::adapter::mapping_output_to_meta_module(&mapping, service_id)
        .map_err(|e| format!("adapter: {}", e))?;
    let generator = alioth_gen::generator::module::ModuleApiGenerator::new();
    let generated = generator.generate(&module).map_err(|e| format!("gen: {}", e))?;

    let mut count = 0;
    for file in &generated.files {
        let target = out_dir.join(&file.path);
        if let Some(p) = target.parent() { std::fs::create_dir_all(p).map_err(|e| format!("mkdir: {}", e))?; }
        std::fs::write(&target, &file.content).map_err(|e| format!("write: {}", e))?;
        count += 1;
    }
    Ok(count)
}

fn tiered(value: Option<&str>, fallback: &str, default_tier: Tier) -> TieredValue {
    let v = value.unwrap_or(fallback).to_string();
    TieredValue { value: v, tier: default_tier, confidence: 0.8, source: "app-agent".into() }
}

fn tier_from_str(s: &str) -> Tier {
    match s {
        "safe" | "Safe" => Tier::Safe,
        "suggest" | "Suggest" => Tier::Suggest,
        _ => Tier::Unclear,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::MappedField;

    #[test]
    fn test_converts_single_entity() {
        let entities = vec![MappedEntity {
            domain_id: "Product".into(),
            table: "isahl.zc_id_production".into(),
            score: 0.95, name_score: 0.9, field_score: 1.0,
            scene_code: Some("YA".into()),
            factor_code: Some("GBA".into()),
            function_code: Some("↑.EE".into()),
            function_confidence: 1.0,
            field_mappings: vec![MappedField {
                json_path: "name".into(),
                column: Some("notice".into()),
                scalar_table: None,
                tier: "safe".into(),
            }],
        }];
        let mo = mapped_entities_to_mapping_output(&entities, "10.0.0");
        assert_eq!(mo.entities[0].name, "Product");
        assert_eq!(mo.entities[0].mapping.table, "isahl.zc_id_production");
        assert_eq!(mo.entities[0].fields[0].column.as_deref(), Some("notice"));
    }
}
