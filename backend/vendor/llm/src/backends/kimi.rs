//! Kimi (Moonshot) provider backend.
//!
//! Kimi offers OpenAI-compatible chat completions at
//! `https://api.moonshot.cn/v1/chat/completions`. Differences from generic
//! OpenAI:
//! - `Authorization: Bearer <key>` (same as OpenAI)
//! - Native `tools` schema with `type: "function"`
//! - Kimi K3 supports `reasoning_effort` as a top-level field (`low`/`high`/`max`).
//! - Kimi K3 uses `max_completion_tokens` (not `max_tokens`).
//! - Temperature is fixed per-model and MUST NOT be sent (will error).
//! - Does NOT support `response_format.type = "json_object"` directly —
//!   instead we use `tools` with a JSON schema or rely on prompt instructions.
//! - Returns tool calls with `arguments` as a JSON string (parses identically
//!   to DeepSeek).

use async_trait::async_trait;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use super::{BackendError, CompletionRequest, CompletionResponse, LlmBackend, ToolCallResult};

const DEFAULT_BASE_URL: &str = "https://api.moonshot.cn";
const DEFAULT_MODEL: &str = "kimi-k3";
const DEFAULT_FLASH_MODEL: &str = "kimi-k2.6";

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct KimiRequest<'a> {
    model: &'a str,
    messages: Vec<KimiMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<KimiTool<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<&'a str>,
    /// Temperature is fixed per-model. Skipped entirely (Kimi errors if sent).
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    /// Used for K3 (deprecated max_tokens). Non-K3 uses max_tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    max_completion_tokens: Option<u64>,
    /// Used for non-K3 models (k2.6, k2.7-code). K3 uses max_completion_tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u64>,
    top_p: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<u8>,
    /// Kimi K3 supports reasoning_effort (low/high/max).
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<&'a str>,
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

        // Model capability branching: K3 uses new fields, legacy models use old.
        let is_k3 = req.model.contains("kimi-k3");

        // Kimi K3 reasoning_effort: low/high/max. Only for K3.
        let reasoning_effort: Option<&'static str> = if is_k3 {
            match req.reasoning_effort {
                Some(crate::types::ReasoningEffort::Minimal) => Some("low"),
                Some(crate::types::ReasoningEffort::Low) => Some("low"),
                Some(crate::types::ReasoningEffort::Medium) => Some("high"),
                Some(crate::types::ReasoningEffort::High) => Some("high"),
                Some(crate::types::ReasoningEffort::XHigh) => Some("max"),
                None => None,
            }
        } else {
            None
        };

        // Temperature is fixed per-model (all Kimi); skip entirely to avoid 400.
        let temperature: Option<f64> = None;

        KimiRequest {
            model: req.model.as_str(),
            messages,
            tools,
            tool_choice: if req.tools.is_empty() {
                None
            } else {
                Some("auto")
            },
            temperature,
            max_completion_tokens: if is_k3 { Some(req.max_tokens) } else { None },
            max_tokens: if is_k3 { None } else { Some(req.max_tokens) },
            top_p: req.top_p,
            // Kimi supports multiple completions per request; default to 1.
            n: Some(1),
            reasoning_effort,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ReasoningEffort;

    fn make_backend() -> KimiBackend {
        KimiBackend::new(
            "test-key".to_string(),
            None,
            "kimi-k3".to_string(),
            "kimi-k2.6".to_string(),
            30,
        )
        .unwrap()
    }

    fn make_request(model: &str, effort: Option<ReasoningEffort>) -> CompletionRequest {
        CompletionRequest {
            model: model.to_string(),
            system: None,
            prompt: "hello".to_string(),
            tools: vec![],
            temperature: 1.0,
            max_tokens: 4096,
            top_p: 1.0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
            reasoning_effort: effort,
            response_format: None,
        }
    }

    #[test]
    fn test_k3_serializes_max_completion_tokens() {
        let backend = make_backend();
        let req = make_request("kimi-k3", None);
        let body = backend.build_body(&req);
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("max_completion_tokens"), "K3 should serialize max_completion_tokens");
        assert!(!json.contains("max_tokens"), "K3 should NOT serialize max_tokens");
        assert!(!json.contains("temperature"), "Kimi should NOT serialize temperature");
    }

    #[test]
    fn test_k26_serializes_max_tokens() {
        let backend = make_backend();
        let req = make_request("kimi-k2.6", None);
        let body = backend.build_body(&req);
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("max_tokens"), "k2.6 should serialize max_tokens");
        assert!(!json.contains("max_completion_tokens"), "k2.6 should NOT serialize max_completion_tokens");
        assert!(!json.contains("temperature"), "Kimi should NOT serialize temperature");
    }

    #[test]
    fn test_k3_reasoning_effort_included() {
        let backend = make_backend();
        let req = make_request("kimi-k3", Some(ReasoningEffort::XHigh));
        let body = backend.build_body(&req);
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("reasoning_effort"), "K3 should include reasoning_effort");
        assert!(json.contains("\"max\""), "K3 XHigh maps to max");
    }

    #[test]
    fn test_k26_reasoning_effort_omitted() {
        let backend = make_backend();
        let req = make_request("kimi-k2.6", Some(ReasoningEffort::High));
        let body = backend.build_body(&req);
        let json = serde_json::to_string(&body).unwrap();
        assert!(!json.contains("reasoning_effort"), "k2.6 should NOT include reasoning_effort");
    }
}
