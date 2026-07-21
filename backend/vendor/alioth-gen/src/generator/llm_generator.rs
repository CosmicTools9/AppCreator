//! LLM-based Code Generator Implementation
//!
//! Provides a concrete `CodeGenerator` implementation that calls
//! DeepSeek-compatible HTTP endpoints to generate code from ontology models.

use crate::generator::ir::llm_contract::{
    CodeGenerationRequest, CodeGenerationResponse, CodeGenerator, GeneratedFile, GenerationError,
    GenerationStatus, LlmProvider, LlmServiceConfig, PrefabContractSpec, ValidationError,
};
use serde::{Deserialize, Serialize};

/// DeepSeek LLM code generator
pub struct DeepSeekCodeGenerator {
    config: LlmServiceConfig,
}

impl DeepSeekCodeGenerator {
    /// Create a new generator with the given configuration
    pub fn new(config: LlmServiceConfig) -> Self {
        Self { config }
    }

    /// Create from environment variables (LLM_API_KEY, LLM_MODEL, LLM_BASE_URL)
    pub fn from_env() -> Result<Self, GenerationError> {
        let api_key = std::env::var("LLM_API_KEY")
            .map_err(|_| GenerationError::new("LLM_API_KEY not set"))?;
        let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "deepseek-v4-pro".to_string());
        let base_url = std::env::var("LLM_BASE_URL").ok();

        Ok(Self::new(LlmServiceConfig {
            provider: LlmProvider::DeepSeek,
            api_key,
            model,
            base_url,
            timeout_seconds: 120,
            max_retries: 3,
            generation_params: Default::default(),
        }))
    }

    fn build_prompt(&self, request: &CodeGenerationRequest) -> String {
        // 使用结构化 OntologyModel → Prompt 转换器 (MVP)
        // 若 ontology_model 可反序列化为 OntologyModel，使用结构化 prompt；
        // 否则回退到原始字符串拼接（兼容旧接口）
        crate::generator::prompt_builder::build_prompt_from_request(request)
    }

    fn base_url(&self) -> String {
        self.config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.deepseek.com/v1".to_string())
    }
}

