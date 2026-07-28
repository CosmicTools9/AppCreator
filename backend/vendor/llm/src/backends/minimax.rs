//! MiniMax provider backend.
//!
//! MiniMax M-series models are accessed at `https://api.minimaxi.com/v1/chat/completions`
//! with OpenAI-compatible request/response schema. Provider-specific quirks:
//! - `Authorization: Bearer <key>` (same as OpenAI)
//! - Models return `reasoning_content` in assistant message when reasoning is
//!   enabled (similar to DeepSeek).
//! - `tool_choice: "auto"` is the default.
//! - MiniMax M3 requires `max_completion_tokens >= 1024` or rejects requests.
//! - MiniMax M3 deprecated `max_tokens` in favor of `max_completion_tokens`.
//! - `response_format` accepts `{"type": "json_object"}` since 2026.

use async_trait::async_trait;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use super::{BackendError, CompletionRequest, CompletionResponse, LlmBackend, ToolCallResult};

const DEFAULT_BASE_URL: &str = "https://api.minimaxi.com";
const DEFAULT_MODEL: &str = "MiniMax-M3";
const DEFAULT_FLASH_MODEL: &str = "MiniMax-M2.7";

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct MiniMaxRequest<'a> {
    model: &'a str,
    messages: Vec<MiniMaxMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<MiniMaxTool<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<&'a str>,
    temperature: f64,
    /// Used for M3 (deprecated max_tokens). Non-M3 uses max_tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    max_completion_tokens: Option<u64>,
    /// Used for non-M3 models (M2.7). M3 uses max_completion_tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u64>,
    top_p: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<MiniMaxResponseFormat<'a>>,
    #[serde(default)]
    stream: bool,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct MiniMaxMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct MiniMaxTool<'a> {
    #[serde(rename = "type")]
    tool_type: &'static str,
    function: MiniMaxFunction<'a>,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct MiniMaxFunction<'a> {
    name: &'a str,
    description: &'a str,
    parameters: &'a serde_json::Value,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct MiniMaxResponseFormat<'a> {
    #[serde(rename = "type")]
    format_type: &'a str,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MiniMaxResponse {
    id: Option<String>,
    model: Option<String>,
    choices: Vec<MiniMaxChoice>,
    usage: Option<MiniMaxUsage>,
    /// MiniMax-specific base_resp envelope (used for error reporting).
    base_resp: Option<MiniMaxBaseResp>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MiniMaxBaseResp {
    status_code: Option<i32>,
    status_msg: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MiniMaxChoice {
    index: u32,
    message: MiniMaxAssistantMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MiniMaxAssistantMessage {
    role: String,
    content: Option<String>,
    /// MiniMax exposes reasoning trace for thinking-capable models (M3, M1).
    reasoning_content: Option<String>,
    tool_calls: Option<Vec<MiniMaxToolCall>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MiniMaxToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: MiniMaxFunctionCall,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MiniMaxFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MiniMaxUsage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    total_tokens: Option<u64>,
}

/// MiniMax error response body (envelope-style).
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MiniMaxErrorBody {
    base_resp: Option<MiniMaxBaseResp>,
    error: Option<MiniMaxErrorDetail>,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MiniMaxErrorDetail {
    message: String,
    #[serde(rename = "type")]
    err_type: Option<String>,
    code: Option<String>,
}

pub struct MiniMaxBackend {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    default_model: String,
    #[allow(dead_code)]
    flash_model: String,
    timeout_seconds: u64,
}

impl MiniMaxBackend {
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        default_model: String,
        flash_model: String,
        timeout_seconds: u64,
    ) -> Result<Self, BackendError> {
        if api_key.is_empty() {
            return Err(BackendError::Auth("MiniMax API key is empty".to_string()));
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
        h.insert("User-Agent", "AliothStudio/0.1 (minimax)".parse().unwrap());
        h
    }

    fn build_body<'a>(&'a self, req: &'a CompletionRequest) -> MiniMaxRequest<'a> {
        let mut messages = Vec::new();
        if let Some(sys) = &req.system {
            if !sys.is_empty() {
                messages.push(MiniMaxMessage {
                    role: "system",
                    content: sys.as_str(),
                });
            }
        }
        messages.push(MiniMaxMessage {
            role: "user",
            content: req.prompt.as_str(),
        });

        let tools: Option<Vec<MiniMaxTool>> = if req.tools.is_empty() {
            None
        } else {
            Some(
                req.tools
                    .iter()
                    .map(|t| MiniMaxTool {
                        tool_type: "function",
                        function: MiniMaxFunction {
                            name: t.name.as_str(),
                            description: t.description.as_str(),
                            parameters: &t.parameters,
                        },
                    })
                    .collect(),
            )
        };

        let response_format: Option<MiniMaxResponseFormat> = req
            .response_format
            .as_deref()
            .map(|t| MiniMaxResponseFormat { format_type: t });
        // Model capability branching: M3 uses max_completion_tokens, legacy uses max_tokens.
        let is_m3 = req.model.contains("MiniMax-M3");

        MiniMaxRequest {
            model: req.model.as_str(),
            messages,
            tools,
            tool_choice: if req.tools.is_empty() {
                None
            } else {
                Some("auto")
            },
            temperature: req.temperature,
            max_completion_tokens: if is_m3 { Some(req.max_tokens) } else { None },
            max_tokens: if is_m3 { None } else { Some(req.max_tokens) },
            top_p: req.top_p,
            response_format,
            stream: false,
        }
    }

    fn parse_error(status: u16, body: &str) -> BackendError {
        if let Ok(parsed) = serde_json::from_str::<MiniMaxErrorBody>(body) {
            let msg = parsed
                .error
                .as_ref()
                .map(|e| e.message.clone())
                .or(parsed.message.clone())
                .or(parsed.base_resp.as_ref().and_then(|b| b.status_msg.clone()))
                .unwrap_or_else(|| body.to_string());

            // MiniMax uses base_resp.status_code for non-HTTP errors.
            if let Some(base) = &parsed.base_resp {
                if let Some(code) = base.status_code {
                    if code != 0 {
                        return match code {
                            1004..=1007 => BackendError::Auth(msg),
                            1008 | 1009 => BackendError::RateLimit(60),
                            _ => BackendError::ProviderStatus { status, body: msg },
                        };
                    }
                }
            }

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

    fn validate_max_tokens(model: &str, max_tokens: u64) -> Result<(), BackendError> {
        if model.contains("MiniMax-M3") && max_tokens < 1024 {
            return Err(BackendError::Config(format!(
                "MiniMax-M3 requires max_tokens >= 1024, got {}",
                max_tokens
            )));
        }
        Ok(())
    }
}

#[async_trait]
impl LlmBackend for MiniMaxBackend {
    fn provider_name(&self) -> &'static str {
        "minimax"
    }

    fn default_model(&self) -> &str {
        &self.default_model
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, BackendError> {
        Self::validate_max_tokens(&req.model, req.max_tokens)?;
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
            let body_text = response.text().await.unwrap_or_default();
            return Err(Self::parse_error(status, &body_text));
        }

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| BackendError::Parse(format!("response body: {}", e)))?;

        let parsed: MiniMaxResponse = serde_json::from_value(raw.clone())
            .map_err(|e| BackendError::Parse(format!("MiniMax envelope: {}", e)))?;

        // MiniMax sometimes returns 200 OK with a non-zero base_resp.status_code.
        if let Some(base) = &parsed.base_resp {
            if let Some(code) = base.status_code {
                if code != 0 {
                    let msg = base
                        .status_msg
                        .clone()
                        .unwrap_or_else(|| "unknown MiniMax error".to_string());
                    return Err(match code {
                        1004..=1007 => BackendError::Auth(msg),
                        1008 | 1009 => BackendError::RateLimit(60),
                        _ => BackendError::ProviderStatus {
                            status: 200,
                            body: msg,
                        },
                    });
                }
            }
        }

        let choice = parsed
            .choices
            .first()
            .ok_or_else(|| BackendError::Parse("no choices in response".to_string()))?;

        let mut text = choice.message.content.clone().unwrap_or_default();
        if let Some(reasoning) = &choice.message.reasoning_content {
            if !reasoning.is_empty() && !text.is_empty() {
                text = format!("{}\n\n{}", reasoning, text);
            } else if text.is_empty() {
                text = reasoning.clone();
            }
        }

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
            text: if text.is_empty() { None } else { Some(text) },
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

    fn make_backend() -> MiniMaxBackend {
        MiniMaxBackend::new(
            "test-key".to_string(),
            None,
            "MiniMax-M3".to_string(),
            "MiniMax-M2.7".to_string(),
            30,
        )
        .unwrap()
    }

    fn make_request(model: &str) -> CompletionRequest {
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
            reasoning_effort: None,
            response_format: None,
        }
    }

    #[test]
    fn m3_serializes_max_completion_tokens() {
        let backend = make_backend();
        let req = make_request("MiniMax-M3");
        let body = backend.build_body(&req);
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("max_completion_tokens"), "M3 should serialize max_completion_tokens");
        assert!(!json.contains("max_tokens"), "M3 should NOT serialize max_tokens");
    }

    #[test]
    fn m27_serializes_max_tokens() {
        let backend = make_backend();
        let req = make_request("MiniMax-M2.7");
        let body = backend.build_body(&req);
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("max_tokens"), "M2.7 should serialize max_tokens");
        assert!(!json.contains("max_completion_tokens"), "M2.7 should NOT serialize max_completion_tokens");
    }

    #[test]
    fn m3_rejects_less_than_1024_tokens() {
        let result = MiniMaxBackend::validate_max_tokens("MiniMax-M3", 512);
        assert!(result.is_err(), "M3 with max_tokens=512 should be rejected");
        match result.unwrap_err() {
            BackendError::Config(msg) => {
                assert!(msg.contains("1024"), "Error should mention 1024 minimum, got: {}", msg);
            }
            e => panic!("Expected BackendError::Config, got {:?}", e),
        }
    }

    #[test]
    fn m3_accepts_1024_tokens() {
        assert!(MiniMaxBackend::validate_max_tokens("MiniMax-M3", 1024).is_ok());
    }

    #[test]
    fn m3_accepts_over_1024_tokens() {
        assert!(MiniMaxBackend::validate_max_tokens("MiniMax-M3", 8192).is_ok());
    }

    #[test]
    fn m27_ignores_validation() {
        // M2.7 has no minimum requirement
        assert!(MiniMaxBackend::validate_max_tokens("MiniMax-M2.7", 512).is_ok());
    }
}
