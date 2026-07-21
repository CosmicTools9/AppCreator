//! Rule Conflict Detection Module
//!
//! Detects conflicts between rules including:
//! - Contradictory rules (same condition, opposite conclusions)
//! - Circular dependencies (rule A triggers B, B triggers A)
//! - Overlapping rules (same conclusion, different conditions)
//! - Redundant rules (same condition and conclusion)
//!
//! ## Example Conflicts
//!
//! ```text
//! // Contradictory rules
//! rule1: IF Person(age >= 18) THEN Adult(Person)
//! rule2: IF Person(age >= 18) THEN NOT Adult(Person)
//!
//! // Circular dependency
//! rule1: IF A(x) THEN B(x)
//! rule2: IF B(x) THEN C(x)
//! rule3: IF C(x) THEN A(x)
//! ```

use crate::generator::ir::ontology::{DomainOntology, OntologyModel};
use runtime_contract::swrl::{ComparisonOp, RuleAtom, SwrlRule, SwrlRuleSet, Term};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Conflict detection report
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConflictReport {
    /// Detected conflicts
    pub conflicts: Vec<RuleConflict>,
    /// Detected circular dependencies
    pub circular_deps: Vec<CircularDependency>,
    /// Detected overlapping rules
    pub overlaps: Vec<RuleOverlap>,
    /// Detected redundant rules
    pub redundancies: Vec<RuleRedundancy>,
    /// Warnings (non-critical issues)
    pub warnings: Vec<ConflictWarning>,
}

impl ConflictReport {
    /// Create a new empty report
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any conflicts were detected
    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty() || !self.circular_deps.is_empty()
    }

    /// Check if any issues were detected (including warnings)
    pub fn has_issues(&self) -> bool {
        self.has_conflicts()
            || !self.overlaps.is_empty()
            || !self.redundancies.is_empty()
            || !self.warnings.is_empty()
    }

    /// Get total number of issues
    pub fn issue_count(&self) -> usize {
        self.conflicts.len()
            + self.circular_deps.len()
            + self.overlaps.len()
            + self.redundancies.len()
            + self.warnings.len()
    }

    /// Add a conflict
    pub fn add_conflict(&mut self, conflict: RuleConflict) {
        self.conflicts.push(conflict);
    }

    /// Add a circular dependency
    pub fn add_circular_dep(&mut self, dep: CircularDependency) {
        self.circular_deps.push(dep);
    }

    /// Add an overlap
    pub fn add_overlap(&mut self, overlap: RuleOverlap) {
        self.overlaps.push(overlap);
    }

    /// Add a redundancy
    pub fn add_redundancy(&mut self, redundancy: RuleRedundancy) {
        self.redundancies.push(redundancy);
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: ConflictWarning) {
        self.warnings.push(warning);
    }

    /// Get all critical issues (conflicts and circular deps)
    pub fn critical_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();

        for conflict in &self.conflicts {
            issues.push(format!("Conflict: {}", conflict.description));
        }

        for dep in &self.circular_deps {
            issues.push(format!("Circular dependency: {}", dep.description));
        }

        issues
    }
}

/// A detected rule conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConflict {
    /// Type of conflict
    pub conflict_type: ConflictType,
    /// Names of conflicting rules
    pub rule_names: Vec<String>,
    /// Human-readable description
    pub description: String,
    /// Suggested resolution
    #[serde(default)]
    pub suggested_resolution: Option<String>,
    /// Severity level
    pub severity: ConflictSeverity,
}

/// Type of conflict
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictType {
    /// Rules have same condition but opposite conclusions
    Contradictory,
    /// Rules produce the same conclusion with mutually exclusive conditions
    MutuallyExclusive,
    /// Rules have inconsistent priorities
    InconsistentPriority,
    /// One rule subsumes another (more general condition, same conclusion)
    Subsumption,
}

/// Conflict severity
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConflictSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for ConflictSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConflictSeverity::Low => write!(f, "Low"),
            ConflictSeverity::Medium => write!(f, "Medium"),
            ConflictSeverity::High => write!(f, "High"),
            ConflictSeverity::Critical => write!(f, "Critical"),
        }
    }
}

/// Circular dependency between rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularDependency {
    /// Names of rules in the cycle
    pub rule_names: Vec<String>,
    /// Description of the cycle
    pub description: String,
    /// Severity
    pub severity: ConflictSeverity,
}