impl CodeGenerator for DeepSeekCodeGenerator {
    fn generate(
        &self,
        request: &CodeGenerationRequest,
    ) -> Result<CodeGenerationResponse, GenerationError> {
        if self.config.api_key.is_empty() {
            return Err(GenerationError::new("API key is empty"));
        }

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.timeout_seconds))
            .build()
            .map_err(|e| GenerationError::new(&format!("HTTP client build failed: {}", e)))?;

        let base_url = self.base_url();
        let url = format!("{}/chat/completions", base_url);

        let mut body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {"role": "system", "content": "You are a code generation assistant."},
                {"role": "user", "content": self.build_prompt(request)}
            ],
            "temperature": self.config.generation_params.temperature,
            "max_tokens": self.config.generation_params.max_tokens,
            "top_p": self.config.generation_params.top_p,
            "frequency_penalty": self.config.generation_params.frequency_penalty,
            "presence_penalty": self.config.generation_params.presence_penalty,
        });

        // DeepSeek v4 reasoning_effort support
        if let Some(ref re) = self.config.generation_params.reasoning_effort {
            if let Some(obj) = body.as_object_mut() {
                obj.insert("reasoning_effort".to_string(), serde_json::json!(re));
            }
        }

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| GenerationError::new(&format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(GenerationError::new(&format!(
                "LLM API error ({}): {}",
                status, text
            )));
        }

        let api_response: LlmApiResponse = response
            .json()
            .map_err(|e| GenerationError::new(&format!("Failed to parse API response: {}", e)))?;

        // Combine content and reasoning_content for DeepSeek v4's thinking output
        let choice = api_response.choices.into_iter().next();
        let content = choice
            .as_ref()
            .map(|c| {
                let mut text = c.message.content.clone();
                if let Some(ref rc) = c.message.reasoning_content {
                    if !rc.is_empty() {
                        if !text.is_empty() {
                            text.push_str("\n\n");
                        }
                        text.push_str(rc);
                    }
                }
                text
            })
            .unwrap_or_default();

        // Build metadata with parsed token usage
        let mut metadata = crate::generator::ir::llm_contract::GenerationMetadata {
            generated_at: chrono::Utc::now().to_rfc3339(),
            model: self.config.model.clone(),
            token_usage: crate::generator::ir::llm_contract::TokenUsage {
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 0,
            },
            generation_time_ms: 0,
            prefab_versions: std::collections::HashMap::new(),
        };
        if let Some(usage) = &api_response.usage {
            metadata.token_usage = crate::generator::ir::llm_contract::TokenUsage {
                input_tokens: usage.prompt_tokens,
                output_tokens: usage.completion_tokens,
                total_tokens: usage.total_tokens,
            };
        }

        // Attempt to parse generated files from the response content
        let generated_files = parse_generated_files(&content).unwrap_or_else(|_| {
            vec![GeneratedFile {
                file_path: "generated.rs".to_string(),
                content: content.clone(),
                file_type: "rust".to_string(),
                description: "Generated code".to_string(),
                dependencies: vec![],
            }]
        });

        Ok(CodeGenerationResponse {
            id: request.id.clone(),
            status: GenerationStatus::Success,
            generated_files,
            placement_instructions: vec![],
            metadata,
            error: None,
        })
    }

    fn validate(&self, response: &CodeGenerationResponse) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if response.status != GenerationStatus::Success {
            errors.push(ValidationError {
                code: "generation_failed".to_string(),
                message: "Generation did not complete successfully".to_string(),
                file_path: None,
                line_number: None,
            });
        }

        if response.generated_files.is_empty() {
            errors.push(ValidationError {
                code: "no_files".to_string(),
                message: "No files were generated".to_string(),
                file_path: None,
                line_number: None,
            });
        }

        // 规约门禁检查（AGENTS.md 核心边界）
        let convention_errors = crate::generator::convention_checker::ConventionChecker::check_all(
            &response.generated_files,
        );
        errors.extend(convention_errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn supported_prefabs(&self) -> Vec<PrefabContractSpec> {
        vec![
            PrefabContractSpec {
                prefab_type: crate::generator::ir::llm_contract::PrefabType::Framework,
                prefab_id: "actix-web-handler".to_string(),
                interface_version: "1.0".to_string(),
                interfaces: vec![],
                data_contracts: vec![],
                configuration_template: None,
                placement_rules: vec![],
            },
            PrefabContractSpec {
                prefab_type: crate::generator::ir::llm_contract::PrefabType::Module,
                prefab_id: "react-component".to_string(),
                interface_version: "1.0".to_string(),
                interfaces: vec![],
                data_contracts: vec![],
                configuration_template: None,
                placement_rules: vec![],
            },
        ]
    }
}

/// Stub generator for testing and offline environments
pub struct StubCodeGenerator;

impl CodeGenerator for StubCodeGenerator {
    fn generate(
        &self,
        request: &CodeGenerationRequest,
    ) -> Result<CodeGenerationResponse, GenerationError> {
        Ok(CodeGenerationResponse {
            id: request.id.clone(),
            status: GenerationStatus::Success,
            generated_files: vec![GeneratedFile {
                file_path: "stub.rs".to_string(),
                content: format!("// Stub generation for {}\n// Configure LLM_API_KEY to use real LLM generation.\n", request.target.module_name),
                file_type: "rust".to_string(),
                description: "Stub file".to_string(),
                dependencies: vec![],
            }],
            placement_instructions: vec![],
            metadata: crate::generator::ir::llm_contract::GenerationMetadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                model: "stub".to_string(),
                token_usage: crate::generator::ir::llm_contract::TokenUsage {
                    input_tokens: 0,
                    output_tokens: 0,
                    total_tokens: 0,
                },
                generation_time_ms: 0,
                prefab_versions: std::collections::HashMap::new(),
            },
            error: None,
        })
    }

    fn validate(&self, _response: &CodeGenerationResponse) -> Result<(), Vec<ValidationError>> {
        Ok(())
    }

    fn supported_prefabs(&self) -> Vec<PrefabContractSpec> {
        vec![]
    }
}

