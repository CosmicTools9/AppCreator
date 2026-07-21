//! Ontology Reasoner
//!
//! 本体推理引擎 - 基于本体语义模型进行关系推理和冲突检测
//! 支持可视化推演所需的关系推导和约束验证

use super::ontology::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};

/// 本体推理引擎
pub struct OntologyReasoner;

/// 缓存频繁访问的本体数据
///
/// 用法：
/// ```rust,ignore
/// let mut cache = OntologyCache::new(&model);
/// let props = cache.get_all_properties(&model, "entity_id");
/// ```
pub struct OntologyCache {
    property_cache: HashMap<String, Vec<OntologyProperty>>,
    model_fingerprint: u64,
}

impl OntologyCache {
    fn compute_fingerprint(model: &OntologyModel) -> u64 {
        let mut hasher = DefaultHasher::new();
        model.domains.len().hash(&mut hasher);
        model.relations.len().hash(&mut hasher);
        model.constraints.len().hash(&mut hasher);
        hasher.finish()
    }

    /// 创建新的缓存实例
    pub fn new(model: &OntologyModel) -> Self {
        Self {
            property_cache: HashMap::new(),
            model_fingerprint: Self::compute_fingerprint(model),
        }
    }

    /// 获取本体的所有属性（含继承），结果缓存
    pub fn get_all_properties(
        &mut self,
        model: &OntologyModel,
        ontology_id: &str,
    ) -> &[OntologyProperty] {
        let fp = Self::compute_fingerprint(model);
        if fp != self.model_fingerprint {
            self.property_cache.clear();
            self.model_fingerprint = fp;
        }
        self.property_cache
            .entry(ontology_id.to_string())
            .or_insert_with(|| OntologyReasoner::get_all_properties(model, ontology_id))
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        self.property_cache.clear();
    }
}

impl OntologyReasoner {
    /// 执行完整本体推理
    pub fn reason(model: &OntologyModel) -> OntologyInferenceResult {
        let mut result = OntologyInferenceResult::default();

        // domain_map 只构建一次，传递给所有子函数
        let domain_map: HashMap<String, &DomainOntology> =
            model.domains.iter().map(|d| (d.id.clone(), d)).collect();

        // 1. 继承关系推理
        Self::infer_inheritance_relations(model, &domain_map, &mut result);

        // 2. 传递关系推理
        Self::infer_transitive_relations(model, &mut result);

        // 3. 对称关系推理
        Self::infer_symmetric_relations(model, &mut result);

        // 4. 属性继承推理
        Self::infer_property_inheritance(model, &domain_map, &mut result);

        // 5. 冲突检测
        Self::detect_conflicts(model, &domain_map, &mut result);

        // 6. 约束传播
        Self::propagate_constraints(model, &domain_map, &mut result);

        // 7. 属性约束推理
        Self::infer_property_constraints(model, &mut result);

        result
    }

    /// 推断继承关系
    fn infer_inheritance_relations(
        model: &OntologyModel,
        domain_map: &HashMap<String, &DomainOntology>,
        result: &mut OntologyInferenceResult,
    ) {
        for domain in &model.domains {
            // 处理父类继承
            for parent_id in &domain.parent_ids {
                if let Some(parent) = domain_map.get(parent_id) {
                    result.inferred_relations.push(InferredRelation {
                        source: domain.id.clone(),
                        target: parent.id.clone(),
                        relation_type: RelationType::Inheritance,
                        inference_source: format!("{} extends {}", domain.name, parent.name),
                    });

                    // 推断属性继承
                    for prop in &parent.properties {
                        let mut inherited_prop = prop.clone();
                        inherited_prop.id = format!("{}.{}", domain.id, prop.id);
                        result.inferred_properties.push(InferredProperty {
                            ontology: domain.id.clone(),
                            property: inherited_prop,
                            inference_source: format!("Inherited from {}", parent.name),
                        });
                    }

                    result.applied_rules.push("inheritance_rule".to_string());
                }
            }

            // 处理等价类
            for equiv_id in &domain.equivalent_ids {
                if let Some(equiv) = domain_map.get(equiv_id) {
                    result.inferred_relations.push(InferredRelation {
                        source: domain.id.clone(),
                        target: equiv.id.clone(),
                        relation_type: RelationType::Custom("equivalent".to_string()),
                        inference_source: format!("{} equivalentTo {}", domain.name, equiv.name),
                    });

                    result.applied_rules.push("equivalence_rule".to_string());
                }
            }
        }
    }