/// Overlapping rules (same conclusion, different conditions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleOverlap {
    /// Names of overlapping rules
    pub rule_names: Vec<String>,
    /// The shared conclusion
    pub conclusion: String,
    /// Description
    pub description: String,
    /// Whether conditions are mutually exclusive
    pub mutually_exclusive: bool,
}

/// Redundant rules (same condition and conclusion)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleRedundancy {
    /// Names of redundant rules
    pub rule_names: Vec<String>,
    /// Description
    pub description: String,
}

/// Conflict warning (non-critical)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictWarning {
    /// Warning message
    pub message: String,
    /// Affected rule names
    pub rule_names: Vec<String>,
    /// Warning type
    pub warning_type: WarningType,
}

/// Type of warning
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WarningType {
    /// Rule is unreachable
    Unreachable,
    /// Rule shadowed by another
    Shadowed,
    /// Rule has no effect
    NoEffect,
    /// Rule has overly complex condition
    ComplexCondition,
}

/// Conflict detector
pub struct ConflictDetector;

impl ConflictDetector {
    /// Analyze a rule set for conflicts
    pub fn analyze(rule_set: &SwrlRuleSet) -> ConflictReport {
        let mut report = ConflictReport::new();

        // Get active rules
        let rules: Vec<&SwrlRule> = rule_set.get_active_rules();

        // Detect contradictory rules
        Self::detect_contradictions(&rules, &mut report);

        // Detect circular dependencies
        Self::detect_circular_deps(&rules, &mut report);

        // Detect overlapping rules
        Self::detect_overlaps(&rules, &mut report);

        // Detect redundancies
        Self::detect_redundancies(&rules, &mut report);

        // Detect unreachable/shadowed rules
        Self::detect_unreachable_rules(&rules, &mut report);

        report
    }

    /// Detect contradictory rules
    fn detect_contradictions(rules: &[&SwrlRule], report: &mut ConflictReport) {
        for i in 0..rules.len() {
            for j in (i + 1)..rules.len() {
                let rule1 = rules[i];
                let rule2 = rules[j];

                // Check if conditions are similar
                if Self::conditions_similar(&rule1.body, &rule2.body) {
                    // Check if conclusions are contradictory
                    if Self::conclusions_contradictory(&rule1.head, &rule2.head) {
                        report.add_conflict(RuleConflict {
                            conflict_type: ConflictType::Contradictory,
                            rule_names: vec![rule1.name.clone(), rule2.name.clone()],
                            description: format!(
                                "Rules '{}' and '{}' have similar conditions but contradictory conclusions",
                                rule1.name, rule2.name
                            ),
                            suggested_resolution: Some(
                                "Review conditions or make conditions mutually exclusive".to_string()
                            ),
                            severity: ConflictSeverity::Critical,
                        });
                    }
                }
            }
        }
    }

    /// Detect circular dependencies
    fn detect_circular_deps(rules: &[&SwrlRule], report: &mut ConflictReport) {
        // Build dependency graph: rule A -> rule B if A's conclusion triggers B's condition
        let mut dependencies: HashMap<String, Vec<String>> = HashMap::new();

        for rule in rules {
            let mut deps = Vec::new();

            // Find which rules could be triggered by this rule's conclusions
            for other in rules {
                if rule.name != other.name {
                    for conclusion in &rule.head {
                        for condition in &other.body {
                            if Self::atom_triggers(conclusion, condition) {
                                deps.push(other.name.clone());
                                break;
                            }
                        }
                    }
                }
            }

            dependencies.insert(rule.name.clone(), deps);
        }

        // Detect cycles using DFS
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for rule in rules {
            if !visited.contains(&rule.name) {
                Self::detect_cycle(
                    &rule.name,
                    &dependencies,
                    &mut visited,
                    &mut rec_stack,
                    &mut Vec::new(),
                    report,
                );
            }
        }
    }

    /// DFS cycle detection
    fn detect_cycle(
        rule_name: &str,
        dependencies: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        report: &mut ConflictReport,
    ) {
        visited.insert(rule_name.to_string());
        rec_stack.insert(rule_name.to_string());
        path.push(rule_name.to_string());

        if let Some(deps) = dependencies.get(rule_name) {
            for dep in deps {
                if !visited.contains(dep) {
                    Self::detect_cycle(dep, dependencies, visited, rec_stack, path, report);
                } else if rec_stack.contains(dep) {
                    // Found a cycle
                    if let Some(pos) = path.iter().position(|r| r == dep) {
                        let cycle: Vec<String> = path[pos..].to_vec();
                        report.add_circular_dep(CircularDependency {
                            rule_names: cycle.clone(),
                            description: format!(
                                "Circular dependency detected: {}",
                                cycle.join(" -> ")
                            ),
                            severity: ConflictSeverity::High,
                        });
                    }
                }
            }
        }

        path.pop();
        rec_stack.remove(rule_name);
    }