// ─── LLM API Types ───

#[derive(Debug, Clone, Deserialize)]
struct LlmApiResponse {
    choices: Vec<LlmChoice>,
    #[serde(default)]
    usage: Option<TokenUsageData>,
}

#[derive(Debug, Clone, Deserialize)]
struct LlmChoice {
    message: LlmMessage,
}

#[derive(Debug, Clone, Deserialize)]
struct LlmMessage {
    content: String,
    /// DeepSeek v4 推理过程的思考链内容
    #[serde(default)]
    reasoning_content: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LlmGeneratedFile {
    path: String,
    content: String,
}

/// DeepSeek v4 API response token usage
#[derive(Debug, Clone, Deserialize)]
struct TokenUsageData {
    #[serde(default)]
    prompt_tokens: u64,
    #[serde(default)]
    completion_tokens: u64,
    #[serde(default)]
    total_tokens: u64,
}

fn parse_generated_files(content: &str) -> Result<Vec<GeneratedFile>, ()> {
    // Try to extract JSON array from markdown code blocks
    let json_str = if let Some(start) = content.find("```json") {
        let start = start + 7;
        if let Some(end) = content[start..].find("```") {
            &content[start..start + end]
        } else {
            content
        }
    } else if let Some(start) = content.find('[') {
        if let Some(end) = content.rfind(']') {
            &content[start..=end]
        } else {
            content
        }
    } else {
        content
    };

    let files: Vec<LlmGeneratedFile> = serde_json::from_str(json_str.trim()).map_err(|_| ())?;

    Ok(files
        .into_iter()
        .map(|f| GeneratedFile {
            file_path: f.path,
            content: f.content,
            file_type: "unknown".to_string(),
            description: String::new(),
            dependencies: vec![],
        })
        .collect())
}

impl GenerationError {
    fn new(msg: &str) -> Self {
        Self {
            code: "llm_error".to_string(),
            message: msg.to_string(),
            details: None,
            suggestions: vec!["Check your API key and network connection".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::llm_contract::{
        CodeGenerationRequest, EnvironmentConfig, GenerationContext, GenerationOptions,
        GenerationTarget, ProjectStructure, TargetPlatform, TargetType,
    };

    #[test]
    fn test_stub_generator() {
        let gen = StubCodeGenerator;
        let request = CodeGenerationRequest {
            id: "test-1".to_string(),
            target: GenerationTarget {
                target_type: TargetType::Module,
                platform: TargetPlatform::Rust,
                output_path: "/tmp".to_string(),
                module_name: "test".to_string(),
            },
            ontology_model: "{}".to_string(),
            user_intent: "test".to_string(),
            prefab_contracts: vec![],
            context: GenerationContext {
                project_structure: ProjectStructure {
                    root_directory: "/tmp".to_string(),
                    directories: vec![],
                    files: vec![],
                },
                existing_code: vec![],
                dependencies: vec![],
                environment: EnvironmentConfig {
                    target_environment: "test".to_string(),
                    runtime_version: "tokio".to_string(),
                    database_config: None,
                    cache_config: None,
                    message_queue_config: None,
                },
            },
            options: GenerationOptions {
                code_style: crate::generator::ir::llm_contract::CodeStyle::Standard,
                comment_level: crate::generator::ir::llm_contract::CommentLevel::Standard,
                error_handling:
                    crate::generator::ir::llm_contract::ErrorHandlingStrategy::ResultType,
                log_level: crate::generator::ir::llm_contract::LogLevel::Info,
                include_tests: true,
                include_docs: true,
                custom_options: std::collections::HashMap::new(),
            },
        };

        let response = gen.generate(&request).unwrap();
        assert_eq!(response.status, GenerationStatus::Success);
        assert!(!response.generated_files.is_empty());
    }
}
