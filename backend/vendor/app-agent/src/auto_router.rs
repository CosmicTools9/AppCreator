//! AutoRouter — 轻量路由决策。
//!
//! 在真实 LLM 调用前，先使用关闭思考的 Flash 模型做一次微型路由调用，
//! 评估当前任务的复杂度，决定使用 Pro/Flash + 何种 reasoning level。
//! 如果路由调用失败（网络/超时/解析错误），回退到基于 AgentState 的本地启发式规则。

use serde::{Deserialize, Serialize};

/// 路由决策
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoutePlan {
    /// 推荐的模型名（如 deepseek-v4-pro, deepseek-v4-flash）
    pub model: ModelTier,
    /// 推荐的 reasoning effort
    pub reasoning_effort: ReasoningTier,
}

/// 模型档次
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelTier {
    /// 旗舰推理模型 — 复杂本体建模、架构设计
    Pro,
    /// 快速经济模型 — 轻量修复、格式转换
    Flash,
}

/// 推理深度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ReasoningTier {
    /// 关闭推理，极速响应
    Off,
    /// 最低推理，简单任务
    Low,
    /// 中等推理，平衡速度与质量
    Medium,
    /// 深度推理，复杂任务
    High,
    /// 最大推理，质量优先
    Max,
}

impl ReasoningTier {
    pub fn as_api_value(&self) -> Option<&'static str> {
        match self {
            Self::Off => None, // 不发送 reasoning_effort
            Self::Low => Some("low"),
            Self::Medium => None, // 默认值，省略以保持缓存
            Self::High => Some("high"),
            Self::Max => Some("high"),
        }
    }
}

impl ModelTier {
    pub fn model_id(&self) -> &'static str {
        match self {
            Self::Pro => "deepseek-v4-pro",
            Self::Flash => "deepseek-v4-flash",
        }
    }
}

/// 路由提示 → LLM 评估当前任务复杂度
const ROUTER_PROMPT: &str = r#"Analyze this task and respond with ONLY a JSON object (no markdown):

{
  "task_complexity": "simple|moderate|complex",
  "needs_reasoning": true|false,
  "reason": "one-line explanation"
}

Rules:
- simple: known patterns, routine CRUD, single entity
- moderate: multi-entity, requires module selection
- complex: novel domain, custom workflow, deep ontology

Respond ONLY with the JSON object. No markdown, no explanation."#;

/// 路由响应格式
#[derive(Debug, Deserialize)]
struct RouterResponse {
    #[serde(default)]
    task_complexity: String,
    #[serde(default)]
    needs_reasoning: bool,
}

/// AutoRouter — 轻量路由决策器
///
/// 每次调用 `route()` 时传入 LLM 引用，避免生命周期自引用问题。
pub struct AutoRouter;

impl Default for AutoRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoRouter {
    pub fn new() -> Self {
        Self
    }

    /// 分析用户需求，返回 RoutePlan
    ///
    /// `llm` — 用于路由调用的 LLM 服务（期望使用 Flash + 低推理成本）
    pub async fn route(
        &self,
        llm: &dyn crate::llm::LlmService,
        user_description: &str,
    ) -> RoutePlan {
        // 先尝试 LLM 路由调用
        match self.llm_call_route(llm, user_description).await {
            Some(plan) => {
                common::telemetry::info!(
                    "AutoRouter: {}/{:?} via LLM",
                    plan.model.model_id(),
                    plan.reasoning_effort
                );
                plan
            }
            None => {
                // 回退：基于内容的本地启发式
                let plan = self.heuristic_route(user_description);
                common::telemetry::info!(
                    "AutoRouter: {}/{:?} via heuristic",
                    plan.model.model_id(),
                    plan.reasoning_effort
                );
                plan
            }
        }
    }

    /// 通过 LLM 调用做路由决策
    async fn llm_call_route(
        &self,
        llm: &dyn crate::llm::LlmService,
        user_description: &str,
    ) -> Option<RoutePlan> {
        let prompt = format!("{}\n\nUser request:\n{}", ROUTER_PROMPT, user_description);

        let response = llm.generate(&prompt).await.ok()?;

        // 尝试解析 JSON 响应
        let cleaned = strip_code_fence(&response);
        let parsed: RouterResponse = serde_json::from_str(&cleaned).ok()?;

        match parsed.task_complexity.as_str() {
            "simple" => Some(RoutePlan {
                model: ModelTier::Flash,
                reasoning_effort: ReasoningTier::Low,
            }),
            "moderate" => {
                if parsed.needs_reasoning {
                    Some(RoutePlan {
                        model: ModelTier::Flash,
                        reasoning_effort: ReasoningTier::Medium,
                    })
                } else {
                    Some(RoutePlan {
                        model: ModelTier::Flash,
                        reasoning_effort: ReasoningTier::Low,
                    })
                }
            }
            "complex" => {
                if parsed.needs_reasoning {
                    Some(RoutePlan {
                        model: ModelTier::Pro,
                        reasoning_effort: ReasoningTier::High,
                    })
                } else {
                    Some(RoutePlan {
                        model: ModelTier::Pro,
                        reasoning_effort: ReasoningTier::Medium,
                    })
                }
            }
            _ => None,
        }
    }

