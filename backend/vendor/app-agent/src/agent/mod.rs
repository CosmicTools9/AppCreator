//! Agent — 子 Agent 委派系统
//!
//! 允许 AppAgent 创建独立的子 Agent 来并行或隔离执行任务。
//! 每个子 Agent 拥有独立的 ConversationContext 和 LLM 调用能力。
//!
//! ## 用法
//! ```ignore
//! let spawner = LocalAgentSpawner::new(llm_service);
//! let handle = spawner.spawn(task).await?;
//! let result = handle.join().await?;
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::harness::{LlmHarness, TaskType};

/// 子 Agent 实例句柄
pub struct AgentHandle {
    /// 子 agent 的完成结果
    result: Arc<Mutex<Option<AgentResult>>>,
}

impl Default for AgentHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentHandle {
    pub fn new() -> Self {
        Self {
            result: Arc::new(Mutex::new(None)),
        }
    }

    /// 等待子 Agent 完成
    pub async fn join(&self) -> Option<AgentResult> {
        self.result.lock().await.take()
    }
}

/// 子 Agent 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    /// 返回给父 Agent 的文本
    pub output: String,
    /// 子 Agent 的最终状态
    pub final_state: serde_json::Value,
    /// 产生的文件
    pub artifacts: Vec<String>,
    /// 执行的步数
    pub steps_executed: usize,
}

/// 子 Agent 任务定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    /// 任务 ID
    pub id: String,
    /// 任务描述（作为 user_description 注入子 context）
    pub description: String,
    /// namespace
    pub namespace: String,
    /// 预期工具调用
    pub expected_tools: Vec<String>,
    /// LLM 任务类型
    pub task_type: TaskType,
}

/// Agent 孵化器 trait
#[async_trait]
pub trait AgentSpawner: Send + Sync {
    /// 创建并执行一个子 Agent
    async fn spawn(&self, task: SubTask) -> Result<AgentResult, String>;
}

/// 进程内子 Agent 孵化器（同进程 tokio task）
/// 进程内子 Agent 孵化器（同进程 tokio task）
pub struct LocalAgentSpawner {
    llm: Arc<dyn crate::llm::LlmService>,
}

impl LocalAgentSpawner {
    pub fn new(llm: Arc<dyn crate::llm::LlmService>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl AgentSpawner for LocalAgentSpawner {
    async fn spawn(&self, task: SubTask) -> Result<AgentResult, String> {
        let harness = LlmHarness::new(&*self.llm)
            .with_task(task.task_type)
            .with_system(format!(
                r#"你是一个子 Agent。你的任务是：

{desc}

独立完成这个任务，不要等待用户输入。完成后输出结果。
输出格式：JSON {{ "result": "...", "files": [...], "summary": "..." }}
"#,
                desc = task.description
            ));

        let result = harness
            .call_with_retry("请开始执行任务。")
            .await
            .map_err(|e| format!("Sub-agent failed: {}", e))?;

        // 尝试解析 LLM 输出为结构化结果
        let parsed: serde_json::Value = serde_json::from_str(&result.raw_text)
            .unwrap_or(serde_json::json!({"result": result.raw_text}));

        let files = parsed
            .get("files")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(AgentResult {
            output: parsed
                .get("result")
                .and_then(|v| v.as_str())
                .unwrap_or(&result.raw_text)
                .to_string(),
            final_state: parsed,
            artifacts: files,
            steps_executed: result.attempts as usize,
        })
    }
}

/// 模拟 Agent 孵化器（测试用）
pub struct MockAgentSpawner;

#[async_trait]
impl AgentSpawner for MockAgentSpawner {
    async fn spawn(&self, _task: SubTask) -> Result<AgentResult, String> {
        Ok(AgentResult {
            output: "mock result".to_string(),
            final_state: serde_json::json!({"mock": true}),
            artifacts: vec![],
            steps_executed: 1,
        })
    }
}