    /// 推断传递关系（BFS + Memoization，O(N·(N+E))）
    fn infer_transitive_relations(model: &OntologyModel, result: &mut OntologyInferenceResult) {
        let mut relation_graph: HashMap<String, Vec<String>> = HashMap::new();

        // 构建关系图
        for relation in &model.relations {
            if relation.relation_type == RelationType::Association {
                relation_graph
                    .entry(relation.source_ontology.clone())
                    .or_default()
                    .push(relation.target_ontology.clone());
            }
        }

        let mut any_inferred = false;
        let sources: Vec<String> = relation_graph.keys().cloned().collect();

        for source in &sources {
            let mut visited = HashSet::new();
            let mut queue = VecDeque::new();

            if let Some(neighbors) = relation_graph.get(source) {
                for n in neighbors {
                    queue.push_back(n.clone());
                }
            }

            while let Some(current) = queue.pop_front() {
                if current == *source {
                    continue;
                }
                if !visited.insert(current.clone()) {
                    continue;
                }

                if let Some(next_neighbors) = relation_graph.get(&current) {
                    for next in next_neighbors {
                        if next != source && !visited.contains(next) {
                            queue.push_back(next.clone());
                        }
                    }
                }
            }

            let direct: HashSet<&str> = relation_graph
                .get(source)
                .map(|v| v.iter().map(|s| s.as_str()).collect())
                .unwrap_or_default();

            for target in &visited {
                if !direct.contains(target.as_str()) {
                    result.inferred_relations.push(InferredRelation {
                        source: source.clone(),
                        target: target.clone(),
                        relation_type: RelationType::Association,
                        inference_source: format!("Transitive: {} -> {}", source, target),
                    });
                    any_inferred = true;
                }
            }
        }

        if any_inferred {
            result.applied_rules.push("transitivity_rule".to_string());
        }
    }

    /// 推断对称关系
    fn infer_symmetric_relations(model: &OntologyModel, result: &mut OntologyInferenceResult) {
        // 预计算所有 (source, target) 对，O(n) 空间换 O(1) 查询
        let existing_pairs: HashSet<(&str, &str)> = model
            .relations
            .iter()
            .map(|r| (r.source_ontology.as_str(), r.target_ontology.as_str()))
            .collect();

        for relation in &model.relations {
            if relation.is_bidirectional {
                let has_reverse = existing_pairs.contains(&(
                    relation.target_ontology.as_str(),
                    relation.source_ontology.as_str(),
                ));

                if !has_reverse {
                    result.inferred_relations.push(InferredRelation {
                        source: relation.target_ontology.clone(),
                        target: relation.source_ontology.clone(),
                        relation_type: relation.relation_type.clone(),
                        inference_source: format!(
                            "Symmetric: {} <-> {}",
                            relation.source_ontology, relation.target_ontology
                        ),
                    });
                }
            }
        }

        if result
            .inferred_relations
            .iter()
            .any(|r| r.inference_source.starts_with("Symmetric"))
        {
            result.applied_rules.push("symmetry_rule".to_string());
        }
    }

