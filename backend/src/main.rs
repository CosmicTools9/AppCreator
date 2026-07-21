use actix_files::NamedFile;
use actix_web::{middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("Panic: {}", info);
        let backtrace = std::backtrace::Backtrace::force_capture();
        eprintln!("Backtrace:\n{:?}", backtrace);
    }));

    eprintln!("Starting AliothStudio AppCreator Service...");

    // Load config
    let server_addr =
        std::env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:49495".to_string());
    let frontend_dir = std::env::var("FRONTEND_DIR").unwrap_or_else(|_| "./frontend".to_string());

    // Initialize logger
    let mut logger_builder =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"));
    logger_builder.format_timestamp_millis();
    logger_builder.init();
    log::info!("AppCreator config loaded: server_addr={}, frontend_dir={}", server_addr, frontend_dir);

    let frontend_dir_data = web::Data::new(frontend_dir);

    log::info!("AppCreator Service listening on {}", server_addr);

    HttpServer::new(move || {
        App::new()
            .app_data(frontend_dir_data.clone())
            .wrap(Logger::default())
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
    HttpResponse::Ok().json(serde_json::json!({
        "service": "app-creator",
        "version": env!("CARGO_PKG_VERSION"),
        "status": "running"
    }))
}

/// SPA fallback: serve requested file if it exists, otherwise serve index.html.
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

    Ok(NamedFile::open(format!("{}/index.html", frontend_dir.as_ref()))?)
}
