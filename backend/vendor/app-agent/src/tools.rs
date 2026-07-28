//! Agent 工具集

pub use crate::state::PlatformCatalog;
use crate::state::{
    CollectionInfo as StateCollectionInfo, FactorInfo as StateFactorInfo,
    FunctionInfo as StateFunctionInfo, ModuleInfo, SceneInfo as StateSceneInfo,
};
use serde::{Deserialize, Serialize};
use sqlx::{AssertSqlSafe, PgPool};

#[derive(Debug, Clone)]
pub enum Tool {
    ListZcIdTables,
    ListOntologyDimensions,
    QueryCollectionFields,
    GenerateExtensionDdl,
    GenerateModuleWiring,
    ValidateDdl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool: String,
    pub data: serde_json::Value,
    pub error: Option<String>,
}

impl ToolResult {
    pub fn ok(tool: &str, data: impl serde::Serialize) -> Self {
        Self {
            tool: tool.to_string(),
            data: serde_json::to_value(data).unwrap_or(serde_json::Value::Null),
            error: None,
        }
    }

    pub fn err(tool: &str, err: impl std::fmt::Display) -> Self {
        Self {
            tool: tool.to_string(),
            data: serde_json::Value::Null,
            error: Some(err.to_string()),
        }
    }
}

pub async fn list_zc_id_tables(pool: &PgPool) -> ToolResult {
    let result = sqlx::query_scalar::<_, String>(
        "SELECT tablename FROM pg_tables WHERE schemaname = 'isahl' AND tablename LIKE 'zc_id_%' ORDER BY tablename"
    )
    .fetch_all(pool)
    .await;

    match result {
        Ok(tables) => ToolResult::ok("list_zc_id_tables", tables),
        Err(e) => ToolResult::err("list_zc_id_tables", e),
    }
}

pub async fn list_ontology_dimensions(pool: &PgPool, namespace: &str) -> ToolResult {
    let catalog = fetch_platform_catalog(pool, namespace).await;
    ToolResult::ok("list_ontology_dimensions", catalog)
}

/// 按用户需求关键词查询相关本体上下文
///
/// 从 meta_ontology + meta_ontology_edges 中检索与用户需求最相关的实体，
/// 返回结构化上下文供 Planner 注入 LLM prompt。
pub async fn query_relevant_ontology(
    pool: &PgPool,
    keywords: &[&str],
    max_entities: usize,
) -> ToolResult {
    if keywords.is_empty() {
        return ToolResult::ok(
            "query_relevant_ontology",
            serde_json::json!({"entities": [], "edges": [], "entity_count": 0, "edge_count": 0}),
        );
    }

    let like_patterns: Vec<String> = keywords.iter().map(|k| format!("%{}%", k)).collect();

    // 1. 找出匹配的 collection（按 name / table_name）
    let mut collection_conditions: Vec<String> = Vec::new();
    for (i, _) in like_patterns.iter().enumerate() {
        collection_conditions.push(format!(
            "name ILIKE ${} OR table_name ILIKE ${}",
            i * 2 + 1,
            i * 2 + 2
        ));
    }
    let collections_query = format!(
        "SELECT name, table_name FROM isahl_meta.meta_collections WHERE ({}) ORDER BY name LIMIT {}",
        collection_conditions.join(" OR "),
        max_entities
    );

    #[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
    struct CollectionRow {
        name: String,
        table_name: String,
    }

    let mut db_query =
        sqlx::query_as::<_, CollectionRow>(AssertSqlSafe(collections_query.as_str()));
    for pattern in &like_patterns {
        db_query = db_query.bind(pattern).bind(pattern);
    }

    let collections: Vec<CollectionRow> = match db_query.fetch_all(pool).await {
        Ok(rows) => rows,
        Err(e) => return ToolResult::err("query_relevant_ontology", e),
    };

    // 2. 找出匹配的 field，并关联到其 collection
    #[derive(Debug, Clone, sqlx::FromRow)]
    struct FieldMatchRow {
        collection_name: String,
        collection_table_name: String,
    }

    let mut field_conditions: Vec<String> = Vec::new();
    for (i, _) in like_patterns.iter().enumerate() {
        field_conditions.push(format!("f.name ILIKE ${}", i + 1));
    }
    let fields_query = format!(
        "SELECT DISTINCT c.name AS collection_name, c.table_name AS collection_table_name
         FROM isahl_meta.meta_fields f
         JOIN isahl_meta.meta_collections c ON c.table_name = f.fk_collection
         WHERE ({})
         ORDER BY c.name, c.table_name
         LIMIT {}",
        field_conditions.join(" OR "),
        max_entities
    );

    let mut field_query = sqlx::query_as::<_, FieldMatchRow>(AssertSqlSafe(fields_query.as_str()));
    for pattern in &like_patterns {
        field_query = field_query.bind(pattern);
    }

    let field_rows: Vec<FieldMatchRow> = match field_query.fetch_all(pool).await {
        Ok(rows) => rows,
        Err(e) => return ToolResult::err("query_relevant_ontology", e),
    };

    // 合并 collection 集合
    let mut all_tables: std::collections::HashMap<String, String> = collections
        .into_iter()
        .map(|c| (c.table_name, c.name))
        .collect();
    for row in field_rows {
        all_tables.insert(row.collection_table_name, row.collection_name);
    }

    // 3. 组装实体输出（兼容旧 `entities` 字段，便于 PlanningPrompt 注入）
    #[derive(Debug, Clone, serde::Serialize)]
    struct Entity {
        concept_name: String,
        table_name: String,
        chain_name: String,
    }

    let entities: Vec<Entity> = all_tables
        .into_iter()
        .map(|(table_name, name)| Entity {
            concept_name: name.clone(),
            table_name,
            chain_name: name,
        })
        .collect();

    // 4. 边信息：当前 DB 无可用 ontology 边视图，返回空数组
    ToolResult::ok(
        "query_relevant_ontology",
        serde_json::json!({
            "entities": entities,
            "edges": [],
            "entity_count": entities.len(),
            "edge_count": 0,
        }),
    )
}

