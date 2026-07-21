//! OWL 本体约束验证器 (IR-2)
//!
//! Validates OWL class and property constraints in GeneratorModel.

use super::ir2::{GeneratorEntity, GeneratorModel};
use std::collections::{HashMap, HashSet};

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub code: ErrorCode,
    pub message: String,
    pub entity: Option<String>,
    pub field: Option<String>,
}

/// Error code for validation errors
#[derive(Debug, Clone)]
pub enum ErrorCode {
    UnknownParentClass,
    CircularInheritance,
    DisjointViolation,
    ConflictingCardinality,
    InvalidMinCardinality,
    InvalidMaxCardinality,
    EquivalentClassMismatch,
    // Phase 23: Rule and Constraint Errors
    InvalidConstraintExpression,
    InvalidRuleSyntax,
    ConstraintViolation,
    RuleConflict,
}

/// OWL Ontology Validator
pub struct OntologyValidator;

impl OntologyValidator {
    /// Validate the entire model
    pub fn validate(model: &GeneratorModel) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Build entity name index
        let entity_names: HashSet<_> = model.entities.iter().map(|e| e.name.raw.clone()).collect();

        for entity in &model.entities {
            // Validate parent classes exist
            Self::validate_parent_classes(entity, &entity_names, &mut errors);

            // Validate no circular inheritance
            Self::validate_no_circular_inheritance(entity, model, &mut errors);

            // Validate disjoint classes exist
            Self::validate_disjoint_classes(entity, &entity_names, &mut errors);

            // Validate cardinality constraints
            Self::validate_cardinality_constraints(entity, &mut errors);

            // Phase 23: Validate constraints
            Self::validate_constraints(entity, &mut errors);

            // Phase 23: Validate SWRL rules
            Self::validate_swrl_rules(entity, &mut errors);
        }

        // Validate equivalent classes
        Self::validate_equivalent_classes(model, &mut errors);

