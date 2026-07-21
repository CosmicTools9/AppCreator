//! 本体规约验证器。

use crate::state::{PlanViolation, PlanViolationKind, PlatformCatalog};
use alioth_gen::generator::ir::ontology::{DomainKind, OntologyModel};
use std::collections::{HashMap, HashSet};

pub fn validate_ontology_model(
    model: &OntologyModel,
    catalog: &PlatformCatalog,
) -> Vec<PlanViolation> {
    let mut violations = Vec::new();
    check_undefined_domains(model, catalog, &mut violations);
    check_inheritance_constraints(model, catalog, &mut violations);
    check_circular_relations(model, &mut violations);
    violations
}

fn check_undefined_domains(
    model: &OntologyModel,
    catalog: &PlatformCatalog,
    violations: &mut Vec<PlanViolation>,
) {
    let known_domains: HashSet<&str> = model.domains.iter().map(|d| d.id.as_str()).collect();
    for rel in &model.relations {
        let src = &rel.source_ontology;
        if !known_domains.contains(src.as_str()) {
            let is_table = catalog.collections.iter().any(|c| c.table_name == *src);
            if !is_table {
                violations.push(PlanViolation {
                    kind: PlanViolationKind::UndefinedReference,
                    detail: format!("relation '{}' references unknown '{}'", rel.id, src),
                    fixable: true,
                });
            }
        }
        let tgt = &rel.target_ontology;
        if !known_domains.contains(tgt.as_str()) {
            let is_table = catalog.collections.iter().any(|c| c.table_name == *tgt);
            if !is_table {
                violations.push(PlanViolation {
                    kind: PlanViolationKind::UndefinedReference,
                    detail: format!("relation '{}' references unknown '{}'", rel.id, tgt),
                    fixable: true,
                });
            }
        }
    }
}

fn check_inheritance_constraints(
    model: &OntologyModel,
    catalog: &PlatformCatalog,
    violations: &mut Vec<PlanViolation>,
) {
    let all_known: HashSet<&str> = catalog
        .collections
        .iter()
        .map(|c| c.table_name.as_str())
        .collect();
    for domain in &model.domains {
        if all_known.contains(domain.id.as_str()) {
            continue;
        }
        if domain.parent_ids.is_empty() {
            if let DomainKind::Entity = &domain.kind {
                violations.push(PlanViolation {
                    kind: PlanViolationKind::MissingCriticalInfo,
                    detail: format!("new entity '{}' has no parent_ids", domain.id),
                    fixable: true,
                });
            }
        }
    }
}

fn check_circular_relations(model: &OntologyModel, violations: &mut Vec<PlanViolation>) {
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for rel in &model.relations {
        let s = rel.source_ontology.as_str();
        let t = rel.target_ontology.as_str();
        if s != t {
            adj.entry(s).or_default().push(t);
        }
    }
    let mut white: HashSet<&str> = adj.keys().copied().collect();
    let mut gray: HashSet<&str> = HashSet::new();
    let mut black: HashSet<&str> = HashSet::new();
    while let Some(seed) = white.iter().next().copied() {
        if dfs_cycle(seed, &adj, &mut white, &mut gray, &mut black) {
            violations.push(PlanViolation {
                kind: PlanViolationKind::CircularDependency,
                detail: "circular relation dependency detected".into(),
                fixable: true,
            });
            break;
        }
    }
}

fn dfs_cycle<'a>(
    node: &'a str,
    adj: &HashMap<&'a str, Vec<&'a str>>,
    white: &mut HashSet<&'a str>,
    gray: &mut HashSet<&'a str>,
    black: &mut HashSet<&'a str>,
) -> bool {
    white.remove(node);
    gray.insert(node);
    if let Some(neighbors) = adj.get(node) {
        for &next in neighbors {
            if gray.contains(next) {
                return true;
            }
            if white.contains(next) && dfs_cycle(next, adj, white, gray, black) {
                return true;
            }
        }
    }
    gray.remove(node);
    black.insert(node);
    false
}
