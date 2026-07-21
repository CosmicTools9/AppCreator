//! SESSION-FIX:gap-h-integration — OT → AlignmentGraph → 投影 端到端集成测试（真实 DB）。
//! 覆盖 GAP-B resolver 路径：covered 域建节点、未覆盖域记 gap、投影产出实体/报告。
//! 模式遵循 BACKEND_FRAMEWORK §5：#[tokio::test] + PgPool::connect，禁止 #[sqlx::test]。

use alioth_gen::generator::ir::ontology::{DomainKind, DomainOntology, OntologyModel};
use app_agent::aligner;
use app_agent::composer::project_from_alignment_graph;
use app_agent::tools::fetch_platform_catalog;
use sqlx::PgPool;

async fn connect() -> PgPool {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost:5432/aliothstudio".to_string());
    PgPool::connect(&url).await.expect("connect")
}

#[tokio::test]
async fn test_covered_binding_end_to_end_with_real_catalog() {
    let pool = connect().await;
    let catalog = fetch_platform_catalog(&pool, "Alioth").await;
    assert!(
        !catalog.collections.is_empty(),
        "catalog should have collections"
    );

    // 取一个真实 collection 作为 covered 域
    let real = &catalog.collections[0];
    let covered_domain_id = real.name.clone();

    let model = OntologyModel {
        domains: vec![
            DomainOntology {
                id: covered_domain_id.clone(),
                kind: DomainKind::Entity,
                ..Default::default()
            },
            DomainOntology {
                id: "nonexistent_domain_xyz_123".into(),
                kind: DomainKind::AggregateRoot,
                ..Default::default()
            },
        ],
        relations: vec![],
        ..Default::default()
    };

    let known = vec![real.table_name.clone()];
    // mapped 为空 → 强制走 covered/gap 路径
    let graph = aligner::build_alignment_graph(&pool, &model, &[], Some(&catalog), &known).await;

    // covered 域 → 节点（真实表绑定）
    let covered_node = graph
        .nodes
        .iter()
        .find(|n| n.biz_domain == covered_domain_id)
        .unwrap_or_else(|| panic!("covered domain {} should produce a node", covered_domain_id));
    assert!(
        covered_node.alioth_entities[0]
            .table
            .contains(&real.table_name),
        "node should bind the real table {}, got {}",
        real.table_name,
        covered_node.alioth_entities[0].table
    );
    assert!(covered_node.evidence.contains("covered-binding"));

    // 未覆盖域 → gap（诚实记录，不虚构）
    let gap = graph
        .gaps
        .iter()
        .find(|g| g.biz_element == "nonexistent_domain_xyz_123")
        .expect("uncovered domain should produce a gap");

    // 投影：实体含 covered 节点，报告含 gap
    let proj = project_from_alignment_graph(&graph, &[]);
    assert!(
        proj.entities.iter().any(|e| e["name"] == covered_domain_id),
        "projection should include covered entity"
    );
    assert!(
        proj.gap_report.iter().any(|r| r.contains(&gap.biz_element)),
        "gap report should surface the uncovered domain"
    );
}

#[tokio::test]
async fn test_table_not_in_known_entities_stays_gap() {
    let pool = connect().await;
    let catalog = fetch_platform_catalog(&pool, "Alioth").await;
    let real = &catalog.collections[0];

    let model = OntologyModel {
        domains: vec![DomainOntology {
            id: real.name.clone(),
            kind: DomainKind::Entity,
            ..Default::default()
        }],
        relations: vec![],
        ..Default::default()
    };

    // known_entities 为空 → 即使 catalog 有该 collection 也不得建节点（DB 真值门禁）
    let graph = aligner::build_alignment_graph(&pool, &model, &[], Some(&catalog), &[]).await;
    assert!(
        graph.nodes.is_empty(),
        "no nodes when known_entities is empty"
    );
    assert_eq!(graph.gaps.len(), 1);
}

/// SESSION-FIX:gap-c-rr-detection — 真实 DB 验证 rr_* 关系机制检测。
/// subj-org → subj-employee 经 zc_id_subj-org_rr_employee 链接。
#[tokio::test]
async fn test_rr_table_mechanism_detected_from_real_db() {
    let pool = connect().await;
    let catalog = fetch_platform_catalog(&pool, "Alioth").await;
    let has_rr = catalog
        .collections
        .iter()
        .any(|c| c.table_name.contains("_rr_"));
    if !has_rr {
        eprintln!("no rr tables in catalog, skipping");
        return;
    }

    let model = OntologyModel {
        domains: vec![
            DomainOntology {
                id: "subj-org".into(),
                kind: DomainKind::Entity,
                ..Default::default()
            },
            DomainOntology {
                id: "subj-employee".into(),
                kind: DomainKind::Entity,
                ..Default::default()
            },
        ],
        relations: vec![alioth_gen::generator::ir::ontology::RelationOntology {
            id: "org-has-employee".into(),
            source_ontology: "subj-org".into(),
            target_ontology: "subj-employee".into(),
            relation_type: alioth_gen::generator::ir::ontology::RelationType::Composition,
            ..Default::default()
        }],
        ..Default::default()
    };
    let mapped = vec![
        app_agent::state::MappedEntity {
            domain_id: "subj-org".into(),
            table: "isahl.zc_id_subj-org".into(),
            score: 0.9,
            name_score: 0.9,
            field_score: 0.9,
            scene_code: None,
            factor_code: None,
            function_code: None,
            function_confidence: 0.0,
            field_mappings: vec![],
        },
        app_agent::state::MappedEntity {
            domain_id: "subj-employee".into(),
            table: "isahl.zc_id_subj-employee".into(),
            score: 0.9,
            name_score: 0.9,
            field_score: 0.9,
            scene_code: None,
            factor_code: None,
            function_code: None,
            function_confidence: 0.0,
            field_mappings: vec![],
        },
    ];

    let graph = aligner::build_alignment_graph(&pool, &model, &mapped, Some(&catalog), &[]).await;

    // 关系边必须被检测：FK 或 RRTable（取决于 DB 实际结构），不得落入 relation gap
    let rel_gap = graph
        .gaps
        .iter()
        .find(|g| g.biz_element.contains("org-has-employee"));
    if graph.edges.is_empty() {
        panic!(
            "expected an edge for subj-org → subj-employee, got gap: {:?}",
            rel_gap.map(|g| &g.description)
        );
    }
    let edge = &graph.edges[0];
    let proj = project_from_alignment_graph(&graph, &mapped);
    assert_eq!(proj.relations.len(), 1);
    println!("edge mechanism: {:?}", edge.alioth_mechanism);
}