/// 生成模块间连接配置
pub async fn generate_module_wiring(used_modules: &[String]) -> ToolResult {
    let wirings: Vec<serde_json::Value> = used_modules
        .iter()
        .map(|m| {
            serde_json::json!({
                "module_id": m,
                "entrypoint": format!("{}_backend::handlers::config", m),
                "scope": format!("/modules/{}", m),
            })
        })
        .collect();

    ToolResult::ok(
        "generate_module_wiring",
        serde_json::json!({"wirings": wirings, "module_count": used_modules.len()}),
    )
}

pub async fn query_collection_fields(pool: &PgPool, collection_name: &str) -> ToolResult {
    #[derive(Debug, Clone, sqlx::FromRow)]
    struct FieldRow {
        name: String,
        field_type: Option<String>,
        description: Option<String>,
    }

    let rows = sqlx::query_as::<_, FieldRow>(
        r#"
        SELECT f.name, f.data_type::text as field_type, f.config->>'description' as description
        FROM isahl_meta.meta_fields f
        JOIN isahl_meta.meta_collections c ON c.table_name = f.fk_collection
        WHERE c.name = $1 AND f.deleted_at IS NULL
        ORDER BY f.name
        "#,
    )
    .bind(collection_name)
    .fetch_all(pool)
    .await;

    match rows {
        Ok(fields) => {
            let result: Vec<FieldInfo> = fields
                .into_iter()
                .map(|r| FieldInfo {
                    name: r.name,
                    field_type: r.field_type.unwrap_or_default(),
                    description: r.description,
                })
                .collect();
            ToolResult::ok("query_collection_fields", result)
        }
        Err(e) => ToolResult::err("query_collection_fields", e),
    }
}

