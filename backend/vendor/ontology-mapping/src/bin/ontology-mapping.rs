//! ontology-mapping CLI — 统一 Rust 入口（替代 scripts/ontology-discover.py）
//!
//! 子命令与 Python 版同名：leafs / search / match / locate / roots / hierarchies /
//! to-mapping-input / map。全部输出 JSON 到 stdout，供 OMP/skill 链与 AppAgent 消费。
//!
//! DB 连接：环境变量 DATABASE_URL（默认 postgres://localhost:5432/aliothstudio_dev）。

use anyhow::Result;
use ontology_mapping::discovery;
use ontology_mapping::OntologyMapper;
use sqlx::postgres::PgPoolOptions;

const DEFAULT_DB: &str = "postgres://localhost:5432/aliothstudio_dev";

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}

fn parse_json_list(s: Option<String>) -> Vec<String> {
    s.and_then(|v| serde_json::from_str::<Vec<String>>(&v).ok())
        .unwrap_or_default()
}

fn print_json<T: serde::Serialize>(v: &T) {
    println!("{}", serde_json::to_string_pretty(v).unwrap());
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    // 无需 DB 的命令
    match cmd {
        "ontology-stale" => {
            // ontology-stale --ns <ns> --scene <scene> [--ontology-output <path>] [--skip-field-check] [--warn-only]
            let ns = arg(&args, "--ns").unwrap_or_default();
            let scene = arg(&args, "--scene").unwrap_or_default();
            let output = arg(&args, "--ontology-output");
            let skip_fields = args.iter().any(|a| a == "--skip-field-check");
            let warn_only = args.iter().any(|a| a == "--warn-only");
            if ns.is_empty() && output.is_none() {
                eprintln!("Usage: ontology-mapping ontology-stale --ns <ns> --scene <scene> [--ontology-output <path>]");
                std::process::exit(2);
            }
            let (report, exit) =
                ontology_mapping::stale::check_stale(&ns, &scene, output.as_deref(), skip_fields)?;
            print_json(&report);
            std::process::exit(if warn_only { 0 } else { exit });
        }
        "gen-service-tests" => {
            // gen-service-tests --manifest <path> [--write]
            let manifest = arg(&args, "--manifest").unwrap_or_default();
            if manifest.is_empty() {
                eprintln!("Usage: ontology-mapping gen-service-tests --manifest <path> [--write]");
                std::process::exit(1);
            }
            let write = args.iter().any(|a| a == "--write");
            let results = ontology_mapping::gen_tests::gen_service_tests(
                std::path::Path::new(&manifest),
                write,
            )?;
            print_json(&results);
            return Ok(());
        }
        "gen-flow-tests" => {
            // gen-flow-tests --manifest <path> [--write] [--llm] [--service <id>]
            let manifest = arg(&args, "--manifest").unwrap_or_default();
            if manifest.is_empty() {
                eprintln!("Usage: ontology-mapping gen-flow-tests --manifest <path> [--write] [--llm] [--service <id>]");
                std::process::exit(1);
            }
            let write = args.iter().any(|a| a == "--write");
            let use_llm = args.iter().any(|a| a == "--llm");
            let target_svc = arg(&args, "--service");
            let (results, scenarios) = ontology_mapping::gen_tests::gen_flow_tests(
                std::path::Path::new(&manifest),
                write,
                use_llm,
                target_svc.as_deref(),
            )
            .await?;
            print_json(&serde_json::json!({ "scenarios": scenarios, "results": results }));
            return Ok(());
        }
        "apply-contracts" => {
            // apply-contracts [--target-module M --target-table T | --init-calibration]
            if args.iter().any(|a| a == "--init-calibration") {
                let inited = ontology_mapping::contracts::init_calibration_all();
                print_json(
                    &serde_json::json!({ "initialized": inited.iter().map(|(m, p)| serde_json::json!({"module": m, "path": p})) .collect::<Vec<_>>() }),
                );
                return Ok(());
            }
            let module = arg(&args, "--target-module").unwrap_or_default();
            let table = arg(&args, "--target-table");
            let ns = arg(&args, "--ns").unwrap_or_else(|| module.clone());
            let matrix = ontology_mapping::contracts::build_consistency_matrix(
                &module,
                &ns,
                table.as_deref(),
            );
            print_json(&matrix);
            if matrix.conflicts == 0 && !module.is_empty() {
                let written = ontology_mapping::contracts::update_calibration(
                    &module,
                    "verified",
                    "与所有引用同表的模块一致",
                )?;
                eprintln!("calibration: {:?}", written);
            }
            return Ok(());
        }
        "prototype-check" => {
            // prototype-check [targets...] [--no-babel] — 退出码: 0 全过 / 1 错误 / 2 仅警告
            let run_babel = !args.iter().any(|a| a == "--no-babel");
            let mut targets: Vec<std::path::PathBuf> = args[2..]
                .iter()
                .filter(|a| !a.starts_with("--"))
                .map(std::path::PathBuf::from)
                .collect();
            if targets.is_empty() {
                targets.push(std::path::PathBuf::from(
                    "Pre-Proc/Alioth/Prototypes/Modules",
                ));
            }
            let (report, exit) =
                ontology_mapping::prototype_check::check_targets(&targets, run_babel);
            print_json(&report);
            std::process::exit(exit);
        }
        "roots" => {
            let roots: Vec<(&str, &str)> = discovery::HIERARCHY_ROOTS.to_vec();
            print_json(&roots);
            return Ok(());
        }
        "hierarchies" => {
            print_json(&discovery::HIERARCHY_ROOTS);
            return Ok(());
        }
        "to-mapping-input" => {
            let entity = arg(&args, "--entity").unwrap_or_default();
            let table = arg(&args, "--table").unwrap_or_default();
            let fields = parse_json_list(arg(&args, "--fields"));
            let scene = arg(&args, "--scene").unwrap_or_default();
            let factors = parse_json_list(arg(&args, "--factor-ids"));
            if scene.is_empty() {
                eprintln!("[ontology-mapping] WARN: 未指定 --scene，scene_code 留空（下游坐标 tier 应为 Unclear，禁止猜测兜底）");
            }
            let input = discovery::to_mapping_input(&entity, &table, &fields, &scene, &factors);
            print_json(&input);
            return Ok(());
        }
        "map" => {
            // map <input.json> [--services <dir>] [--rules <path>]
            let input_path = args.get(2).cloned().unwrap_or_default();
            let services = arg(&args, "--services")
                .unwrap_or_else(|| "Pre-Proc/Alioth/Sources/Services".into());
            let rules = arg(&args, "--rules")
                .unwrap_or_else(|| "Meta/backend/ontology-mapping/rules.yaml".into());
            let content = std::fs::read_to_string(&input_path)?;
            let input: ontology_mapping::MappingInput = serde_json::from_str(&content)?;
            let mapper = OntologyMapper::load(&rules, &services)?;
            let output = mapper.map(&input);
            print_json(&output);
            return Ok(());
        }
        _ => {}
    }

    // 需要 DB 的命令
    // sqlx 0.9 默认经 whoami 取系统用户名（macOS 上解析为 anonymous 而失败），
    // 无 DATABASE_URL 时按 libpq 惯例回退到 $USER@localhost。
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        let user = std::env::var("USER").unwrap_or_else(|_| "postgres".into());
        format!("postgres://{user}@localhost:5432/aliothstudio_dev")
    });
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&db_url)
        .await?;

    let top: usize = arg(&args, "--top")
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);

    match cmd {
        "model-graph" => {
            let graph = ontology_mapping::model_graph::load_graph(&pool).await?;
            print_json(&graph);
        }
        "gap" => {
            // gap <proto.html>
            let proto = args.get(2).cloned().unwrap_or_default();
            let report = ontology_mapping::gap::analyze(std::path::Path::new(&proto))?;
            print_json(&report);
        }
        "leafs" => {
            let parent = arg(&args, "--parent").unwrap_or_else(|| "zc_id_lifecycle".into());
            let leafs = discovery::query_leafs(&pool, &parent).await?;
            print_json(&leafs);
        }
        "search" => {
            let entity = arg(&args, "--entity").unwrap_or_default();
            let parent = arg(&args, "--parent").unwrap_or_else(|| "zc_id_lifecycle".into());
            let cands = discovery::search_tables(&pool, &entity, &parent, top).await?;
            print_json(&cands);
        }
        "match" => {
            let entity = arg(&args, "--entity").unwrap_or_default();
            let fields = parse_json_list(arg(&args, "--fields"));
            let parent = arg(&args, "--parent");
            let cands =
                discovery::match_tables(&pool, &entity, &fields, parent.as_deref(), top).await?;
            print_json(&cands);
        }
        "locate" => {
            let entity = arg(&args, "--entity").unwrap_or_default();
            let hint = arg(&args, "--hint");
            let fields = parse_json_list(arg(&args, "--fields"));
            let cands =
                discovery::locate_tables(&pool, &entity, hint.as_deref(), &fields, top).await?;
            print_json(&cands);
            }
            "transfer" => {
            // transfer --namespace <ns> [--services-dir <dir>] [--rules <path>] [--gaps <json>]
            let ns = arg(&args, "--namespace").unwrap_or_default();
            let services_dir = arg(&args, "--services-dir").unwrap_or_else(|| {
                format!("Pre-Proc/{}/Sources/Services", ns)
            });
            let rules_path = arg(&args, "--rules").unwrap_or_else(|| {
                "Meta/backend/ontology-mapping/rules.yaml".to_string()
            });
            let gaps_json = arg(&args, "--gaps").unwrap_or_else(|| "[]".to_string());
            let gaps: Vec<serde_json::Value> = serde_json::from_str(&gaps_json).unwrap_or_default();

            let mapper = OntologyMapper::load(&rules_path, &services_dir)?;
            let root = std::path::Path::new(".");
            let mut mapped: Vec<serde_json::Value> = Vec::new();
            for gap in &gaps {
                let domain_id = gap.get("domain_id").and_then(|v| v.as_str()).unwrap_or("");
                let fields: Vec<String> = gap.get("new_fields")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|f| f.get("name").and_then(|n| n.as_str()).map(|s| s.to_string())).collect())
                    .unwrap_or_default();
                if domain_id.is_empty() { continue; }
                let candidates = discovery::match_tables(&pool, domain_id, &fields, None, 3).await?;
                if let Some(best) = candidates.first() {
                    if best.score >= 0.5 {
                        let input = discovery::to_mapping_input(domain_id, &best.table, &fields, "", &[]);
                        let output = mapper.map(&input);
                        if let Some(entity) = output.entities.first() {
                            mapped.push(serde_json::json!({
                                "domain_id": domain_id,
                                "table": best.table,
                                "score": best.score,
                                "function_code": entity.coordinates.function.value,
                                "function_confidence": entity.coordinates.function.confidence,
                                "fields": entity.fields,
                            }));
                        }
                    }
                }
            }
            print_json(&serde_json::json!({ "mapped": mapped, "total": gaps.len(), "matched": mapped.len() }));
        }
        _ => {
            eprintln!("ontology-mapping — 本体映射发现与字段映射 CLI");
            eprintln!();
            eprintln!("用法:");
            eprintln!("  ontology-mapping ontology-stale --ns <ns> --scene <scene> [--ontology-output <path>] [--warn-only]");
            eprintln!("  ontology-mapping gen-service-tests --manifest <path> [--write]");
            eprintln!("  ontology-mapping gen-flow-tests --manifest <path> [--write] [--llm] [--service <id>]");
            eprintln!("  ontology-mapping prototype-check [targets...] [--no-babel]");
            eprintln!("  ontology-mapping model-graph");
            eprintln!("  ontology-mapping gap <proto.html>");
            eprintln!("  ontology-mapping to-mapping-input --entity <name> --table <table> [--fields '<json>'] [--scene <code>] [--factor-ids '<json>']");
            eprintln!("  ontology-mapping map <input.json> [--services <dir>] [--rules <path>]");
            eprintln!("  ontology-mapping transfer --namespace <ns> [--services-dir <dir>] [--rules <path>] [--gaps '<json>']");
            eprintln!("  ontology-mapping roots | hierarchies");
            eprintln!("DB: DATABASE_URL（默认 {DEFAULT_DB}）");
            std::process::exit(if cmd.is_empty() { 0 } else { 1 });
        }
    }
    Ok(())
}