    /// Detect overlapping rules
    fn detect_overlaps(rules: &[&SwrlRule], report: &mut ConflictReport) {
        // Group rules by conclusion class
        let mut by_conclusion: HashMap<String, Vec<&SwrlRule>> = HashMap::new();

        for rule in rules {
            for atom in &rule.head {
                if let RuleAtom::ClassAssertion(class_name, _) = atom {
                    by_conclusion
                        .entry(class_name.clone())
                        .or_default()
                        .push(rule);
                }
            }
        }

        // Check for overlaps
        for (conclusion, rule_group) in by_conclusion {
            if rule_group.len() > 1 {
                // Check if conditions are mutually exclusive
                let mut mutually_exclusive = false;

                for i in 0..rule_group.len() {
                    for j in (i + 1)..rule_group.len() {
                        if Self::conditions_mutually_exclusive(
                            &rule_group[i].body,
                            &rule_group[j].body,
                        ) {
                            mutually_exclusive = true;
                            break;
                        }
                    }
                }

                if !mutually_exclusive {
                    let rule_names: Vec<String> =
                        rule_group.iter().map(|r| r.name.clone()).collect();

                    report.add_overlap(RuleOverlap {
                        rule_names: rule_names.clone(),
                        conclusion: conclusion.clone(),
                        description: format!(
                            "Rules {} all conclude {} but conditions may overlap",
                            rule_names.join(", "),
                            conclusion
                        ),
                        mutually_exclusive: false,
                    });
                }
            }
        }
    }

    /// Detect redundant rules
    fn detect_redundancies(rules: &[&SwrlRule], report: &mut ConflictReport) {
        for i in 0..rules.len() {
            for j in (i + 1)..rules.len() {
                let rule1 = rules[i];
                let rule2 = rules[j];

                // Check if both conditions and conclusions are equivalent
                if Self::conditions_equivalent(&rule1.body, &rule2.body)
                    && Self::conclusions_equivalent(&rule1.head, &rule2.head)
                {
                    report.add_redundancy(RuleRedundancy {
                        rule_names: vec![rule1.name.clone(), rule2.name.clone()],
                        description: format!(
                            "Rules '{}' and '{}' are redundant (same conditions and conclusions)",
                            rule1.name, rule2.name
                        ),
                    });
                }
            }
        }
    }

    /// Detect unreachable or shadowed rules
    fn detect_unreachable_rules(rules: &[&SwrlRule], report: &mut ConflictReport) {
        // A rule is shadowed if another rule has the same or more general conclusion
        // and the same or more general condition
        for i in 0..rules.len() {
            for j in 0..rules.len() {
                if i != j {
                    let rule1 = rules[i];
                    let rule2 = rules[j];

                    // Check if rule2 shadows rule1
                    if Self::conditions_subsume(&rule2.body, &rule1.body)
                        && Self::conclusions_subsume(&rule2.head, &rule1.head)
                        && rule2.priority >= rule1.priority
                    {
                        report.add_warning(ConflictWarning {
                            message: format!(
                                "Rule '{}' is shadowed by rule '{}'",
                                rule1.name, rule2.name
                            ),
                            rule_names: vec![rule1.name.clone(), rule2.name.clone()],
                            warning_type: WarningType::Shadowed,
                        });
                    }
                }
            }
        }
    }

    /// Check if two conditions are similar
    fn conditions_similar(cond1: &[RuleAtom], cond2: &[RuleAtom]) -> bool {
        // Simple check: same number of atoms and some overlap
        if cond1.len() != cond2.len() {
            return false;
        }

        let mut matches = 0;
        for atom1 in cond1 {
            for atom2 in cond2 {
                if Self::atoms_similar(atom1, atom2) {
                    matches += 1;
                    break;
                }
            }
        }

        matches == cond1.len()
    }

