use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::Serialize;
use sqlx::PgPool;


#[derive(Debug, Serialize)]
struct GapEntry {
    junction_table: String,
    left_entity: String,
    right_table: String,
    field_name: String,
    has_m2n: bool,
}

fn parse_rr_name(name: &str) -> Option<(String, String)> {
    let clean = name.trim_start_matches("zc_id_");
    let pos = clean.find("_rr_")?;
    Some((clean[..pos].to_string(), clean[pos + 4..].to_string()))
}

async fn query_junction_tables(pool: &PgPool) -> Result<Vec<String>> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT table_name FROM information_schema.tables \
         WHERE table_schema='isahl' AND table_name LIKE '%_rr_%' ORDER BY table_name"
    ).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

async fn field_exists(pool: &PgPool, code: &str) -> Result<bool> {
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT COUNT(*)>0 FROM isahl_meta.meta_fields WHERE code=$1"
    ).bind(code).fetch_one(pool).await.unwrap_or(false))
}

fn create_meta_api_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default()
}

fn api_url() -> String {
    std::env::var("META_API_URL").unwrap_or_else(|_| "http://127.0.0.1:4949".into())
}

async fn login(client: &reqwest::Client) -> Result<String> {
    let url = format!("{}/api/meta/auth/login", api_url());
    let user = std::env::var("META_USER").unwrap_or_else(|_| "admin".into());
    let pass = std::env::var("META_PASS").unwrap_or_else(|_| "admin".into());
    let resp = client.post(&url)
        .json(&serde_json::json!({"username": user, "password": pass, "provider": "local"}))
        .send().await
        .with_context(|| format!("Meta login failed at {url}"))?;
    let body: serde_json::Value = resp.json().await?;
    body["token"].as_str().map(|s| s.to_string())
        .or_else(|| body["data"]["token"].as_str().map(|s| s.to_string()))
        .or_else(|| std::env::var("META_API_JWT").ok())
        .context("No JWT token in login response")
}

async fn create_field(client: &reqwest::Client, token: &str, payload: &serde_json::Value) -> Result<()> {
    let url = format!("{}/api/meta/internal/fields", api_url());
    let mut h = HeaderMap::new();
    h.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {token}")).unwrap());
    h.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let resp = client.post(&url).headers(h).json(payload).send().await
        .with_context(|| format!("Meta API POST failed at {url}"))?;
    let status = resp.status();
    let body: serde_json::Value = resp.json().await.unwrap_or_default();
    if !status.is_success() {
        anyhow::bail!("Meta API error (HTTP {status}): {body}");
    }
    Ok(())
}

fn read_arg(args: &[String], name: &str) -> String {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned().unwrap_or_default()
}

pub async fn cmd_report(pool: &PgPool, _args: &[String]) -> Result<()> {
    let tables = query_junction_tables(pool).await?;
    let mut found = 0;
    for t in &tables {
        if let Some((l, r)) = parse_rr_name(t) {
            let code = format!("{l}_rr_{r}");
            let exists = field_exists(pool, &code).await.unwrap_or(false);
            println!("  {t} | {l} → {r} | exists={exists}");
            found += 1;
        }
    }
    println!("Total: {found} junction tables");
    Ok(())
}

pub async fn cmd_apply(pool: &PgPool, args: &[String]) -> Result<()> {
    let dry_run = args.iter().any(|a| a == "--dry-run");
    let _mapping_override = read_arg(args, "--mapping-override");
    let _backup = read_arg(args, "--backup");
    let tables = query_junction_tables(pool).await?;
    let mut gaps: Vec<GapEntry> = Vec::new();

    for t in &tables {
        if let Some((l, r)) = parse_rr_name(t) {
            let code = format!("{l}_rr_{r}");
            let has_m2n = field_exists(pool, &code).await.unwrap_or(false);
            if !has_m2n {
                gaps.push(GapEntry {
                    junction_table: t.clone(),
                    left_entity: l,
                    right_table: format!("zc_id_{r}"),
                    field_name: code,
                    has_m2n: false,
                });
            }
        }
    }

    if gaps.is_empty() {
        println!("✅ No gaps found — all M2N fields exist");
        return Ok(());
    }

    println!("Found {} missing M2N fields:", gaps.len());
    for g in &gaps {
        println!("  {} ({} → {})", g.field_name, g.left_entity, g.right_table);
    }

    if dry_run {
        println!("Dry-run: would create {} fields", gaps.len());
        return Ok(());
    }

    let client = create_meta_api_client();
    let token = login(&client).await?;
    let mut created = 0;
    let mut errors = 0;

    for g in &gaps {
        let payload = serde_json::json!({
            "collection_id": g.left_entity,
            "name": g.field_name,
            "display_name": g.field_name,
            "field_type": "m2n",
            "reference_config": {
                "target_table": g.right_table,
                "junction_table": g.junction_table,
            },
        });
        match create_field(&client, &token, &payload).await {
            Ok(()) => { println!("  ✅ {} created", g.field_name); created += 1; }
            Err(e) => { eprintln!("  ❌ {} failed: {e}", g.field_name); errors += 1; }
        }
    }

    println!("Created: {created}, Errors: {errors}");
    if errors > 0 { std::process::exit(1); }
    Ok(())
}

pub async fn cmd_verify(pool: &PgPool, _args: &[String]) -> Result<()> {
    let tables = query_junction_tables(pool).await?;
    let mut ok = 0;
    let mut miss = 0;
    for t in &tables {
        if let Some((l, r)) = parse_rr_name(t) {
            let code = format!("{l}_rr_{r}");
            if field_exists(pool, &code).await.unwrap_or(false) {
                ok += 1;
            } else {
                println!("  ❌ {code} missing");
                miss += 1;
            }
        }
    }
    println!("OK={ok} MISSING={miss}");
    if miss > 0 { std::process::exit(1); }
    Ok(())
}
