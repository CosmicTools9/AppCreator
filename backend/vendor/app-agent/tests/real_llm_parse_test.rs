use app_agent::planner::parse_and_validate;
use app_agent::tools::fetch_platform_catalog;

async fn connect_test_db() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost:5432/aliothstudio_test".to_string());
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("connect_test_db failed")
}

#[tokio::test]
async fn test_real_llm_response_parsing() {
    let pool = connect_test_db().await;
    let catalog = fetch_platform_catalog(&pool, "Alioth").await;

    // 注：原响应使用模块 "inventory"，但该模块在 meta_collections.modules 移除后
    // 不再出现在 PlatformCatalog 中；改为已存在的 "system-settings" 以保持测试可用。
    let response = r#"{"ontology":{"id":"warehouse-inventory","name":"仓库库存管理","version":"1.0","domains":[],"relations":[],"constraints":[{"id":"inv-quantity-non-negative","name":"库存数量非负","constraint_type":"Structural","scope":{"target_ontology":"zc_id_inve-materials"},"expression":"quantity >= 0","severity":"Error"}],"computations":[{"id":"compute-stock-on-hand","name":"计算在手库存","computation_type":"Derivation","inputs":["zc_id_inve-materials.inbound_qty","zc_id_inve-materials.outbound_qty"],"outputs":["zc_id_inve-materials.on_hand_qty"],"formula":"on_hand_qty = inbound_qty - outbound_qty","trigger_conditions":["OnUpdate"]}],"transaction_lifecycle":null},"used_modules":["system-settings"],"known_entities":["zc_id_inve-materials","zc_id_stor-plc-warehouse"],"missing_info":[],"workflow_steps":[]}"#;

    let result = parse_and_validate(response, &catalog);
    println!("fix_log: {:?}", result.fix_log);
    println!("missing_info: {:?}", result.missing_info);
    println!("warnings: {:?}", result.warnings);
    println!("used_modules: {:?}", result.used_modules);
    println!("ontology.domains.len: {}", result.ontology.domains.len());
    assert!(
        result.missing_info.is_empty(),
        "Should have no missing_info, got: {:?}",
        result.missing_info
    );
}
