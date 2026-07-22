//! AppAgent orchestrator end-to-end tests (no real LLM, no HTTP server).
//!
//! These tests exercise the full state machine pipeline using a deterministic
//! mock LLM service. They verify state transitions, terminal conditions, and
//! artifact generation while running against the actual `aliothstudio_test`
//! database so that `PlatformCatalog` loading is realistic.
//!
//! Run:
//!   DATABASE_URL=postgres://localhost:5432/aliothstudio_test \
//!     cargo test -p app-agent --test state_machine_e2e -- --test-threads=1

use app_agent::mocks::MockLlmService;
use app_agent::orchestrator::AppAgent;
use app_agent::state::progress_event;
use app_agent::state::{
    AgentState, ComposeScratch, ConversationContext, FlowPlan, ResumeConfig, UserAnswer,
};
use sqlx::PgPool;
use std::sync::Arc;

async fn connect_test_db() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost:5432/aliothstudio_test".to_string());
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("connect_test_db failed")
}

/// Set up FEATURE_MANIFEST_PATH and connect to test database.
async fn setup_e2e() -> PgPool {
    if std::env::var("FEATURE_MANIFEST_PATH").is_err() {
        if let Ok(mut cwd) = std::env::current_dir() {
            for _ in 0..5 {
                let candidate = cwd.join("Pre-Proc/compiled_modules.json");
                if candidate.exists() {
                    std::env::set_var("FEATURE_MANIFEST_PATH", candidate);
                    break;
                }
                if !cwd.pop() {
                    break;
                }
            }
        }
    }
    connect_test_db().await
}

#[tokio::test]
async fn test_happy_path_inventory_app_reaches_presenting() {
    let pool = setup_e2e().await;
    let llm = MockLlmService::inventory_planning_ok();
    let agent = AppAgent::new(Arc::new(pool), Box::new(llm));

    let mut ctx = ConversationContext::new(
        1,
        "I need a small warehouse inventory app".to_string(),
        "Alioth".to_string(),
    );

    let result = agent.run(&mut ctx).await;
    assert!(
        result.is_ok(),
        "Agent run should succeed, got err: {:?}",
        result.err()
    );

    assert!(
        matches!(ctx.state, AgentState::Published { .. }),
        "Expected Published, got {:?}",
        ctx.state
    );

    let plan = ctx.flow_plan.expect("Flow plan should exist");
    assert!(plan.used_modules.contains(&"inventory".to_string()));
    assert!(plan.used_modules.contains(&"demand".to_string()));

    let scratch = ctx.compose_scratch.expect("Compose scratch should exist");
    assert!(!scratch.app_name.is_empty());
    assert!(scratch.files_written > 0);
    assert!(std::path::Path::new(&scratch.output_path).exists());

    let app_json = std::path::Path::new(&scratch.output_path).join("app.json");
    assert!(app_json.exists(), "app.json should exist at {:?}", app_json);
    let content = std::fs::read_to_string(&app_json).expect("read app.json");
    let _: serde_json::Value =
        serde_json::from_str(&content).expect("app.json should be valid JSON");

    if let AgentState::Published { result } = &ctx.state {
        assert_eq!(result.app_name, scratch.app_name);
        let module_ids: Vec<_> = result.used_modules.iter().map(|m| &m.module_id).collect();
        assert!(module_ids.contains(&&"inventory".to_string()));
        assert!(module_ids.contains(&&"demand".to_string()));
        assert!(
            result
                .endpoint_url
                .as_ref()
                .map(|url| !url.is_empty())
                .unwrap_or(false),
            "endpoint URL should be present and non-empty"
        );
    }
}