/// 从数据库加载平台 catalog（场景/因子/功能 × 实体集合）
///
/// 供 App Agent 内部 `load_platform_catalog` 以及测试调试使用。
/// `namespace` 决定从哪个 Pre-Proc 命名空间读取模块元数据(module.json)。
pub async fn fetch_platform_catalog(pool: &PgPool, namespace: &str) -> PlatformCatalog {
    #[derive(Debug, Clone, sqlx::FromRow)]
    struct SceneRow {
        id: i64,
        code: String,
        notice: String,
    }
    #[derive(Debug, Clone, sqlx::FromRow)]
    struct FactorRow {
        id: i64,
        code: String,
        notice: String,
    }
    #[derive(Debug, Clone, sqlx::FromRow)]
    struct FunctionRow {
        id: i64,
        code: String,
        notice: String,
    }
    #[derive(Debug, Clone, sqlx::FromRow)]
    struct ColRow {
        name: String,
        table_name: Option<String>,
    }

    let scenes: Vec<SceneRow> =
        sqlx::query_as("SELECT id, code, notice FROM isahl.zc_id_scene ORDER BY code")
            .fetch_all(pool)
            .await
            .unwrap_or_default();

    let factors: Vec<FactorRow> =
        sqlx::query_as("SELECT id, code, notice FROM isahl.zc_id_factor ORDER BY code")
            .fetch_all(pool)
            .await
            .unwrap_or_default();

    let functions: Vec<FunctionRow> =
        sqlx::query_as("SELECT id, code, notice FROM isahl.zc_id_function ORDER BY code")
            .fetch_all(pool)
            .await
            .unwrap_or_default();

    let col_rows: Vec<ColRow> =
        sqlx::query_as("SELECT name, table_name FROM isahl_meta.meta_collections ORDER BY name")
            .fetch_all(pool)
            .await
            .unwrap_or_default();

    // 查询所有 collection 的字段信息
    #[derive(Debug, Clone, sqlx::FromRow)]
    struct FieldRow {
        collection_name: String,
        name: String,
        field_type: Option<String>,
        description: Option<String>,
    }
    let field_rows: Vec<FieldRow> = sqlx::query_as(
        "SELECT c.name as collection_name, f.name, f.data_type::text as field_type, f.config->>'description' as description
         FROM isahl_meta.meta_fields f
         JOIN isahl_meta.meta_collections c ON c.table_name = f.fk_collection
         ORDER BY c.name, f.name",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut fields_map: std::collections::HashMap<String, Vec<crate::state::FieldInfo>> =
        std::collections::HashMap::new();
    for row in field_rows {
        fields_map
            .entry(row.collection_name.clone())
            .or_default()
            .push(crate::state::FieldInfo {
                name: row.name.clone(),
                field_type: row.field_type.clone().unwrap_or_default(),
                description: row.description.clone(),
            });
    }

    // 全局 collections：DB 已移除 meta_collections.modules，不再按模块分组
    let collections: Vec<StateCollectionInfo> = col_rows
        .into_iter()
        .map(|row| StateCollectionInfo {
            id: 0,
            name: row.name.clone(),
            table_name: row.table_name.clone().unwrap_or_default(),
            parent_table: None,
            inheritance_depth: 0,
            child_tables: vec![],
            fields: fields_map.remove(&row.name).unwrap_or_default(),
        })
        .collect();

    // 模块元数据从 Pre-Proc/{namespace}/Sources/Modules/*/module.json 发现，与 DB 集合解耦
    let modules = discover_modules_from_namespace(namespace).await;

    PlatformCatalog {
        modules,
        collections,
        scenes: scenes
            .into_iter()
            .map(|r| StateSceneInfo {
                id: r.id,
                code: r.code,
                notice: r.notice,
            })
            .collect(),
        factors: factors
            .into_iter()
            .map(|r| StateFactorInfo {
                id: r.id,
                code: r.code,
                notice: r.notice,
            })
            .collect(),
        functions: functions
            .into_iter()
            .map(|r| StateFunctionInfo {
                id: r.id,
                code: r.code,
                notice: r.notice,
            })
            .collect(),
        status_bases: Vec::new(),
        lifecycle_entities: vec![
            "zc_id_bill".to_string(),
            "zc_id_event".to_string(),
            "zc_id_entity".to_string(),
            "zc_id_status".to_string(),
        ],
        inheritance: load_inheritance_tree(pool).await.unwrap_or_default(),
    }
}

/// 从 Pre-Proc/{namespace}/Sources/Modules 目录发现模块元数据。
///
/// 与 DB 集合完全解耦：module.json 提供模块 id/name/语义覆盖，
/// 集合详情统一进入 PlatformCatalog.collections。
async fn discover_modules_from_namespace(namespace: &str) -> Vec<ModuleInfo> {
    let modules_dir = crate::composer::resolve_project_root()
        .join("Pre-Proc")
        .join(namespace)
        .join("Sources")
        .join("Modules");

    let mut entries = match tokio::fs::read_dir(&modules_dir).await {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    let mut modules = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        let Ok(file_type) = entry.file_type().await else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        let module_json_path = modules_dir.join(&id).join("module.json");
        let mut module = ModuleInfo {
            id: id.clone(),
            name: id.clone(),
            collections: vec![],
            extension_points: vec![],
        };
        if let Ok(content) = tokio::fs::read_to_string(&module_json_path).await {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
                    module.name = name.to_string();
                }
                // 读取 extensionPoints
                if let Some(eps) = json.get("extensionPoints") {
                    if let Ok(points) =
                        serde_json::from_value::<runtime_engine::ModuleExtensionPoints>(eps.clone())
                    {
                        module.extension_points = vec![points];
                    }
                }
            }
        }
        modules.push(module);
    }

    modules.sort_by(|a, b| a.id.cmp(&b.id));
    modules
}

