//! app_agent — 应用构建 Agent

pub mod agent;
pub mod auto_router;
pub mod composer;
pub mod compressor;
pub mod execution_log;
pub mod extender;
pub mod harness;
pub mod llm;
pub mod memory;
pub mod mocks;
pub mod models;
pub mod orchestrator;
pub mod planner;
pub mod schema;
pub mod service_gen;
pub mod skills;
pub mod state;
pub mod tool_registry;
pub use execution_log::generate_state_diagram;
pub use execution_log::write_session_execution_log;
pub mod evaluate;
pub mod validator;
pub use evaluate::{
    evaluate, evaluate_with, Artifacts, CritiqueItem, EvalDimension, EvalReport, Judger, LlmJudger,
    RuleBasedJudger, MAX_EVAL_ITERATIONS, THRESHOLD,
};
pub use mocks::{MockLlmService, MockResponse};
pub mod tools;
pub use auto_router::{AutoRouter, ModelTier, ReasoningTier, RoutePlan};
pub use harness::{HarnessResult, LlmHarness, TaskType};
pub use llm::{GenerationOverrides, LlmError, LlmService};
pub use models::{ModelCapabilities, ModelEntry, ModelRegistry};
pub mod aligner;
pub use orchestrator::{progress_percent, state_name, AppAgent};
use sqlx::PgPool;
pub use state::{
    AgentProgress, AgentState, AgentToolCall, AppMeta, BuildResult, BusinessRulePlan,
    ComputationPlan, ConstraintPlan, ConversationContext, FlowPlan, MissingInfo, ModuleUsage,
    PlatformCatalog, Question, ResumeConfig, StepDetail, StepResult, UserAnswer, YamlOperation,
    YamlPatch,
};
pub use tools::{Tool, ToolResult};

pub async fn init(_pool: &PgPool) -> anyhow::Result<()> {
    common::telemetry::info!("Initializing app-agent module...");
    Ok(())
}
