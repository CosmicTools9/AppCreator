//! AppAgent 执行流水日志 — 全量日志模型 + 内存缓冲 + 磁盘持久化
//!
//! # 三通道暴露
//!
//! 1. 内存：`ConversationContext.execution_log` Vec — 供 REST API `/execution-log` 实时读取
//! 2. 磁盘：`execution.log`（JSON Lines）— 由 `rebuild_execution_log()` 持久化到 `output_path`
//! 3. WebSocket：通过 `AgentProgress` 的 `event_kind: "execution_log"` 推送
//!
//! # 日志模型
//!
//! - `ExecutionEvent` — tagged union，每种事件有自己的结构化字段
//! - `ExecutionLogEntry` — 一次日志记录（含 session_id / timestamp / level）
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 日志级别
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
        }
    }
}

/// 执行事件 — tagged union
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum ExecutionEvent {
    /// 状态机状态转移
    StateTransition {
        from: String,
        to: String,
        elapsed_ms: u64,
    },
    /// LLM 调用
    LlmCall {
        model_id: String,
        reasoning_effort: String,
        prompt_len: usize,
        response_len: usize,
        retry_count: u32,
        token_usage: Option<u32>,
        /// 各次重试的延迟（ms），首次即成功则单元素
        latencies_ms: Vec<u64>,
    },
    /// 子进程执行
    Subprocess {
        command: String,
        exit_code: Option<i32>,
        stdout: String,
        stderr: String,
        duration_ms: u64,
    },
    /// 文件写入
    FileWrite { path: String, size: u64 },
    /// 验证结果
    Validation {
        kind: String, // "app_json" | "extension_yaml" | "compilation" | "health_check"
        passed: bool,
        detail: String,
    },
    /// 技能步骤执行
    SkillStep {
        skill_name: String,
        track: usize,
        step: usize,
        completed: bool,
        summary: String,
    },
    /// 技能内工具调用（ExecutingSkill 真实执行 tool_call）
    ToolCall {
        tool: String,
        success: bool,
        detail: Option<String>,
    },
}

/// 一次完整的日志记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub session_id: i64,
    #[serde(flatten)]
    pub event: ExecutionEvent,
}

impl ExecutionLogEntry {
    /// JSON Lines 序列化（单行 compact JSON）
    pub fn to_json_line(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            format!(
                r#"{{"timestamp":"{}","level":"error","session_id":{},"event_type":"state_transition","from":"serialization_error","to":"serialization_error","elapsed_ms":0}}"#,
                Utc::now().to_rfc3339(),
                self.session_id
            )
        })
    }
}

/// 从 execution.log 文件读取所有日志条目
pub fn read_execution_log(path: &str) -> Result<Vec<ExecutionLogEntry>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read execution.log: {}", e))?;

    let mut entries = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str::<ExecutionLogEntry>(line) {
            Ok(entry) => entries.push(entry),
            Err(e) => {
                common::telemetry::warn!("execution.log line {} parse error: {}", i + 1, e);
            }
        }
    }
    Ok(entries)
}

/// 从 ConversationContext 的 execution_log Vec 重建 execution.log 文件
pub fn rebuild_execution_log(output_path: &str, entries: &[ExecutionLogEntry]) {
    use std::io::Write;
    let log_path = format!("{}/execution.log", output_path);
    let path = Path::new(&log_path);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::File::create(path) {
        Ok(mut file) => {
            for entry in entries {
                let _ = writeln!(file, "{}", entry.to_json_line());
            }
        }
        Err(e) => {
            common::telemetry::error!("Failed to rebuild execution.log at {}: {}", log_path, e);
        }
    }
}