async fn load_inheritance_tree(
    pool: &PgPool,
) -> Result<Vec<crate::state::InheritanceEntry>, sqlx::Error> {
    #[derive(Debug, Clone, sqlx::FromRow)]
    struct InheritRow {
        depth: i32,
        child_table: String,
        parent_table: String,
        path: String,
    }
    let rows = sqlx::query_as::<_, InheritRow>(
        "SELECT depth, sub AS child_table, parent AS parent_table, path FROM isahl_meta.devv_inherits_union ORDER BY depth, parent, sub"
    )
    .fetch_all(pool)
    .await?;

    // Group children by parent for efficient lookup
    use std::collections::HashMap;
    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
    for row in &rows {
        children_map
            .entry(row.parent_table.clone())
            .or_default()
            .push(row.child_table.clone());
    }

    Ok(rows
        .into_iter()
        .map(|r| crate::state::InheritanceEntry {
            depth: r.depth,
            child_table: r.child_table.clone(),
            parent_table: r.parent_table.clone(),
            path: r.path,
            children: children_map.remove(&r.child_table).unwrap_or_default(),
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInfo {
    pub name: String,
    pub field_type: String,
    pub description: Option<String>,
}

// ──────────────────────────────────────────────────────────────────────────
// YAML 操作工具（META_AI_SPEC §7）
// ──────────────────────────────────────────────────────────────────────────

fn extensions_dir(namespace: &str, app_name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("Pre-Proc")
        .join(namespace)
        .join("Apps")
        .join(app_name)
        .join("extensions")
}

/// 写入 `Pre-Proc/{ns}/Apps/{app}/request-no-impl/gap-{slug}.md` 缺口文档。
///
/// 用于记录 App Agent 无法直接实现的需求（未编译模块、本体缺口等），
/// 不丢失需求，但不生成路由/不建表。
pub async fn write_request_no_impl(
    namespace: &str,
    app_name: &str,
    slug: &str,
    content: &str,
) -> Result<(), String> {
    let safe_slug = slug.replace("..", "").replace(['/', '\\'], "-");
    let dir = std::path::PathBuf::from("Pre-Proc")
        .join(namespace)
        .join("Apps")
        .join(app_name)
        .join("request-no-impl");
    if let Err(e) = tokio::fs::create_dir_all(&dir).await {
        return Err(format!("create request-no-impl dir failed: {}", e));
    }
    let path = dir.join(format!("gap-{}.md", safe_slug));
    tokio::fs::write(&path, content)
        .await
        .map_err(|e| format!("write {} failed: {}", path.display(), e))
}

/// 读取 Gateway 编译进单体的模块 ID 集合。
///
/// 解析 `Gateway/backend/Cargo.toml` 的 `[features].all-modules` 列表
/// （`module-<id>` → `<id>`）。这是文件系统级别的跨容器契约（非 HTTP 调用，
/// 遵 CONTAINER_BOUNDARY）。解析失败返回空集，调用方据此跳过校验。
/// 优先级：feature_manifest.json（Gateway build.rs 输出） > Cargo.toml 解析
pub fn compiled_module_ids() -> std::collections::HashSet<String> {
    use std::collections::HashSet;
    let mut ids: HashSet<String> = HashSet::new();

    // 1. 优先读取 Gateway 编译产出的 compiled_modules.json
    let manifest_path = std::env::var("FEATURE_MANIFEST_PATH")
        .unwrap_or_else(|_| "../../Pre-Proc/compiled_modules.json".to_string());
    if let Ok(content) = std::fs::read_to_string(&manifest_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(arr) = json.get("compiled_module_ids").and_then(|v| v.as_array()) {
                for id in arr {
                    if let Some(s) = id.as_str() {
                        ids.insert(s.to_string());
                    }
                }
                if !ids.is_empty() {
                    common::telemetry::info!(
                        "Loaded {} compiled module IDs from compiled_modules.json",
                        ids.len()
                    );
                    return ids;
                }
            }
        }
    }

    // 2. Fallback: 解析 Gateway/backend/Cargo.toml 的 all-modules feature
    common::telemetry::info!(
        "compiled_modules.json not found or empty; falling back to Cargo.toml parsing"
    );
    let cargo_path = std::env::var("GATEWAY_CARGO_TOML")
        .unwrap_or_else(|_| "../../Gateway/backend/Cargo.toml".to_string());
    let content = match std::fs::read_to_string(&cargo_path) {
        Ok(c) => c,
        Err(e) => {
            common::telemetry::warn!("compiled_module_ids: failed to read {}: {}", cargo_path, e);
            return ids;
        }
    };
    // 提取 all-modules = [ ... ] 块内的所有 "module-xxx" 条目。
    let mut in_block = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if !in_block {
            if trimmed.starts_with("all-modules") && trimmed.contains('[') {
                in_block = true;
                if trimmed.contains(']') {
                    collect_module_features(trimmed, &mut ids);
                    break;
                }
            }
            continue;
        }
        if trimmed.contains(']') {
            collect_module_features(trimmed, &mut ids);
            break;
        }
        collect_module_features(trimmed, &mut ids);
    }
    ids
}

fn collect_module_features(line: &str, ids: &mut std::collections::HashSet<String>) {
    // 匹配形如 "module-xxx" 的带引号条目，剥离 module- 前缀。
    for part in line.split(',') {
        let token = part.trim().trim_matches(['"', '[', ']', ' ']);
        if let Some(id) = token.strip_prefix("module-") {
            if !id.is_empty() {
                ids.insert(id.to_string());
            }
        }
    }
}

fn sanitize_file_name(name: &str) -> Result<String, String> {
    let sanitized = name.replace("..", "").replace(['/', '\\'], "");
    if sanitized.is_empty() || !sanitized.ends_with(".yaml") {
        return Err(format!("Invalid extension file name: {}", name));
    }
    Ok(sanitized)
}
pub async fn read_extension_yaml(namespace: &str, app_name: &str, file_name: &str) -> ToolResult {
    let file_name = match sanitize_file_name(file_name) {
        Ok(f) => f,
        Err(e) => return ToolResult::err("read_extension_yaml", e),
    };
    let path = extensions_dir(namespace, app_name).join(&file_name);
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => {
            // 预校验 YAML 格式
            match yaml_serde::from_str::<yaml_serde::Value>(&content) {
                Ok(value) => ToolResult::ok(
                    "read_extension_yaml",
                    serde_json::json!({
                        "file": file_name,
                        "content": content,
                        "parsed": value,
                    }),
                ),
                Err(e) => {
                    ToolResult::err("read_extension_yaml", format!("YAML parse error: {}", e))
                }
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => ToolResult::err(
            "read_extension_yaml",
            format!("File not found: {}", path.display()),
        ),
        Err(e) => ToolResult::err("read_extension_yaml", e),
    }
}

/// 写入/覆盖 extension YAML 文件
pub async fn write_extension_yaml(
    namespace: &str,
    app_name: &str,
    file_name: &str,
    content: &str,
) -> ToolResult {
    let file_name = match sanitize_file_name(file_name) {
        Ok(f) => f,
        Err(e) => return ToolResult::err("write_extension_yaml", e),
    };
    if let Err(e) = yaml_serde::from_str::<yaml_serde::Value>(content) {
        return ToolResult::err(
            "write_extension_yaml",
            format!("Invalid YAML content: {}", e),
        );
    }
    let dir = extensions_dir(namespace, app_name);
    let path = dir.join(&file_name);
    if let Err(e) = tokio::fs::create_dir_all(&dir).await {
        return ToolResult::err("write_extension_yaml", e);
    }
    if tokio::fs::metadata(&path).await.is_ok() {
        let bak = path.with_extension("yaml.bak");
        if let Err(e) = tokio::fs::copy(&path, &bak).await {
            return ToolResult::err("write_extension_yaml", format!("Backup failed: {}", e));
        }
    }
    match tokio::fs::write(&path, content).await {
        Ok(_) => ToolResult::ok(
            "write_extension_yaml",
            serde_json::json!({
                "file": file_name,
                "path": path.to_string_lossy().to_string(),
                "bytes_written": content.len(),
            }),
        ),
        Err(e) => ToolResult::err("write_extension_yaml", e),
    }
}
/// 列出 extensions 目录下的所有 YAML 文件
pub async fn list_extension_files(namespace: &str, app_name: &str) -> ToolResult {
    let dir = extensions_dir(namespace, app_name);
    match tokio::fs::read_dir(&dir).await {
        Ok(mut entries) => {
            let mut files = Vec::new();
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".yaml") && !name.ends_with(".bak") {
                    let meta = entry.metadata().await.ok();
                    files.push(serde_json::json!({
                        "name": name,
                        "size": meta.as_ref().map(|m| m.len()),
                        "modified": meta.as_ref().and_then(|m| m.modified().ok())
                            .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339()),
                    }));
                }
            }
            ToolResult::ok(
                "list_extension_files",
                serde_json::json!({ "files": files }),
            )
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            ToolResult::ok("list_extension_files", serde_json::json!({ "files": [] }))
        }
        Err(e) => ToolResult::err("list_extension_files", e),
    }
}