        // Phase 23: Validate rule conflicts across all entities
        Self::validate_rule_conflicts(model, &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate parent classes exist
    fn validate_parent_classes(
        entity: &GeneratorEntity,
        entity_names: &HashSet<String>,
        errors: &mut Vec<ValidationError>,
    ) {
        for parent in &entity.parent_classes {
            if !entity_names.contains(&parent.raw) {
                errors.push(ValidationError {
                    code: ErrorCode::UnknownParentClass,
                    message: format!("Parent class '{}' not found", parent.raw),
                    entity: Some(entity.name.raw.clone()),
                    field: None,
                });
            }
        }
    }

    /// Validate no circular inheritance using DFS
    fn validate_no_circular_inheritance(
        entity: &GeneratorEntity,
        model: &GeneratorModel,
        errors: &mut Vec<ValidationError>,
    ) {
        let mut visited = HashSet::new();
        let mut stack = vec![entity.name.raw.clone()];
        let mut path = vec![entity.name.raw.clone()];

        while let Some(current) = stack.pop() {
            if visited.contains(&current) {
                // Check if this creates a cycle back to the original entity
                if current == entity.name.raw {
                    errors.push(ValidationError {
                        code: ErrorCode::CircularInheritance,
                        message: format!("Circular inheritance detected: {}", path.join(" -> ")),
                        entity: Some(entity.name.raw.clone()),
                        field: None,
                    });
                }
                continue;
            }

            visited.insert(current.clone());

            if let Some(e) = model.entities.iter().find(|e| e.name.raw == current) {
                for parent in &e.parent_classes {
                    stack.push(parent.raw.clone());
                    path.push(parent.raw.clone());
                }
            }
        }
    }

    /// Validate disjoint classes exist
    fn validate_disjoint_classes(
        entity: &GeneratorEntity,
        entity_names: &HashSet<String>,
        errors: &mut Vec<ValidationError>,
    ) {
        for disjoint in &entity.disjoint_classes {
            if !entity_names.contains(disjoint) {
                errors.push(ValidationError {
                    code: ErrorCode::DisjointViolation,
                    message: format!("Disjoint class '{}' not found", disjoint),
                    entity: Some(entity.name.raw.clone()),
                    field: None,
                });
            }
        }
    }

    /// Validate cardinality constraints
    pub fn validate_cardinality_constraints(
        entity: &GeneratorEntity,
        errors: &mut Vec<ValidationError>,
    ) {
        for field in &entity.fields {
            // Validate minCardinality <= maxCardinality
            if let (Some(min), Some(max)) = (field.min_cardinality, field.max_cardinality) {
                if min > max {
                    errors.push(ValidationError {
                        code: ErrorCode::ConflictingCardinality,
                        message: format!("minCardinality({}) > maxCardinality({})", min, max),
                        entity: Some(entity.name.raw.clone()),
                        field: Some(field.name.raw.clone()),
                    });
                }
            }

            // Validate minCardinality with nullable
            if let Some(min) = field.min_cardinality {
                if field.nullable && min > 0 {
                    errors.push(ValidationError {
                        code: ErrorCode::InvalidMinCardinality,
                        message: String::from("Nullable field cannot have minCardinality > 0"),
                        entity: Some(entity.name.raw.clone()),
                        field: Some(field.name.raw.clone()),
                    });
                }
            }
        }
    }

    /// Validate equivalent classes consistency
    fn validate_equivalent_classes(model: &GeneratorModel, errors: &mut Vec<ValidationError>) {
        // Build entity map
        let entity_map: HashMap<_, _> = model
            .entities
            .iter()
            .map(|e| (e.name.raw.clone(), e))
            .collect();

        for entity in &model.entities {
            for equiv_name in &entity.equivalent_classes {
                if let Some(equiv_entity) = entity_map.get(equiv_name) {
                    // Check field count consistency (basic check)
                    if entity.fields.len() != equiv_entity.fields.len() {
                        errors.push(ValidationError {
                            code: ErrorCode::EquivalentClassMismatch,
                            message: format!(
                                "Equivalent class '{}' has different field count ({} vs {})",
                                equiv_name,
                                entity.fields.len(),
                                equiv_entity.fields.len()
                            ),
                            entity: Some(entity.name.raw.clone()),
                            field: None,
                        });
                    }
                }
            }
        }
    }

    // Phase 23: Constraint and Rule Validation

    /// Validate constraints for an entity
    fn validate_constraints(entity: &GeneratorEntity, errors: &mut Vec<ValidationError>) {
        use runtime_contract::expression::parse_constraint_expression;

        // Validate entity-level constraints
        for constraint in &entity.constraints {
            if let Err(e) = parse_constraint_expression(&constraint.expression) {
                errors.push(ValidationError {
                    code: ErrorCode::InvalidConstraintExpression,
                    message: format!(
                        "Invalid constraint expression '{}': {}",
                        constraint.expression, e
                    ),
                    entity: Some(entity.name.raw.clone()),
                    field: constraint.field_name.clone(),
                });
            }
        }

        // Validate field-level constraints
        for field in &entity.fields {
            for constraint in &field.constraints {
                if let Err(e) = parse_constraint_expression(&constraint.expression) {
                    errors.push(ValidationError {
                        code: ErrorCode::InvalidConstraintExpression,
                        message: format!(
                            "Invalid constraint expression '{}': {}",
                            constraint.expression, e
                        ),
                        entity: Some(entity.name.raw.clone()),
                        field: Some(field.name.raw.clone()),
                    });
                }
            }
        }
    }

    /// Validate SWRL rules for an entity
    fn validate_swrl_rules(entity: &GeneratorEntity, errors: &mut Vec<ValidationError>) {
        for rule in &entity.swrl_rules {
            // Basic validation: rule must have body and head
            if rule.body.trim().is_empty() {
                errors.push(ValidationError {
                    code: ErrorCode::InvalidRuleSyntax,
                    message: format!("SWRL rule '{}' has empty body (IF)", rule.name),
                    entity: Some(entity.name.raw.clone()),
                    field: None,
                });
            }

            if rule.head.trim().is_empty() {
                errors.push(ValidationError {
                    code: ErrorCode::InvalidRuleSyntax,
                    message: format!("SWRL rule '{}' has empty head (THEN)", rule.name),
                    entity: Some(entity.name.raw.clone()),
                    field: None,
                });
            }

            // Note: Additional syntax validation could be done here
            // using parse_swrl_rule from the rules module
        }
    }

    /// Validate rule conflicts across all entities
    fn validate_rule_conflicts(model: &GeneratorModel, errors: &mut Vec<ValidationError>) {
        use std::collections::{HashMap, HashSet};

        // Collect all business rules with their entity context
        #[derive(Debug, Clone)]
        struct RuleRef {
            name: String,
            condition: String,
            #[allow(dead_code)]
            action: Option<String>,
            trigger: String,
            entity: String,
        }

        let mut all_rules: Vec<RuleRef> = Vec::new();
        let mut entity_fields: HashMap<String, HashSet<String>> = HashMap::new();

        for entity in &model.entities {
            let mut fields = HashSet::new();
            for field in &entity.fields {
                fields.insert(field.name.raw.clone());
                fields.insert(field.name.snake.clone());
            }
            entity_fields.insert(entity.name.raw.clone(), fields);

            for rule in &entity.business_rules {
                all_rules.push(RuleRef {
                    name: rule.name.clone(),
                    condition: rule.condition.clone(),
                    action: rule.action.clone(),
                    trigger: rule.trigger.clone(),
                    entity: entity.name.raw.clone(),
                });
            }
        }

        // 1. Detect circular dependencies in rule references
        let _rule_names: HashSet<String> = all_rules.iter().map(|r| r.name.clone()).collect();
        let mut dependency_graph: HashMap<String, Vec<String>> = HashMap::new();

        for rule in &all_rules {
            let mut deps = Vec::new();
            for other in &all_rules {
                if other.name != rule.name && rule.condition.contains(&other.name) {
                    deps.push(other.name.clone());
                }
            }
            dependency_graph.insert(rule.name.clone(), deps);
        }

        // DFS cycle detection
        fn has_cycle(
            graph: &HashMap<String, Vec<String>>,
            node: &str,
            visited: &mut HashSet<String>,
            stack: &mut HashSet<String>,
            path: &mut Vec<String>,
        ) -> Option<Vec<String>> {
            visited.insert(node.to_string());
            stack.insert(node.to_string());
            path.push(node.to_string());

            if let Some(neighbors) = graph.get(node) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        if let Some(cycle) = has_cycle(graph, neighbor, visited, stack, path) {
                            return Some(cycle);
                        }
                    } else if stack.contains(neighbor) {
                        // Found cycle - extract cycle from path
                        let idx = path.iter().position(|p| p == neighbor).unwrap_or(0);
                        let cycle = path[idx..].to_vec();
                        return Some(cycle);
                    }
                }
            }

