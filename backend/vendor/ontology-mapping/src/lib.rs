pub mod collector;
pub mod contracts;
pub mod discovery;
pub mod gap;
pub mod gen_tests;
pub mod generator;
pub mod inferrer;
pub mod matcher;
pub mod model_graph;
pub mod output;
pub mod patch_m2n;
pub mod pipeline_state;
pub mod prototype_check;
pub mod rules;
pub mod stale;
pub mod sync;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

use output::*;
use rules::RuleSet;

pub use output::{EntityInput, FieldInput, MappingInput, NestedEntityInput, NestedInput};

use crate::collector::FactorEntityMapping;

pub struct OntologyMapper {
    rules: RuleSet,
    known_entities: HashMap<String, FactorEntityMapping>,
}

impl OntologyMapper {
    pub fn load(rules_path: impl AsRef<Path>, services_dir: impl AsRef<Path>) -> Result<Self> {
        let rules = RuleSet::load(rules_path)?;
        let mappings = collector::collect_service_mappings(services_dir)?;
        let known_entities = mappings.into_iter().map(|m| (m.name.clone(), m)).collect();
        Ok(Self {
            rules,
            known_entities,
        })
    }

    pub fn map(&self, input: &MappingInput) -> MappingOutput {
        let field_matcher = matcher::FieldMatcher::new(&self.rules.field_patterns);
        let scalar_matcher = matcher::ScalarMatcher::new(&self.rules.scalar_inference);
        let nesting_matcher = matcher::NestingMatcher::new(&self.rules.nesting_rules);
        let coordinate_inferrer =
            inferrer::CoordinateInferrer::new(&self.rules.coordinate_inference);

        let mut entities = Vec::new();
        let mut safe_count = 0usize;
        let mut suggest_count = 0usize;
        let mut unclear_count = 0usize;

        for entity in &input.entities {
            let mut fields: Vec<FieldMapping> = Vec::new();
            let mut relationships: Vec<RelationshipMapping> = Vec::new();

            // Collect sibling field names for contextual matching
            let sibling_names: Vec<&str> = entity.fields.iter().map(|f| f.name.as_str()).collect();

            for field in &entity.fields {
                // Try scalar first, then field pattern
                let mapping = scalar_matcher
                    .match_scalar(&field.name)
                    .or_else(|| field_matcher.match_field(&field.name, &sibling_names, None));

                if let Some(m) = mapping {
                    match m.tier {
                        Tier::Safe => safe_count += 1,
                        Tier::Suggest => suggest_count += 1,
                        Tier::Unclear => unclear_count += 1,
                    }
                    fields.push(m);
                } else {
                    unclear_count += 1;
                    fields.push(FieldMapping {
                        json_path: field.name.clone(),
                        column: None,
                        scalar_table: None,
                        ref_table: None,
                        tier: Tier::Unclear,
                        confidence: 0.0,
                        source: "no_match".into(),
                        alternatives: vec![],
                    });
                }
            }

            // Process nested entities
            for nested in &entity.nested {
                if let Some(rel) = nesting_matcher.decide_nesting(nested, false, false) {
                    match rel.tier {
                        Tier::Safe => safe_count += 1,
                        Tier::Suggest => suggest_count += 1,
                        Tier::Unclear => unclear_count += 1,
                    }
                    relationships.push(rel);
                }
            }

            let coordinates = coordinate_inferrer.infer(&entity.name, input);

            // 优先使用输入中指定的 table（来自 discovery 预先发现）
            let entity_mapping = if let Some(ref table_name) = entity.table {
                let qual = if table_name.starts_with("isahl.") {
                    table_name.clone()
                } else {
                    format!("isahl.{}", table_name)
                };
                EntityMapping {
                    table: qual,
                    inherits: None,
                    source: "discovery".into(),
                    tier: Tier::Safe,
                    confidence: 0.90,
                }
            } else {
                // 否则查 service.json 已知实体
                self.known_entities
                    .get(&entity.name)
                    .map(|known| EntityMapping {
                        table: known.table.clone(),
                        inherits: known.inherits.clone(),
                        source: "factor_match".into(),
                        tier: Tier::Safe,
                        confidence: 1.0,
                    })
                    .unwrap_or_else(|| EntityMapping {
                        table: String::new(),
                        inherits: None,
                        source: "rule_match".into(),
                        tier: Tier::Suggest,
                        confidence: 0.70,
                    })
            };

            entities.push(MappedEntity {
                name: entity.name.clone(),
                mapping: entity_mapping,
                coordinates,
                fields,
                relationships,
            });
        }

        MappingOutput {
            meta: OutputMeta {
                tool_version: "0.1.0".into(),
                alioth_model: self.rules.alioth_model.clone(),
            },
            entities,
            summary: TierSummary {
                safe: safe_count,
                suggest: suggest_count,
                unclear: unclear_count,
            },
        }
    }
}