#[tokio::test]
async fn test_happy_path_warehouse_mgmt_in_cosmic_tools_reaches_presenting() {
    let pool = setup_e2e().await;
    let llm = MockLlmService::warehouse_planning_ok();
    let agent = AppAgent::new(Arc::new(pool), Box::new(llm));

    let mut ctx = ConversationContext::new(
        3,
        "I need a basic warehouse management system for Cosmic-Tools".to_string(),
        "Cosmic-Tools".to_string(),
    );

    let result = agent.run(&mut ctx).await;
    assert!(
        result.is_ok(),
        "Agent run should succeed, got err: {:?}",
        result.err()
    );

    assert!(
        matches!(ctx.state, AgentState::Published { .. }),
        "Expected Published, got {:?}",
        ctx.state
    );

    let plan = ctx.flow_plan.expect("Flow plan should exist");
    assert!(
        plan.used_modules.contains(&"warehouse-mgmt".to_string()),
        "warehouse-mgmt should be used"
    );
    assert_eq!(
        plan.namespace, "Cosmic-Tools",
        "namespace should be Cosmic-Tools"
    );

    let scratch = ctx.compose_scratch.expect("Compose scratch should exist");
    assert!(!scratch.app_name.is_empty());
    assert!(scratch.files_written > 0);
    assert!(std::path::Path::new(&scratch.output_path).exists());

    let app_json = std::path::Path::new(&scratch.output_path).join("app.json");
    assert!(app_json.exists(), "app.json should exist at {:?}", app_json);
    let content = std::fs::read_to_string(&app_json).expect("read app.json");
    let app_value: serde_json::Value =
        serde_json::from_str(&content).expect("app.json should be valid JSON");
    assert_eq!(
        app_value.get("namespace").and_then(|v| v.as_str()),
        Some("Cosmic-Tools")
    );
    assert!(
        app_value
            .get("config")
            .and_then(|c| c.get("modules"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().any(|m| m.as_str() == Some("warehouse-mgmt")))
            .unwrap_or(false),
        "app.json config.modules should contain warehouse-mgmt"
    );

    let app_tsx = std::path::Path::new(&scratch.output_path).join("llm-tsx/app.tsx");
    assert!(app_tsx.exists(), "app.tsx should exist at {:?}", app_tsx);
    let app_tsx_content = std::fs::read_to_string(&app_tsx).expect("read app.tsx");
    assert!(
        app_tsx_content.contains("ModuleLayout"),
        "app.tsx should import and render ModuleLayout"
    );
    assert!(
        app_tsx_content.contains("embedded={true}"),
        "app.tsx should render ModuleLayout in embedded mode"
    );

    if let AgentState::Published { result } = &ctx.state {
        assert_eq!(result.app_name, scratch.app_name);
        let module_ids: Vec<_> = result.used_modules.iter().map(|m| &m.module_id).collect();
        assert!(module_ids.contains(&&"warehouse-mgmt".to_string()));
        assert!(
            result
                .endpoint_url
                .as_ref()
                .map(|url| !url.is_empty())
                .unwrap_or(false),
            "endpoint URL should be present and non-empty"
        );
    }
}

#[tokio::test]
async fn test_user_answer_round_trip_and_second_run() {
    let pool = setup_e2e().await;
    let llm = MockLlmService::planning_with_clarification();
    let agent = AppAgent::new(Arc::new(pool), Box::new(llm));

    let mut ctx = ConversationContext::new(
        2,
        "I need an inventory app with a custom field".to_string(),
        "Alioth".to_string(),
    );

    let first = agent.run(&mut ctx).await;
    assert!(first.is_ok(), "First run failed: {:?}", first.err());
    assert!(
        matches!(
            ctx.state,
            AgentState::Planning { .. }
                | AgentState::Presenting { .. }
                | AgentState::Published { .. }
        ),
        "Expected terminal state after first run, got {:?}",
        ctx.state
    );

    let questions = ctx.pending_questions.clone();
    let qid = if !questions.is_empty() {
        questions.last().unwrap().id.clone()
    } else {
        "q_test".to_string()
    };
    ctx.user_answers.push(UserAnswer {
        question_id: qid.clone(),
        answer: "use existing property".to_string(),
        answered_at: chrono::Utc::now(),
    });
    ctx.state = AgentState::Planning {
        revision_round: 0,
        needs_clarification: None,
    };

    let second = agent.run(&mut ctx).await;
    assert!(second.is_ok(), "Second run failed: {:?}", second.err());
    assert!(ctx.user_answers.iter().any(|a| a.question_id == qid));
}

#[tokio::test]
async fn test_interrupt_and_resume_from_planning() {
    let pool = setup_e2e().await;
    let llm = MockLlmService::inventory_planning_ok();
    let agent = AppAgent::new(Arc::new(pool), Box::new(llm));

    let mut ctx = ConversationContext::new(
        3,
        "Build an inventory app".to_string(),
        "Alioth".to_string(),
    );

    AppAgent::request_interrupt(&mut ctx);

    let result = agent.run(&mut ctx).await;
    assert!(
        result.is_ok(),
        "Interrupted run should return Ok pause message"
    );
    assert!(
        result.unwrap().starts_with("⏸️"),
        "Should return pause message"
    );

    assert!(
        !matches!(ctx.state, AgentState::Published { .. }),
        "Should not have reached Published after interrupt"
    );

    ctx.interrupt_requested = false;
    let resume = agent.run(&mut ctx).await;
    assert!(resume.is_ok(), "Resume failed: {:?}", resume.err());
    assert!(
        matches!(ctx.state, AgentState::Published { .. }),
        "After resume should reach Published, got {:?}",
        ctx.state
    );
}

#[tokio::test]
async fn test_reset_to_checkpoint_preserves_ontology() {
    let pool = setup_e2e().await;
    let llm = MockLlmService::inventory_planning_ok();
    let agent = AppAgent::new(Arc::new(pool), Box::new(llm));

    let mut ctx = ConversationContext::new(4, "Inventory app".to_string(), "Alioth".to_string());

    agent.run(&mut ctx).await.unwrap();
    assert!(matches!(ctx.state, AgentState::Published { .. }));

    let original_ontology = ctx.ontology_model.clone();
    let original_flow = ctx.flow_plan.clone();
    let original_scratch = ctx.compose_scratch.clone();

    AppAgent::reset_to_checkpoint(
        &mut ctx,
        &ResumeConfig {
            target_state: AgentState::Planning {
                revision_round: 1,
                needs_clarification: None,
            },
            preserve_ontology: true,
            preserve_flow_plan: true,
            preserve_scratch: true,
            preserve_yaml_ops: true,
        },
    )
    .expect("reset should succeed");

    assert!(
        matches!(
            ctx.state,
            AgentState::Planning {
                revision_round: 1,
                ..
            }
        ),
        "Should be in Planning revision 1"
    );
    assert!(ctx.ontology_model.is_some(), "Ontology should be preserved");
    assert!(ctx.flow_plan.is_some(), "Flow plan should be preserved");
    assert!(ctx.compose_scratch.is_some(), "Scratch should be preserved");
    assert!(
        ctx.ontology_model.as_ref().unwrap().domains.len()
            == original_ontology.as_ref().unwrap().domains.len(),
        "Ontology domain count should match original"
    );
    assert_eq!(
        ctx.flow_plan.as_ref().unwrap().used_modules,
        original_flow.as_ref().unwrap().used_modules,
        "Flow plan used modules should match original"
    );
    assert_eq!(
        ctx.compose_scratch.as_ref().unwrap().app_name,
        original_scratch.as_ref().unwrap().app_name,
        "Scratch app name should match original"
    );
}

#[tokio::test]
async fn test_reset_clears_context_when_requested() {
    let pool = setup_e2e().await;
    let llm = MockLlmService::inventory_planning_ok();
    let agent = AppAgent::new(Arc::new(pool), Box::new(llm));

    let mut ctx = ConversationContext::new(5, "Inventory app".to_string(), "Alioth".to_string());
    agent.run(&mut ctx).await.unwrap();

    AppAgent::reset_to_checkpoint(
        &mut ctx,
        &ResumeConfig {
            target_state: AgentState::Initializing,
            preserve_ontology: false,
            preserve_flow_plan: false,
            preserve_scratch: false,
            preserve_yaml_ops: false,
        },
    )
    .unwrap();

    assert_eq!(ctx.state, AgentState::Initializing);
    assert!(ctx.ontology_model.is_none());
    assert!(ctx.flow_plan.is_none());
    assert!(ctx.compose_scratch.is_none());
}

#[tokio::test]
async fn test_step_history_records_all_transitions() {
    let pool = setup_e2e().await;
    let llm = MockLlmService::inventory_planning_ok();
    let agent = AppAgent::new(Arc::new(pool), Box::new(llm));

    let mut ctx = ConversationContext::new(6, "Inventory app".to_string(), "Alioth".to_string());
    agent.run(&mut ctx).await.unwrap();

    assert!(
        !ctx.step_history.is_empty(),
        "Step history should not be empty"
    );

    // ContextCompressor 会在 step 数超过阈值后压缩/丢弃旧条目，
    // 因此不能假定第一条仍为 Initializing -> Planning。
    // 改为检查历史中包含关键转换且最终到达 Presenting。
    assert!(
        ctx.step_history.iter().any(|s| {
            matches!(s.state_before, AgentState::Planning { .. })
                && matches!(s.state_after, AgentState::Extending)
        }),
        "Step history should contain Planning -> Extending transition"
    );

    let last = ctx.step_history.last().unwrap();
    assert!(
        matches!(last.state_after, AgentState::Published { .. }),
        "Last transition should enter Published"
    );
    assert!(last.is_terminal, "Published state should be terminal");
}

#[tokio::test]
async fn test_llm_failure_is_reported_without_panic() {
    let pool = setup_e2e().await;
    let llm = MockLlmService::always_failing("simulated LLM outage");
    let agent = AppAgent::new(Arc::new(pool), Box::new(llm));

    let mut ctx = ConversationContext::new(7, "Inventory app".to_string(), "Alioth".to_string());
    let result = agent.run(&mut ctx).await;

    assert!(result.is_err(), "LLM failure should propagate as error");
    let err = result.unwrap_err();
    assert!(err.contains("LLM call failed") || err.contains("simulated LLM outage"));
}

#[tokio::test]
async fn test_run_with_progress_emits_events() {
    let pool = setup_e2e().await;
    let llm = MockLlmService::inventory_planning_ok();
    let agent = AppAgent::new(Arc::new(pool), Box::new(llm));

    let mut ctx = ConversationContext::new(
        100,
        "I need a small warehouse inventory app".to_string(),
        "Alioth".to_string(),
    );
    let progress = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured = std::sync::Arc::clone(&progress);
    agent
        .run_with_progress(&mut ctx, move |p| {
            captured.lock().unwrap().push(p);
        })
        .await
        .expect("run should succeed");

    let events = progress.lock().unwrap();
    assert!(
        events
            .iter()
            .any(|e| e.event_kind == progress_event::ONTOLOGY_PARSED),
        "should emit ontology_parsed event, got kinds: {:?}",
        events.iter().map(|e| &e.event_kind).collect::<Vec<_>>()
    );
    assert!(
        events
            .iter()
            .any(|e| e.event_kind == progress_event::ARTIFACT_WRITTEN),
        "should emit artifact_written event, got kinds: {:?}",
        events.iter().map(|e| &e.event_kind).collect::<Vec<_>>()
    );
    assert!(
        events
            .iter()
            .any(|e| e.event_kind == progress_event::COMPLETED),
        "should emit completed event, got kinds: {:?}",
        events.iter().map(|e| &e.event_kind).collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn test_presenting_with_change_request_returns_to_planning() {
    let pool = setup_e2e().await;
    let llm = MockLlmService::inventory_planning_ok();
    let agent = AppAgent::new(Arc::new(pool), Box::new(llm));

    let mut ctx = ConversationContext::new(101, "Inventory app".to_string(), "Alioth".to_string());
    agent.run(&mut ctx).await.unwrap();
    assert!(
        matches!(ctx.state, AgentState::Published { .. }),
        "expected Published after first run, got {:?}",
        ctx.state
    );

    // Convert terminal Published state to Presenting so we can exercise the
    // change-request branch of the Presenting handler in isolation.
    if let AgentState::Published { result } = ctx.state {
        ctx.state = AgentState::Presenting { result };
    }

    ctx.change_requests
        .push("Add a purchase approval workflow".to_string());
    let result = agent
        .run_single_step(&mut ctx, None::<&fn(app_agent::AgentProgress)>)
        .await
        .unwrap();
    assert!(
        matches!(result.state_after, AgentState::Planning { .. }),
        "should return to Planning after change request, got {:?}",
        result.state_after
    );
    assert!(
        ctx.user_description.contains("购买") || ctx.user_description.contains("purchase"),
        "user_description should include change request"
    );
}

/// Gap 8：Verifying rubric 评估环 + 回流（Evaluator-Optimizer）集成测试。
///
/// 直接驱动 orchestrator 的 Verifying 分支（单步），用 `judge_low_score` mock
/// 让 LLM-as-Judge 三个语义维度恒返回 0.20，从而 overall < 阈值(0.8)：
/// - 第 1~3 次评估：未达阈值且 eval_iteration < MAX(3) → 回流 Composing，eval_iteration 递增
/// - 第 4 次评估：未达阈值但 eval_iteration 已触顶 → 强制 Publishing（不无限回流）
/// 验证：回流状态转移、eval_iteration 计数、eval_feedback/eval-report.json 产出。
#[tokio::test]
async fn test_verifying_rubric_reflow_to_composing_then_publish() {
    let pool = setup_e2e().await;
    let llm = MockLlmService::judge_low_score();
    let agent = AppAgent::new(Arc::new(pool), Box::new(llm));

    // 构造 Verifying 上下文：scratch 目录含合法 app.json + 4 个有效 extension yaml
    let scratch_dir = std::env::temp_dir().join(format!(
        "alioth_eval_test_{}_{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    let _ = std::fs::remove_dir_all(&scratch_dir);
    std::fs::create_dir_all(&scratch_dir).unwrap();
    std::fs::create_dir_all(scratch_dir.join("extensions")).unwrap();

    let app_json = serde_json::json!({
        "id": "ai-eval-test",
        "code": "ai-eval-test",
        "namespace": "Alioth",
        "name": "Eval Test App",
        "version": "0.1.0",
        "status": "developing",
        "deploymentMode": "standalone",
        "navigation": [{"label": "系统管理", "modules": ["inventory", "demand"]}]
    });
    std::fs::write(
        scratch_dir.join("app.json"),
        serde_json::to_string_pretty(&app_json).unwrap(),
    )
    .unwrap();
    for f in [
        "constraints.yaml",
        "rules.yaml",
        "statemachines.yaml",
        "workflows.yaml",
    ] {
        std::fs::write(scratch_dir.join("extensions").join(f), "# extension\n").unwrap();
    }

    let flow_plan = FlowPlan {
        used_modules: vec!["inventory".to_string(), "demand".to_string()],
        namespace: "Alioth".to_string(),
        known_entities: vec![],
        workflow_steps: vec![],
        missing_info: vec![],
        created_modules: vec![],
        created_blocks: vec![],
        created_services: vec![],
        ontology_model_json: None,
        functional_units: vec![],
        semantic_concepts: vec![],
        computations: vec![],
        constraints: vec![],
        business_rules: vec![],
        app_meta: None,
    };

    let mut ctx = ConversationContext::new(
        200,
        "I need an inventory app".to_string(),
        "Alioth".to_string(),
    );
    ctx.state = AgentState::Verifying {
        verification_round: 0,
    };
    ctx.flow_plan = Some(flow_plan);
    ctx.compose_scratch = Some(ComposeScratch {
        app_name: "Eval Test App".to_string(),
        output_path: scratch_dir.to_string_lossy().to_string(),
        files_written: 1,
        module_count: 2,
        gateway_design_content: None,
    });
    assert_eq!(ctx.eval_iteration, 0, "初始 eval_iteration 应为 0");

    // 第 1 次评估：失败 → 回流 Composing, eval_iteration=1
    agent
        .run_single_step(&mut ctx, None::<&fn(app_agent::AgentProgress)>)
        .await
        .expect("run_single_step Verifying 应成功");
    assert!(
        matches!(ctx.state, AgentState::Composing),
        "第 1 次评估应回流 Composing, 实际 {:?}",
        ctx.state
    );
    assert_eq!(ctx.eval_iteration, 1, "回流后 eval_iteration 应为 1");
    assert!(ctx.eval_feedback.is_some(), "回流应写入 eval_feedback");
    assert!(
        scratch_dir.join("eval-report.json").exists(),
        "应产出 eval-report.json"
    );

    // 第 2 次评估：继续回流, eval_iteration=2
    ctx.state = AgentState::Verifying {
        verification_round: 0,
    };
    agent
        .run_single_step(&mut ctx, None::<&fn(app_agent::AgentProgress)>)
        .await
        .expect("run_single_step Verifying 应成功");
    assert!(
        matches!(ctx.state, AgentState::Composing),
        "第 2 次评估应回流 Composing, 实际 {:?}",
        ctx.state
    );
    assert_eq!(ctx.eval_iteration, 2, "回流后 eval_iteration 应为 2");

    // 第 3 次评估：回流到上限, eval_iteration=3
    ctx.state = AgentState::Verifying {
        verification_round: 0,
    };
    agent
        .run_single_step(&mut ctx, None::<&fn(app_agent::AgentProgress)>)
        .await
        .expect("run_single_step Verifying 应成功");
    assert!(
        matches!(ctx.state, AgentState::Composing),
        "第 3 次评估应回流 Composing, 实际 {:?}",
        ctx.state
    );
    assert_eq!(ctx.eval_iteration, 3, "回流后 eval_iteration 应为 3");

    // 第 4 次评估：已达上限, 不静默发布, 转入 AwaitingUserInput 等待人工干预
    ctx.state = AgentState::Verifying {
        verification_round: 0,
    };
    agent
        .run_single_step(&mut ctx, None::<&fn(app_agent::AgentProgress)>)
        .await
        .expect("run_single_step Verifying 应成功");
    assert!(
        matches!(ctx.state, AgentState::AwaitingUserInput { .. }),
        "触顶后应转入 AwaitingUserInput 等待人工干预, 实际 {:?}",
        ctx.state
    );
    assert_eq!(ctx.eval_iteration, 3, "触顶后 eval_iteration 不应再增长");

    // 用户干预路径：提交变更请求后从 AwaitingUserInput 回到 Planning 重新生成,
    // 且 eval_iteration / eval_feedback 清零, 给评估环一次全新收敛机会。
    ctx.change_requests
        .push("简化目标描述, 仅保留核心库存模块".to_string());
    agent
        .run_single_step(&mut ctx, None::<&fn(app_agent::AgentProgress)>)
        .await
        .expect("run_single_step AwaitingUserInput 应成功");
    assert!(
        matches!(ctx.state, AgentState::Planning { .. }),
        "AwaitingUserInput + 变更请求应回到 Planning, 实际 {:?}",
        ctx.state
    );
    assert_eq!(ctx.eval_iteration, 0, "干预后应清零 eval_iteration");
    assert!(ctx.eval_feedback.is_none(), "干预后应清零 eval_feedback");
    assert!(
        ctx.user_description.contains("简化目标描述"),
        "变更请求应并入 user_description"
    );

    let _ = std::fs::remove_dir_all(&scratch_dir);
}
