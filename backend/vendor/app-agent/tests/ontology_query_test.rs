use app_agent::tools::query_relevant_ontology;

async fn connect_test_db() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost:5432/aliothstudio_test".to_string());
    sqlx::PgPool::connect(&database_url).await.expect("connect_test_db failed")
}



#[tokio::test]
async fn test_query_relevant_ontology_with_inventory_keywords() {
    let pool = connect_test_db().await;

    // 模拟用户输入 "I need an inventory management system for a small warehouse"
    let keywords = vec!["inventory", "management", "system", "warehouse"];
    let result =
        query_relevant_ontology(&pool, &keywords.iter().map(|s| *s).collect::<Vec<_>>(), 50).await;

    assert!(result.error.is_none(), "Query failed: {:?}", result.error);

    let entity_count = result
        .data
        .get("entity_count")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    println!("Entity count: {}", entity_count);
    println!(
        "Edge count: {}",
        result
            .data
            .get("edge_count")
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
    );

    assert!(
        entity_count > 0,
        "Expected non-zero entities for inventory keywords"
    );

    if let Some(entities) = result.data.get("entities").and_then(|v| v.as_array()) {
        println!("Matched entities:");
        for (i, e) in entities.iter().take(10).enumerate() {
            let name = e
                .get("concept_name")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let table = e.get("table_name").and_then(|v| v.as_str()).unwrap_or("?");
            println!("  {}: {} ({})", i + 1, name, table);
        }
    }
}
