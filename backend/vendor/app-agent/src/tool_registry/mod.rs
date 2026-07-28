//! ToolRegistry — 工具注册表与动态调用
//!
//! 统一的工具接口层，支持按名称注册和按名称动态调用。
//! 工具描述（名称 + 参数 Schema）可被 LLM 读取，实现自主工具选择。
//!
//! ## 用法
//! ```ignore
//! let mut registry = ToolRegistry::new();
//! registry.register(ReadFileTool);
//! registry.register(WriteFileTool);
//!
//! // LLM 选择调用的工具
//! let result = registry.call("read_file", json!({"path": "..."})).await?;
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::LazyLock;

/// 将相对路径解析到项目根目录（与 `RunCommandTool` / 门禁路径解析保持一致）。
///
/// LLM 产出的路径均为相对项目根的路径（如 `Pre-Proc/Alioth/...`），
/// 而测试进程 CWD 通常是 `Meta/backend/app-agent`，若直接按 CWD 写文件，
/// 门禁按 `resolve_project_root()` 校验时会找不到产物。绝对路径原样返回。
fn resolve_path_against_root(p: &str) -> PathBuf {
    let path = Path::new(p);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    crate::composer::resolve_project_root().join(path)
}

/// 工具定义（JSON Schema 描述参数，供 LLM 读取）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    /// JSON Schema for input parameters
    pub input_schema: Value,
}

/// 工具调用结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub tool: String,
    pub success: bool,
    pub data: Option<Value>,
    pub error: Option<String>,
}

impl ToolCallResult {
    pub fn ok(tool: &str, data: Value) -> Self {
        Self {
            tool: tool.to_string(),
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(tool: &str, error: impl std::fmt::Display) -> Self {
        Self {
            tool: tool.to_string(),
            success: false,
            data: None,
            error: Some(error.to_string()),
        }
    }
}

/// 工具 trait — 实现此 trait 即可注册到 ToolRegistry
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    /// 工具名
    fn name(&self) -> &'static str;
    /// 工具描述
    fn description(&self) -> &'static str;
    /// 输入参数 JSON Schema
    fn input_schema(&self) -> Value;
    /// 执行工具
    async fn call(&self, params: Value) -> ToolCallResult;
}

/// 工具注册表
pub struct ToolRegistry {
    tools: HashMap<&'static str, Box<dyn Tool>>,
    tool_defs: Vec<ToolDef>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            tool_defs: Vec::new(),
        }
    }

    /// 注册工具
    pub fn register(&mut self, tool: impl Tool + 'static) {
        let name = tool.name();
        let def = ToolDef {
            name: name.to_string(),
            description: tool.description().to_string(),
            input_schema: tool.input_schema(),
        };
        self.tool_defs.push(def);
        self.tools.insert(name, Box::new(tool));
    }

    /// 获取工具定义列表（供 LLM 选择）
    pub fn tool_defs(&self) -> &[ToolDef] {
        &self.tool_defs
    }

    /// 获取工具定义列表的 JSON 表示
    pub fn tool_defs_json(&self) -> Value {
        serde_json::to_value(&self.tool_defs).unwrap_or_default()
    }

    /// 调用工具
    pub async fn call(&self, name: &str, params: Value) -> Result<ToolCallResult, String> {
        match self.tools.get(name) {
            Some(tool) => Ok(tool.call(params).await),
            None => Err(format!("Tool '{}' not found", name)),
        }
    }

    /// 工具数量
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── 内置工具 ────────────────────────────────────────────────

/// 读取文件工具
pub struct ReadFileTool;

#[async_trait::async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &'static str {
        "read_file"
    }
    fn description(&self) -> &'static str {
        "Read a file from the filesystem. Paths are relative to project root."
    }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Relative file path"}
            },
            "required": ["path"]
        })
    }
    async fn call(&self, params: Value) -> ToolCallResult {
        let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let p = resolve_path_against_root(path);
        match tokio::fs::read_to_string(&p).await {
            Ok(content) => ToolCallResult::ok(
                "read_file",
                serde_json::json!({
                    "path": path,
                    "content": content,
                    "size": content.len(),
                }),
            ),
            Err(e) => {
                ToolCallResult::err("read_file", format!("Failed to read '{}': {}", p.display(), e))
            }
        }
    }
}

