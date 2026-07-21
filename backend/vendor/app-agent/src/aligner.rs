//! OntologyAligner — biz-ontology ↔ alioth-ontology 语义对齐图构建器。

use crate::state::*;
use alioth_gen::generator::ir::ontology::{DomainKind, OntologyModel, RelationType};
use sqlx::PgPool;

pub async fn build_alignment_graph(
    pool: &PgPool,
    model: &OntologyModel,
    mapped: &[MappedEntity],
    catalog: Option<&PlatformCatalog>,
    known_entities: &[String],
) -> AlignmentGraph {
    let domain_by_name: std::collections::HashMap<&str, &MappedEntity> =
        mapped.iter().map(|m| (m.domain_id.as_str(), m)).collect();

    let (nodes, mut gaps) =
        build_domain_nodes_and_gaps(model, &domain_by_name, catalog, known_entities);

    let mut edges = Vec::new();
    for rel in &model.relations {
        let source_mapped = domain_by_name.contains_key(rel.source_ontology.as_str());
        let target_mapped = domain_by_name.contains_key(rel.target_ontology.as_str());
        if !source_mapped || !target_mapped {
            record_missing_endpoint_gap(&mut gaps, rel, source_mapped, target_mapped);
            continue;
        }
        let source_table = domain_by_name[rel.source_ontology.as_str()].table.as_str();
        let target_table = domain_by_name[rel.target_ontology.as_str()].table.as_str();

        if let Some((mechanism, evidence)) =
            detect_relationship_mechanism(pool, source_table, target_table, &rel.relation_type)
                .await
        {
            edges.push(AlignmentEdge {
                biz_rel_id: rel.id.clone(),
                biz_rel_type: format!("{:?}", rel.relation_type),
                alioth_mechanism: mechanism,
                evidence: Some(evidence),
            });
        } else {
            gaps.push(AlignmentGap {
                biz_element: format!(
                    "relation {} ({} → {})",
                    rel.id, rel.source_ontology, rel.target_ontology
                ),
                description: "未找到 DB 中的 FK 列或 r_* 关系表证据".into(),
                suggested_alioth_entities: vec![],
            });
        }
    }

    AlignmentGraph { nodes, edges, gaps }
}

fn primary_role(kind: &DomainKind) -> String {
    match kind {
        DomainKind::AggregateRoot => "aggregate_root".into(),
        DomainKind::Entity => "entity".into(),
        DomainKind::ValueObject => "value_object".into(),
        _ => "entity".into(),
    }
}

fn build_node_from_mapping(
    domain: &alioth_gen::generator::ir::ontology::DomainOntology,
    m: &MappedEntity,
) -> AlignmentNode {
    AlignmentNode {
        biz_domain: domain.id.clone(),
        biz_kind: format!("{:?}", domain.kind),
        alioth_entities: vec![AliothBinding {
            table: m.table.clone(),
            role: primary_role(&domain.kind),
            coordinates: Some(CoordinatesSnapshot {
                scene: m.scene_code.clone(),
                factor: m.factor_code.clone(),
                function: m.function_code.clone(),
                function_confidence: m.function_confidence,
            }),
            field_mappings: m.field_mappings.clone(),
            constraints: vec![],
        }],
        evidence: format!(
            "discovery score={}, name_score={}, field_score={}",
            m.score, m.name_score, m.field_score
        ),
        confidence: m.score,
    }
}

fn build_domain_nodes_and_gaps(
    model: &OntologyModel,
    domain_by_name: &std::collections::HashMap<&str, &MappedEntity>,
    catalog: Option<&PlatformCatalog>,
    known_entities: &[String],
) -> (Vec<AlignmentNode>, Vec<AlignmentGap>) {
    let mut nodes = Vec::new();
    let mut gaps = Vec::new();
    for domain in &model.domains {
        if let Some(m) = domain_by_name.get(domain.id.as_str()) {
            nodes.push(build_node_from_mapping(domain, m));
        } else if let Some(node) = resolve_covered_binding(domain, catalog, known_entities) {
            // SESSION-FIX:gap-b-covered-binding
            nodes.push(node);
        } else {
            gaps.push(AlignmentGap {
                biz_element: domain.id.clone(),
                description: "未在 OntologyTransfer 中找到 alioth 表匹配，且未被已知模块覆盖"
                    .into(),
                suggested_alioth_entities: vec![],
            });
        }
    }
    (nodes, gaps)
}

