//! GeneratorModel (IR-2) → MetaModule (IR-1) 转换
//!
//! 将 OntologyModel→GeneratorModel 的输出转换为 ModuleApiGenerator
//! 和 ModuleFrontendGenerator 可消费的 MetaModule。

use super::module::*;
use super::{
    GeneratorConstraintLevel, GeneratorFieldType, GeneratorModel, GeneratorRelationType,
    GeneratorValidationType,
};

/// 从 GeneratorModel (IR-2) 构建 MetaModule (IR-1)
pub fn meta_module_from_generator_model(
    model: &GeneratorModel,
    name: impl Into<String>,
) -> MetaModule {
    let entities: Vec<MetaEntity> = model
        .entities
        .iter()
        .map(|ge| {
            let fields: Vec<MetaField> = ge
                .fields
                .iter()
                .map(|gf| MetaField {
                    name: gf.name.raw.clone(),
                    field_type: map_field_type(&gf.field_type),
                    description: gf.description.clone(),
                    nullable: gf.nullable,
                    unique: gf.unique,
                    indexed: gf.indexed,
                    default_value: gf.default_value.clone(),
                    validations: gf
                        .validations
                        .iter()
                        .map(|gv| MetaValidation {
                            validation_type: map_validation_type(&gv.validation_type),
                            params: gv.params.clone(),
                        })
                        .collect(),
                    annotations: vec![],
                    domain: gf.domain.clone(),
                    range: gf.range.clone(),
                    min_cardinality: gf.min_cardinality,
                    max_cardinality: gf.max_cardinality,
                    is_functional: gf.is_functional,
                    constraints: vec![],
                    field_permission: Default::default(),
                    throws_clauses: vec![],
                    quality_rules: vec![],
                })
                .collect();

            let relations: Vec<MetaRelation> = ge
                .relations
                .iter()
                .map(|gr| MetaRelation {
                    name: gr.name.clone(),
                    target_entity: gr.target_entity.clone(),
                    relation_type: map_relation_type(&gr.relation_type),
                    nullable: gr.nullable,
                    via: None,
                    target_table: None,
                })
                .collect();

            let state_machine = if ge.state_machine.enabled {
                MetaStateMachine {
                    enabled: true,
                    states: ge
                        .state_machine
                        .states
                        .iter()
                        .map(|s| s.name.clone())
                        .collect(),
                    initial_state: ge.state_machine.initial_state.clone(),
                    state_field: Some(ge.state_machine.state_field.clone()),
                }
            } else {
                MetaStateMachine {
                    enabled: false,
                    ..Default::default()
                }
            };

            MetaEntity {
                name: ge.name.raw.clone(),
                description: ge.description.clone(),
                fields,
                relations,
                annotations: vec![],
                parent_classes: ge.parent_classes.iter().map(|pn| pn.raw.clone()).collect(),
                equivalent_classes: ge.equivalent_classes.clone(),
                disjoint_classes: ge.disjoint_classes.clone(),
                is_abstract: ge.is_abstract,
                table_name: None,
                state_machine,
                transitions: ge
                    .transitions
                    .iter()
                    .map(|gt| MetaTransition {
                        event: gt.event.clone(),
                        from: gt.from.clone(),
                        to: gt.to.clone(),
                        guard: gt.guard.clone(),
                        action: gt.action.clone(),
                    })
                    .collect(),
                lifecycle_hooks: ge
                    .lifecycle_hooks
                    .iter()
                    .map(|gh| MetaLifecycleHook {
                        event: gh.event.clone(),
                        function_name: gh.function_name.clone(),
                        from_state: gh.from_state.clone(),
                        to_state: gh.to_state.clone(),
                        order: gh.order,
                    })
                    .collect(),
                business_rules: ge
                    .business_rules
                    .iter()
                    .map(|gbr| MetaBusinessRule {
                        name: gbr.name.clone(),
                        condition: gbr.condition.clone(),
                        action: gbr.action.clone(),
                        error_message: gbr.error_message.clone(),
                        priority: gbr.priority,
                        trigger: gbr.trigger.clone(),
                    })
                    .collect(),
                swrl_rules: ge
                    .swrl_rules
                    .iter()
                    .map(|gsr| MetaSwrlRule {
                        name: gsr.name.clone(),
                        description: gsr.description.clone(),
                        body: gsr.body.clone(),
                        head: gsr.head.clone(),
                        priority: gsr.priority,
                        active: gsr.active,
                    })
                    .collect(),
                constraints: ge
                    .constraints
                    .iter()
                    .map(|gc| MetaConstraint {
                        name: gc.name.clone(),
                        expression: gc.expression.clone(),
                        level: match gc.level {
                            GeneratorConstraintLevel::Field => MetaConstraintLevel::Field,
                            GeneratorConstraintLevel::Entity => MetaConstraintLevel::Entity,
                        },
                        error_message: gc.error_message.clone(),
                        error_code: gc.error_code.clone(),
                        active: gc.active,
                        blocking: gc.blocking,
                        field_name: gc.field_name.clone(),
                    })
                    .collect(),
                // 4D space types differ between IR-1 and IR-2; skip for now
                permission_config: Default::default(),
                permission_inheritance: Default::default(),
                permission_conflict_resolution: Default::default(),
                quality_rules: vec![],
            }
        })
        .collect();

    let mut module = MetaModule::new(name);
    module.entities = entities;
    module.pages = MetaModule::infer_pages(&module.entities);
    module
}

fn map_field_type(ft: &GeneratorFieldType) -> MetaFieldType {
    match ft {
        GeneratorFieldType::Text => MetaFieldType::String,
        GeneratorFieldType::Integer => MetaFieldType::Integer,
        GeneratorFieldType::BigInt => MetaFieldType::Long,
        GeneratorFieldType::Decimal => MetaFieldType::Decimal,
        GeneratorFieldType::Boolean => MetaFieldType::Boolean,
        GeneratorFieldType::DateTime => MetaFieldType::DateTime,
        GeneratorFieldType::Uuid => MetaFieldType::Uuid,
        GeneratorFieldType::Json => MetaFieldType::Json,
        GeneratorFieldType::Enum(s) => MetaFieldType::Enum(s.clone()),
        GeneratorFieldType::Reference(s) => MetaFieldType::Reference(s.clone()),
    }
}

fn map_validation_type(vt: &GeneratorValidationType) -> MetaValidationType {
    match vt {
        GeneratorValidationType::MinLength => MetaValidationType::MinLength,
        GeneratorValidationType::MaxLength => MetaValidationType::MaxLength,
        GeneratorValidationType::Pattern => MetaValidationType::Pattern,
        GeneratorValidationType::Min => MetaValidationType::Min,
        GeneratorValidationType::Max => MetaValidationType::Max,
        GeneratorValidationType::Email => MetaValidationType::Email,
        GeneratorValidationType::Url => MetaValidationType::Url,
        GeneratorValidationType::Custom(s) => MetaValidationType::Custom(s.clone()),
    }
}

fn map_relation_type(rt: &GeneratorRelationType) -> MetaRelationType {
    match rt {
        GeneratorRelationType::OneToOne => MetaRelationType::OneToOne,
        GeneratorRelationType::OneToMany => MetaRelationType::OneToMany,
        GeneratorRelationType::ManyToOne => MetaRelationType::ManyToOne,
        GeneratorRelationType::ManyToMany => MetaRelationType::ManyToMany,
        GeneratorRelationType::ManyHasMany => MetaRelationType::ManyHasMany,
    }
}