    /// 基于内容的本地启发式路由（LLM 调用失败时的回退）
    fn heuristic_route(&self, user_description: &str) -> RoutePlan {
        let lower = user_description.to_lowercase();

        // 关键词检测：复杂任务标记
        let complex_indicators = [
            "workflow",
            "审批",
            "流程",
            "状态机",
            "state machine",
            "custom",
            "自定义",
            "novel",
            "novel domain",
            "integration",
            "集成",
            "多步骤",
            "multi-step",
        ];
        let moderate_indicators = [
            "order",
            "订单",
            "product",
            "产品",
            "inventory",
            "库存",
            "module",
            "模块",
            "多个",
            "multiple",
            "关联",
            "relation",
            "relationship",
        ];

        let has_complex = complex_indicators.iter().any(|k| lower.contains(k));
        let has_moderate = moderate_indicators.iter().any(|k| lower.contains(k));

        if has_complex {
            RoutePlan {
                model: ModelTier::Pro,
                reasoning_effort: ReasoningTier::High,
            }
        } else if has_moderate {
            RoutePlan {
                model: ModelTier::Flash,
                reasoning_effort: ReasoningTier::Medium,
            }
        } else {
            RoutePlan {
                model: ModelTier::Flash,
                reasoning_effort: ReasoningTier::Low,
            }
        }
    }
}

/// 剥离 markdown code fence
fn strip_code_fence(text: &str) -> String {
    let trimmed = text.trim();
    if let Some(rest) = trimmed.strip_prefix("```") {
        if let Some(start) = rest.find('\n') {
            let body = &rest[start + 1..];
            if let Some(end) = body.rfind("```") {
                return body[..end].trim().to_string();
            }
            return body.trim().to_string();
        }
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heuristic_simple() {
        let router = AutoRouter::new();
        let plan = router.heuristic_route("创建一个客户管理功能");
        assert_eq!(plan.model, ModelTier::Flash);
        assert_eq!(plan.reasoning_effort, ReasoningTier::Low);
    }

    #[test]
    fn test_heuristic_moderate() {
        let router = AutoRouter::new();
        let plan = router.heuristic_route("创建订单管理，关联产品和库存模块");
        assert_eq!(plan.model, ModelTier::Flash);
        assert_eq!(plan.reasoning_effort, ReasoningTier::Medium);
    }

    #[test]
    fn test_heuristic_complex() {
        let router = AutoRouter::new();
        let plan = router.heuristic_route("创建一个采购审批流程，包含自定义状态机和多步骤工作流");
        assert_eq!(plan.model, ModelTier::Pro);
        assert_eq!(plan.reasoning_effort, ReasoningTier::High);
    }

    #[test]
    fn test_strip_code_fence() {
        let input = "```json\n{\"task_complexity\": \"simple\"}\n```";
        assert_eq!(strip_code_fence(input), "{\"task_complexity\": \"simple\"}");
    }

    #[test]
    fn test_strip_code_fence_no_fence() {
        let input = "{\"task_complexity\": \"simple\"}";
        assert_eq!(strip_code_fence(input), "{\"task_complexity\": \"simple\"}");
    }

    #[test]
    fn test_model_tier_id() {
        assert_eq!(ModelTier::Pro.model_id(), "deepseek-v4-pro");
        assert_eq!(ModelTier::Flash.model_id(), "deepseek-v4-flash");
    }

    #[test]
    fn test_reasoning_api_value() {
        assert_eq!(ReasoningTier::Off.as_api_value(), None);
        assert_eq!(ReasoningTier::Low.as_api_value(), Some("low"));
        assert_eq!(ReasoningTier::Medium.as_api_value(), None);
        assert_eq!(ReasoningTier::High.as_api_value(), Some("high"));
        assert_eq!(ReasoningTier::Max.as_api_value(), Some("high"));
    }

    #[allow(dead_code)]
    struct MockLlm;
    #[async_trait::async_trait]
    impl crate::llm::LlmService for MockLlm {
        async fn generate(&self, _prompt: &str) -> Result<String, crate::llm::LlmError> {
            Err(crate::llm::LlmError {
                message: "mock".into(),
            })
        }
    }
}