    /// 推断属性继承
    fn infer_property_inheritance(
        model: &OntologyModel,
        domain_map: &HashMap<String, &DomainOntology>,
        result: &mut OntologyInferenceResult,
    ) {
        for domain in &model.domains {
            for parent_id in &domain.parent_ids {
                if let Some(parent) = domain_map.get(parent_id) {
                    for prop in &parent.properties {
                        let exists = domain.properties.iter().any(|p| p.name == prop.name);
                        if !exists {
                            let mut inherited = prop.clone();
                            inherited.id = format!("inherited_{}", prop.id);
                            inherited.domain = domain.id.clone();
                            result.inferred_properties.push(InferredProperty {
                                ontology: domain.id.clone(),
                                property: inherited,
                                inference_source: "Property inheritance".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    /// 冲突检测
    fn detect_conflicts(
        model: &OntologyModel,
        domain_map: &HashMap<String, &DomainOntology>,
        result: &mut OntologyInferenceResult,
    ) {
        // 1. 互斥类冲突检测
        for domain in &model.domains {
            for disjoint_id in &domain.disjoint_ids {
                if let Some(disjoint) = domain_map.get(disjoint_id) {
                    // 避免构建两个完整 HashSet，直接遍历检查
                    let mut has_common = false;
                    for pid in &domain.parent_ids {
                        if disjoint.parent_ids.contains(pid) {
                            has_common = true;
                            break;
                        }
                    }

                    if has_common {
                        let common_parents: Vec<String> = domain
                            .parent_ids
                            .iter()
                            .filter(|p| disjoint.parent_ids.contains(*p))
                            .cloned()
                            .collect();

                        result.conflicts.push(OntologyConflict {
                            conflict_type: ConflictType::DisjointConflict,
                            description: format!(
                                "Disjoint classes {} and {} share common parent(s): {:?}",
                                domain.name, disjoint.name, common_parents
                            ),
                            involved_ontologies: vec![domain.id.clone(), disjoint.id.clone()],
                            severity: ConflictSeverity::Warning,
                        });
                    }
                }
            }
        }

        // 2. 关系冲突检测
        for relation in &model.relations {
            if !domain_map.contains_key(&relation.source_ontology) {
                result.conflicts.push(OntologyConflict {
                    conflict_type: ConflictType::RelationConflict,
                    description: format!(
                        "Relation {} references non-existent source ontology: {}",
                        relation.name, relation.source_ontology
                    ),
                    involved_ontologies: vec![relation.id.clone()],
                    severity: ConflictSeverity::Error,
                });
            }

            if !domain_map.contains_key(&relation.target_ontology) {
                result.conflicts.push(OntologyConflict {
                    conflict_type: ConflictType::RelationConflict,
                    description: format!(
                        "Relation {} references non-existent target ontology: {}",
                        relation.name, relation.target_ontology
                    ),
                    involved_ontologies: vec![relation.id.clone()],
                    severity: ConflictSeverity::Error,
                });
            }
        }

        // 3. 约束冲突检测
        for constraint in &model.constraints {
            if !domain_map.contains_key(&constraint.scope.target_ontology) {
                result.conflicts.push(OntologyConflict {
                    conflict_type: ConflictType::ConstraintConflict,
                    description: format!(
                        "Constraint {} targets non-existent ontology: {}",
                        constraint.name, constraint.scope.target_ontology
                    ),
                    involved_ontologies: vec![constraint.id.clone()],
                    severity: ConflictSeverity::Error,
                });
            }
        }
    }

    /// 约束传播
    fn propagate_constraints(
        model: &OntologyModel,
        domain_map: &HashMap<String, &DomainOntology>,
        result: &mut OntologyInferenceResult,
    ) {
        for domain in &model.domains {
            for parent_id in &domain.parent_ids {
                if let Some(parent) = domain_map.get(parent_id) {
                    for constraint in &model.constraints {
                        if constraint.scope.target_ontology == parent.id {
                            let mut propagated = constraint.clone();
                            propagated.id = format!("propagated_{}", constraint.id);
                            propagated.scope.target_ontology = domain.id.clone();

                            result.inferred_constraints.push(InferredConstraint {
                                constraint: propagated,
                                inference_source: format!("Propagated from {}", parent.name),
                            });
                        }
                    }
                }
            }
        }

        if !result.inferred_constraints.is_empty() {
            result
                .applied_rules
                .push("constraint_propagation".to_string());
        }
    }

    /// 属性约束推理
    fn infer_property_constraints(model: &OntologyModel, result: &mut OntologyInferenceResult) {
        for domain in &model.domains {
            for prop in &domain.properties {
                // Required
                if prop.required {
                    result.inferred_constraints.push(InferredConstraint {
                        constraint: ConstraintOntology {
                            id: format!("{}_{}_required", domain.id, prop.id),
                            name: format!("{} required", prop.name),
                            constraint_type: ConstraintOntologyType::DataQuality,
                            scope: ConstraintScope {
                                target_ontology: domain.id.clone(),
                                target_property: Some(prop.name.clone()),
                                context: vec![],
                            },
                            expression: "required".to_string(),
                            description: Some(format!("属性 {} 是必需的", prop.name)),
                            error_message_template: Some(format!("字段 '{}' 不能为空", prop.name)),
                            severity: ConstraintSeverity::Error,
                        },
                        inference_source: format!(
                            "Derived from required flag of {}.{}",
                            domain.name, prop.name
                        ),
                    });
                }

                // Unique (functional or max cardinality 1)
                if prop.is_functional || prop.cardinality.max == Some(1) {
                    result.inferred_constraints.push(InferredConstraint {
                        constraint: ConstraintOntology {
                            id: format!("{}_{}_unique", domain.id, prop.id),
                            name: format!("{} unique", prop.name),
                            constraint_type: ConstraintOntologyType::Structural,
                            scope: ConstraintScope {
                                target_ontology: domain.id.clone(),
                                target_property: Some(prop.name.clone()),
                                context: vec![],
                            },
                            expression: "unique".to_string(),
                            description: Some(format!("属性 {} 具有唯一性", prop.name)),
                            error_message_template: Some(format!(
                                "字段 '{}' 的值必须唯一",
                                prop.name
                            )),
                            severity: ConstraintSeverity::Error,
                        },
                        inference_source: format!(
                            "Derived from functional/max-cardinality of {}.{}",
                            domain.name, prop.name
                        ),
                    });
                }

                // Cardinality
                if prop.cardinality.min.is_some()
                    || prop.cardinality.max.is_some()
                    || prop.cardinality.exact.is_some()
                {
                    result.inferred_constraints.push(InferredConstraint {
                        constraint: ConstraintOntology {
                            id: format!("{}_{}_cardinality", domain.id, prop.id),
                            name: format!("{} cardinality", prop.name),
                            constraint_type: ConstraintOntologyType::Structural,
                            scope: ConstraintScope {
                                target_ontology: domain.id.clone(),
                                target_property: Some(prop.name.clone()),
                                context: vec![],
                            },
                            expression: format!(
                                "cardinality(min={:?},max={:?},exact={:?})",
                                prop.cardinality.min, prop.cardinality.max, prop.cardinality.exact
                            ),
                            description: Some(format!("属性 {} 的基数约束", prop.name)),
                            error_message_template: Some(format!(
                                "字段 '{}' 的基数不符合要求",
                                prop.name
                            )),
                            severity: ConstraintSeverity::Error,
                        },
                        inference_source: format!(
                            "Derived from cardinality of {}.{}",
                            domain.name, prop.name
                        ),
                    });
                }
            }
        }
    }

    /// 本体一致性检查
    pub fn consistency_check(model: &OntologyModel) -> Vec<OntologyConflict> {
        let mut conflicts = vec![];
        let domain_map: HashMap<String, &DomainOntology> =
            model.domains.iter().map(|d| (d.id.clone(), d)).collect();

        // 1. 类层次循环检测
        for domain in &model.domains {
            let mut visited = HashSet::new();
            let mut rec_stack = HashSet::new();
            let mut path = vec![];
            Self::detect_cycle_in_hierarchy(
                domain.id.as_str(),
                &domain_map,
                &mut visited,
                &mut rec_stack,
                &mut path,
                &mut conflicts,
            );
        }

        // 2. 属性域/范围校验
        let primitives: HashSet<&str> = [
            "string", "int", "bigint", "decimal", "bool", "datetime", "float", "date", "text",
            "uuid", "json",
        ]
        .iter()
        .cloned()
        .collect();

        for domain in &model.domains {
            for prop in &domain.properties {
                if prop.domain != domain.id {
                    conflicts.push(OntologyConflict {
                        conflict_type: ConflictType::RelationConflict,
                        description: format!(
                            "Property '{}' domain '{}' does not match owning ontology '{}'",
                            prop.name, prop.domain, domain.id
                        ),
                        involved_ontologies: vec![domain.id.clone(), prop.id.clone()],
                        severity: ConflictSeverity::Error,
                    });
                }

                if !primitives.contains(prop.range.as_str())
                    && !domain_map.contains_key(&prop.range)
                {
                    conflicts.push(OntologyConflict {
                        conflict_type: ConflictType::RelationConflict,
                        description: format!(
                            "Property '{}' range '{}' is neither a primitive type nor a known ontology",
                            prop.name, prop.range
                        ),
                        involved_ontologies: vec![domain.id.clone(), prop.id.clone()],
                        severity: ConflictSeverity::Error,
                    });
                }
            }
        }

        conflicts
    }

    fn detect_cycle_in_hierarchy(
        ontology_id: &str,
        domain_map: &HashMap<String, &DomainOntology>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        conflicts: &mut Vec<OntologyConflict>,
    ) {
        visited.insert(ontology_id.to_string());
        rec_stack.insert(ontology_id.to_string());
        path.push(ontology_id.to_string());

        if let Some(domain) = domain_map.get(ontology_id) {
            for parent_id in &domain.parent_ids {
                if !visited.contains(parent_id) {
                    Self::detect_cycle_in_hierarchy(
                        parent_id, domain_map, visited, rec_stack, path, conflicts,
                    );
                } else if rec_stack.contains(parent_id) {
                    if let Some(pos) = path.iter().position(|p| p == parent_id) {
                        let cycle = path[pos..].to_vec();
                        conflicts.push(OntologyConflict {
                            conflict_type: ConflictType::InheritanceConflict,
                            description: format!(
                                "Circular inheritance detected: {}",
                                cycle.join(" -> ")
                            ),
                            involved_ontologies: cycle.clone(),
                            severity: ConflictSeverity::Error,
                        });
                    }
                }
            }
        }

        path.pop();
        rec_stack.remove(ontology_id);
    }

    /// 获取本体的所有属性（包括继承的）
    pub fn get_all_properties(model: &OntologyModel, ontology_id: &str) -> Vec<OntologyProperty> {
        let domain_map: HashMap<String, &DomainOntology> =
            model.domains.iter().map(|d| (d.id.clone(), d)).collect();

        let mut properties = Vec::new();
        let mut visited = HashSet::new();

        Self::collect_properties_recursive(ontology_id, &domain_map, &mut properties, &mut visited);

        properties
    }

    fn collect_properties_recursive(
        ontology_id: &str,
        domain_map: &HashMap<String, &DomainOntology>,
        properties: &mut Vec<OntologyProperty>,
        visited: &mut HashSet<String>,
    ) {
        if !visited.insert(ontology_id.to_string()) {
            return;
        }

        if let Some(domain) = domain_map.get(ontology_id) {
            // 先添加父类的属性
            for parent_id in &domain.parent_ids {
                Self::collect_properties_recursive(parent_id, domain_map, properties, visited);
            }

            // 再添加当前类的属性（使用 extend + cloned 避免重复分配）
            properties.extend(domain.properties.iter().cloned());
        }
    }

    /// 检查两个本体是否有关系路径
    pub fn has_relation_path(model: &OntologyModel, from: &str, to: &str) -> bool {
        // 预构建双向邻接表，避免每次 BFS 都扫描全部 relations
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for relation in &model.relations {
            adj.entry(relation.source_ontology.clone())
                .or_default()
                .push(relation.target_ontology.clone());
            if relation.is_bidirectional {
                adj.entry(relation.target_ontology.clone())
                    .or_default()
                    .push(relation.source_ontology.clone());
            }
        }

        let mut visited = HashSet::new();
        let mut stack = vec![from.to_string()];

        while let Some(current) = stack.pop() {
            if current == to {
                return true;
            }
            if !visited.insert(current.clone()) {
                continue;
            }
            if let Some(neighbors) = adj.get(&current) {
                for next in neighbors {
                    if !visited.contains(next) {
                        stack.push(next.clone());
                    }
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn create_test_model() -> OntologyModel {
        OntologyModel {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: None,
            version: "1.0".to_string(),
            domains: vec![
                DomainOntology {
                    id: "animal".to_string(),
                    name: "Animal".to_string(),
                    description: None,
                    kind: DomainKind::Entity,
                    parent_ids: vec![],
                    equivalent_ids: vec![],
                    disjoint_ids: vec![],
                    properties: vec![OntologyProperty {
                        id: "name".to_string(),
                        name: "name".to_string(),
                        property_type: PropertyType::DataProperty,
                        required: true,
                        cardinality: Cardinality {
                            min: Some(1),
                            max: Some(1),
                            exact: None,
                        },
                        domain: "animal".to_string(),
                        range: "string".to_string(),
                        is_functional: true,
                        is_transitive: false,
                        is_symmetric: false,
                        constraints: vec![],
                        semantic_description: None,
                    }],
                    prefab_contract: None,
                },
                DomainOntology {
                    id: "dog".to_string(),
                    name: "Dog".to_string(),
                    description: None,
                    kind: DomainKind::Entity,
                    parent_ids: vec!["animal".to_string()],
                    equivalent_ids: vec![],
                    disjoint_ids: vec![],
                    properties: vec![],
                    prefab_contract: None,
                },
            ],
            transaction_lifecycle: None,
            relations: vec![],
            constraints: vec![],
            computations: vec![],
            namespaces: HashMap::new(),
            metadata: OntologyMetadata::default(),
        }
    }

    #[test]
    fn test_inheritance_inference() {
        let model = create_test_model();
        let result = OntologyReasoner::reason(&model);

        // 应该推断出 Dog 继承 Animal 的属性
        let dog_props: Vec<_> = result
            .inferred_properties
            .iter()
            .filter(|p| p.ontology == "dog")
            .collect();

        assert!(!dog_props.is_empty(), "Should infer inherited properties");
        assert_eq!(dog_props[0].property.name, "name");
    }

    #[test]
    fn test_relation_path() {
        let mut model = create_test_model();
        model.relations.push(RelationOntology {
            id: "has-owner".to_string(),
            name: "hasOwner".to_string(),
            relation_type: RelationType::Association,
            source_ontology: "dog".to_string(),
            target_ontology: "animal".to_string(),
            is_bidirectional: false,
            properties: vec![],
            constraints: vec![],
            semantic_description: None,
        });

        assert!(OntologyReasoner::has_relation_path(&model, "dog", "animal"));
        assert!(!OntologyReasoner::has_relation_path(
            &model, "animal", "dog"
        ));
    }

    #[test]
    fn test_circular_inheritance() {
        let mut model = create_test_model();
        // A -> B -> C -> A
        model.domains.push(DomainOntology {
            id: "a".to_string(),
            name: "A".to_string(),
            description: None,
            kind: DomainKind::Entity,
            parent_ids: vec!["c".to_string()],
            equivalent_ids: vec![],
            disjoint_ids: vec![],
            properties: vec![],
            prefab_contract: None,
        });
        model.domains.push(DomainOntology {
            id: "b".to_string(),
            name: "B".to_string(),
            description: None,
            kind: DomainKind::Entity,
            parent_ids: vec!["a".to_string()],
            equivalent_ids: vec![],
            disjoint_ids: vec![],
            properties: vec![],
            prefab_contract: None,
        });
        model.domains.push(DomainOntology {
            id: "c".to_string(),
            name: "C".to_string(),
            description: None,
            kind: DomainKind::Entity,
            parent_ids: vec!["b".to_string()],
            equivalent_ids: vec![],
            disjoint_ids: vec![],
            properties: vec![],
            prefab_contract: None,
        });

        let conflicts = OntologyReasoner::consistency_check(&model);
        assert!(
            conflicts
                .iter()
                .any(|c| c.conflict_type == ConflictType::InheritanceConflict),
            "Should detect circular inheritance"
        );
        assert!(
            conflicts
                .iter()
                .all(|c| c.severity == ConflictSeverity::Error),
            "All consistency errors should be Error severity"
        );
    }

    #[test]
    fn test_property_domain_mismatch() {
        let mut model = create_test_model();
        model.domains[0].properties.push(OntologyProperty {
            id: "wrong".to_string(),
            name: "wrong".to_string(),
            property_type: PropertyType::DataProperty,
            required: false,
            cardinality: Cardinality {
                min: None,
                max: None,
                exact: None,
            },
            domain: "nonexistent".to_string(),
            range: "string".to_string(),
            is_functional: false,
            is_transitive: false,
            is_symmetric: false,
            constraints: vec![],
            semantic_description: None,
        });

        let conflicts = OntologyReasoner::consistency_check(&model);
        assert!(
            conflicts.iter().any(|c| c.description.contains("domain")),
            "Should detect property domain mismatch"
        );
    }

    #[test]
    fn test_constraint_inference() {
        let mut model = create_test_model();
        model.domains[0].properties[0].required = true;
        model.domains[0].properties[0].is_functional = true;
        model.domains[0].properties[0].cardinality.max = Some(1);

        let result = OntologyReasoner::reason(&model);
        let required_count = result
            .inferred_constraints
            .iter()
            .filter(|c| c.constraint.expression == "required")
            .count();
        let unique_count = result
            .inferred_constraints
            .iter()
            .filter(|c| c.constraint.expression == "unique")
            .count();
        let cardinality_count = result
            .inferred_constraints
            .iter()
            .filter(|c| c.constraint.expression.starts_with("cardinality"))
            .count();

        assert_eq!(required_count, 1, "Should infer required constraint");
        assert_eq!(unique_count, 1, "Should infer unique constraint");
        assert_eq!(cardinality_count, 1, "Should infer cardinality constraint");
    }

    #[test]
    fn test_transitive_performance() {
        let mut model = OntologyModel {
            id: "perf".to_string(),
            name: "Perf".to_string(),
            description: None,
            version: "1.0".to_string(),
            domains: vec![],
            transaction_lifecycle: None,
            relations: vec![],
            constraints: vec![],
            computations: vec![],
            namespaces: HashMap::new(),
            metadata: OntologyMetadata::default(),
        };

        let n = 1000;
        for i in 0..n {
            model.domains.push(DomainOntology {
                id: format!("c{}", i),
                name: format!("C{}", i),
                description: None,
                kind: DomainKind::Entity,
                parent_ids: vec![],
                equivalent_ids: vec![],
                disjoint_ids: vec![],
                properties: vec![],
                prefab_contract: None,
            });
        }

        // Chain relations: c0 -> c1 -> c2 -> ... -> c999
        for i in 0..(n - 1) {
            model.relations.push(RelationOntology {
                id: format!("r{}", i),
                name: format!("R{}", i),
                relation_type: RelationType::Association,
                source_ontology: format!("c{}", i),
                target_ontology: format!("c{}", i + 1),
                is_bidirectional: false,
                properties: vec![],
                constraints: vec![],
                semantic_description: None,
            });
        }

        let start = std::time::Instant::now();
        let result = OntologyReasoner::reason(&model);
        let elapsed = start.elapsed();

        // Should infer transitive relations for the chain
        assert!(
            !result.inferred_relations.is_empty(),
            "Should infer transitive relations"
        );
        // c0 should reach c999 transitively
        assert!(
            result
                .inferred_relations
                .iter()
                .any(|r| r.source == "c0" && r.target == format!("c{}", n - 1)),
            "Should infer c0 -> c{} transitive relation",
            n - 1
        );
        assert!(
            elapsed.as_millis() < 1500,
            "Reasoning should complete within 1500ms, took {}ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_ontology_cache() {
        let model = create_test_model();
        let mut cache = OntologyCache::new(&model);

        let props1 = cache.get_all_properties(&model, "animal");
        assert_eq!(props1.len(), 1);
        assert_eq!(props1[0].name, "name");

        // dog 继承 animal 的 name 属性
        let props2 = cache.get_all_properties(&model, "dog");
        assert_eq!(props2.len(), 1);
        assert_eq!(props2[0].name, "name");

        // 第二次读取应命中缓存
        let props3 = cache.get_all_properties(&model, "animal");
        assert_eq!(props3.len(), 1);
    }

    #[test]
    fn test_symmetric_relation_optimization() {
        let mut model = create_test_model();
        model.relations.push(RelationOntology {
            id: "r1".to_string(),
            name: "R1".to_string(),
            relation_type: RelationType::Association,
            source_ontology: "animal".to_string(),
            target_ontology: "dog".to_string(),
            is_bidirectional: true,
            properties: vec![],
            constraints: vec![],
            semantic_description: None,
        });
        // 添加反向关系，不应再推断
        model.relations.push(RelationOntology {
            id: "r2".to_string(),
            name: "R2".to_string(),
            relation_type: RelationType::Association,
            source_ontology: "dog".to_string(),
            target_ontology: "animal".to_string(),
            is_bidirectional: false,
            properties: vec![],
            constraints: vec![],
            semantic_description: None,
        });

        let result = OntologyReasoner::reason(&model);
        let symmetric_count = result
            .inferred_relations
            .iter()
            .filter(|r| r.inference_source.starts_with("Symmetric"))
            .count();
        // r1 已经是双向且存在显式反向 r2，不应再推断对称关系
        assert_eq!(
            symmetric_count, 0,
            "Should not infer symmetric when reverse exists"
        );
    }
}
