use actix_files::NamedFile;
use actix_web::{middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer};
use app_creator::chat;
use app_creator::handlers;
use app_creator::middleware::SsoAuthMiddleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("Panic: {}", info);
        let backtrace = std::backtrace::Backtrace::force_capture();
        eprintln!("Backtrace:\n{:?}", backtrace);
    }));

    eprintln!("Starting AliothStudio AppCreator Service...");

    // Database pool (required for AppAgent persistence)
    let database_url = std::env::var("DATABASE_URL").expect(
        "DATABASE_URL must be set. Use scripts/db/ensure-schema.sh to initialize the schema.",
    );
    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");
    let pool_data: web::Data<sqlx::PgPool> = web::Data::new(pool);

    // Auto-patch: ensure required schema + enum values + tables at startup
    chat::ensure_isahl_meta_schema(pool_data.as_ref())
        .await
        .expect("Failed to ensure isahl_meta schema");
    chat::ensure_chat_session_status_values(pool_data.as_ref())
        .await
        .expect("Failed to ensure chat_session_status enum values");
    chat::ensure_app_creator_tables(pool_data.as_ref())
        .await
        .expect("Failed to create app_creator tables");

    // LLM service (required for AppAgent)
    let llm_config = llm::LlmServiceConfig::from_env();
    let llm_service = llm::LlmService::new(llm_config)
        .expect("Failed to initialize LLM service; check LLM_PROVIDER / LLM_API_KEY");
    let llm_data = web::Data::new(llm_service);

    // 认证模式初始化（SSO/Standalone 双模式判定）
    app_creator::auth_config::init_auth_config();

    // Config
    let server_addr =
        std::env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:49495".to_string());
    let frontend_dir = std::env::var("FRONTEND_DIR").unwrap_or_else(|_| "./frontend".to_string());

    // Auth middleware verifies SSO ES256 JWT and injects RequestContext.
    // Public paths (/health, /api/creator/status) are handled internally.

    // Logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();
    log::info!("AppCreator config loaded: server_addr={}", server_addr);

    let frontend_dir_data = web::Data::new(frontend_dir);

    log::info!("AppCreator Service listening on {}", server_addr);

    HttpServer::new(move || {
        App::new()
            // API v1 — all /api/creator/* endpoints
            .service(
                web::scope("/api/creator")
                    // P1 — Chat sessions (AppAgent-driven)
                    .configure(chat::configure_routes)
                    // P4 — App FS Repository
                    .route("/apps", web::get().to(handlers::list_apps))
                    .route("/apps/{code}", web::get().to(handlers::get_app))
                    .route("/apps/{code}", web::delete().to(handlers::delete_app))
                    // P0 — App creation (via AppAgent)
                    .route("/apps", web::post().to(chat::create_app_handler))
                    // P3 — User
                    .route("/user/me", web::get().to(handlers::get_current_user))
                    .route("/auth/login", web::post().to(handlers::login_standalone))
                    .route("/auth/refresh", web::post().to(handlers::refresh_token)),
            )
            .wrap(Logger::default())
            .wrap(common::RateLimitMiddleware::per_ip_any(
                &["/api/creator/auth/login"],
                10.0,
                10.0 / 60.0,
            ))
            .wrap(SsoAuthMiddleware::new())
            .app_data(pool_data.clone())
            .app_data(llm_data.clone())
            .app_data(frontend_dir_data.clone())
            .route("/health", web::get().to(health_check))
            .route("/api/creator/status", web::get().to(api_status))
            .default_service(web::route().to(spa_fallback))
    })
    .bind(&server_addr)?
    .run()
    .await
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

async fn api_status() -> HttpResponse {
    let cfg = app_creator::auth_config::auth_config();
    let auth_mode = match cfg.mode {
        app_creator::auth_config::AuthMode::Sso => "sso",
        app_creator::auth_config::AuthMode::Standalone => "standalone",
    };
    HttpResponse::Ok().json(serde_json::json!({
        "service": "app-creator",
        "version": env!("CARGO_PKG_VERSION"),
        "status": "running",
        "auth_mode": auth_mode,
    }))
}

/// SPA fallback: serve requested file or index.html.
async fn spa_fallback(
    req: HttpRequest,
    frontend_dir: web::Data<String>,
) -> actix_web::Result<NamedFile> {
    let path = req.path();
    let safe_path = path.trim_start_matches('/');
    let file_path = std::path::Path::new(frontend_dir.as_ref()).join(safe_path);

    if file_path.is_file() {
        return Ok(NamedFile::open(file_path)?);
    }
    Ok(NamedFile::open(format!(
        "{}/index.html",
        frontend_dir.as_ref()
    ))?)
}