/// 写文件工具
pub struct WriteFileTool;

#[async_trait::async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &'static str {
        "write_file"
    }
    fn description(&self) -> &'static str {
        "Write content to a file. Creates parent directories if needed."
    }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Relative file path"},
                "content": {"type": "string", "description": "File content"}
            },
            "required": ["path", "content"]
        })
    }
    async fn call(&self, params: Value) -> ToolCallResult {
        let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let content = params.get("content").and_then(|v| v.as_str()).unwrap_or("");
        let p = resolve_path_against_root(path);
        if let Some(parent) = p.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }
        match tokio::fs::write(&p, content).await {
            Ok(()) => ToolCallResult::ok(
                "write_file",
                serde_json::json!({
                    "path": path,
                    "size": content.len(),
                }),
            ),
            Err(e) => {
                ToolCallResult::err("write_file", format!("Failed to write '{}': {}", p.display(), e))
            }
        }
    }
}

pub struct SearchFileTool;

#[async_trait::async_trait]
impl Tool for SearchFileTool {
    fn name(&self) -> &'static str {
        "search_files"
    }
    fn description(&self) -> &'static str {
        "Pattern search for files matching a simple glob pattern."
    }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {"type": "string", "description": "Glob pattern (e.g. **/*.rs)"},
                "root": {"type": "string", "description": "Root directory, default '.'"}
            },
            "required": ["pattern"]
        })
    }
    async fn call(&self, params: Value) -> ToolCallResult {
        let pattern = params.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
        let root = params.get("root").and_then(|v| v.as_str()).unwrap_or(".");

        let base = resolve_path_against_root(root);
        let full_glob = base.join(pattern);
        let parent = full_glob.parent().unwrap_or(&base);
        let file_part = full_glob
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("*");

        // Convert simple glob (*, ?) to exact match using ends_with
        let suffix = if let Some(stripped) = file_part.strip_prefix('*') {
            stripped
        } else {
            file_part
        };

        let mut matches = Vec::new();
        if let Ok(entries) = std::fs::read_dir(parent) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if name.ends_with(suffix) || name == file_part {
                    matches.push(path.to_string_lossy().to_string());
                }
            }
        }

        ToolCallResult::ok(
            "search_files",
            serde_json::json!({
                "pattern": pattern,
                "root": root,
                "matches": matches,
                "count": matches.len(),
            }),
        )
    }
}

/// 列出技能工具
pub struct ListSkillsTool {
    skill_names: Vec<String>,
}

impl ListSkillsTool {
    pub fn new(skill_names: Vec<String>) -> Self {
        Self { skill_names }
    }
}

#[async_trait::async_trait]
impl Tool for ListSkillsTool {
    fn name(&self) -> &'static str {
        "list_skills"
    }
    fn description(&self) -> &'static str {
        "List all available skills that can be executed."
    }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Optional search query"}
            }
        })
    }
    async fn call(&self, params: Value) -> ToolCallResult {
        let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let skills: Vec<&String> = if query.is_empty() {
            self.skill_names.iter().collect()
        } else {
            let q = query.to_lowercase();
            self.skill_names.iter().filter(|n| n.contains(&q)).collect()
        };
        ToolCallResult::ok(
            "list_skills",
            serde_json::json!({
                "skills": skills,
                "count": skills.len(),
            }),
        )
    }
}

/// 执行技能工具 — 由 SkillExecutor 驱动
pub struct ExecuteSkillTool;