/// SESSION-FIX:gap-b-covered-binding — 从 PlatformCatalog ∩ known_entities 解析覆盖域绑定。
/// 规则：col.table_name ∈ known_entities（DB 真表验证）且 domain.id == col.name 或 strip 前缀后的 table_name。
fn resolve_covered_binding(
    domain: &alioth_gen::generator::ir::ontology::DomainOntology,
    catalog: Option<&PlatformCatalog>,
    known_entities: &[String],
) -> Option<AlignmentNode> {
    let catalog = catalog?;
    for col in &catalog.collections {
        if !known_entities.contains(&col.table_name) {
            continue;
        }
        let stripped = col
            .table_name
            .strip_prefix("zc_id_")
            .or_else(|| col.table_name.strip_prefix("zc_ad_"))
            .unwrap_or(&col.table_name);
        if domain.id == col.name || domain.id == stripped {
            let table = if col.table_name.starts_with("isahl.") {
                col.table_name.clone()
            } else {
                format!("isahl.{}", col.table_name)
            };
            return Some(AlignmentNode {
                biz_domain: domain.id.clone(),
                biz_kind: format!("{:?}", domain.kind),
                alioth_entities: vec![AliothBinding {
                    table,
                    role: primary_role(&domain.kind),
                    coordinates: None,
                    field_mappings: vec![],
                    constraints: vec![],
                }],
                evidence: format!(
                    "covered-binding: catalog[{}] ∩ known_entities",
                    col.table_name
                ),
                confidence: 0.7,
            });
        }
    }
    None
}

fn record_missing_endpoint_gap(
    gaps: &mut Vec<AlignmentGap>,
    rel: &alioth_gen::generator::ir::ontology::RelationOntology,
    source_mapped: bool,
    target_mapped: bool,
) {
    let missing = format!(
        "{}{}",
        if !source_mapped {
            format!("source[{}] ", rel.source_ontology)
        } else {
            String::new()
        },
        if !target_mapped {
            format!("target[{}]", rel.target_ontology)
        } else {
            String::new()
        }
    );
    gaps.push(AlignmentGap {
        biz_element: format!(
            "relation {} ({} → {})",
            rel.id, rel.source_ontology, rel.target_ontology
        ),
        description: format!("端点未映射: {missing}"),
        suggested_alioth_entities: vec![],
    });
}