/// 结构化 Patch YAML 文件
///
/// path 表达式支持：
/// - 数组索引: "constraints[0].expression"
/// - 对象键匹配: "rules/{name=check_price}"
/// - 嵌套路径: "workflows[0].steps[1].action"
pub async fn patch_extension_yaml(
    namespace: &str,
    app_name: &str,
    file_name: &str,
    patches: &[crate::state::YamlPatch],
) -> ToolResult {
    let file_name = match sanitize_file_name(file_name) {
        Ok(f) => f,
        Err(e) => return ToolResult::err("patch_extension_yaml", e),
    };
    let path = extensions_dir(namespace, app_name).join(&file_name);

    // 读取现有 YAML
    let content = match tokio::fs::read_to_string(&path).await {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return ToolResult::err(
                "patch_extension_yaml",
                format!("File not found: {}", path.display()),
            );
        }
        Err(e) => return ToolResult::err("patch_extension_yaml", e),
    };

    let mut root: yaml_serde::Value = match yaml_serde::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            return ToolResult::err("patch_extension_yaml", format!("YAML parse error: {}", e))
        }
    };

    let mut applied = Vec::new();
    let mut errors = Vec::new();

    for patch in patches {
        match apply_yaml_patch(&mut root, &patch.path, patch.value.clone()) {
            Ok(()) => applied.push(patch.path.clone()),
            Err(e) => errors.push(format!("{}: {}", patch.path, e)),
        }
    }

    if !errors.is_empty() && applied.is_empty() {
        return ToolResult::err(
            "patch_extension_yaml",
            format!("All patches failed: {}", errors.join("; ")),
        );
    }

    // 序列化回 YAML
    let new_yaml = match yaml_serde::to_string(&root) {
        Ok(y) => y,
        Err(e) => {
            return ToolResult::err("patch_extension_yaml", format!("Serialize error: {}", e))
        }
    };

    // 写入
    match tokio::fs::write(&path, &new_yaml).await {
        Ok(_) => ToolResult::ok(
            "patch_extension_yaml",
            serde_json::json!({
                "file": file_name,
                "applied": applied,
                "errors": errors,
                "new_content": new_yaml,
            }),
        ),
        Err(e) => ToolResult::err("patch_extension_yaml", e),
    }
}

