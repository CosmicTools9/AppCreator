//! OntologyModel → GeneratorModel Transformer
//!
//! Converts the semantic Ontology IR into the generation-oriented GeneratorModel (IR-2).
//! This is the primary bridge between ontology-driven design and code generation.

use super::ontology::{
    Cardinality, ComputationOntology, ConstraintOntology, ConstraintSeverity, DomainKind,
    DomainOntology, OntologyModel, OntologyProperty, PropertyType, RelationOntology, RelationType,
    TransactionLifecycle,
};
use super::{
    naming, GeneratorBusinessRule, GeneratorConstraint, GeneratorConstraintLevel, GeneratorEntity,
    GeneratorEnum, GeneratorField, GeneratorFieldType, GeneratorLifecycleHook, GeneratorModel,
    GeneratorRelation, GeneratorRelationType, GeneratorState, GeneratorStateMachine,
    GeneratorTransition, ModelMetadata, ModuleDependency, PrimaryKeyType,
};
use serde::Deserialize;

// ── module.json parsing helpers ──────────────────────────────────────────
#[derive(Deserialize)]
struct ModuleJson {
    #[serde(default)]
    backend_crate: Option<String>,
    #[serde(default)]
    crate_name: Option<String>,
    #[serde(default)]
    extension_points: Option<ExtensionPoints>,
    #[serde(default)]
    dependencies: Option<Dependencies>,
}
#[derive(Deserialize)]
struct ExtensionPoints {
    #[serde(default)]
    entities: Vec<ExtensionEntity>,
}
#[derive(Deserialize)]
struct ExtensionEntity {
    #[serde(default)]
    entity_name: String,
    #[serde(default)]
    table_name: String,
}
#[derive(Deserialize)]
struct Dependencies {
    #[serde(default)]
    events: Option<Events>,
}
#[derive(Deserialize)]
struct Events {
    #[serde(default)]
    publishes: Vec<String>,
}
/// Transforms an `OntologyModel` into a `GeneratorModel`.
pub struct OntologyTransformer;

