//! LLM service framework.
//!
//! Each provider (DeepSeek / Kimi / MiniMax) has its own dedicated backend
//! implementation under `backends/`. The high-level [`LlmService`] façade
//! dispatches to the backend based on `LlmProvider`.

pub mod backends;
pub mod service;
pub mod types;

pub use backends::{
    BackendError, CompletionRequest, CompletionResponse, LlmBackend, ToolCallResult,
};
pub use service::{LlmError, LlmService};
pub use types::{
    GenerationParams, LlmProvider, LlmResponse, LlmServiceConfig, ReasoningEffort, ToolCall,
    ToolDefinition,
};
