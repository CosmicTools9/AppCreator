use app_agent::tools::fetch_platform_catalog;

async fn connect_test_db() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost:5432/aliothstudio_test".to_string());
    sqlx::PgPool::connect(&database_url).await.expect("connect_test_db failed")
}



#[tokio::test]
async fn test_catalog_has_modules() {
    let pool = connect_test_db().await;
    let catalog = fetch_platform_catalog(&pool, "Alioth").await;
    println!("Modules count: {}", catalog.modules.len());
    println!("Collections count: {}", catalog.collections.len());
    println!("Scenes count: {}", catalog.scenes.len());
    println!("Factors count: {}", catalog.factors.len());
    println!("Functions count: {}", catalog.functions.len());
    for m in &catalog.modules {
        println!("  module: id={}", m.id);
    }
    assert!(!catalog.modules.is_empty(), "Catalog should have modules");
    assert!(
        !catalog.collections.is_empty(),
        "Catalog should have global collections"
    );
}