/// 将 serde_json::Value 转换为 yaml_serde::Value
fn json_to_yaml(v: serde_json::Value) -> Result<yaml_serde::Value, String> {
    let json_str = serde_json::to_string(&v).map_err(|e| format!("JSON serialize: {}", e))?;
    yaml_serde::from_str(&json_str).map_err(|e| format!("YAML deserialize: {}", e))
}

fn apply_yaml_patch(
    root: &mut yaml_serde::Value,
    path: &str,
    value: Option<serde_json::Value>,
) -> Result<(), String> {
    let segments: Vec<&str> = path.split('.').collect();
    if segments.is_empty() {
        return Err("Empty path".to_string());
    }

    let mut current = root;

    for (idx, segment) in segments.iter().enumerate() {
        let is_last = idx == segments.len() - 1;

        // 解析数组索引，如 "constraints[0]"
        let (key, array_idx) = if let Some(bracket_start) = segment.find('[') {
            let bracket_end = segment.find(']').ok_or("Unmatched bracket")?;
            let key = &segment[..bracket_start];
            let idx_str = &segment[bracket_start + 1..bracket_end];
            let idx: usize = idx_str.parse().map_err(|_| "Invalid array index")?;
            (key, Some(idx))
        } else {
            (*segment, None)
        };

        // 解析对象键匹配，如 "rules/{name=check_price}"
        let (obj_key, obj_match) = if let Some(brace_start) = key.find("/{") {
            let brace_end = key.find('}').ok_or("Unmatched brace")?;
            let obj_key = &key[..brace_start];
            let match_expr = &key[brace_start + 2..brace_end];
            let parts: Vec<&str> = match_expr.split('=').collect();
            if parts.len() != 2 {
                return Err("Invalid match expression".to_string());
            }
            (obj_key, Some((parts[0].to_string(), parts[1].to_string())))
        } else {
            (key, None)
        };

        if is_last {
            // 最终段：设置/删除值
            match (array_idx, obj_match) {
                (Some(idx), _) => {
                    let arr = current.as_sequence_mut().ok_or("Expected array")?;
                    if idx >= arr.len() {
                        return Err("Array index out of bounds".to_string());
                    }
                    if let Some(v) = value {
                        arr[idx] = json_to_yaml(v)?;
                    } else {
                        arr.remove(idx);
                    }
                }
                (None, Some((match_key, match_val))) => {
                    let seq = current
                        .as_sequence_mut()
                        .ok_or("Expected array for object match")?;
                    let found = seq
                        .iter_mut()
                        .find(|item| {
                            item.as_mapping()
                                .and_then(|m| m.get(yaml_serde::Value::String(match_key.clone())))
                                .map(|v| v.as_str() == Some(&match_val))
                                .unwrap_or(false)
                        })
                        .ok_or("No matching object found")?;
                    if let Some(v) = value {
                        *found = json_to_yaml(v)?;
                    } else {
                        // 标记删除，但这里简化处理：将整个对象设为 Null
                        *found = yaml_serde::Value::Null;
                    }
                }
                (None, None) => {
                    let map = current.as_mapping_mut().ok_or("Expected mapping")?;
                    if let Some(v) = value {
                        map.insert(
                            yaml_serde::Value::String(obj_key.to_string()),
                            json_to_yaml(v)?,
                        );
                    } else {
                        map.remove(yaml_serde::Value::String(obj_key.to_string()));
                    }
                }
            }
            return Ok(());
        }

        // 非最终段：导航到下一层
        match (array_idx, obj_match) {
            (Some(idx), _) => {
                let arr = current.as_sequence_mut().ok_or("Expected array")?;
                if idx >= arr.len() {
                    return Err("Array index out of bounds".to_string());
                }
                current = &mut arr[idx];
            }
            (None, Some((match_key, match_val))) => {
                let seq = current
                    .as_sequence_mut()
                    .ok_or("Expected array for object match")?;
                current = seq
                    .iter_mut()
                    .find(|item| {
                        item.as_mapping()
                            .and_then(|m| m.get(yaml_serde::Value::String(match_key.clone())))
                            .map(|v| v.as_str() == Some(&match_val))
                            .unwrap_or(false)
                    })
                    .ok_or("No matching object found")?;
            }
            (None, None) => {
                let map = current.as_mapping_mut().ok_or("Expected mapping")?;
                let key_val = yaml_serde::Value::String(obj_key.to_string());
                if !map.contains_key(&key_val) {
                    map.insert(
                        key_val.clone(),
                        yaml_serde::Value::Mapping(Default::default()),
                    );
                }
                current = map.get_mut(&key_val).unwrap();
            }
        }
    }

    Ok(())
}

