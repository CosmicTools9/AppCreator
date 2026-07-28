//! Chat session repository + AppAgent lifecycle integration tests.
//!
//! Tests run against `aliothstudio_test` (or `DATABASE_URL`) using `#[tokio::test]`
//! per project rules. They exercise the chat repository functions and a single
//! AppAgent step with a mock LLM service.

use app_agent::mocks::MockLlmService;
use app_agent::{AppAgent, ConversationContext};
use app_creator::chat;
use common::testing::{connect_test_db, setup_test_schema_light};


/// Connect to test DB and verify we're on a `*_test` database.
async fn setup() -> sqlx::PgPool {
    let pool = connect_test_db().await;
    setup_test_schema_light(&pool)
        .await
        .expect("Tests must run against a *_test database (set DATABASE_URL accordingly)");
    pool
}
#[tokio::test]
async fn chat_session_create_get_roundtrip() {
    let pool = setup().await;

    let row = chat::create_session(&pool, "Integration test session", None, "Alioth")
        .await
        .expect("create_session failed");

    assert_eq!(row.title, "Integration test session");
    assert_eq!(row.namespace, "Alioth");
    assert_eq!(row.status, "active");

    let fetched = chat::get_session(&pool, row.id)
        .await
        .expect("get_session failed")
        .expect("session not found");
    assert_eq!(fetched.id, row.id);
}

#[tokio::test]
async fn chat_message_add_and_list_roundtrip() {
    let pool = setup().await;

    let session = chat::create_session(&pool, "Message roundtrip", None, "Alioth")
        .await
        .expect("create_session failed");

    let msg = chat::add_message(&pool, session.id, "user", "Hello, agent!")
        .await
        .expect("add_message failed");
    assert_eq!(msg.role, "user");
    assert_eq!(msg.content, "Hello, agent!");

    let msgs = chat::list_messages(&pool, session.id)
        .await
        .expect("list_messages failed");
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].content, "Hello, agent!");
}

#[tokio::test]
async fn agent_context_save_and_load_roundtrip() {
    let pool = setup().await;

    let session = chat::create_session(&pool, "Context roundtrip", None, "Alioth")
        .await
        .expect("create_session failed");

    let mut ctx = ConversationContext::new(session.id, "tester".into(), "Alioth".into());
    ctx.user_description = "I need a warehouse management app".into();

    chat::save_agent_context(&pool, session.id, &ctx)
        .await
        .expect("save_agent_context failed");

    let loaded = chat::load_agent_context(&pool, session.id)
        .await
        .expect("load_agent_context failed")
        .expect("context missing");

    assert_eq!(loaded.user_description, ctx.user_description);
    assert_eq!(loaded.namespace, Some("Alioth".to_string()));
}

#[tokio::test]
async fn list_sessions_filters_by_namespace_and_orders_desc() {
    let pool = setup().await;

    let uniq = chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default();
    let ns = format!("ListTest-{uniq}");
    let other_ns = format!("ListTestOther-{uniq}");
    let s1 = chat::create_session(&pool, "first", None, &ns)
        .await
        .expect("create s1 failed");
    let s2 = chat::create_session(&pool, "second", None, &ns)
        .await
        .expect("create s2 failed");
    let _other = chat::create_session(&pool, "other", None, &other_ns)
        .await
        .expect("create other failed");

    let rows = chat::list_sessions(&pool, Some(&ns), 20)
        .await
        .expect("list_sessions failed");

    assert!(rows.len() >= 2, "expected at least 2 sessions in namespace");
    let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    assert!(ids.contains(&s1.id) && ids.contains(&s2.id));
    // 全部结果都属于目标 namespace
    assert!(rows.iter().all(|r| r.namespace == ns));
    // updated_at DESC：相邻项单调不增
    for w in rows.windows(2) {
        assert!(
            w[0].updated_at >= w[1].updated_at,
            "must be ordered by updated_at DESC"
        );
    }

    // limit 生效
    let limited = chat::list_sessions(&pool, Some(&ns), 1)
        .await
        .expect("list_sessions with limit failed");
    assert_eq!(limited.len(), 1);
}

#[tokio::test]
async fn app_agent_single_step_with_mock_llm() {
    let pool = setup().await;

    let session = chat::create_session(&pool, "Agent step", None, "Alioth")
        .await
        .expect("create_session failed");

    let mut ctx = ConversationContext::new(session.id, "tester".into(), "Alioth".into());
    ctx.user_description = "warehouse management".into();

    let mock_llm = MockLlmService::from_handler(|_prompt| {
        app_agent::mocks::MockResponse::RouterJson(
            r#"{"task_complexity":"low","needs_reasoning":false}"#.to_string(),
        )
    });
    let agent = AppAgent::new(std::sync::Arc::new(pool), Box::new(mock_llm));

    let result = agent
        .run_single_step(&mut ctx, None::<&fn(app_agent::AgentProgress)>)
        .await;

    // The mock LLM returns a default response; the step should transition the state machine.
    assert!(
        result.is_ok(),
        "run_single_step should succeed with mock LLM: {:?}",
        result.err()
    );
}
#[tokio::test]
async fn claim_session_sequential() {
    let pool = setup().await;
    chat::ensure_app_creator_tables(&pool).await.unwrap();
    chat::ensure_chat_session_status_values(&pool).await.unwrap();
    let session = chat::create_session(&pool, "Claim test", None, "Alioth")
        .await.expect("create_session failed");
    assert!(chat::claim_session_for_generation(&pool, session.id).await.unwrap());
    assert!(!chat::claim_session_for_generation(&pool, session.id).await.unwrap());
    chat::update_session_status(&pool, session.id, "active").await.unwrap();
}

#[tokio::test]
async fn owner_check_denies_ownerless() {
    let pool = setup().await;
    chat::ensure_app_creator_tables(&pool).await.unwrap();
    let session = chat::create_session(&pool, "Owner test", None, "Alioth")
        .await.expect("create_session failed");
    assert!(chat::check_session_owner(&pool, session.id, 1).await.is_some());
}

#[tokio::test]
async fn owner_check_allows_exact() {
    let pool = setup().await;
    chat::ensure_app_creator_tables(&pool).await.unwrap();
    let session = chat::create_session_with_owner(&pool, "Owner test", None, "Alioth", 42)
        .await.expect("create_session_with_owner failed");
    assert!(chat::check_session_owner(&pool, session.id, 42).await.is_none());
    assert!(chat::check_session_owner(&pool, session.id, 1).await.is_some());
}