impl OntologyTransformer {
    /// Transform ontology into generator model.
    pub fn transform(model: &OntologyModel, used_modules: Vec<String>) -> GeneratorModel {
        let mut entities = Vec::new();
        let mut enums = Vec::new();

        // Build entity index for stable cross-reference lookup
        let mut entity_index: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for (idx, domain) in model.domains.iter().enumerate() {
            match domain.kind {
                DomainKind::Entity | DomainKind::AggregateRoot | DomainKind::ValueObject => {
                    entity_index.insert(domain.id.clone(), idx);
                    entity_index.insert(domain.name.clone(), idx);
                    entities.push(Self::domain_to_entity(domain, model));
                }
                DomainKind::Enumeration => {
                    enums.push(Self::domain_to_enum(domain));
                }
                DomainKind::DomainService | DomainKind::DomainEvent => {
                    // Services and events are not directly generated as entities;
                    // they may be handled by specialized generators in the future.
                }
            }
        }

        // Apply lifecycle/state-machine definitions
        if let Some(lifecycle) = &model.transaction_lifecycle {
            Self::apply_lifecycle(&mut entities, &entity_index, lifecycle);
        }

        // Apply relation ontologies
        for relation in &model.relations {
            Self::apply_relation(&mut entities, &entity_index, relation);
        }

        // Apply constraints
        for constraint in &model.constraints {
            Self::apply_constraint(&mut entities, &entity_index, constraint);
        }

        // Apply computations (business rules)
        for computation in &model.computations {
            Self::apply_computation(&mut entities, &entity_index, computation);
        }

        let external_deps = Self::load_module_dependencies(&used_modules);
        GeneratorModel {
            i18n_config: None,
            entities,
            enums,
            metadata: ModelMetadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                generator_version: crate::ALIOTH_STUDIO_VERSION.clone(),
            },
            exceptions: vec![],
            exception_handlers: vec![],
            external_dependencies: external_deps,
        }
    }

    // ── Domain → Entity / Enum ───────────────────────────────────────────────

    fn domain_to_entity(domain: &DomainOntology, model: &OntologyModel) -> GeneratorEntity {
        let mut fields = Vec::new();
        let mut relations = Vec::new();
        let mut annotations = Vec::new();

        for prop in &domain.properties {
            match prop.property_type {
                PropertyType::DataProperty => {
                    fields.push(Self::property_to_field(prop));
                }
                PropertyType::ObjectProperty => {
                    relations.push(Self::property_to_relation(prop));
                }
                PropertyType::AnnotationProperty => {
                    // Map annotation properties to entity annotations
                    let mut params = std::collections::HashMap::new();
                    if let Some(ref desc) = prop.semantic_description {
                        params.insert("description".to_string(), desc.clone());
                    }
                    annotations.push(super::GeneratorAnnotation {
                        name: prop.name.clone(),
                        params,
                    });
                }
            }
        }

        // Compute inheritance depth from parent chain
        let inheritance_depth = Self::compute_inheritance_depth(domain, model);

        GeneratorEntity {
            name: naming::build_entity_name(&domain.name),
            description: domain.description.clone(),
            fields,
            relations,
            annotations,
            primary_key_type: PrimaryKeyType::BigInt,
            parent_classes: domain
                .parent_ids
                .iter()
                .map(|id| naming::build_entity_name(id))
                .collect(),
            equivalent_classes: domain.equivalent_ids.clone(),
            disjoint_classes: domain.disjoint_ids.clone(),
            is_abstract: false,
            inheritance_depth,
            state_machine: GeneratorStateMachine::default(),
            transitions: vec![],
            lifecycle_hooks: vec![],
            business_rules: vec![],
            swrl_rules: vec![],
            constraints: vec![],
            quality_rules: vec![],
            quality_config: Default::default(),
            parent_tables: vec![],
        }
    }

    fn domain_to_enum(domain: &DomainOntology) -> GeneratorEnum {
        // Enumeration values are derived from property names.
        // Each property represents one enum variant.
        let values: Vec<String> = domain.properties.iter().map(|p| p.name.clone()).collect();

        GeneratorEnum {
            name: domain.name.clone(),
            values,
        }
    }

    /// Compute inheritance depth by walking parent chains.
    fn compute_inheritance_depth(domain: &DomainOntology, model: &OntologyModel) -> u32 {
        let mut depth = 0u32;
        let mut current = domain;
        let domain_map: std::collections::HashMap<&str, &DomainOntology> =
            model.domains.iter().map(|d| (d.id.as_str(), d)).collect();
        let mut visited = std::collections::HashSet::new();
        visited.insert(domain.id.as_str());

        while let Some(parent_id) = current.parent_ids.first() {
            if !visited.insert(parent_id.as_str()) {
                break; // cycle detected
            }
            if let Some(parent) = domain_map.get(parent_id.as_str()) {
                depth += 1;
                current = parent;
            } else {
                break;
            }
        }
        depth
    }

    // ── Property → Field ─────────────────────────────────────────────────────

    fn property_to_field(prop: &OntologyProperty) -> GeneratorField {
        let field_type = Self::map_property_range(&prop.range);
        let validations = prop
            .constraints
            .iter()
            .filter_map(Self::map_property_constraint)
            .collect();

        // Handle Cardinality.exact: sync to min/max if present
        let (min_cardinality, max_cardinality) = if let Some(exact) = prop.cardinality.exact {
            (Some(exact), Some(exact))
        } else {
            (prop.cardinality.min, prop.cardinality.max)
        };

        GeneratorField {
            name: naming::build_field_name(&prop.name),
            field_type,
            description: prop.semantic_description.clone(),
            nullable: !prop.required,
            unique: prop
                .constraints
                .iter()
                .any(|c| matches!(c.constraint_type, super::ontology::ConstraintType::Unique)),
            indexed: false,
            default_value: None,
            validations,
            annotations: vec![],
            domain: Some(prop.domain.clone()),
            range: Some(prop.range.clone()),
            min_cardinality,
            max_cardinality,
            is_functional: prop.is_functional,
            constraints: vec![],
            throws_clauses: vec![],
            quality_rules: vec![],
        }
    }

    fn property_to_relation(prop: &OntologyProperty) -> GeneratorRelation {
        let relation_type = Self::infer_relation_type(&prop.cardinality);
        GeneratorRelation {
            name: prop.name.clone(),
            target_entity: prop.range.clone(),
            relation_type,
            nullable: !prop.required,
        }
    }

    // ── Lifecycle / State Machine ────────────────────────────────────────────

    fn apply_lifecycle(
        entities: &mut [GeneratorEntity],
        entity_index: &std::collections::HashMap<String, usize>,
        lifecycle: &TransactionLifecycle,
    ) {
        for ref_id in lifecycle.phases.iter().flat_map(|p| &p.related_ontologies) {
            if let Some(&idx) = entity_index.get(ref_id) {
                if let Some(entity) = entities.get_mut(idx) {
                    entity.state_machine = Self::lifecycle_to_state_machine(lifecycle);
                    entity.transitions = Self::lifecycle_to_transitions(lifecycle);
                    entity.lifecycle_hooks = Self::lifecycle_to_hooks(lifecycle);
                }
            }
        }
    }

    fn lifecycle_to_state_machine(lifecycle: &TransactionLifecycle) -> GeneratorStateMachine {
        GeneratorStateMachine {
            enabled: true,
            states: lifecycle
                .phases
                .iter()
                .map(|phase| GeneratorState {
                    name: phase.name.clone(),
                    pascal_name: Self::to_pascal(&phase.name),
                    snake_name: Self::to_snake(&phase.name),
                    is_final: phase.is_terminal,
                })
                .collect(),
            initial_state: lifecycle
                .phases
                .iter()
                .min_by_key(|p| p.order)
                .map(|p| p.name.clone()),
            state_field: "status".to_string(),
            state_enum_name: format!("{}Status", Self::to_pascal(&lifecycle.name)),
        }
    }

    fn lifecycle_to_transitions(lifecycle: &TransactionLifecycle) -> Vec<GeneratorTransition> {
        lifecycle
            .transitions
            .iter()
            .map(|t| GeneratorTransition {
                event: t.trigger_event.clone(),
                event_snake: Self::to_snake(&t.trigger_event),
                from: vec![t.from_phase.clone()],
                to: t.to_phase.clone(),
                guard: t.guard_conditions.first().cloned(),
                action: t.actions.first().map(|a| a.description.clone()),
            })
            .collect()
    }

    fn lifecycle_to_hooks(lifecycle: &TransactionLifecycle) -> Vec<GeneratorLifecycleHook> {
        let mut hooks = Vec::new();
        for transition in &lifecycle.transitions {
            for action in &transition.actions {
                hooks.push(GeneratorLifecycleHook {
                    event: "onTransition".to_string(),
                    function_name: action.description.clone(),
                    function_name_snake: Self::to_snake(&action.description),
                    from_state: Some(transition.from_phase.clone()),
                    to_state: Some(transition.to_phase.clone()),
                    order: 0,
                    is_async: false,
                });
            }
        }
        hooks
    }

    // ── Relations ────────────────────────────────────────────────────────────

    fn apply_relation(
        entities: &mut [GeneratorEntity],
        entity_index: &std::collections::HashMap<String, usize>,
        relation: &RelationOntology,
    ) {
        // Add relation to source entity
        if let Some(&idx) = entity_index.get(&relation.source_ontology) {
            if let Some(entity) = entities.get_mut(idx) {
                entity.relations.push(GeneratorRelation {
                    name: relation.name.clone(),
                    target_entity: relation.target_ontology.clone(),
                    relation_type: Self::map_relation_type(&relation.relation_type),
                    nullable: true,
                });
            }
        }

        // If bidirectional, add reverse relation to target entity
        if relation.is_bidirectional {
            if let Some(&idx) = entity_index.get(&relation.target_ontology) {
                if let Some(entity) = entities.get_mut(idx) {
                    entity.relations.push(GeneratorRelation {
                        name: relation.name.clone(),
                        target_entity: relation.source_ontology.clone(),
                        relation_type: Self::reverse_relation_type(&relation.relation_type),
                        nullable: true,
                    });
                }
            }
        }
    }

    // ── Constraints ──────────────────────────────────────────────────────────

    fn apply_constraint(
        entities: &mut [GeneratorEntity],
        entity_index: &std::collections::HashMap<String, usize>,
        constraint: &ConstraintOntology,
    ) {
        // Level is determined by scope: field-level if target_property exists, else entity-level
        let level = if constraint.scope.target_property.is_some() {
            GeneratorConstraintLevel::Field
        } else {
            GeneratorConstraintLevel::Entity
        };

        let gen_constraint = GeneratorConstraint {
            name: Some(constraint.name.clone()),
            expression: constraint.expression.clone(),
            level,
            error_message: constraint.error_message_template.clone(),
            error_code: None,
            active: true,
            blocking: matches!(constraint.severity, ConstraintSeverity::Error),
            field_name: constraint.scope.target_property.clone(),
        };

        if let Some(&idx) = entity_index.get(&constraint.scope.target_ontology) {
            if let Some(entity) = entities.get_mut(idx) {
                if let Some(ref field_name) = constraint.scope.target_property {
                    // Attach to field if specified
                    if let Some(field) = entity
                        .fields
                        .iter_mut()
                        .find(|f| f.name.raw == *field_name || f.name.snake == *field_name)
                    {
                        field.constraints.push(gen_constraint);
                    } else {
                        entity.constraints.push(gen_constraint);
                    }
                } else {
                    entity.constraints.push(gen_constraint);
                }
            }
        }
    }

    // ── Computations (Business Rules) ────────────────────────────────────────

    fn apply_computation(
        entities: &mut [GeneratorEntity],
        entity_index: &std::collections::HashMap<String, usize>,
        computation: &ComputationOntology,
    ) {
        // Determine trigger from computation type and trigger conditions
        let trigger = if computation.trigger_conditions.is_empty() {
            "always".to_string()
        } else {
            computation.trigger_conditions.join(", ")
        };

        let rule = GeneratorBusinessRule {
            name: computation.name.clone(),
            name_snake: Self::to_snake(&computation.name),
            condition: computation.formula.clone(),
            action: computation.outputs.first().map(|o| o.name.clone()),
            error_message: computation.description.clone(),
            error_code: None,
            priority: 0,
            trigger,
        };

        // Attach to all input entities
        for input in &computation.inputs {
            if let Some(&idx) = entity_index.get(&input.source_ontology) {
                if let Some(entity) = entities.get_mut(idx) {
                    entity.business_rules.push(rule.clone());
                }
            }
        }
    }

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn map_property_range(range: &str) -> GeneratorFieldType {
        naming::map_field_type(range)
    }

    #[allow(dead_code)]
    fn _legacy_map_property_range(range: &str) -> GeneratorFieldType {
        match range.to_lowercase().as_str() {
            "string" | "text" | "varchar" | "char" => GeneratorFieldType::Text,
            "integer" | "int" | "i32" | "int4" => GeneratorFieldType::Integer,
            "bigint" | "long" | "i64" | "int8" => GeneratorFieldType::BigInt,
            "decimal" | "numeric" | "float" | "number" => GeneratorFieldType::Decimal,
            "double" | "real" => GeneratorFieldType::Decimal,
            "boolean" | "bool" => GeneratorFieldType::Boolean,
            "datetime" | "date" | "timestamp" | "time" => GeneratorFieldType::DateTime,
            "uuid" => GeneratorFieldType::Uuid,
            "json" | "jsonb" | "object" | "array" => GeneratorFieldType::Json,
            r => {
                if let Some(stripped) = r.strip_prefix("enum:") {
                    GeneratorFieldType::Enum(stripped.to_string())
                } else if let Some(stripped) = r.strip_prefix("ref:") {
                    GeneratorFieldType::Reference(stripped.to_string())
                } else if let Some(stripped) = r.strip_prefix("reference:") {
                    GeneratorFieldType::Reference(stripped.to_string())
                } else if r.ends_with("enum") {
                    GeneratorFieldType::Enum(r.to_string())
                } else {
                    common::telemetry::warn!(
                        "Unknown ontology property range '{}', defaulting to Reference",
                        r
                    );
                    GeneratorFieldType::Reference(r.to_string())
                }
            }
        }
    }

    fn map_property_constraint(
        constraint: &super::ontology::PropertyConstraint,
    ) -> Option<super::GeneratorValidation> {
        use super::GeneratorValidationType;
        let (validation_type, params) = match constraint.constraint_type {
            super::ontology::ConstraintType::Range => {
                let parts: Vec<&str> = constraint.value.split(',').collect();
                let mut p = std::collections::HashMap::new();
                if let Some(min) = parts.first() {
                    p.insert("min".to_string(), min.trim().to_string());
                }
                if let Some(max) = parts.get(1) {
                    p.insert("max".to_string(), max.trim().to_string());
                }
                (GeneratorValidationType::Min, p)
            }
            super::ontology::ConstraintType::Pattern => {
                let mut p = std::collections::HashMap::new();
                p.insert("pattern".to_string(), constraint.value.clone());
                (GeneratorValidationType::Pattern, p)
            }
            super::ontology::ConstraintType::Enum => {
                let mut p = std::collections::HashMap::new();
                p.insert("values".to_string(), constraint.value.clone());
                (GeneratorValidationType::Custom("enum".to_string()), p)
            }
            super::ontology::ConstraintType::Unique => (
                GeneratorValidationType::Custom("unique".to_string()),
                std::collections::HashMap::new(),
            ),
            super::ontology::ConstraintType::Custom => {
                let mut p = std::collections::HashMap::new();
                p.insert("expression".to_string(), constraint.value.clone());
                (GeneratorValidationType::Custom("custom".to_string()), p)
            }
        };
        Some(super::GeneratorValidation {
            validation_type,
            params,
        })
    }

    fn infer_relation_type(cardinality: &Cardinality) -> GeneratorRelationType {
        match (cardinality.min, cardinality.max) {
            (Some(1), Some(1)) => GeneratorRelationType::OneToOne,
            (Some(0) | None, Some(1)) => GeneratorRelationType::ManyToOne,
            (Some(1), None) => GeneratorRelationType::OneToMany,
            (Some(0) | None, None) => GeneratorRelationType::ManyToMany,
            // Fallback: if exact is specified, treat as OneToOne
            (_, _) if cardinality.exact == Some(1) => GeneratorRelationType::OneToOne,
            // Any other explicit many-side cardinality
            (_, Some(n)) if n > 1 => GeneratorRelationType::ManyToMany,
            _ => GeneratorRelationType::ManyToOne,
        }
    }

    fn map_relation_type(rt: &RelationType) -> GeneratorRelationType {
        match rt {
            RelationType::Association => GeneratorRelationType::ManyToOne,
            RelationType::Aggregation => GeneratorRelationType::OneToMany,
            RelationType::Composition => GeneratorRelationType::OneToOne,
            RelationType::Inheritance => GeneratorRelationType::ManyToOne,
            RelationType::Dependency => GeneratorRelationType::ManyToOne,
            RelationType::Realization => GeneratorRelationType::ManyToOne,
            RelationType::Custom(_) => GeneratorRelationType::ManyToOne,
        }
    }

    fn reverse_relation_type(rt: &RelationType) -> GeneratorRelationType {
        match rt {
            RelationType::Association => GeneratorRelationType::OneToMany,
            RelationType::Aggregation => GeneratorRelationType::ManyToOne,
            RelationType::Composition => GeneratorRelationType::OneToOne,
            RelationType::Inheritance => GeneratorRelationType::ManyHasMany,
            RelationType::Dependency => GeneratorRelationType::OneToMany,
            RelationType::Realization => GeneratorRelationType::OneToMany,
            RelationType::Custom(_) => GeneratorRelationType::ManyToOne,
        }
    }

    fn to_snake(s: &str) -> String {
        use convert_case::{Case, Casing};
        s.to_case(Case::Snake)
    }

    fn to_pascal(s: &str) -> String {
        use convert_case::{Case, Casing};
        s.to_case(Case::Pascal)
    }
    // ── Module dependency loading ─────────────────────────────────────────
    /// Load external module dependencies from `Modules/{id}/module.json` files.
    fn load_module_dependencies(used_modules: &[String]) -> Vec<ModuleDependency> {
        let workspace_root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../..");
        let mut deps = Vec::new();
        for module_id in used_modules {
            let json_path = format!(
                "{}/Pre-Proc/Alioth/Sources/Modules/{}/module.json",
                workspace_root, module_id
            );
            let content = match std::fs::read_to_string(&json_path) {
                Ok(c) => c,
                Err(e) => {
                    common::telemetry::warn!(
                        "Failed to read module.json for '{}': {}",
                        module_id,
                        e
                    );
                    continue;
                }
            };
            let module_json: ModuleJson = match serde_json::from_str(&content) {
                Ok(m) => m,
                Err(e) => {
                    common::telemetry::warn!(
                        "Failed to parse module.json for '{}': {}",
                        module_id,
                        e
                    );
                    continue;
                }
            };
            let crate_name = module_json
                .backend_crate
                .or(module_json.crate_name)
                .unwrap_or_else(|| format!("{}-backend", module_id));
            let path = format!(
                "../../Pre-Proc/{}/Sources/Modules/{}/backend",
                "Alioth", module_id
            );
            let (exported_tables, extension_points) =
                if let Some(ep) = &module_json.extension_points {
                    let tables: Vec<String> =
                        ep.entities.iter().map(|e| e.table_name.clone()).collect();
                    let points: Vec<String> =
                        ep.entities.iter().map(|e| e.entity_name.clone()).collect();
                    (tables, points)
                } else {
                    (vec![], vec![])
                };
            let exported_events = module_json
                .dependencies
                .and_then(|d| d.events)
                .map(|e| e.publishes)
                .unwrap_or_default();
            deps.push(ModuleDependency {
                module_id: module_id.clone(),
                crate_name,
                path,
                exported_tables,
                exported_events,
                extension_points,
            });
        }
        deps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::ontology::{
        ConstraintType, DomainKind, OntologyMetadata, OntologyProperty, PropertyConstraint,
        PropertyType,
    };

    #[test]
    fn test_transform_empty_model() {
        let ontology = OntologyModel {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: None,
            version: "1.0".to_string(),
            domains: vec![],
            transaction_lifecycle: None,
            relations: vec![],
            constraints: vec![],
            computations: vec![],
            namespaces: std::collections::HashMap::new(),
            metadata: OntologyMetadata::default(),
        };

        let model = OntologyTransformer::transform(&ontology, vec![]);
        assert!(model.entities.is_empty());
        assert!(model.enums.is_empty());
    }

    #[test]
    fn test_transform_entity_domain() {
        let domain = DomainOntology {
            id: "user".to_string(),
            name: "User".to_string(),
            description: Some("A user".to_string()),
            kind: DomainKind::Entity,
            parent_ids: vec![],
            equivalent_ids: vec![],
            disjoint_ids: vec![],
            properties: vec![OntologyProperty {
                id: "email".to_string(),
                name: "email".to_string(),
                property_type: PropertyType::DataProperty,
                required: true,
                cardinality: Cardinality {
                    min: Some(1),
                    max: Some(1),
                    exact: None,
                },
                domain: "User".to_string(),
                range: "String".to_string(),
                is_functional: true,
                is_transitive: false,
                is_symmetric: false,
                constraints: vec![PropertyConstraint {
                    constraint_type: ConstraintType::Pattern,
                    value: r"^[^@]+@[^@]+$".to_string(),
                    description: Some("Must be a valid email".to_string()),
                }],
                semantic_description: Some("User email address".to_string()),
            }],
            prefab_contract: None,
        };

        let ontology = OntologyModel {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: None,
            version: "1.0".to_string(),
            domains: vec![domain],
            transaction_lifecycle: None,
            relations: vec![],
            constraints: vec![],
            computations: vec![],
            namespaces: std::collections::HashMap::new(),
            metadata: OntologyMetadata::default(),
        };

        let model = OntologyTransformer::transform(&ontology, vec![]);
        assert_eq!(model.entities.len(), 1);
        let entity = &model.entities[0];
        assert_eq!(entity.name.raw, "User");
        assert_eq!(entity.fields.len(), 1);
        assert_eq!(entity.fields[0].name.raw, "email");
        assert!(!entity.fields[0].nullable);
    }
}
