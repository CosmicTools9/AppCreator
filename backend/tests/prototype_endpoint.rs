//! Prototype 预览端点集成测试（真实 DB + 真实 HTTP 链路 + 真实产物文件）。
//!
//! 验证 GET /api/creator/sessions/{id}/prototype：
//! - 有效 SSO token + 已组合 session → 200 text/html（真实产物 I_need_a）
//! - 无 token → 401
//! - session 未组合（无 app_name���→ 404

use actix_web::{test, web, App};
use app_creator::{chat, middleware::SsoAuthMiddleware};
use common::testing::connect_test_db;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

const TEST_PRIVATE_KEY_PEM: &str =
    include_str!("../../../SSO/backend/tests/fixtures/sso_jwt_private.pem");
const TEST_PUBLIC_KEY_PEM: &str =
    include_str!("../../../SSO/backend/tests/fixtures/sso_jwt_public.pem");

fn make_token() -> String {
    #[derive(serde::Serialize)]
    struct Claims {
        sub: String,
        email: String,
        exp: i64,
        iat: i64,
        iss: String,
        namespace: String,
    }
    let now = chrono::Utc::now().timestamp();
    let claims = Claims {
        sub: "1".to_string(),
        email: "tester@alioth.dev".to_string(),
        exp: now + 600,
        iat: now,
        iss: "alioth-sso".to_string(),
        namespace: "Cosmic-Tools".to_string(),
    };
    encode(
        &Header::new(Algorithm::ES256),
        &claims,
        &EncodingKey::from_ec_pem(TEST_PRIVATE_KEY_PEM.as_bytes()).unwrap(),
    )
    .unwrap()
}
fn project_root() -> String {
    format!("{}/../..", env!("CARGO_MANIFEST_DIR"))
}

#[tokio::test]
async fn prototype_endpoint_serves_real_artifact() {
    // 根目录解析依赖环境变量（避免改动进程 CWD 影响并行测试）
    unsafe {
        std::env::set_var("APPCREATOR_PROJECT_ROOT", project_root());
        std::env::set_var("SSO_JWT_PUBLIC_KEY", TEST_PUBLIC_KEY_PEM);
    }
    app_creator::auth_config::init_auth_config();

    let pool = connect_test_db().await;
    let session = chat::create_session(&pool, "prototype e2e", None, "Cosmic-Tools")
        .await
        .expect("create_session failed");
    let mut ctx = app_agent::ConversationContext::new(
        session.id,
        "e2e".to_string(),
        "Cosmic-Tools".to_string(),
    );
    ctx.app_name = Some("I_need_a".to_string());
    chat::save_agent_context(&pool, session.id, &ctx)
        .await
        .expect("save_agent_context failed");

    let app = test::init_service(
        App::new()
            .wrap(SsoAuthMiddleware::new())
            .app_data(web::Data::new(pool.clone()))
            .service(web::scope("/api/creator").configure(chat::configure_routes)),
    )
    .await;

    // 1. 有效 token → 200 text/html
    let req = test::TestRequest::get()
        .uri(&format!("/api/creator/sessions/{}/prototype", session.id))
        .insert_header(("Authorization", format!("Bearer {}", make_token())))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    assert!(content_type.starts_with("text/html"), "got: {content_type}");
    let body = test::read_body(resp).await;
    assert!(!body.is_empty(), "prototype.html body must not be empty");

    // 2. 无 token → 401
    let req = test::TestRequest::get()
        .uri(&format!("/api/creator/sessions/{}/prototype", session.id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // 3. 未组合 session（无 app_name）→ 404
    let session2 = chat::create_session(&pool, "prototype e2e not-ready", None, "Cosmic-Tools")
        .await
        .expect("create_session failed");
    let ctx2 = app_agent::ConversationContext::new(
        session2.id,
        "e2e".to_string(),
        "Cosmic-Tools".to_string(),
    );
    chat::save_agent_context(&pool, session2.id, &ctx2)
        .await
        .expect("save_agent_context failed");
    let req = test::TestRequest::get()
        .uri(&format!("/api/creator/sessions/{}/prototype", session2.id))
        .insert_header(("Authorization", format!("Bearer {}", make_token())))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}
