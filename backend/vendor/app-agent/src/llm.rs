//! LLM 服务抽象
//!
//! 定义 Agent 调用的 LLM 接口。支持生成参数按需覆盖。

use crate::auto_router::ModelTier;
use async_trait::async_trait;

/// 轻量级生成参数 — 可覆盖 Agent 的默认 LLM 配置。
/// 字段均为 Option，仅非 None 的字段会覆盖默认值。
#[derive(Debug, Clone, Default)]
pub struct GenerationOverrides {
    /// 较低 temperature → 更确定性的输出
    pub temperature: Option<f64>,
    /// 最大输出 token 数
    pub max_tokens: Option<u64>,
    /// DeepSeek v4 reasoning effort: minimal/low/medium/high/xhigh
    pub reasoning_effort: Option<&'static str>,
    /// 模型档次选择（用于多模型适配器）
    pub model_tier: Option<ModelTier>,
    /// OpenAI-compatible response_format, e.g. `"json_object"`.
    pub response_format: Option<String>,
}

#[async_trait]
pub trait LlmService: Send + Sync {
    async fn generate(&self, prompt: &str) -> Result<String, LlmError>;

    /// 带 system prompt 的生成。默认实现将 system 与 user prompt 合并。
    async fn generate_with_system(&self, system: &str, prompt: &str) -> Result<String, LlmError> {
        let full = if system.is_empty() {
            prompt.to_string()
        } else {
            format!("{}\n\n{}", system, prompt)
        };
        self.generate(&full).await
    }

    /// 带参数覆盖的生成。用于 Harness 层按 TaskType 分档。
    /// 默认忽略参数，交给具体实现决定是否应用。
    async fn generate_with_params(
        &self,
        system: &str,
        prompt: &str,
        _overrides: GenerationOverrides,
    ) -> Result<String, LlmError> {
        self.generate_with_system(system, prompt).await
    }
}

#[derive(Debug, Clone)]
pub struct LlmError {
    pub message: String,
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for LlmError {}

impl From<String> for LlmError {
    fn from(message: String) -> Self {
        Self { message }
    }
}

impl From<&str> for LlmError {
    fn from(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpLlmService {
    endpoint: String,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl HttpLlmService {
    pub fn new(endpoint: String, api_key: Option<String>) -> Self {
        Self {
            endpoint,
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn generate(&self, prompt: &str) -> Result<String, LlmError> {
        #[derive(serde::Serialize)]
        struct RequestBody {
            prompt: String,
        }

        #[derive(serde::Deserialize)]
        struct ResponseBody {
            content: String,
        }

        let mut request = self.client.post(&self.endpoint).json(&RequestBody {
            prompt: prompt.to_string(),
        });

        if let Some(ref key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        let response = request.send().await.map_err(|e| LlmError {
            message: format!("HTTP request failed: {}", e),
        })?;

        if !response.status().is_success() {
            return Err(LlmError {
                message: format!("LLM request failed with status: {}", response.status()),
            });
        }

        let body: ResponseBody = response.json().await.map_err(|e| LlmError {
            message: format!("Failed to parse response: {}", e),
        })?;

        Ok(body.content)
    }
}

#[async_trait]
impl LlmService for HttpLlmService {
    async fn generate(&self, prompt: &str) -> Result<String, LlmError> {
        HttpLlmService::generate(self, prompt).await
    }
}