async fn detect_relationship_mechanism(
    pool: &PgPool,
    source_table: &str,
    target_table: &str,
    _rel_type: &RelationType,
) -> Option<(AliothRelationMechanism, DbRelationEvidence)> {
    let clean_source = source_table.strip_prefix("isahl.").unwrap_or(source_table);
    let clean_target = target_table.strip_prefix("isahl.").unwrap_or(target_table);

    let fk_sql = "SELECT column_name FROM information_schema.columns WHERE table_schema = 'isahl' AND table_name = $1 AND ordinal_position <= 60 AND column_name LIKE 'fk_%'";
    let fk_rows: Vec<(String,)> = sqlx::query_as(fk_sql)
        .bind(clean_source)
        .fetch_all(pool)
        .await
        .unwrap_or_default();
    for (col,) in &fk_rows {
        let suffix = col.strip_prefix("fk_").unwrap_or("");
        if clean_target.contains(suffix) {
            return Some((
                AliothRelationMechanism::FK {
                    column: col.clone(),
                    target_table: target_table.to_string(),
                },
                DbRelationEvidence {
                    query_kind: "fk_column".into(),
                    schema: "isahl".into(),
                    relation_table: clean_source.to_string(),
                    relation_column: Some(col.clone()),
                    target_table: clean_target.to_string(),
                },
            ));
        }
    }

    // SESSION-FIX:gap-c-rr-detection — rr_* 实体-实体关系表（111 张真实表）。
    // 命名约定：zc_id_<left>_rr_<right>（DB 抽查验证：87/111 left 精确、111/111 right 模糊命中）。
    // `_r_` 表（36 张）是实体→标量属性关系，不是 domain-domain 证据，不再用于此。
    let rr_sql = "SELECT table_name FROM isahl_meta.meta_collections WHERE table_name LIKE '%\\_rr\\_%' ESCAPE '\\'";
    let rr_rows: Vec<(String,)> = sqlx::query_as(rr_sql)
        .fetch_all(pool)
        .await
        .unwrap_or_default();
    for (rr_table,) in &rr_rows {
        // source 锚定：<source>_rr_ 必须出现在表名中
        if !rr_table.contains(&format!("{}_rr_", clean_source)) {
            continue;
        }
        // target 匹配：_rr_ 后缀与 clean_target 互相包含（处理 subj-employee ↔ employee 类前缀差异）
        let suffix = rr_table.split("_rr_").nth(1).unwrap_or("");
        let target_match = clean_target == suffix
            || clean_target.ends_with(&format!("-{}", suffix))
            || suffix.ends_with(clean_target)
            || clean_target.contains(suffix)
            || suffix.contains(clean_target);
        if target_match {
            return Some((
                AliothRelationMechanism::RRTable {
                    table: rr_table.clone(),
                },
                DbRelationEvidence {
                    query_kind: "rr_table".into(),
                    schema: "isahl_meta".into(),
                    relation_table: rr_table.clone(),
                    relation_column: None,
                    target_table: clean_target.to_string(),
                },
            ));
        }
    }
    None
}

// SESSION-FIX:gap-e-coordinate-confirmation — 层2 坐标确认纯 helper。
/// 为缺 scene/factor 坐标的 mapped 实体生成确认问题（选项来自 catalog 真实维度）。
pub fn build_coordinate_questions(
    mapped: &[MappedEntity],
    catalog: &PlatformCatalog,
) -> Vec<crate::state::Question> {
    let mut questions = Vec::new();
    for m in mapped {
        if m.scene_code.is_none() {
            questions.push(crate::state::Question {
                id: format!("coord_scene_{}", m.domain_id),
                category: crate::state::MissingInfoCategory::SceneAmbiguity,
                question: format!(
                    "实体 `{}`（表 {}）需要 scene 坐标。请从以下维度选择：",
                    m.domain_id, m.table
                ),
                options: catalog
                    .scenes
                    .iter()
                    .map(|s| format!("{} ({})", s.code, s.notice))
                    .collect(),
                required: false,
            });
        }
        if m.factor_code.is_none() {
            questions.push(crate::state::Question {
                id: format!("coord_factor_{}", m.domain_id),
                category: crate::state::MissingInfoCategory::SceneAmbiguity,
                question: format!(
                    "实体 `{}`（表 {}）需要 factor 坐标。请从以下维度选择：",
                    m.domain_id, m.table
                ),
                options: catalog
                    .factors
                    .iter()
                    .map(|f| format!("{} ({})", f.code, f.notice))
                    .collect(),
                required: false,
            });
        }
    }
    questions
}

