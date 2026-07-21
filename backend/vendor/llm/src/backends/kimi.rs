//! Kimi (Moonshot) provider backend.
//!
//! Kimi offers OpenAI-compatible chat completions at
//! `https://api.moonshot.cn/v1/chat/completions`. Differences from generic
//! OpenAI:
//! - `Authorization: Bearer <key>` (same as OpenAI)
//! - Native `tools` schema with `type: "function"`
//! - Does NOT support `reasoning_effort` (Kimi models have built-in thinking
//!   variants; selection is done by model name, e.g. `kimi-k2-thinking`).
//! - Does NOT support `response_format.type = "json_object"` directly —
//!   instead we use `tools` with a JSON schema or rely on prompt instructions.
//! - Returns tool calls with `arguments` as a JSON string (parses identically
//!   to DeepSeek).

use async_trait::async_trait;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use super::{BackendError, CompletionRequest, CompletionResponse, LlmBackend, ToolCallResult};

const DEFAULT_BASE_URL: &str = "https://api.moonshot.cn";
const DEFAULT_MODEL: &str = "kimi-k2.6";
const DEFAULT_FLASH_MODEL: &str = "kimi-k2.5";

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct KimiRequest<'a> {
    model: &'a str,
    messages: Vec<KimiMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<KimiTool<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<&'a str>,
    temperature: f64,
    max_tokens: u64,
    top_p: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<u8>,
    #[serde(default)]
    stream: bool,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct KimiMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct KimiTool<'a> {
    #[serde(rename = "type")]
    tool_type: &'static str,
    function: KimiFunction<'a>,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct KimiFunction<'a> {
    name: &'a str,
    description: &'a str,
    parameters: &'a serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct KimiResponse {
    id: Option<String>,
    model: Option<String>,
    choices: Vec<KimiChoice>,
    usage: Option<KimiUsage>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct KimiChoice {
    index: u32,
    message: KimiAssistantMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct KimiAssistantMessage {
    role: String,
    content: Option<String>,
    tool_calls: Option<Vec<KimiToolCall>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct KimiToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: KimiFunctionCall,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct KimiFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct KimiUsage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    total_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct KimiErrorBody {
    error: Option<KimiErrorDetail>,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct KimiErrorDetail {
    message: String,
    #[serde(rename = "type")]
    err_type: Option<String>,
    code: Option<String>,
}

pub struct KimiBackend {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    default_model: String,
    #[allow(dead_code)]
    flash_model: String,
    timeout_seconds: u64,
}

impl KimiBackend {
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        default_model: String,
        flash_model: String,
        timeout_seconds: u64,
    ) -> Result<Self, BackendError> {
        if api_key.is_empty() {
            return Err(BackendError::Auth("Kimi API key is empty".to_string()));
        }
        Ok(Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(timeout_seconds))
                .build()
                .map_err(|e| BackendError::Config(e.to_string()))?,
            base_url: base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
            api_key,
            default_model,
            flash_model,
            timeout_seconds,
        })
    }

    fn endpoint(&self) -> String {
        // Kimi's chat completions endpoint is at /v1/chat/completions.
        format!(
            "{}/v1/chat/completions",
            self.base_url.trim_end_matches('/')
        )
    }

    fn build_headers(&self) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        h.insert(
            AUTHORIZATION,
            format!("Bearer {}", self.api_key).parse().unwrap(),
        );
        h.insert("User-Agent", "AliothStudio/0.1 (kimi)".parse().unwrap());
        h
    }

    fn build_body<'a>(&'a self, req: &'a CompletionRequest) -> KimiRequest<'a> {
        let mut messages = Vec::new();
        if let Some(sys) = &req.system {
            if !sys.is_empty() {
                messages.push(KimiMessage {
                    role: "system",
                    content: sys.as_str(),
                });
            }
        }
        messages.push(KimiMessage {
            role: "user",
            content: req.prompt.as_str(),
        });

        let tools: Option<Vec<KimiTool>> = if req.tools.is_empty() {
            None
        } else {
            Some(
                req.tools
                    .iter()
                    .map(|t| KimiTool {
                        tool_type: "function",
                        function: KimiFunction {
                            name: t.name.as_str(),
                            description: t.description.as_str(),
                            parameters: &t.parameters,
                        },
                    })
                    .collect(),
            )
        };

        KimiRequest {
            model: req.model.as_str(),
            messages,
            tools,
            tool_choice: if req.tools.is_empty() {
                None
            } else {
                Some("auto")
            },
            temperature: req.temperature,
            max_tokens: req.max_tokens,
            top_p: req.top_p,
            // Kimi supports multiple completions per request; default to 1.
            n: Some(1),
            stream: false,
        }
    }

    fn parse_error(status: u16, body: &str) -> BackendError {
        if let Ok(parsed) = serde_json::from_str::<KimiErrorBody>(body) {
            let msg = parsed
                .error
                .as_ref()
                .map(|e| e.message.clone())
                .or(parsed.message)
                .unwrap_or_else(|| body.to_string());
            return match status {
                401 | 403 => BackendError::Auth(msg),
                429 => BackendError::RateLimit(60),
                _ => BackendError::ProviderStatus { status, body: msg },
            };
        }
        if status == 429 {
            return BackendError::RateLimit(60);
        }
        if status == 401 || status == 403 {
            return BackendError::Auth(body.to_string());
        }
        BackendError::ProviderStatus {
            status,
            body: body.to_string(),
        }
    }
}

#[async_trait]
impl LlmBackend for KimiBackend {
    fn provider_name(&self) -> &'static str {
        "kimi"
    }

    fn default_model(&self) -> &str {
        &self.default_model
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, BackendError> {
        let url = self.endpoint();
        let body = self.build_body(&req);
        let headers = self.build_headers();

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    BackendError::Timeout(self.timeout_seconds)
                } else {
                    BackendError::Transport(e.to_string())
                }
            })?;

        let status = response.status().as_u16();
        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Self::parse_error(status, &body));
        }

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| BackendError::Parse(format!("response body: {}", e)))?;

        let parsed: KimiResponse = serde_json::from_value(raw.clone())
            .map_err(|e| BackendError::Parse(format!("Kimi envelope: {}", e)))?;

        let choice = parsed
            .choices
            .first()
            .ok_or_else(|| BackendError::Parse("no choices in response".to_string()))?;

        let text = choice.message.content.clone();
        let tool_calls: Vec<ToolCallResult> = choice
            .message
            .tool_calls
            .as_ref()
            .map(|tcs| {
                tcs.iter()
                    .map(|tc| ToolCallResult {
                        id: tc.id.clone(),
                        name: tc.function.name.clone(),
                        arguments: serde_json::from_str(&tc.function.arguments)
                            .unwrap_or(serde_json::Value::String(tc.function.arguments.clone())),
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(CompletionResponse {
            text: if text.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                None
            } else {
                text
            },
            tool_calls,
            raw,
        })
    }
}

pub const fn default_model() -> &'static str {
    DEFAULT_MODEL
}

pub const fn default_flash_model() -> &'static str {
    DEFAULT_FLASH_MODEL
}
