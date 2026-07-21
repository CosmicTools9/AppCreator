//! `LlmService` is the high-level façade that the rest of the project uses.
//!
//! Internally it holds a `Box<dyn LlmBackend>` selected by `LlmProvider` and
//! applies per-request timeouts and parameter overrides.
use super::backends::{
    build_backend, BackendError, CompletionRequest, CompletionResponse, LlmBackend,
};
use super::types::{LlmProvider, LlmResponse, LlmServiceConfig, ToolCall, ToolDefinition};
use std::time::Duration;
/// Wrapper error used by the public API.
///
/// Backends produce `BackendError`; we expose it as `LlmError` so the
/// rest of the codebase doesn't have to import the backend module.
pub type LlmError = BackendError;

pub struct LlmService {
    backend: Box<dyn LlmBackend>,
    config: LlmServiceConfig,
}

impl LlmService {
    pub fn new(config: LlmServiceConfig) -> Result<Self, LlmError> {
        let mut config = config;
        let provider = std::mem::replace(&mut config.provider, LlmProvider::DeepSeek);
        let backend = build_backend(
            provider,
            config.api_key.clone(),
            config.base_url.clone(),
            config.model.clone(),
            config.flash_model.clone(),
            config.timeout_seconds,
        )?;
        Ok(Self { backend, config })
    }

    /// Plain text generation (no system prompt, no overrides).
    pub async fn generate(&self, prompt: &str) -> Result<String, LlmError> {
        self.generate_with_overrides(prompt, None, None, None, None)
            .await
    }

    /// 带参数覆盖的生成。适用于 Harness 层按 TaskType 动态调节。
    ///
    /// 未设置的参数（None）使用构造时的默认值。
    pub async fn generate_with_overrides(
        &self,
        prompt: &str,
        temperature: Option<f64>,
        max_tokens: Option<u64>,
        reasoning_effort: Option<&str>,
        response_format: Option<&str>,
    ) -> Result<String, LlmError> {
        let req = self.build_request(
            None,
            prompt,
            &[],
            temperature,
            max_tokens,
            reasoning_effort,
            response_format,
            None,
        )?;
        let resp = self.complete_with_timeout(req).await?;
        Ok(resp.text.unwrap_or_default())
    }

    /// 带 system prompt + 模型切换的生成。
    /// 将 system prompt 作为 API system message 发送。
    #[allow(clippy::too_many_arguments)]
    pub async fn generate_with_system_preamble(
        &self,
        system: &str,
        prompt: &str,
        temperature: Option<f64>,
        max_tokens: Option<u64>,
        reasoning_effort: Option<&str>,
        response_format: Option<&str>,
        model_override: Option<&str>,
    ) -> Result<String, LlmError> {
        let req = self.build_request(
            Some(system),
            prompt,
            &[],
            temperature,
            max_tokens,
            reasoning_effort,
            response_format,
            model_override,
        )?;
        let resp = self.complete_with_timeout(req).await?;
        Ok(resp.text.unwrap_or_default())
    }

    /// 带工具调用的生成
    pub async fn generate_with_tools(
        &self,
        prompt: &str,
        tools: &[ToolDefinition],
    ) -> Result<LlmResponse, LlmError> {
        let req = self.build_request(None, prompt, tools, None, None, None, None, None)?;
        let resp = self.complete_with_timeout(req).await?;
        Ok(to_llm_response(resp))
    }

    pub fn provider_name(&self) -> &str {
        self.backend.provider_name()
    }

    pub fn config(&self) -> &LlmServiceConfig {
        &self.config
    }

    pub async fn verify(&self) -> Result<(), LlmError> {
        self.backend.verify().await
    }

    #[allow(clippy::too_many_arguments)]
    fn build_request(
        &self,
        system: Option<&str>,
        prompt: &str,
        tools: &[ToolDefinition],
        temperature: Option<f64>,
        max_tokens: Option<u64>,
        reasoning_effort: Option<&str>,
        response_format: Option<&str>,
        model_override: Option<&str>,
    ) -> Result<CompletionRequest, LlmError> {
        let params = &self.config.generation_params;

        let reasoning_effort = reasoning_effort.and_then(super::types::ReasoningEffort::parse);

        let temperature = temperature.unwrap_or(params.temperature);
        let max_tokens = max_tokens.unwrap_or(params.max_tokens);
        let response_format = response_format
            .map(|s| s.to_string())
            .or_else(|| params.response_format.clone());

        let model = model_override
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.config.model.clone());

        Ok(CompletionRequest {
            model,
            system: system.map(|s| s.to_string()),
            prompt: prompt.to_string(),
            tools: tools.to_vec(),
            temperature,
            max_tokens,
            top_p: params.top_p,
            frequency_penalty: params.frequency_penalty,
            presence_penalty: params.presence_penalty,
            reasoning_effort,
            response_format,
        })
    }

    async fn complete_with_timeout(
        &self,
        req: CompletionRequest,
    ) -> Result<CompletionResponse, LlmError> {
        let timeout = self.config.timeout_seconds;
        let backend = &self.backend;
        match tokio::time::timeout(Duration::from_secs(timeout), backend.complete(req)).await {
            Ok(result) => result,
            Err(_) => Err(BackendError::Timeout(timeout)),
        }
    }
}

fn to_llm_response(resp: CompletionResponse) -> LlmResponse {
    if resp.tool_calls.is_empty() {
        LlmResponse::Text(resp.text.unwrap_or_default())
    } else {
        let tool_calls: Vec<ToolCall> = resp
            .tool_calls
            .into_iter()
            .map(|tc| ToolCall {
                id: tc.id,
                name: tc.name,
                arguments: tc.arguments,
            })
            .collect();
        LlmResponse::ToolCalls(tool_calls)
    }
}