    /// Check if two atoms are similar
    fn atoms_similar(atom1: &RuleAtom, atom2: &RuleAtom) -> bool {
        match (atom1, atom2) {
            (RuleAtom::ClassAssertion(name1, term1), RuleAtom::ClassAssertion(name2, term2)) => {
                name1 == name2 && Self::terms_similar(term1, term2)
            }
            (
                RuleAtom::PropertyAssertion(prop1, sub1, obj1),
                RuleAtom::PropertyAssertion(prop2, sub2, obj2),
            ) => {
                prop1 == prop2 && Self::terms_similar(sub1, sub2) && Self::terms_similar(obj1, obj2)
            }
            (RuleAtom::Comparison(t1_1, op1, t1_2), RuleAtom::Comparison(t2_1, op2, t2_2)) => {
                op1 == op2 && Self::terms_similar(t1_1, t2_1) && Self::terms_similar(t1_2, t2_2)
            }
            _ => false,
        }
    }

    /// Check if two terms are similar
    fn terms_similar(term1: &Term, term2: &Term) -> bool {
        match (term1, term2) {
            (Term::Variable(_), Term::Variable(_)) => true,
            (Term::Individual(i1), Term::Individual(i2)) => i1 == i2,
            (Term::Literal(l1), Term::Literal(l2)) => l1 == l2,
            _ => false,
        }
    }