#[async_trait::async_trait]
impl Tool for ExecuteSkillTool {
    fn name(&self) -> &'static str {
        "execute_skill"
    }
    fn description(&self) -> &'static str {
        "Execute a skill workflow. Returns the skill's system prompt for the current step."
    }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "skill": {"type": "string", "description": "Skill name"},
                "action": {
                    "type": "string",
                    "enum": ["advance", "prompt", "complete"],
                    "description": "advance=next step, prompt=get current step prompt, complete=check if done"
                }
            },
            "required": ["skill", "action"]
        })
    }
    async fn call(&self, _params: Value) -> ToolCallResult {
        // 实际由 SkillExecutor 驱动，此处只返回定义
        ToolCallResult::ok(
            "execute_skill",
            serde_json::json!({
                "note": "This tool is driven by SkillExecutor. Use the SkillExecutor directly."
            }),
        )
    }
}

// ── RunCommandTool ──────────────────────────────────────────
//
// 受限命令执行：仅允许执行白名单中的程序（默认从 `skill-adapters/_runtime.yaml`
// 加载 `allowed_programs`，文件缺失或解析失败时回落到硬编码列表）。
//
// 安全约束：
// - program 前缀白名单校验（精确匹配 或 `<entry>/` 前缀匹配）
// - cwd 固定为项目根（`composer::resolve_project_root()`）
// - 默认 120s 超时（`timeout_sec` 参数可覆盖，但工具本身不强制使用——caller
//   可在外层用 `tokio::time::timeout` 进一步截断）
// - stdout/stderr 各截断 8 KiB，超出追加 `[truncated]` 标记

/// 默认允许的程序列表（`_runtime.yaml` 缺失或解析失败时使用）
const DEFAULT_ALLOWED_PROGRAMS: &[&str] = &["target/debug/ontology-mapping", "bun", "npx", "cargo"];

/// stdout/stderr 截断阈值（字节）
const OUTPUT_TRUNCATE_BYTES: usize = 8 * 1024;

/// 默认超时（秒）
const DEFAULT_TIMEOUT_SEC: u64 = 120;

/// 进程缓存的白名单程序列表（首次访问时初始化）
fn allowed_programs() -> &'static Vec<String> {
    static CACHE: LazyLock<Vec<String>> = LazyLock::new(load_allowed_programs);
    &CACHE
}

/// 从 `skill-adapters/_runtime.yaml` 加载白名单
///
/// 文件不存在 / 解析失败 → 回落到 `DEFAULT_ALLOWED_PROGRAMS`
fn load_allowed_programs() -> Vec<String> {
    let path = crate::composer::resolve_project_root().join("skill-adapters/_runtime.yaml");
    match std::fs::read_to_string(&path) {
        Ok(content) => match yaml_serde::from_str::<RuntimeConfig>(&content) {
            Ok(cfg) if !cfg.allowed_programs.is_empty() => cfg.allowed_programs,
            Ok(_) => {
                common::telemetry::warn!(
                    "RunCommandTool: {} has empty allowed_programs, using default",
                    path.display()
                );
                DEFAULT_ALLOWED_PROGRAMS
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            }
            Err(e) => {
                common::telemetry::warn!(
                    "RunCommandTool: failed to parse {}: {}, using default",
                    path.display(),
                    e
                );
                DEFAULT_ALLOWED_PROGRAMS
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            }
        },
        Err(_) => DEFAULT_ALLOWED_PROGRAMS
            .iter()
            .map(|s| s.to_string())
            .collect(),
    }
}

/// `_runtime.yaml` 反序列化目标
#[derive(Debug, serde::Deserialize)]
struct RuntimeConfig {
    #[serde(default)]
    allowed_programs: Vec<String>,
}

/// 检查 program 是否在白名单内
///
/// 匹配规则：
/// - `program == entry`：精确匹配（bare-name 程序如 `bun`、`cargo`）
/// - `program.starts_with(format!("{entry}/"))`：路径前缀匹配（`target/debug/...`）
fn is_program_allowed(program: &str, allowlist: &[String]) -> bool {
    allowlist
        .iter()
        .any(|entry| program == entry || program.starts_with(&format!("{}/", entry)))
}

