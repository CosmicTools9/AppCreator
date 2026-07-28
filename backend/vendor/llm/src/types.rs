use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// DeepSeek v4 / OpenAI o-series reasoning effort level.
///
/// Aligned with oh-my-pi's `Effort` enum (5 levels):
/// https://github.com/can1357/oh-my-pi — model-thinking.ts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningEffort {
    Minimal,
    Low,
    Medium,
    High,
    #[serde(rename = "xhigh")]
    XHigh,
}

impl std::fmt::Display for ReasoningEffort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Minimal => write!(f, "minimal"),
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::XHigh => write!(f, "xhigh"),
        }
    }
}

impl ReasoningEffort {
    /// Parse a string from the API / env config into the typed enum.
    /// Returns `None` for unrecognised values.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "minimal" => Some(Self::Minimal),
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            "xhigh" => Some(Self::XHigh),
            _ => None,
        }
    }

    /// Returns the API-level string value for the reasoning effort.
    /// Returns `None` for variants that should not be sent.
    ///
    /// DeepSeek v4 accepts: low, medium, high.
    /// minimal → "low" (lowest available), xhigh → "high" (highest available).
    /// medium → None (let API apply its default, preserving prefix cache).
    pub fn as_api_value(&self) -> Option<&'static str> {
        match self {
            Self::Minimal => Some("low"),
            Self::Low => Some("low"),
            Self::Medium => None, // default; omit to preserve prefix cache
            Self::High => Some("high"),
            Self::XHigh => Some("high"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmServiceConfig {
    pub provider: LlmProvider,
    pub api_key: String,
    pub model: String,
    pub flash_model: String,
    pub base_url: Option<String>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub generation_params: GenerationParams,
}

impl LlmServiceConfig {
    /// 从环境变量解析 LLM 配置
    ///
    /// 默认适配当前 `LLM_PROVIDER` 的模型名和 API 地址，
    /// 可通过 `LLM_MODEL` / `LLM_FLASH_MODEL` / `LLM_BASE_URL` 单独覆盖。
    pub fn from_env() -> Self {
        let provider = LlmProvider::from_env();
        let default_model = provider.default_model().to_string();
        let default_flash = provider.default_flash_model().to_string();
        let default_base_url = provider.default_base_url().to_string();
        Self {
            provider,
            api_key: std::env::var("LLM_API_KEY").unwrap_or_default(),
            model: std::env::var("LLM_MODEL").unwrap_or(default_model),
            flash_model: std::env::var("LLM_FLASH_MODEL").unwrap_or(default_flash),
            base_url: std::env::var("LLM_BASE_URL")
                .ok()
                .or(Some(default_base_url)),
            timeout_seconds: std::env::var("LLM_TIMEOUT_SECONDS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300),
            max_retries: std::env::var("LLM_MAX_RETRIES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3),
            generation_params: GenerationParams {
                temperature: std::env::var("LLM_TEMPERATURE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1.0),
                max_tokens: std::env::var("LLM_MAX_TOKENS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(4096),
                top_p: 1.0,
                frequency_penalty: 0.0,
                presence_penalty: 0.0,
                reasoning_effort: std::env::var("LLM_REASONING_EFFORT")
                    .ok()
                    .and_then(|s| match s.as_str() {
                        "minimal" => Some(ReasoningEffort::Minimal),
                        "low" => Some(ReasoningEffort::Low),
                        "medium" => Some(ReasoningEffort::Medium),
                        "high" => Some(ReasoningEffort::High),
                        "xhigh" => Some(ReasoningEffort::XHigh),
                        _ => None,
                    })
                    .unwrap_or(ReasoningEffort::Medium),
                response_format: std::env::var("LLM_RESPONSE_FORMAT").ok(),
            },
        }
    }
}

impl LlmProvider {
    /// 从环境变量 LLM_PROVIDER 解析，支持 kimi / minimax / deepseek，默认 DeepSeek
    pub fn from_env() -> Self {
        match std::env::var("LLM_PROVIDER")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "kimi" => Self::Kimi,
            "minimax" => Self::MiniMax,
            _ => Self::DeepSeek,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DeepSeek => "deepseek",
            Self::Kimi => "kimi",
            Self::MiniMax => "minimax",
        }
    }

    /// 默认旗舰模型（Pro 档）
    pub fn default_model(&self) -> &'static str {
        match self {
            Self::DeepSeek => "deepseek-v4-pro",
            Self::Kimi => "kimi-k2.6",
            Self::MiniMax => "MiniMax-M3",
        }
    }
    /// 默认经济模型（Flash 档）
    pub fn default_flash_model(&self) -> &'static str {
        match self {
            Self::DeepSeek => "deepseek-v4-flash",
            Self::Kimi => "kimi-k2.6",
            Self::MiniMax => "MiniMax-M2.7",
        }
    }

    /// 默认 API 地址
    pub fn default_base_url(&self) -> &'static str {
        match self {
            Self::DeepSeek => "https://api.deepseek.com",
            Self::Kimi => "https://api.moonshot.cn",
            Self::MiniMax => "https://api.minimaxi.com",
        }
    }
}

impl FromStr for LlmProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "deepseek" => Ok(Self::DeepSeek),
            "kimi" => Ok(Self::Kimi),
            "minimax" => Ok(Self::MiniMax),
            _ => Err(format!(
                "Unknown LLM provider '{}'. Supported: deepseek, kimi, minimax",
                s
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum LlmProvider {
    /// DeepSeek (OpenAI-compatible API)
    #[default]
    DeepSeek,
    /// Kimi / Moonshot (OpenAI-compatible API)
    Kimi,
    /// MiniMax (OpenAI-compatible API)
    MiniMax,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationParams {
    pub temperature: f64,
    pub max_tokens: u64,
    pub top_p: f64,
    pub frequency_penalty: f64,
    pub presence_penalty: f64,
    /// DeepSeek reasoning effort (low / medium / high).
    #[serde(default = "default_reasoning_effort")]
    pub reasoning_effort: ReasoningEffort,
    /// OpenAI-compatible response_format, e.g. `"json_object"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
}

fn default_reasoning_effort() -> ReasoningEffort {
    ReasoningEffort::Medium
}

/// Get the max_tokens ceiling for a given task type (oh-my-pi inspired).
/// Returns a recommended upper bound, not a hard limit.
pub fn recommended_max_tokens(task: &str) -> u64 {
    match task {
        "ontology_planning" => 16384,
        "code_generation" => 32768,
        "semantic_repair" => 4096,
        "format_correction" => 8192,
        _ => 4096,
    }
}

impl GenerationParams {
    /// 将可选覆盖参数合并到此配置中，返回新实例。
    /// 用于 Harness 层按 TaskType 动态调节 temperature/effort/tokens。
    pub fn with_overrides(
        &self,
        temperature: Option<f64>,
        max_tokens: Option<u64>,
        reasoning_effort: Option<&str>,
        response_format: Option<&str>,
    ) -> Self {
        let mut p = self.clone();
        if let Some(t) = temperature {
            p.temperature = t;
        }
        if let Some(t) = max_tokens {
            p.max_tokens = t;
        }
        if let Some(re) = reasoning_effort {
            match re {
                "minimal" => p.reasoning_effort = ReasoningEffort::Minimal,
                "low" => p.reasoning_effort = ReasoningEffort::Low,
                "medium" => p.reasoning_effort = ReasoningEffort::Medium,
                "high" => p.reasoning_effort = ReasoningEffort::High,
                "xhigh" => p.reasoning_effort = ReasoningEffort::XHigh,
                _ => {}
            }
        }
        if let Some(rf) = response_format {
            p.response_format = Some(rf.to_string());
        }
        p
    }
}

impl Default for GenerationParams {
    fn default() -> Self {
        Self {
            temperature: 1.0,
            max_tokens: 4096,
            top_p: 1.0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
            reasoning_effort: ReasoningEffort::Medium,
            response_format: None,
        }
    }
}

/// LLM 工具定义（Function Calling Schema）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// LLM 工具调用请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// LLM 响应（文本或工具调用）
#[derive(Debug, Clone)]
pub enum LlmResponse {
    Text(String),
    ToolCalls(Vec<ToolCall>),
}
