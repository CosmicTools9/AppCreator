//! Rules Module - SWRL-style Rules and Constraints
//!
//! This module provides:
//! - SWRL-style rule definitions (IF-THEN)
//! - Constraint validation (@constraint annotation)
//! - Conflict detection between rules
//!
//! ## Example DSL
//!
//! ```dsl
//! // SWRL-style rules
//! rule "adultDefinition"
//! IF Person(age >= 18)
//! THEN Adult(Person)
//!
//! rule "managerAccess"
//! IF Employee(role = "manager") AND Department(budget > 100000)
//! THEN HasApprovalAuthority(Employee, Department)
//!
//! // Constraints
//! @constraint("price > 0 AND price < 1000000")
//! field price: Decimal
//!
//! @constraint("startDate < endDate")
//! entity Project {
//!     startDate: DateTime
//!     endDate: DateTime
//! }
//! ```

pub mod conflict;
pub mod constraint;
pub mod engine;

// SWRL rule types from runtime-contract
pub use runtime_contract::swrl::{
    ComparisonOp, LiteralValue, RuleAtom, RuleParseError, SwrlRule, SwrlRuleSet, Term,
};
// Re-export RuleEvaluation and RuleTrigger from behavior module
pub use runtime_contract::behavior::{RuleEvaluation, RuleEvaluationSummary, RuleTrigger};

// Re-export constraint types
pub use constraint::{
    extract_field_references, parse_constraint_expression, BinaryOp, Constraint, ConstraintExpr,
    ConstraintLevel, ConstraintLiteral, ConstraintParseError, ConstraintViolation, Constraints,
    UnaryOp,
};

// Re-export conflict detection types
pub use conflict::{
    CircularDependency, ConflictDetector, ConflictReport, ConflictSeverity, ConflictType,
    ConflictWarning, Inconsistency, InconsistencyReport, InconsistencyType, RuleConflict,
    RuleOverlap, RuleRedundancy, WarningType,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete rule and constraint definition for an entity
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntityRules {
    /// SWRL-style inference rules
    #[serde(default)]
    pub swrl_rules: Vec<SwrlRule>,
    /// Field and entity constraints
    #[serde(default)]
    pub constraints: Constraints,
    /// Rule set for efficient lookup
    #[serde(skip)]
    pub rule_set: Option<SwrlRuleSet>,
}

impl EntityRules {
    /// Create a new empty entity rules container
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a SWRL rule
    pub fn add_swrl_rule(&mut self, rule: SwrlRule) {
        self.swrl_rules.push(rule);
        self.rule_set = None; // Invalidate cache
    }

    /// Add a constraint
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.add(constraint);
    }

    /// Get or build the rule set
    pub fn rule_set(&mut self) -> &SwrlRuleSet {
        if self.rule_set.is_none() {
            let mut set = SwrlRuleSet::new();
            for rule in &self.swrl_rules {
                set.add(rule.clone());
            }
            self.rule_set = Some(set);
        }
        self.rule_set.as_ref().unwrap()
    }

    /// Check if this entity has any rules or constraints
    pub fn has_rules(&self) -> bool {
        !self.swrl_rules.is_empty() || !self.constraints.constraints.is_empty()
    }

    /// Validate rules for conflicts
    pub fn validate(&self) -> ConflictReport {
        let mut set = SwrlRuleSet::new();
        for rule in &self.swrl_rules {
            set.add(rule.clone());
        }
        ConflictDetector::analyze(&set)
    }

    /// Get all active constraints for a field
    pub fn field_constraints(&self, field_name: &str) -> Vec<&Constraint> {
        self.constraints.for_field(field_name)
    }

    /// Get all entity-level constraints
    pub fn entity_constraints(&self) -> Vec<&Constraint> {
        self.constraints.entity_constraints()
    }
}

/// Rule and constraint metadata for code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesMetadata {
    /// Number of SWRL rules
    pub swrl_rule_count: usize,
    /// Number of constraints
    pub constraint_count: usize,
    /// Number of field constraints
    pub field_constraint_count: usize,
    /// Number of entity constraints
    pub entity_constraint_count: usize,
    /// Has conflict warnings
    pub has_conflicts: bool,
}

impl From<&EntityRules> for RulesMetadata {
    fn from(rules: &EntityRules) -> Self {
        let field_constraints = rules
            .constraints
            .constraints
            .iter()
            .filter(|c| c.level == ConstraintLevel::Field)
            .count();
        let entity_constraints = rules
            .constraints
            .constraints
            .iter()
            .filter(|c| c.level == ConstraintLevel::Entity)
            .count();

        Self {
            swrl_rule_count: rules.swrl_rules.len(),
            constraint_count: rules.constraints.constraints.len(),
            field_constraint_count: field_constraints,
            entity_constraint_count: entity_constraints,
            has_conflicts: false, // Would need to run validation
        }
    }
}

/// Rule execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleContext {
    /// Entity being processed
    pub entity_name: String,
    /// Current field values
    pub field_values: HashMap<String, serde_json::Value>,
    /// Current state (if state machine)
    #[serde(default)]
    pub current_state: Option<String>,
    /// Operation being performed
    pub operation: RuleOperation,
    /// User context
    #[serde(default)]
    pub fk_user: Option<String>,
    /// Timestamp
    pub timestamp: String,
}

/// Type of operation triggering rule evaluation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RuleOperation {
    Create,
    Update,
    Delete,
    Transition,
    Query,
}

impl std::fmt::Display for RuleOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleOperation::Create => write!(f, "create"),
            RuleOperation::Update => write!(f, "update"),
            RuleOperation::Delete => write!(f, "delete"),
            RuleOperation::Transition => write!(f, "transition"),
            RuleOperation::Query => write!(f, "query"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_rules_new() {
        let rules = EntityRules::new();
        assert!(!rules.has_rules());
        assert!(rules.swrl_rules.is_empty());
        assert!(rules.constraints.constraints.is_empty());
    }

    #[test]
    fn test_entity_rules_add() {
        let mut rules = EntityRules::new();

        let rule = SwrlRule::new("testRule");
        rules.add_swrl_rule(rule);

        let constraint = Constraint::field("price > 0");
        rules.add_constraint(constraint);

        assert!(rules.has_rules());
        assert_eq!(rules.swrl_rules.len(), 1);
        assert_eq!(rules.constraints.constraints.len(), 1);
    }

    #[test]
    fn test_rule_context() {
        let mut field_values = HashMap::new();
        field_values.insert("price".to_string(), serde_json::json!(100));

        let context = RuleContext {
            entity_name: "Product".to_string(),
            field_values,
            current_state: None,
            operation: RuleOperation::Create,
            fk_user: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        assert_eq!(context.entity_name, "Product");
        assert_eq!(context.operation, RuleOperation::Create);
    }

    #[test]
    fn test_rules_metadata() {
        let mut rules = EntityRules::new();
        rules.add_swrl_rule(SwrlRule::new("rule1"));
        rules.add_constraint(Constraint::field("price > 0"));
        rules.add_constraint(Constraint::entity("a < b"));

        let meta = RulesMetadata::from(&rules);
        assert_eq!(meta.swrl_rule_count, 1);
        assert_eq!(meta.constraint_count, 2);
        assert_eq!(meta.field_constraint_count, 1);
        assert_eq!(meta.entity_constraint_count, 1);
    }
}