// ══════════════════════════════════════════════════════════════════════════
// 7-Stage Pipeline Tools: Module/Scene/Factor Scaffold Generators
// ══════════════════════════════════════════════════════════════════════════

/// Create a module.json scaffold from a FlowPlan.
/// Writes to Pre-Proc/{namespace}/Sources/Modules/{id}/module.json
/// （与 create_block_scaffold/create_service_scaffold 及 Gateway 发现的 Pre-Proc 布局一致；
/// 原 `Modules/{namespace}/{id}` 路径错误，导致 alioth-module 门禁找不到 module.json）。
pub async fn create_module_scaffold(
    project_root: &str,
    namespace: &str,
    module_id: &str,
    name: &str,
    description: &str,
    block_ids: &[String],
) -> Result<String, String> {
    let dir = std::path::Path::new(project_root)
        .join("Pre-Proc")
        .join(namespace)
        .join("Sources")
        .join("Modules")
        .join(module_id);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| e.to_string())?;

    let first_id = block_ids
        .first()
        .cloned()
        .unwrap_or_else(|| "default".to_string());

    let group_id = "default";

    let module = serde_json::json!({
        "id": module_id,
        "namespace": namespace,
        "name": name,
        "description": description,
        "category": "business",
        "status": "active",
        "version": "0.1.0",
        "routePrefix": format!("/{}", module_id),
        "icon": "Package",
        "hasBackend": true,
        "hasFrontend": true,
        "hasWebview": false,
        "layer": 0,
        "blocks": block_ids.iter().map(|id| {
            serde_json::json!({ "id": id, "group": group_id })
        }).collect::<Vec<_>>(),
        "blockAssembly": {
            "mode": "multi-block",
            "shell": "ModuleLayout",
            "navigation": {
                "groups": [
                    { "id": group_id, "label": "Default", "icon": "FileText" }
                ],
                "defaultBlock": first_id,
                "collapseBehavior": "width"
            },
            "blocks": block_ids.iter().enumerate().map(|(i, id)| {
                serde_json::json!({
                    "id": id,
                    "label": id,
                    "group": group_id,
                    "order": i,
                    "icon": "FileText"
                })
            }).collect::<Vec<_>>(),
            "stateContract": {
                "shared": ["globalQuery", "userContext"],
                "isolated": ["search", "filter", "page", "selectedId"]
            },
            "serviceBindings": serde_json::Value::Object(Default::default())
        },
        "min_alioth_version": "10.0.0"
    });

    let path = dir.join("module.json");
    let content = serde_json::to_string_pretty(&module).map_err(|e| e.to_string())?;
    tokio::fs::write(&path, &content)
        .await
        .map_err(|e| e.to_string())?;
    common::telemetry::info!("Created module scaffold: {}", path.display());
    Ok(module_id.to_string())
}

