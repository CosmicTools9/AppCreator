use app_agent::planner::PlanningPrompt;

async fn connect_test_db() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost:5432/aliothstudio_test".to_string());
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("connect_test_db failed")
}

#[tokio::test]
async fn test_prompt_length() {
    let pool = connect_test_db().await;

    // 加载 catalog
    let catalog = app_agent::tools::fetch_platform_catalog(&pool, "Alioth").await;

    println!("Catalog modules count: {}", catalog.modules.len());
    println!("Catalog collections count: {}", catalog.collections.len());
    for m in &catalog.modules {
        println!("  Module: {} ({})", m.id, m.name);
    }

    let compiled_modules = std::collections::HashSet::new();
    let prompt = PlanningPrompt::new(
        "I need an inventory management system for a small warehouse",
        &catalog,
        None,
        &[],
        None,
        &compiled_modules,
    );

    println!("\n=== SYSTEM PROMPT LENGTH ===");
    println!("Chars: {}", prompt.system.len());
    println!("Lines: {}", prompt.system.lines().count());

    println!("\n=== USER PROMPT LENGTH ===");
    println!("Chars: {}", prompt.user.len());
    println!("Lines: {}", prompt.user.lines().count());

    // 打印 user prompt 的前 3000 字符（按 char 边界截断，避免切到汉字中间）
    println!("\n=== USER PROMPT PREVIEW ===");
    let preview_end = prompt
        .user
        .char_indices()
        .nth(3000)
        .map(|(i, _)| i)
        .unwrap_or(prompt.user.len());
    println!("{}", &prompt.user[..preview_end]);
}