/// 生成 AppAgent 状态机 Mermaid 图
///
/// `current_state`: 当前状态的调试表示（如 "Planning { revision_round: 0, needs_clarification: None }"）
/// `visited_states`: 已访问过的状态列表（按访问顺序）
pub fn generate_state_diagram(current_state: &str, visited_states: &[String]) -> String {
    // 所有状态定义（按执行流程分组）
    let all_states = [
        "Initializing",
        "SemanticAnalysis",
        "FunctionDecomposition",
        "OntologyAnalysis",
        "Planning",
        "Extending",
        "Generating",
        "GeneratingFrontend",
        "Composing",
        "Verifying",
        "Publishing",
        "Published",
        "Presenting",
        "ModuleCreation",
        "BlockCreation",
        "OntologyTransfer",
        "ServiceAPI",
        "ExecutingSkill",
    ];

    // 状态别名 — Mermaid 不支持状态名含特殊字符
    let state_alias = |s: &&str| -> &str {
        match *s {
            "Initializing" => "init",
            "SemanticAnalysis" => "semantic",
            "FunctionDecomposition" => "func_decomp",
            "OntologyAnalysis" => "ontology",
            "Planning" => "planning",
            "Extending" => "extending",
            "Generating" => "generating",
            "GeneratingFrontend" => "gen_frontend",
            "Composing" => "composing",
            "Verifying" => "verifying",
            "Publishing" => "publishing",
            "Published" => "published",
            "Presenting" => "presenting",
            "ModuleCreation" => "module_creat",
            "BlockCreation" => "block_creat",
            "OntologyTransfer" => "onto_transfer",
            "ServiceAPI" => "service_api",
            "ExecutingSkill" => "skill_exec",
            _ => "unknown",
        }
    };

    // 提取基础状态名（去掉参数部分）
    fn base_state(state: &str) -> String {
        state.split('{').next().unwrap_or(state).trim().to_string()
    }

    let base_current = base_state(current_state);

    let mut diagram = String::from("stateDiagram-v2\n");
    diagram.push_str("    direction LR\n\n");

    // classDef 必须在引用前声明
    diagram.push_str("    classDef current fill:#e0f0ff,stroke:#2563eb,stroke-width:3px\n");
    diagram.push_str("    classDef visited fill:#f0fdf4,stroke:#16a34a\n");
    diagram.push_str("    classDef unvisited fill:#f3f4f6,stroke:#9ca3af\n\n");

    // 状态定义
    for state in &all_states {
        let alias = state_alias(state);
        diagram.push_str(&format!("    {}[{}]\n", alias, state));
    }

    diagram.push('\n');

    // 转移边
    let transitions = [
        ("init", "semantic"),
        ("semantic", "func_decomp"),
        ("func_decomp", "ontology"),
        ("ontology", "planning"),
        ("planning", "extending"),
        ("planning", "planning"), // 自修复循环
        ("planning", "module_creat"),
        ("planning", "skill_exec"),
        ("extending", "generating"),
        ("generating", "gen_frontend"),
        ("gen_frontend", "composing"),
        ("composing", "verifying"),
        ("verifying", "publishing"),
        ("verifying", "planning"), // auto-fix 回退
        ("publishing", "published"),
        ("publishing", "planning"), // retry 回退
        ("published", "presenting"),
        ("module_creat", "block_creat"),
        ("block_creat", "onto_transfer"),
        ("onto_transfer", "service_api"),
        ("service_api", "publishing"),
        ("skill_exec", "planning"), // 技能完成后回退到 Planning
    ];

    for (from, to) in &transitions {
        if from != to {
            diagram.push_str(&format!("    {} --> {}\n", from, to));
        } else {
            diagram.push_str(&format!("    {} --> {}: 自修复\n", from, to));
        }
    }

    diagram.push('\n');

    // 应用样式
    for state in &all_states {
        let alias = state_alias(state);
        let is_current = base_current.as_str() == *state;
        let visited = visited_states.iter().any(|v| base_state(v) == *state);

        if is_current {
            diagram.push_str(&format!("    class {} current\n", alias));
        } else if visited {
            diagram.push_str(&format!("    class {} visited\n", alias));
        } else {
            diagram.push_str(&format!("    class {} unvisited\n", alias));
        }
    }

    diagram
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::{read_execution_log, ExecutionEvent, ExecutionLogEntry, LogLevel};
    use chrono::Utc;

    #[test]
    fn test_entry_serialization() {
        let entry = ExecutionLogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            session_id: 42,
            event: ExecutionEvent::StateTransition {
                from: "Initializing".to_string(),
                to: "SemanticAnalysis".to_string(),
                elapsed_ms: 123,
            },
        };
        let json = entry.to_json_line();
        let deserialized: ExecutionLogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.session_id, 42);
        assert!(matches!(
            deserialized.event,
            ExecutionEvent::StateTransition { .. }
        ));
    }

    #[test]
    fn test_llm_call_serialization() {
        let entry = ExecutionLogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            session_id: 1,
            event: ExecutionEvent::LlmCall {
                model_id: "deepseek-v4".to_string(),
                reasoning_effort: "high".to_string(),
                prompt_len: 1024,
                response_len: 2048,
                retry_count: 0,
                token_usage: Some(1500),
                latencies_ms: vec![1200],
            },
        };
        let json = entry.to_json_line();
        let deserialized: ExecutionLogEntry = serde_json::from_str(&json).unwrap();
        if let ExecutionEvent::LlmCall { model_id, .. } = &deserialized.event {
            assert_eq!(model_id, "deepseek-v4");
        } else {
            panic!("Expected LlmCall event");
        }
    }

    #[test]
    fn test_read_execution_log() {
        use std::io::Write;
        let dir = std::env::temp_dir().join("exec-log-test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("execution.log");

        let entry1 = ExecutionLogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            session_id: 1,
            event: ExecutionEvent::StateTransition {
                from: "A".to_string(),
                to: "B".to_string(),
                elapsed_ms: 100,
            },
        };
        let entry2 = ExecutionLogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Warn,
            session_id: 1,
            event: ExecutionEvent::Validation {
                kind: "app_json".to_string(),
                passed: false,
                detail: "missing field".to_string(),
            },
        };

        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(file, "{}", entry1.to_json_line()).unwrap();
        writeln!(file, "{}", entry2.to_json_line()).unwrap();
        drop(file);

        let entries = read_execution_log(path.to_str().unwrap()).unwrap();
        assert_eq!(entries.len(), 2);

        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// 写入 session 级 execution.log（在确定 output_path 之前使用）
/// 写入到系统临时目录下的 `.alioth-session/{session_id}/execution.log`
pub fn write_session_execution_log(session_id: i64, entries: &[ExecutionLogEntry]) {
    use std::io::Write;
    let dir = std::env::temp_dir()
        .join(".alioth-session")
        .join(format!("{}", session_id));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("execution.log");

    match std::fs::File::create(&path) {
        Ok(mut file) => {
            for entry in entries {
                let _ = writeln!(file, "{}", entry.to_json_line());
            }
        }
        Err(e) => {
            common::telemetry::error!(
                "Failed to write session execution.log for session {}: {}",
                session_id,
                e
            );
        }
    }
}