/// Create a block.json scaffold.
/// Writes to Pre-Proc/{namespace}/Sources/Blocks/{id}/block.json.
pub async fn create_block_scaffold(
    project_root: &str,
    namespace: &str,
    block_id: &str,
    name: &str,
    _factor_ids: &[String],
) -> Result<String, String> {
    let dir = std::path::Path::new(project_root)
        .join("Pre-Proc")
        .join(namespace)
        .join("Sources")
        .join("Blocks")
        .join(block_id);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| e.to_string())?;

    let block = serde_json::json!({
        "id": block_id,
        "namespace": namespace,
        "name": name,
        "version": "0.1.0",
        "prototypeVersion": "v1",
        "services": [],
        "sharing": {
            "mode": "single",
            "ownerModule": format!("{}/{}", namespace, block_id),
            "consumers": []
        }
    });

    let path = dir.join("block.json");
    let content = serde_json::to_string_pretty(&block).map_err(|e| e.to_string())?;
    tokio::fs::write(&path, &content)
        .await
        .map_err(|e| e.to_string())?;
    common::telemetry::info!("Created block scaffold: {}", path.display());
    Ok(block_id.to_string())
}

/// Create a service.json scaffold.
/// Writes to Pre-Proc/{namespace}/Sources/Services/{id}/service.json.
pub async fn create_service_scaffold(
    project_root: &str,
    namespace: &str,
    service_id: &str,
    domain: &str,
    entity_names: &[String],
) -> Result<String, String> {
    let dir = std::path::Path::new(project_root)
        .join("Pre-Proc")
        .join(namespace)
        .join("Sources")
        .join("Services")
        .join(service_id);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| e.to_string())?;

    let entities: Vec<serde_json::Value> = entity_names
        .iter()
        .map(|name| {
            serde_json::json!({
                "name": name,
                "table": format!("zc_id_{}", name.to_lowercase()),
                "fields": []
            })
        })
        .collect();

    let service = serde_json::json!({
        "id": service_id,
        "namespace": namespace,
        "domain": domain,
        "layer": 0,
        "dtoDependencies": [],
        "dtoExposes": {
            "refs": entity_names.to_vec(),
            "queries": ["list_refs", "get_refs"]
        },
        "backendCrate": format!("{}-service-{}", namespace.to_lowercase(), service_id),
        "hasBackend": true,
        "hasFrontend": false,
        "version": "0.1.0",
        "publishes": [],
        "subscribes": [],
        "services": [],
        "ontology": {
            "entities": entities
        }
    });

    let path = dir.join("service.json");
    let content = serde_json::to_string_pretty(&service).map_err(|e| e.to_string())?;
    tokio::fs::write(&path, &content)
        .await
        .map_err(|e| e.to_string())?;
    common::telemetry::info!("Created service scaffold: {}", path.display());
    Ok(service_id.to_string())
}
/// Derive module/block/factor IDs from FlowPlan entities.
/// Returns (module_ids, block_ids, factor_ids).
pub fn derive_artifact_ids(
    plan: &crate::state::FlowPlan,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let domain = plan
        .known_entities
        .first()
        .cloned()
        .unwrap_or_else(|| "default".to_string());

    let module_slug = slugify(&domain);
    let module_ids = vec![format!("{}-app", module_slug)];
    let block_ids = plan
        .workflow_steps
        .iter()
        .map(|s| format!("{}-{}", module_slug, slugify(s)))
        .collect::<Vec<_>>();
    let factor_ids = vec![module_slug.clone()];

    (module_ids, block_ids, factor_ids)
}

fn slugify(s: &str) -> String {
    s.chars()
        .filter_map(|c| {
            if c.is_alphanumeric() {
                Some(c.to_ascii_lowercase())
            } else if c == ' ' || c == '-' || c == '_' {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
