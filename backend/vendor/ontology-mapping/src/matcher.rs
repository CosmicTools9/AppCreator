use crate::output::{FieldMapping, Tier};
use crate::rules::FieldPatterns;
use regex::Regex;

// ---------------------------------------------------------------------------
// Field pattern matcher
// ---------------------------------------------------------------------------

pub struct FieldMatcher<'a> {
    patterns: &'a FieldPatterns,
}

impl<'a> FieldMatcher<'a> {
    pub fn new(patterns: &'a FieldPatterns) -> Self {
        Self { patterns }
    }

    pub fn match_field(
        &self,
        field_name: &str,
        sibling_fields: &[&str],
        entity_type: Option<&str>,
    ) -> Option<FieldMapping> {
        // 1. Try exact patterns
        for p in &self.patterns.exact {
            if let Ok(re) = Regex::new(&p.pattern) {
                if re.is_match(field_name) {
                    return Some(FieldMapping {
                        json_path: field_name.into(),
                        column: Some(p.column.clone()),
                        scalar_table: None,
                        ref_table: None,
                        tier: tier_from_confidence(p.confidence),
                        confidence: p.confidence,
                        source: "exact_pattern".into(),
                        alternatives: vec![],
                    });
                }
            }
        }

        // 2. Try prefix patterns
        for p in &self.patterns.prefix {
            if let Ok(re) = Regex::new(&p.pattern) {
                if re.is_match(field_name) {
                    let rest = re.replace(field_name, "").to_string();
                    let column = p
                        .column_template
                        .replace("{field}", field_name)
                        .replace("{rest}", &rest);
                    return Some(FieldMapping {
                        json_path: field_name.into(),
                        column: Some(column),
                        scalar_table: None,
                        ref_table: None,
                        tier: tier_from_confidence(p.confidence),
                        confidence: p.confidence,
                        source: "prefix_pattern".into(),
                        alternatives: vec![],
                    });
                }
            }
        }

        // 3. Try semantic groups
        for g in &self.patterns.semantic_groups {
            if g.triggers.iter().any(|t| t == field_name) {
                return Some(FieldMapping {
                    json_path: field_name.into(),
                    column: Some(g.preference.clone()),
                    scalar_table: None,
                    ref_table: None,
                    tier: tier_from_confidence(g.confidence),
                    confidence: g.confidence,
                    source: "semantic_group".into(),
                    alternatives: vec![],
                });
            }
        }

        // 4. Try contextual patterns
        for c in &self.patterns.contextual {
            if let Ok(re) = Regex::new(&c.pattern) {
                if re.is_match(field_name) {
                    for cand in &c.candidates {
                        if self.check_when(&cand.when, sibling_fields, entity_type) {
                            return Some(FieldMapping {
                                json_path: field_name.into(),
                                column: Some(cand.column.clone()),
                                scalar_table: None,
                                ref_table: None,
                                tier: tier_from_confidence(cand.confidence),
                                confidence: cand.confidence,
                                source: "contextual".into(),
                                alternatives: c
                                    .candidates
                                    .iter()
                                    .filter(|x| x.column != cand.column)
                                    .map(|x| x.column.clone())
                                    .collect(),
                            });
                        }
                    }
                    // Default fallback
                    return Some(FieldMapping {
                        json_path: field_name.into(),
                        column: Some(c.default.clone()),
                        scalar_table: None,
                        ref_table: None,
                        tier: tier_from_confidence(c.confidence),
                        confidence: c.confidence,
                        source: "contextual_default".into(),
                        alternatives: c.candidates.iter().map(|x| x.column.clone()).collect(),
                    });
                }
            }
        }

        None
    }

    fn check_when(
        &self,
        when: &crate::rules::CandidateWhen,
        siblings: &[&str],
        entity_type: Option<&str>,
    ) -> bool {
        // Check entity_type condition if present
        if let Some(ref required_entity_type) = when.entity_type {
            if let Some(actual) = entity_type {
                if actual != required_entity_type.as_str() {
                    return false;
                }
            } else {
                // No entity_type provided but candidate requires one => skip
                return false;
            }
        }

        // Check siblings condition
        for required in &when.siblings_contain {
            if !siblings.contains(&required.as_str()) {
                return false;
            }
        }
        true
    }
}

// ---------------------------------------------------------------------------
// Confidence → Tier
// ---------------------------------------------------------------------------

pub fn tier_from_confidence(c: f64) -> Tier {
    if c >= 0.85 {
        Tier::Safe
    } else if c >= 0.50 {
        Tier::Suggest
    } else {
        Tier::Unclear
    }
}
// ---------------------------------------------------------------------------
// Scalar inference matcher
// ---------------------------------------------------------------------------

use crate::rules::ScalarInference;

pub struct ScalarMatcher<'a> {
    rules: &'a ScalarInference,
}

impl<'a> ScalarMatcher<'a> {
    pub fn new(rules: &'a ScalarInference) -> Self {
        Self { rules }
    }

    pub fn match_scalar(&self, field_name: &str) -> Option<FieldMapping> {
        let lower = field_name.to_lowercase();
        for rule in &self.rules.rules {
            if rule
                .triggers
                .iter()
                .any(|t| lower == t.as_str() || lower.contains(t.as_str()))
            {
                return Some(FieldMapping {
                    json_path: field_name.into(),
                    column: Some(rule.column_prefix.clone()),
                    scalar_table: Some(rule.scalar_table.clone()),
                    ref_table: None,
                    tier: tier_from_confidence(rule.confidence),
                    confidence: rule.confidence,
                    source: "scalar_inference".into(),
                    alternatives: vec![],
                });
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Nesting decision matcher
// ---------------------------------------------------------------------------

use crate::output::{NestedInput, RelationshipMapping};
use crate::rules::NestingRule;

pub struct NestingMatcher<'a> {
    rules: &'a [NestingRule],
}

impl<'a> NestingMatcher<'a> {
    pub fn new(rules: &'a [NestingRule]) -> Self {
        Self { rules }
    }

    pub fn decide_nesting(
        &self,
        nested: &NestedInput,
        fields_shared: bool,
        structure_variable: bool,
    ) -> Option<RelationshipMapping> {
        let field_count = nested.items.fields.len();

        for rule in self.rules {
            // Check structure condition if present
            if let Some(ref required_structure) = rule.when.structure {
                if required_structure == "variable" && !structure_variable {
                    continue;
                }
            }

            // Check shared condition if present
            if let Some(required_shared) = rule.when.shared {
                if required_shared && !fields_shared {
                    continue;
                }
            }

            let matches = match (&rule.when.is_array, &rule.when.is_object) {
                (true, _) if nested.nested_type == "array" => {
                    self.check_count(field_count, &rule.when.element_has_fields)
                }
                (_, true) if nested.nested_type == "object" => {
                    self.check_count(field_count, &rule.when.fields_count)
                }
                _ => false,
            };

            if matches {
                return Some(RelationshipMapping {
                    target: nested.items.name.clone(),
                    rel_type: rule
                        .relationship
                        .clone()
                        .unwrap_or_else(|| rule.action.clone()),
                    via: None,
                    tier: tier_from_confidence(rule.confidence),
                    confidence: rule.confidence,
                    source: "nesting_rule".into(),
                });
            }
        }
        None
    }

    fn check_count(&self, actual: usize, constraint: &Option<String>) -> bool {
        match constraint.as_deref() {
            Some(">2") => actual > 2,
            Some("≤3") => actual <= 3,
            Some(">5") => actual > 5,
            _ => true,
        }
    }
}