/// 把 `coord_scene_<domain>` / `coord_factor_<domain>` 答案确定性应用到 mapped 实体。
/// 答案必须匹配 catalog 中真实维度 code（或选项前缀"code (notice)"形式），否则忽略。
/// 返回成功应用的坐标数。
pub fn apply_coordinate_answers(
    mapped: &mut [MappedEntity],
    answers: &[crate::state::UserAnswer],
    catalog: &PlatformCatalog,
) -> usize {
    let mut applied = 0;
    for ans in answers {
        let (kind, domain_id) = if let Some(d) = ans.question_id.strip_prefix("coord_scene_") {
            ("scene", d)
        } else if let Some(d) = ans.question_id.strip_prefix("coord_factor_") {
            ("factor", d)
        } else {
            continue;
        };
        // 答案可能是裸 code 或 "code (notice)" 选项形式——取空格前第一段
        let code = ans.answer.split(' ').next().unwrap_or("").trim();
        if code.is_empty() {
            continue;
        }
        let Some(entity) = mapped.iter_mut().find(|m| m.domain_id == domain_id) else {
            continue;
        };
        match kind {
            "scene" => {
                if catalog.scenes.iter().any(|s| s.code == code) {
                    entity.scene_code = Some(code.to_string());
                    applied += 1;
                }
            }
            "factor" if catalog.factors.iter().any(|f| f.code == code) => {
                entity.factor_code = Some(code.to_string());
                applied += 1;
            }
            _ => {}
        }
    }
    applied
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primary_role() {
        assert_eq!(primary_role(&DomainKind::AggregateRoot), "aggregate_root");
        assert_eq!(primary_role(&DomainKind::Entity), "entity");
        assert_eq!(primary_role(&DomainKind::ValueObject), "value_object");
        assert_eq!(primary_role(&DomainKind::DomainService), "entity");
    }

    #[test]
    fn test_build_nodes_and_gaps_no_db() {
        use alioth_gen::generator::ir::ontology::{DomainOntology, RelationOntology};
        let model = OntologyModel {
            domains: vec![
                DomainOntology {
                    id: "mapped_domain".into(),
                    kind: DomainKind::Entity,
                    ..Default::default()
                },
                DomainOntology {
                    id: "unmapped_domain".into(),
                    kind: DomainKind::AggregateRoot,
                    ..Default::default()
                },
            ],
            relations: vec![RelationOntology {
                id: "rel1".into(),
                source_ontology: "mapped_domain".into(),
                target_ontology: "unmapped_domain".into(),
                relation_type: RelationType::Composition,
                ..Default::default()
            }],
            ..Default::default()
        };
        let mapped = vec![MappedEntity {
            domain_id: "mapped_domain".into(),
            table: "isahl.zc_id_test".into(),
            score: 0.8,
            name_score: 0.6,
            field_score: 0.4,
            scene_code: None,
            factor_code: None,
            function_code: Some("↓_GG".into()),
            function_confidence: 0.75,
            field_mappings: vec![],
        }];
        let by_name: std::collections::HashMap<&str, &MappedEntity> =
            mapped.iter().map(|m| (m.domain_id.as_str(), m)).collect();

        let (nodes, gaps) = build_domain_nodes_and_gaps(&model, &by_name, None, &[]);
        assert_eq!(nodes.len(), 1);
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].biz_element, "unmapped_domain");

        let mut rel_gaps: Vec<AlignmentGap> = Vec::new();
        for rel in &model.relations {
            let sm = by_name.contains_key(rel.source_ontology.as_str());
            let tm = by_name.contains_key(rel.target_ontology.as_str());
            if !sm || !tm {
                record_missing_endpoint_gap(&mut rel_gaps, rel, sm, tm);
            }
        }
        assert_eq!(rel_gaps.len(), 1);
        assert!(rel_gaps[0].biz_element.contains("rel1"));
    }

    // SESSION-FIX:gap-b-resolver-tests
    #[test]
    fn test_resolve_covered_binding_match() {
        use alioth_gen::generator::ir::ontology::DomainOntology;
        let domain = DomainOntology {
            id: "inventory".into(),
            kind: DomainKind::AggregateRoot,
            ..Default::default()
        };
        let catalog = PlatformCatalog {
            collections: vec![CollectionInfo {
                id: 1,
                name: "inventory".into(),
                table_name: "zc_id_inventory".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let known = vec!["zc_id_inventory".to_string()];
        let node = resolve_covered_binding(&domain, Some(&catalog), &known);
        assert!(node.is_some());
        let node = node.unwrap();
        assert_eq!(node.biz_domain, "inventory");
        assert_eq!(node.alioth_entities[0].table, "isahl.zc_id_inventory");
    }

    #[test]
    fn test_resolve_covered_binding_table_not_known_returns_none() {
        use alioth_gen::generator::ir::ontology::DomainOntology;
        let domain = DomainOntology {
            id: "inventory".into(),
            kind: DomainKind::Entity,
            ..Default::default()
        };
        let catalog = PlatformCatalog {
            collections: vec![CollectionInfo {
                id: 1,
                name: "inventory".into(),
                table_name: "zc_id_inventory".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let known: Vec<String> = vec![];
        assert!(resolve_covered_binding(&domain, Some(&catalog), &known).is_none());
    }

    #[test]
    fn test_resolve_covered_binding_name_mismatch_returns_none() {
        use alioth_gen::generator::ir::ontology::DomainOntology;
        let domain = DomainOntology {
            id: "warehouse".into(),
            kind: DomainKind::Entity,
            ..Default::default()
        };
        let catalog = PlatformCatalog {
            collections: vec![CollectionInfo {
                id: 1,
                name: "inventory".into(),
                table_name: "zc_id_inventory".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let known = vec!["zc_id_inventory".to_string()];
        assert!(resolve_covered_binding(&domain, Some(&catalog), &known).is_none());
    }
    // SESSION-FIX:gap-e-coordinate-tests
    fn coord_test_catalog() -> PlatformCatalog {
        PlatformCatalog {
            scenes: vec![SceneInfo {
                id: 1,
                code: "SC-LOG".into(),
                notice: "物流场景".into(),
            }],
            factors: vec![FactorInfo {
                id: 1,
                code: "FA-WH".into(),
                notice: "仓储要素".into(),
            }],
            ..Default::default()
        }
    }

    fn coord_test_entity(domain: &str) -> MappedEntity {
        MappedEntity {
            domain_id: domain.into(),
            table: "isahl.zc_id_x".into(),
            score: 0.8,
            name_score: 0.6,
            field_score: 0.4,
            scene_code: None,
            factor_code: None,
            function_code: None,
            function_confidence: 0.0,
            field_mappings: vec![],
        }
    }

    #[test]
    fn test_build_coordinate_questions_for_missing() {
        let catalog = coord_test_catalog();
        let mapped = vec![coord_test_entity("dom1")];
        let qs = build_coordinate_questions(&mapped, &catalog);
        assert_eq!(qs.len(), 2);
        assert_eq!(qs[0].id, "coord_scene_dom1");
        assert_eq!(qs[1].id, "coord_factor_dom1");
        assert!(qs[0].options.iter().any(|o| o.contains("SC-LOG")));
        assert!(qs[1].options.iter().any(|o| o.contains("FA-WH")));
    }

    #[test]
    fn test_apply_coordinate_answers_validates_against_catalog() {
        let catalog = coord_test_catalog();
        let mut mapped = vec![coord_test_entity("dom1")];
        let answers = vec![
            crate::state::UserAnswer {
                question_id: "coord_scene_dom1".into(),
                answer: "SC-LOG (物流场景)".into(),
                answered_at: chrono::Utc::now(),
            },
            crate::state::UserAnswer {
                question_id: "coord_factor_dom1".into(),
                answer: "FA-NOTEXIST".into(),
                answered_at: chrono::Utc::now(),
            },
        ];
        let applied = apply_coordinate_answers(&mut mapped, &answers, &catalog);
        assert_eq!(applied, 1);
        assert_eq!(mapped[0].scene_code, Some("SC-LOG".into()));
        assert_eq!(mapped[0].factor_code, None); // 非法 code 被拒绝
    }

    #[test]
    fn test_apply_coordinate_answers_ignores_unrelated() {
        let catalog = coord_test_catalog();
        let mut mapped = vec![coord_test_entity("dom1")];
        let answers = vec![crate::state::UserAnswer {
            question_id: "other_question".into(),
            answer: "SC-LOG".into(),
            answered_at: chrono::Utc::now(),
        }];
        assert_eq!(apply_coordinate_answers(&mut mapped, &answers, &catalog), 0);
        assert_eq!(mapped[0].scene_code, None);
    }
}
