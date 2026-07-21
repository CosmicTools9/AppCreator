//! Chat session repository + AppAgent lifecycle integration tests.
//!
//! Tests run against `aliothstudio_test` (or `DATABASE_URL`) using `#[tokio::test]`
//! per project rules. They exercise the chat repository functions and a single
//! AppAgent step with a mock LLM service.

use app_agent::mocks::MockLlmService;
use app_agent::{AppAgent, ConversationContext};
use app_creator::chat;
use common::testing::connect_test_db;

#[tokio::test]
async fn chat_session_create_get_roundtrip() {
    let pool = connect_test_db().await;

    let row = chat::create_session(&pool,
        "Integration test session",
        None,
        "Alioth",
    )
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
    let pool = connect_test_db().await;

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
    let pool = connect_test_db().await;

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
async fn app_agent_single_step_with_mock_llm() {
    let pool = connect_test_db().await;

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
        .run_single_step(&mut ctx,
            None::<&fn(app_agent::AgentProgress)>,
        )
        .await;

    // The mock LLM returns a default response; the step should transition the state machine.
    assert!(
        result.is_ok(),
        "run_single_step should succeed with mock LLM: {:?}",
        result.err()
    );
}