    /// Check if two conclusions are contradictory
    fn conclusions_contradictory(head1: &[RuleAtom], head2: &[RuleAtom]) -> bool {
        // Check if one conclusion negates the other
        for atom1 in head1 {
            for atom2 in head2 {
                if Self::atoms_contradictory(atom1, atom2) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if two atoms are contradictory
    fn atoms_contradictory(atom1: &RuleAtom, atom2: &RuleAtom) -> bool {
        match (atom1, atom2) {
            (RuleAtom::ClassAssertion(name1, term1), RuleAtom::ClassAssertion(name2, term2)) => {
                // Check for NOT relationship (simplified - in practice would check ontology)
                let not_name1 = format!("Not{}", name1);
                let not_name2 = format!("Not{}", name2);
                (name1 == &not_name2 || name2 == &not_name1) && Self::terms_similar(term1, term2)
            }
            (RuleAtom::Comparison(t1_1, op1, t1_2), RuleAtom::Comparison(t2_1, op2, t2_2)) => {
                // Check for contradictory comparisons like x > 5 and x < 3
                Self::terms_similar(t1_1, t2_1)
                    && Self::terms_similar(t1_2, t2_2)
                    && Self::comparison_ops_contradictory(*op1, *op2)
            }
            _ => false,
        }
    }

    /// Check if two comparison operators are contradictory
    fn comparison_ops_contradictory(op1: ComparisonOp, op2: ComparisonOp) -> bool {
        matches!(
            (op1, op2),
            (ComparisonOp::Gt, ComparisonOp::Le)
                | (ComparisonOp::Ge, ComparisonOp::Lt)
                | (ComparisonOp::Lt, ComparisonOp::Ge)
                | (ComparisonOp::Le, ComparisonOp::Gt)
        )
    }

    /// Check if one atom triggers another
    fn atom_triggers(conclusion: &RuleAtom, condition: &RuleAtom) -> bool {
        // Simplified: check if conclusion class assertion matches condition class assertion
        match (conclusion, condition) {
            (
                RuleAtom::ClassAssertion(c_name, c_term),
                RuleAtom::ClassAssertion(cond_name, cond_term),
            ) => c_name == cond_name && Self::terms_unify(c_term, cond_term),
            _ => false,
        }
    }

    /// Check if two terms can unify
    fn terms_unify(term1: &Term, term2: &Term) -> bool {
        match (term1, term2) {
            (Term::Variable(_), _) => true,
            (_, Term::Variable(_)) => true,
            (Term::Individual(i1), Term::Individual(i2)) => i1 == i2,
            (Term::Literal(l1), Term::Literal(l2)) => l1 == l2,
            _ => false,
        }
    }

    /// Check if conditions are mutually exclusive
    fn conditions_mutually_exclusive(cond1: &[RuleAtom], cond2: &[RuleAtom]) -> bool {
        // Check if there's a direct contradiction in conditions
        for atom1 in cond1 {
            for atom2 in cond2 {
                if Self::atoms_contradictory(atom1, atom2) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if conditions are equivalent
    fn conditions_equivalent(cond1: &[RuleAtom], cond2: &[RuleAtom]) -> bool {
        if cond1.len() != cond2.len() {
            return false;
        }

        for atom1 in cond1 {
            let mut found = false;
            for atom2 in cond2 {
                if Self::atoms_equivalent(atom1, atom2) {
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }

        true
    }

    /// Check if atoms are equivalent
    fn atoms_equivalent(atom1: &RuleAtom, atom2: &RuleAtom) -> bool {
        match (atom1, atom2) {
            (RuleAtom::ClassAssertion(n1, t1), RuleAtom::ClassAssertion(n2, t2)) => {
                n1 == n2 && t1 == t2
            }
            (RuleAtom::PropertyAssertion(p1, s1, o1), RuleAtom::PropertyAssertion(p2, s2, o2)) => {
                p1 == p2 && s1 == s2 && o1 == o2
            }
            (RuleAtom::Comparison(t1_1, op1, t1_2), RuleAtom::Comparison(t2_1, op2, t2_2)) => {
                op1 == op2 && t1_1 == t2_1 && t1_2 == t2_2
            }
            _ => false,
        }
    }

    /// Check if conclusions are equivalent
    fn conclusions_equivalent(head1: &[RuleAtom], head2: &[RuleAtom]) -> bool {
        Self::conditions_equivalent(head1, head2)
    }

    /// Check if conditions subsume (more general than)
    fn conditions_subsume(general: &[RuleAtom], specific: &[RuleAtom]) -> bool {
        // General conditions should be a subset of specific conditions
        for gen_atom in general {
            let mut found = false;
            for spec_atom in specific {
                if Self::atoms_subsume(gen_atom, spec_atom) {
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }
        true
    }

    /// Check if one atom subsumes another
    fn atoms_subsume(general: &RuleAtom, specific: &RuleAtom) -> bool {
        match (general, specific) {
            (
                RuleAtom::ClassAssertion(g_name, g_term),
                RuleAtom::ClassAssertion(s_name, s_term),
            ) => g_name == s_name && Self::term_subsumes(g_term, s_term),
            (
                RuleAtom::PropertyAssertion(g_p, g_s, g_o),
                RuleAtom::PropertyAssertion(s_p, s_s, s_o),
            ) => g_p == s_p && Self::term_subsumes(g_s, s_s) && Self::term_subsumes(g_o, s_o),
            _ => false,
        }
    }

    /// Check if one term subsumes another
    fn term_subsumes(general: &Term, specific: &Term) -> bool {
        match (general, specific) {
            (Term::Variable(_), _) => true,
            (Term::Individual(g), Term::Individual(s)) => g == s,
            (Term::Literal(g), Term::Literal(s)) => g == s,
            _ => false,
        }
    }

    /// Check if conclusions subsume
    fn conclusions_subsume(general: &[RuleAtom], specific: &[RuleAtom]) -> bool {
        Self::conditions_subsume(general, specific)
    }
}

/// Inconsistency report for ontology validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InconsistencyReport {
    /// Whether the ontology is consistent
    pub is_consistent: bool,
    /// List of inconsistencies
    pub inconsistencies: Vec<Inconsistency>,
}

/// An inconsistency in the ontology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inconsistency {
    /// Type of inconsistency
    pub inconsistency_type: InconsistencyType,
    /// Description
    pub description: String,
    /// Related entities/classes
    pub related_entities: Vec<String>,
    /// Related rules
    pub related_rules: Vec<String>,
}

/// Type of inconsistency
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum InconsistencyType {
    /// Class is both subclass and disjoint with another
    SubclassAndDisjoint,
    /// Property domain/range mismatch
    PropertyMismatch,
    /// Cardinality violation
    CardinalityViolation,
    /// Circular subclass relationship
    CircularSubclass,
    /// Contradictory facts
    ContradictoryFacts,
}

// ============================================================================
// Module Conflict Detection
// ============================================================================

/// Module conflict type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModuleConflictType {
    /// Dependency cycle between modules
    DependencyCycle,
    /// Namespace collision
    NamespaceCollision,
    /// Version mismatch for same module
    VersionMismatch,
}

/// A module-level conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConflict {
    /// Conflict type
    pub conflict_type: ModuleConflictType,
    /// Human-readable description
    pub description: String,
    /// Severity
    pub severity: ConflictSeverity,
}

/// Module conflict report
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModuleConflictReport {
    /// Dependency conflicts
    pub dependency_conflicts: Vec<ModuleConflict>,
    /// Namespace conflicts
    pub namespace_conflicts: Vec<ModuleConflict>,
    /// Version conflicts
    pub version_conflicts: Vec<ModuleConflict>,
}

impl ModuleConflictReport {
    /// Create a new empty report
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any conflicts were detected
    pub fn has_conflicts(&self) -> bool {
        !self.dependency_conflicts.is_empty()
            || !self.namespace_conflicts.is_empty()
            || !self.version_conflicts.is_empty()
    }

    /// Total number of conflicts
    pub fn total_count(&self) -> usize {
        self.dependency_conflicts.len()
            + self.namespace_conflicts.len()
            + self.version_conflicts.len()
    }
}

/// Module conflict detector
pub struct ModuleConflictDetector;

impl ModuleConflictDetector {
    /// Analyze an ontology model for module-level conflicts
    pub fn analyze(model: &OntologyModel) -> ModuleConflictReport {
        let mut report = ModuleConflictReport::new();

        Self::detect_dependency_cycles(model, &mut report);
        Self::detect_namespace_collisions(model, &mut report);
        Self::detect_version_mismatches(model, &mut report);

        report
    }

    /// Detect internal dependency cycles via prefab_contract references
    fn detect_dependency_cycles(model: &OntologyModel, report: &mut ModuleConflictReport) {
        let domain_map: HashMap<String, &DomainOntology> =
            model.domains.iter().map(|d| (d.id.clone(), d)).collect();

        // Build graph: domain id -> prefab_id (target domain id)
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        for domain in &model.domains {
            if let Some(ref contract) = domain.prefab_contract {
                // If prefab_id matches another domain id, treat it as a dependency
                if domain_map.contains_key(&contract.prefab_id) {
                    graph
                        .entry(domain.id.clone())
                        .or_default()
                        .push(contract.prefab_id.clone());
                }
            }
        }

        // Also check metadata.dependencies self-reference
        if model.metadata.dependencies.contains(&model.id) {
            report.dependency_conflicts.push(ModuleConflict {
                conflict_type: ModuleConflictType::DependencyCycle,
                description: format!("Ontology '{}' declares a dependency on itself", model.id),
                severity: ConflictSeverity::Critical,
            });
        }

        // DFS cycle detection in prefab graph
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for domain in &model.domains {
            if !visited.contains(&domain.id) {
                Self::detect_cycle_dfs(
                    &domain.id,
                    &graph,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    report,
                );
            }
        }
    }

    fn detect_cycle_dfs(
        node: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        report: &mut ModuleConflictReport,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(deps) = graph.get(node) {
            for dep in deps {
                if !visited.contains(dep) {
                    Self::detect_cycle_dfs(dep, graph, visited, rec_stack, path, report);
                } else if rec_stack.contains(dep) {
                    if let Some(pos) = path.iter().position(|p| p == dep) {
                        let cycle = path[pos..].to_vec();
                        report.dependency_conflicts.push(ModuleConflict {
                            conflict_type: ModuleConflictType::DependencyCycle,
                            description: format!(
                                "Module dependency cycle detected: {}",
                                cycle.join(" -> ")
                            ),
                            severity: ConflictSeverity::High,
                        });
                    }
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
    }

    /// Detect namespace collisions (different prefixes mapping to same URI)
    fn detect_namespace_collisions(model: &OntologyModel, report: &mut ModuleConflictReport) {
        let mut uri_to_prefix: HashMap<String, String> = HashMap::new();
        for (prefix, uri) in &model.namespaces {
            if let Some(existing_prefix) = uri_to_prefix.get(uri) {
                report.namespace_conflicts.push(ModuleConflict {
                    conflict_type: ModuleConflictType::NamespaceCollision,
                    description: format!(
                        "Namespaces '{}' and '{}' both map to '{}'",
                        existing_prefix, prefix, uri
                    ),
                    severity: ConflictSeverity::Medium,
                });
            } else {
                uri_to_prefix.insert(uri.clone(), prefix.clone());
            }
        }
    }

    /// Detect version mismatches for the same prefab across domains
    fn detect_version_mismatches(model: &OntologyModel, report: &mut ModuleConflictReport) {
        let mut prefab_versions: HashMap<String, Vec<String>> = HashMap::new();
        for domain in &model.domains {
            if let Some(ref contract) = domain.prefab_contract {
                prefab_versions
                    .entry(contract.prefab_id.clone())
                    .or_default()
                    .push(contract.interface_version.clone());
            }
        }

        for (prefab_id, versions) in &prefab_versions {
            let unique_versions: HashSet<String> = versions.iter().cloned().collect();
            if unique_versions.len() > 1 {
                report.version_conflicts.push(ModuleConflict {
                    conflict_type: ModuleConflictType::VersionMismatch,
                    description: format!(
                        "Prefab '{}' has incompatible version requirements: {:?}",
                        prefab_id, versions
                    ),
                    severity: ConflictSeverity::High,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_contract::swrl::{RuleAtom, SwrlRule, SwrlRuleSet, Term};

    #[test]
    fn test_detect_contradictory_rules() {
        let mut rule_set = SwrlRuleSet::new();

        // Rule 1: IF Person(age >= 18) THEN Adult(Person)
        let rule1 = SwrlRule::new("adultRule")
            .add_condition(RuleAtom::class("Person", Term::variable("x")))
            .add_condition(RuleAtom::Comparison(
                Term::variable("age"),
                ComparisonOp::Ge,
                Term::integer(18),
            ))
            .add_conclusion(RuleAtom::class("Adult", Term::variable("x")));

        // Rule 2: IF Person(age >= 18) THEN NOT Adult(Person)
        let rule2 = SwrlRule::new("notAdultRule")
            .add_condition(RuleAtom::class("Person", Term::variable("x")))
            .add_condition(RuleAtom::Comparison(
                Term::variable("age"),
                ComparisonOp::Ge,
                Term::integer(18),
            ))
            .add_conclusion(RuleAtom::class("NotAdult", Term::variable("x")));

        rule_set.add(rule1);
        rule_set.add(rule2);

        let report = ConflictDetector::analyze(&rule_set);
        assert!(report.has_conflicts());
    }

    #[test]
    fn test_detect_circular_dependency() {
        let mut rule_set = SwrlRuleSet::new();

        // A -> B -> C -> A
        let rule1 = SwrlRule::new("ruleA")
            .add_condition(RuleAtom::class("A", Term::variable("x")))
            .add_conclusion(RuleAtom::class("B", Term::variable("x")));

        let rule2 = SwrlRule::new("ruleB")
            .add_condition(RuleAtom::class("B", Term::variable("x")))
            .add_conclusion(RuleAtom::class("C", Term::variable("x")));

        let rule3 = SwrlRule::new("ruleC")
            .add_condition(RuleAtom::class("C", Term::variable("x")))
            .add_conclusion(RuleAtom::class("A", Term::variable("x")));

        rule_set.add(rule1);
        rule_set.add(rule2);
        rule_set.add(rule3);

        let report = ConflictDetector::analyze(&rule_set);
        assert!(!report.circular_deps.is_empty());
    }

    #[test]
    fn test_detect_redundant_rules() {
        let mut rule_set = SwrlRuleSet::new();

        let rule1 = SwrlRule::new("rule1")
            .add_condition(RuleAtom::class("Person", Term::variable("x")))
            .add_conclusion(RuleAtom::class("Adult", Term::variable("x")));

        let rule2 = SwrlRule::new("rule2")
            .add_condition(RuleAtom::class("Person", Term::variable("x")))
            .add_conclusion(RuleAtom::class("Adult", Term::variable("x")));

        rule_set.add(rule1);
        rule_set.add(rule2);

        let report = ConflictDetector::analyze(&rule_set);
        assert!(!report.redundancies.is_empty());
    }

    #[test]
    fn test_conflict_report() {
        let mut report = ConflictReport::new();
        assert!(!report.has_conflicts());
        assert!(!report.has_issues());
        assert_eq!(report.issue_count(), 0);

        report.add_warning(ConflictWarning {
            message: "Test warning".to_string(),
            rule_names: vec!["rule1".to_string()],
            warning_type: WarningType::Unreachable,
        });

        assert!(report.has_issues());
        assert_eq!(report.issue_count(), 1);
    }

    #[test]
    fn test_module_dependency_conflict() {
        let model = OntologyModel {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: None,
            version: "1.0".to_string(),
            domains: vec![
                DomainOntology {
                    id: "order".to_string(),
                    name: "Order".to_string(),
                    description: None,
                    kind: crate::generator::ir::ontology::DomainKind::AggregateRoot,
                    parent_ids: vec![],
                    equivalent_ids: vec![],
                    disjoint_ids: vec![],
                    properties: vec![],
                    prefab_contract: Some(crate::generator::ir::ontology::PrefabContract {
                        prefab_type: crate::generator::ir::ontology::PrefabType::Module,
                        prefab_id: "inventory".to_string(),
                        interface_version: "1.0".to_string(),
                        interfaces: vec![],
                        configuration: HashMap::new(),
                    }),
                },
                DomainOntology {
                    id: "inventory".to_string(),
                    name: "Inventory".to_string(),
                    description: None,
                    kind: crate::generator::ir::ontology::DomainKind::Entity,
                    parent_ids: vec![],
                    equivalent_ids: vec![],
                    disjoint_ids: vec![],
                    properties: vec![],
                    prefab_contract: Some(crate::generator::ir::ontology::PrefabContract {
                        prefab_type: crate::generator::ir::ontology::PrefabType::Module,
                        prefab_id: "order".to_string(),
                        interface_version: "1.0".to_string(),
                        interfaces: vec![],
                        configuration: HashMap::new(),
                    }),
                },
            ],
            transaction_lifecycle: None,
            relations: vec![],
            constraints: vec![],
            computations: vec![],
            namespaces: HashMap::new(),
            metadata: crate::generator::ir::ontology::OntologyMetadata::default(),
        };

        let report = ModuleConflictDetector::analyze(&model);
        assert!(
            report.has_conflicts(),
            "Should detect module dependency cycle"
        );
        assert!(
            report
                .dependency_conflicts
                .iter()
                .any(|c| { c.conflict_type == ModuleConflictType::DependencyCycle }),
            "Should have a DependencyCycle conflict"
        );
    }

    #[test]
    fn test_namespace_conflict() {
        let model = OntologyModel {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: None,
            version: "1.0".to_string(),
            domains: vec![],
            transaction_lifecycle: None,
            relations: vec![],
            constraints: vec![],
            computations: vec![],
            namespaces: {
                let mut ns = HashMap::new();
                ns.insert("ns1".to_string(), "http://example.org".to_string());
                ns.insert("ns2".to_string(), "http://example.org".to_string());
                ns
            },
            metadata: crate::generator::ir::ontology::OntologyMetadata::default(),
        };

        let report = ModuleConflictDetector::analyze(&model);
        assert!(
            report
                .namespace_conflicts
                .iter()
                .any(|c| { c.conflict_type == ModuleConflictType::NamespaceCollision }),
            "Should detect namespace collision"
        );
    }

    #[test]
    fn test_version_conflict() {
        let model = OntologyModel {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: None,
            version: "1.0".to_string(),
            domains: vec![
                DomainOntology {
                    id: "a".to_string(),
                    name: "A".to_string(),
                    description: None,
                    kind: crate::generator::ir::ontology::DomainKind::Entity,
                    parent_ids: vec![],
                    equivalent_ids: vec![],
                    disjoint_ids: vec![],
                    properties: vec![],
                    prefab_contract: Some(crate::generator::ir::ontology::PrefabContract {
                        prefab_type: crate::generator::ir::ontology::PrefabType::Module,
                        prefab_id: "shared".to_string(),
                        interface_version: "1.0".to_string(),
                        interfaces: vec![],
                        configuration: HashMap::new(),
                    }),
                },
                DomainOntology {
                    id: "b".to_string(),
                    name: "B".to_string(),
                    description: None,
                    kind: crate::generator::ir::ontology::DomainKind::Entity,
                    parent_ids: vec![],
                    equivalent_ids: vec![],
                    disjoint_ids: vec![],
                    properties: vec![],
                    prefab_contract: Some(crate::generator::ir::ontology::PrefabContract {
                        prefab_type: crate::generator::ir::ontology::PrefabType::Module,
                        prefab_id: "shared".to_string(),
                        interface_version: "2.0".to_string(),
                        interfaces: vec![],
                        configuration: HashMap::new(),
                    }),
                },
            ],
            transaction_lifecycle: None,
            relations: vec![],
            constraints: vec![],
            computations: vec![],
            namespaces: HashMap::new(),
            metadata: crate::generator::ir::ontology::OntologyMetadata::default(),
        };

        let report = ModuleConflictDetector::analyze(&model);
        assert!(
            report
                .version_conflicts
                .iter()
                .any(|c| { c.conflict_type == ModuleConflictType::VersionMismatch }),
            "Should detect version mismatch"
        );
    }
}