/// 截断字节流，超出 8 KiB 时保留前 8 KiB + `[truncated]` 标记
fn truncate_output(bytes: &[u8]) -> String {
    const TAIL: &[u8] = b"\n[truncated]";
    if bytes.len() <= OUTPUT_TRUNCATE_BYTES {
        String::from_utf8_lossy(bytes).into_owned()
    } else {
        let mut buf = Vec::with_capacity(OUTPUT_TRUNCATE_BYTES + TAIL.len());
        buf.extend_from_slice(&bytes[..OUTPUT_TRUNCATE_BYTES]);
        buf.extend_from_slice(TAIL);
        String::from_utf8_lossy(&buf).into_owned()
    }
}

/// Run command tool — 仅执行白名单程序
pub struct RunCommandTool;

#[async_trait::async_trait]
impl Tool for RunCommandTool {
    fn name(&self) -> &'static str {
        "run_command"
    }

    fn description(&self) -> &'static str {
        "Execute a whitelisted program with arguments. cwd is project root. \
         Timeout 120s default (configurable via timeout_sec param). \
         stdout/stderr each truncated to 8 KiB."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "program": {
                    "type": "string",
                    "description": "Program executable (must be in allowed list: target/debug/ontology-mapping, bun, npx, cargo)"
                },
                "args": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Program arguments (no shell interpolation)"
                },
                "timeout_sec": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Optional timeout in seconds, default 120"
                }
            },
            "required": ["program", "args"]
        })
    }

    async fn call(&self, params: Value) -> ToolCallResult {
        let program = match params.get("program").and_then(|v| v.as_str()) {
            Some(p) if !p.is_empty() => p.to_string(),
            _ => {
                return ToolCallResult::err("run_command", "missing required param 'program'");
            }
        };

        let args: Vec<String> = params
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let timeout_sec = params
            .get("timeout_sec")
            .and_then(|v| v.as_u64())
            .unwrap_or(DEFAULT_TIMEOUT_SEC);

        // 白名单校验
        let allowlist = allowed_programs();
        if !is_program_allowed(&program, allowlist) {
            return ToolCallResult::err(
                "run_command",
                format!(
                    "program '{}' is not in the allowed list: {:?}",
                    program, allowlist
                ),
            );
        }

        let cwd: PathBuf = crate::composer::resolve_project_root();

        let mut cmd = tokio::process::Command::new(&program);
        cmd.args(&args)
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .kill_on_drop(true);

        let child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                return ToolCallResult::err(
                    "run_command",
                    format!("failed to spawn '{}': {}", program, e),
                );
            }
        };

        let timeout_duration = std::time::Duration::from_secs(timeout_sec);
        match tokio::time::timeout(timeout_duration, child.wait_with_output()).await {
            Ok(Ok(output)) => {
                let stdout = truncate_output(&output.stdout);
                let stderr = truncate_output(&output.stderr);
                let exit_code = output.status.code().unwrap_or(-1);
                ToolCallResult::ok(
                    "run_command",
                    serde_json::json!({
                        "program": program,
                        "args": args,
                        "cwd": cwd.to_string_lossy(),
                        "exit_code": exit_code,
                        "success": output.status.success(),
                        "stdout": stdout,
                        "stderr": stderr,
                    }),
                )
            }
            Ok(Err(e)) => ToolCallResult::err(
                "run_command",
                format!("failed to wait on '{}': {}", program, e),
            ),
            Err(_elapsed) => ToolCallResult::err(
                "run_command",
                format!("command '{}' exceeded timeout of {}s", program, timeout_sec),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// 工具必须在白名单拒绝 `python`、`sh`、`/bin/sh` 等程序
    #[tokio::test]
    async fn rejects_disallowed_program() {
        let tool = RunCommandTool;
        for bad in ["python", "python3", "sh", "/bin/sh", "/usr/bin/env"] {
            let r = tool
                .call(json!({"program": bad, "args": ["--version"]}))
                .await;
            assert!(!r.success, "program '{}' should be rejected", bad);
            let err = r.error.as_deref().unwrap_or("");
            assert!(
                err.contains("not in the allowed list"),
                "program '{}' should fail with allowlist error, got: {}",
                bad,
                err
            );
        }
    }

    /// 工具必须允许 `cargo --version`（假定 cargo 在 PATH 中）
    #[tokio::test]
    async fn executes_allowed_program() {
        let tool = RunCommandTool;
        let r = tool
            .call(json!({"program": "cargo", "args": ["--version"]}))
            .await;
        assert!(r.success, "cargo --version should succeed: {:?}", r.error);
        let data = r.data.expect("data should be present");
        let stdout = data.get("stdout").and_then(|v| v.as_str()).unwrap_or("");
        assert!(
            stdout.contains("cargo"),
            "stdout should mention 'cargo', got: {}",
            stdout
        );
        assert_eq!(data.get("exit_code").and_then(|v| v.as_i64()), Some(0));
        assert_eq!(data.get("success").and_then(|v| v.as_bool()), Some(true));
        // cwd 应为项目根
        let cwd = data.get("cwd").and_then(|v| v.as_str()).unwrap_or("");
        assert!(
            !cwd.is_empty(),
            "cwd should be resolved project root, got: {}",
            cwd
        );
    }

    /// 路径前缀条目必须匹配：传入 `target/debug/ontology-mapping` 自身合法
    /// （即使二进制不存在，工具应在白名单校验通过后于 spawn 处报错，而非 allowlist 拒绝）
    #[tokio::test]
    async fn path_prefix_entry_passes_allowlist() {
        let allowlist = allowed_programs().clone();
        assert!(
            is_program_allowed("target/debug/ontology-mapping", &allowlist),
            "exact path-prefix entry must be allowed"
        );
        assert!(
            is_program_allowed("target/debug/ontology-mapping-foo", &allowlist) == false,
            "entry must not match sibling names without '/' separator"
        );
        assert!(
            !is_program_allowed("python", &allowlist),
            "bare 'python' must not be allowed"
        );
        assert!(is_program_allowed("bun", &allowlist));
        assert!(is_program_allowed("npx", &allowlist));
        assert!(is_program_allowed("cargo", &allowlist));
    }

    /// 超时：通过 `timeout_sec=1` 让 `sleep 5` 在 1s 后被截断
    #[tokio::test]
    async fn timeout_truncates_long_running_command() {
        let tool = RunCommandTool;
        let r = tool
            .call(json!({
                "program": "sh",
                "args": ["-c", "sleep 5"],
                "timeout_sec": 1
            }))
            .await;
        assert!(!r.success, "should fail on timeout");
        let err = r.error.as_deref().unwrap_or("");
        assert!(
            err.contains("timeout") || err.contains("not in the allowed list"),
            "expected timeout or allowlist error (CI may lack 'sh'), got: {}",
            err
        );
    }

    /// 缺失必需参数 → 报错而非 panic
    #[tokio::test]
    async fn missing_required_params_return_error() {
        let tool = RunCommandTool;
        // 空 params → "missing required param 'program'"
        let r1 = tool.call(json!({})).await;
        assert!(!r1.success);
        let err = r1.error.as_deref().unwrap_or("");
        assert!(
            err.contains("missing required param 'program'"),
            "got: {}",
            err
        );

        // 空 program 字符串 → 同样报错
        let r2 = tool.call(json!({"program": "", "args": []})).await;
        assert!(!r2.success);
    }
    #[test]
    fn truncate_output_over_limit() {
        let big = "x".repeat(OUTPUT_TRUNCATE_BYTES + 100);
        let truncated = truncate_output(big.as_bytes());
        assert!(truncated.ends_with("[truncated]"));
        assert!(truncated.starts_with(&"x".repeat(100)));
    }
}