            path.pop();
            stack.remove(node);
            None
        }

        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        for rule in &all_rules {
            if !visited.contains(&rule.name) {
                let mut path = Vec::new();
                if let Some(cycle) = has_cycle(
                    &dependency_graph,
                    &rule.name,
                    &mut visited,
                    &mut stack,
                    &mut path,
                ) {
                    errors.push(ValidationError {
                        code: ErrorCode::RuleConflict,
                        message: format!(
                            "Circular rule dependency detected: {}",
                            cycle.join(" -> ")
                        ),
                        entity: Some(rule.entity.clone()),
                        field: None,
                    });
                }
            }
        }

        // 2. Detect contradictory rules (same entity, same field, opposite conditions)
        // Extract field references from conditions and look for opposing operators
        let opposing_pairs: [(Vec<&str>, Vec<&str>); 3] = [
            (vec![">=", ">"], vec!["<=", "<"]),
            (vec!["==", "="], vec!["!=", "<>"]),
            (vec!["AND"], vec!["OR"]),
        ];

        for i in 0..all_rules.len() {
            for j in (i + 1)..all_rules.len() {
                let a = &all_rules[i];
                let b = &all_rules[j];

                // Only compare rules on the same entity with the same trigger
                if a.entity != b.entity || a.trigger != b.trigger {
                    continue;
                }

                let a_fields = entity_fields.get(&a.entity).cloned().unwrap_or_default();
                let mut common_fields = Vec::new();
                for field in &a_fields {
                    if a.condition.contains(field) && b.condition.contains(field) {
                        common_fields.push(field.clone());
                    }
                }

                if common_fields.is_empty() {
                    continue;
                }

                for (ops_a, ops_b) in &opposing_pairs {
                    let a_has_op = ops_a.iter().any(|op| a.condition.contains(op));
                    let b_has_op = ops_b.iter().any(|op| b.condition.contains(op));
                    if a_has_op && b_has_op {
                        errors.push(ValidationError {
                            code: ErrorCode::RuleConflict,
                            message: format!(
                                "Potentially contradictory rules '{}' and '{}' on fields {:?}",
                                a.name, b.name, common_fields
                            ),
                            entity: Some(a.entity.clone()),
                            field: Some(common_fields.join(", ")),
                        });
                        break;
                    }
                }
            }
        }

        // 3. Detect redundant rules (same condition + trigger)
        for i in 0..all_rules.len() {
            for j in (i + 1)..all_rules.len() {
                let a = &all_rules[i];
                let b = &all_rules[j];

                if a.condition.trim() == b.condition.trim()
                    && a.trigger == b.trigger
                    && a.entity == b.entity
                {
                    errors.push(ValidationError {
                        code: ErrorCode::RuleConflict,
                        message: format!(
                            "Redundant rules '{}' and '{}' have identical condition and trigger",
                            a.name, b.name
                        ),
                        entity: Some(a.entity.clone()),
                        field: None,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::ir2::{
        EntityName, FieldName, GeneratorField, GeneratorFieldType, GeneratorModel,
    };
    use super::*;
    use crate::test_utils::{create_test_entity, create_test_field};

    #[test]
    fn test_valid_inheritance() {
        let model = GeneratorModel {
            i18n_config: None,
            entities: vec![
                create_test_entity("Animal", vec![], vec![create_test_field("name", false)]),
                create_test_entity(
                    "Dog",
                    vec!["Animal"],
                    vec![create_test_field("breed", false)],
                ),
            ],
            enums: vec![],
            metadata: Default::default(),
            exceptions: vec![],
            exception_handlers: vec![],
            external_dependencies: vec![],
        };

        assert!(OntologyValidator::validate(&model).is_ok());
    }

    #[test]
    fn test_unknown_parent() {
        let model = GeneratorModel {
            i18n_config: None,
            entities: vec![create_test_entity("Dog", vec!["UnknownParent"], vec![])],
            enums: vec![],
            metadata: Default::default(),
            exceptions: vec![],
            external_dependencies: vec![],
            exception_handlers: vec![],
        };

        let result = OntologyValidator::validate(&model);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(matches!(errors[0].code, ErrorCode::UnknownParentClass));
    }

    #[test]
    fn test_circular_inheritance_detection() {
        let model = GeneratorModel {
            i18n_config: None,
            entities: vec![
                GeneratorEntity {
                    name: EntityName {
                        raw: "A".to_string(),
                        snake: "a".to_string(),
                        camel: "a".to_string(),
                        pascal: "A".to_string(),
                        kebab: "a".to_string(),
                        screaming_snake: "A".to_string(),
                        plural_snake: "as".to_string(),
                        plural_pascal: "As".to_string(),
                        plural_kebab: "as".to_string(),
                    },
                    description: None,
                    fields: vec![],
                    relations: vec![],
                    annotations: vec![],
                    primary_key_type: Default::default(),
                    parent_classes: vec![EntityName {
                        raw: "B".to_string(),
                        snake: "b".to_string(),
                        camel: "b".to_string(),
                        pascal: "B".to_string(),
                        kebab: "b".to_string(),
                        screaming_snake: "B".to_string(),
                        plural_snake: "bs".to_string(),
                        plural_pascal: "Bs".to_string(),
                        plural_kebab: "bs".to_string(),
                    }],
                    equivalent_classes: vec![],
                    disjoint_classes: vec![],
                    is_abstract: false,
                    inheritance_depth: 0,
                    state_machine: Default::default(),
                    transitions: vec![],
                    lifecycle_hooks: vec![],
                    business_rules: vec![],
                    swrl_rules: vec![],
                    constraints: vec![],
                    quality_rules: vec![],
                    quality_config: Default::default(),
                    parent_tables: vec![],
                },
                GeneratorEntity {
                    name: EntityName {
                        raw: "B".to_string(),
                        snake: "b".to_string(),
                        camel: "b".to_string(),
                        pascal: "B".to_string(),
                        kebab: "b".to_string(),
                        screaming_snake: "B".to_string(),
                        plural_snake: "bs".to_string(),
                        plural_pascal: "Bs".to_string(),
                        plural_kebab: "bs".to_string(),
                    },
                    description: None,
                    fields: vec![],
                    relations: vec![],
                    annotations: vec![],
                    primary_key_type: Default::default(),
                    parent_classes: vec![EntityName {
                        raw: "C".to_string(),
                        snake: "c".to_string(),
                        camel: "c".to_string(),
                        pascal: "C".to_string(),
                        kebab: "c".to_string(),
                        screaming_snake: "C".to_string(),
                        plural_snake: "cs".to_string(),
                        plural_pascal: "Cs".to_string(),
                        plural_kebab: "cs".to_string(),
                    }],
                    equivalent_classes: vec![],
                    disjoint_classes: vec![],
                    is_abstract: false,
                    inheritance_depth: 0,
                    state_machine: Default::default(),
                    transitions: vec![],
                    lifecycle_hooks: vec![],
                    business_rules: vec![],
                    swrl_rules: vec![],
                    constraints: vec![],
                    quality_rules: vec![],
                    quality_config: Default::default(),
                    parent_tables: vec![],
                },
                GeneratorEntity {
                    name: EntityName {
                        raw: "C".to_string(),
                        snake: "c".to_string(),
                        camel: "c".to_string(),
                        pascal: "C".to_string(),
                        kebab: "c".to_string(),
                        screaming_snake: "C".to_string(),
                        plural_snake: "cs".to_string(),
                        plural_pascal: "Cs".to_string(),
                        plural_kebab: "cs".to_string(),
                    },
                    description: None,
                    fields: vec![],
                    relations: vec![],
                    annotations: vec![],
                    primary_key_type: Default::default(),
                    parent_classes: vec![EntityName {
                        raw: "A".to_string(),
                        snake: "a".to_string(),
                        camel: "a".to_string(),
                        pascal: "A".to_string(),
                        kebab: "a".to_string(),
                        screaming_snake: "A".to_string(),
                        plural_snake: "as".to_string(),
                        plural_pascal: "As".to_string(),
                        plural_kebab: "as".to_string(),
                    }],
                    equivalent_classes: vec![],
                    disjoint_classes: vec![],
                    is_abstract: false,
                    inheritance_depth: 0,
                    state_machine: Default::default(),
                    transitions: vec![],
                    lifecycle_hooks: vec![],
                    business_rules: vec![],
                    swrl_rules: vec![],
                    constraints: vec![],
                    quality_rules: vec![],
                    quality_config: Default::default(),
                    parent_tables: vec![],
                },
            ],
            enums: vec![],
            metadata: Default::default(),
            exceptions: vec![],
            external_dependencies: vec![],
            exception_handlers: vec![],
        };

        let result = OntologyValidator::validate(&model);
        assert!(result.is_err());
    }

    #[test]
    fn test_conflicting_cardinality() {
        let mut entity = create_test_entity("Test", vec![], vec![]);
        entity.fields = vec![GeneratorField {
            name: FieldName {
                raw: "items".to_string(),
                snake: "items".to_string(),
                camel: "items".to_string(),
                pascal: "Items".to_string(),
            },
            field_type: GeneratorFieldType::Text,
            description: None,
            nullable: false,
            unique: false,
            indexed: false,
            default_value: None,
            validations: vec![],
            annotations: vec![],
            domain: None,
            range: None,
            min_cardinality: Some(5),
            max_cardinality: Some(2),
            is_functional: false,
            constraints: vec![],
            throws_clauses: vec![],
            quality_rules: vec![],
        }];

        let mut errors = vec![];
        OntologyValidator::validate_cardinality_constraints(&entity, &mut errors);

        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0].code, ErrorCode::ConflictingCardinality));
    }

    #[test]
    fn test_nullable_with_min_cardinality() {
        let mut entity = create_test_entity("Test", vec![], vec![]);
        entity.fields = vec![GeneratorField {
            name: FieldName {
                raw: "items".to_string(),
                snake: "items".to_string(),
                camel: "items".to_string(),
                pascal: "Items".to_string(),
            },
            field_type: GeneratorFieldType::Text,
            description: None,
            nullable: true,
            unique: false,
            indexed: false,
            default_value: None,
            validations: vec![],
            annotations: vec![],
            domain: None,
            range: None,
            min_cardinality: Some(1),
            max_cardinality: None,
            is_functional: false,
            constraints: vec![],
            throws_clauses: vec![],
            quality_rules: vec![],
        }];

        let mut errors = vec![];
        OntologyValidator::validate_cardinality_constraints(&entity, &mut errors);

        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0].code, ErrorCode::InvalidMinCardinality));
    }
}
