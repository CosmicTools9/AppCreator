//! LLM Provider-specific backends.
//!
//! Each Provider (DeepSeek, Kimi, MiniMax) implements its own HTTP client that
//! calls the Provider's native API directly. There is no shared "OpenAI
//! compatible" layer; each backend handles its own request/response format,
//! authentication header, error mapping, and rate-limit reporting.

pub mod deepseek;
pub mod kimi;
pub mod minimax;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::types::{LlmProvider, ReasoningEffort, ToolDefinition};

/// Provider-agnostic completion request.
///
/// Each backend maps this struct to its own request body via `serialize_body`.
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    /// Model identifier as the backend should pass to its API.
    /// Usually the user-selected model (e.g. `deepseek-v4-pro`).
    pub model: String,
    /// System / developer prompt. Backend may map to its native key
    /// (`system` for OpenAI-compatible, `preamble` for rig, etc.).
    pub system: Option<String>,
    /// User prompt.
    pub prompt: String,
    /// Tools for function calling.
    pub tools: Vec<ToolDefinition>,
    /// Sampling temperature.
    pub temperature: f64,
    /// Maximum output tokens.
    pub max_tokens: u64,
    /// Top-p nucleus sampling.
    pub top_p: f64,
    /// Frequency penalty.
    pub frequency_penalty: f64,
    /// Presence penalty.
    pub presence_penalty: f64,
    /// Reasoning effort hint (semantic level). Backend may translate to its
    /// native format (DeepSeek accepts `low|medium|high`; others may use
    /// thinking variants or different keys).
    pub reasoning_effort: Option<ReasoningEffort>,
    /// Response format hint. Backend may translate to its native type
    /// (`json_object`, `{"type":"json_schema",...}`).
    pub response_format: Option<String>,
}

/// Provider-agnostic completion response.
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    /// Plain text content (if no tool calls).
    pub text: Option<String>,
    /// Tool/function calls requested by the model.
    pub tool_calls: Vec<ToolCallResult>,
    /// Raw provider response (for logging / debugging).
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Rate-limit information reported by the backend (parsed from response headers).
#[derive(Debug, Clone, Default)]
pub struct RateLimitInfo {
    pub remaining_requests: Option<u64>,
    pub remaining_tokens: Option<u64>,
    pub reset_seconds: Option<u64>,
}

/// Common error type for all backends.
#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("HTTP transport error: {0}")]
    Transport(String),
    #[error("Request timeout after {0}s")]
    Timeout(u64),
    #[error("Provider returned status {status}: {body}")]
    ProviderStatus { status: u16, body: String },
    #[error("Failed to parse response: {0}")]
    Parse(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Authentication error: {0}")]
    Auth(String),
    #[error("Rate limit exceeded; reset in {0}s")]
    RateLimit(u64),
}

/// Provider-specific HTTP backend.
#[async_trait]
pub trait LlmBackend: Send + Sync {
    /// Provider name (matches `LlmProvider::as_str()`).
    fn provider_name(&self) -> &'static str;

    /// Send a completion request to the provider.
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, BackendError>;

    /// Probe with a tiny request to verify connectivity (e.g. on startup).
    async fn verify(&self) -> Result<(), BackendError> {
        // Default: send a 1-token "ping" request.
        let req = CompletionRequest {
            model: self.default_model().to_string(),
            system: None,
            prompt: "ping".to_string(),
            tools: vec![],
            temperature: 0.0,
            max_tokens: 1,
            top_p: 1.0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
            reasoning_effort: None,
            response_format: None,
        };
        self.complete(req).await.map(|_| ())
    }

    /// Default model name to use for verify() and as a fallback.
    fn default_model(&self) -> &str;
}

/// Build the appropriate backend for the given provider + config.
pub fn build_backend(
    provider: LlmProvider,
    api_key: String,
    base_url: Option<String>,
    default_model: String,
    #[allow(dead_code)] flash_model: String,
    timeout_seconds: u64,
) -> Result<Box<dyn LlmBackend>, BackendError> {
    let backend: Box<dyn LlmBackend> = match provider {
        LlmProvider::DeepSeek => Box::new(deepseek::DeepSeekBackend::new(
            api_key,
            base_url,
            default_model,
            flash_model,
            timeout_seconds,
        )?),
        LlmProvider::Kimi => Box::new(kimi::KimiBackend::new(
            api_key,
            base_url,
            default_model,
            flash_model,
            timeout_seconds,
        )?),
        LlmProvider::MiniMax => Box::new(minimax::MiniMaxBackend::new(
            api_key,
            base_url,
            default_model,
            flash_model,
            timeout_seconds,
        )?),
    };
    Ok(backend)
}
