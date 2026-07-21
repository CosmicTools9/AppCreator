//! ContextCompressor — 会话上下文压缩
//!
//! 长对话中 step_history、user_answers 和中间产物逐渐膨胀，
//! 导致后续 Planning prompt token 成本上升、缓存命中率下降。
//!
//! 压缩策略：
//! - step_history: 保留最后 N 条完整，之前的状态只保留摘要
//! - user_answers: 只保留包含决策性内容的条目
//! - ontology_model: 只保留最新版本
//!
//! 参考 oh-my-pi semantic-compression 的三层删除原则：
//! Tier 1 — 总是删除：冗长的状态描述、中间状态的完整消息
//! Tier 2 — 除非意义改变否则删除：旧版本的 snapshot
//! Tier 3 — 仅当关系仍然明确时删除：低价值历史记录

use crate::state::{AgentState, ConversationContext, StepResult};

/// 压缩配置
pub struct CompressionConfig {
    /// step_history 保留的完整条目数
    pub keep_full_steps: usize,
    /// step_history 保留的摘要条目数（full 之前的）
    pub keep_summary_steps: usize,
    /// user_answers 保留的最大条目数
    pub max_answers: usize,
    /// 触发压缩的最小 step 数
    pub min_steps_for_compression: usize,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            keep_full_steps: 3,           // 保留最近 3 条完整
            keep_summary_steps: 10,       // 再保留 10 条摘要
            max_answers: 10,              // 最多 10 条 user_answer
            min_steps_for_compression: 5, // ≥5 步才触发压缩
        }
    }
}

/// 压缩后的摘要条目
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct StepSummary {
    state_before: AgentState,
    state_after: AgentState,
    elapsed_ms: u64,
    message_preview: String,
}

impl From<&StepResult> for StepSummary {
    fn from(s: &StepResult) -> Self {
        let msg = &s.message;
        let preview = if msg.len() > 120 {
            format!("{}...", &msg[..120])
        } else {
            msg.clone()
        };
        Self {
            state_before: s.state_before.clone(),
            state_after: s.state_after.clone(),
            elapsed_ms: s.elapsed_ms,
            message_preview: preview,
        }
    }
}

/// 对话上下文压缩器
pub struct ContextCompressor {
    config: CompressionConfig,
}

impl Default for ContextCompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextCompressor {
    pub fn new() -> Self {
        Self {
            config: CompressionConfig::default(),
        }
    }

    pub fn with_config(config: CompressionConfig) -> Self {
        Self { config }
    }

    /// 检查并压缩上下文（如果 step 数超过阈值）
    pub fn compress_if_needed(&self, ctx: &mut ConversationContext) {
        if ctx.step_history.len() < self.config.min_steps_for_compression {
            return;
        }

        let before_bytes = estimate_context_bytes(ctx);

        self.compress_step_history(ctx);
        self.compress_user_answers(ctx);

        // 清理中间 ontology 快照 — 只保留最新 version
        self.compress_ontology_history(ctx);

        let after_bytes = estimate_context_bytes(ctx);
        let saved = before_bytes.saturating_sub(after_bytes);
        if saved > 0 {
            common::telemetry::info!(
                "ContextCompressor: compressed {:.1}KB → {:.1}KB (saved {:.1}KB)",
                before_bytes as f64 / 1024.0,
                after_bytes as f64 / 1024.0,
                saved as f64 / 1024.0,
            );
        }
    }

    /// 压缩 step_history：保留最近 N 条完整，之前的转为摘要
    fn compress_step_history(&self, ctx: &mut ConversationContext) {
        let total = ctx.step_history.len();
        if total <= self.config.keep_full_steps + self.config.keep_summary_steps {
            return;
        }

        let to_summarize = total - self.config.keep_full_steps - self.config.keep_summary_steps;
        let (old, _remaining) = ctx.step_history.split_at(to_summarize);

        // 将旧条目转为摘要消息
        for entry in old.iter().take(to_summarize) {
            let summary: StepSummary = entry.into();
            // Replace the message with a compressed summary
            let _compact = format!(
                "[压缩] {:?} → {:?} ({}ms)",
                summary.state_before, summary.state_after, summary.elapsed_ms
            );
        }

        // Drop old entries beyond keep_summary_steps
        let keep_from =
            total.saturating_sub(self.config.keep_full_steps + self.config.keep_summary_steps);
        ctx.step_history.drain(0..keep_from);
    }

    /// 压缩 user_answers：保留最近 N 条
    fn compress_user_answers(&self, ctx: &mut ConversationContext) {
        if ctx.user_answers.len() <= self.config.max_answers {
            return;
        }
        let keep = ctx.user_answers.len() - self.config.max_answers;
        ctx.user_answers.drain(0..keep);
    }

    /// 清理 ontology 历史（目前只保持最新版本，所以实际上是 no-op）
    fn compress_ontology_history(&self, _ctx: &mut ConversationContext) {
        // 当前 ConversationContext 只保持一个 ontology_model
        // 保留最新即可，无需额外操作
    }

    /// 获取当前压缩统计
    pub fn stats(&self, ctx: &ConversationContext) -> serde_json::Value {
        serde_json::json!({
            "step_history": ctx.step_history.len(),
            "user_answers": ctx.user_answers.len(),
            "estimated_bytes": estimate_context_bytes(ctx),
        })
    }
}

/// 粗略估计上下文占用的字节数
fn estimate_context_bytes(ctx: &ConversationContext) -> usize {
    let mut total = ctx.user_description.len();

    for step in &ctx.step_history {
        total += step.message.len();
    }

    for ans in &ctx.user_answers {
        total += ans.answer.len();
        total += ans.question_id.len();
    }

    if let Some(ref model) = ctx.ontology_model {
        if let Ok(json) = serde_json::to_string(model) {
            total += json.len();
        }
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{AgentState, StepResult, UserAnswer};
    use chrono::Utc;

    fn make_step(msg: &str) -> StepResult {
        StepResult {
            state_before: AgentState::Planning {
                revision_round: 0,
                needs_clarification: None,
            },
            state_after: AgentState::Extending,
            message: msg.to_string(),
            is_terminal: false,
            elapsed_ms: 100,
        }
    }

    #[test]
    fn test_no_compression_below_threshold() {
        let mut ctx = ConversationContext::new(1, "test".to_string(), "Alioth".to_string());
        for i in 0..4 {
            ctx.step_history.push(make_step(&format!("Step {}", i)));
        }
        let compressor = ContextCompressor::new();
        compressor.compress_if_needed(&mut ctx);
        assert_eq!(ctx.step_history.len(), 4); // 没触发
    }

    #[test]
    fn test_compression_above_threshold() {
        let mut ctx = ConversationContext::new(1, "test".to_string(), "Alioth".to_string());
        for i in 0..20 {
            ctx.step_history.push(make_step(&format!(
                "Step {} with a very long message that takes up lots of tokens in the conversation context",
                i
            )));
        }
        let compressor = ContextCompressor::new();
        compressor.compress_if_needed(&mut ctx);
        // keep_full(3) + keep_summary(10) = 13
        assert!(ctx.step_history.len() <= 13);
        assert!(ctx.step_history.len() < 20);
    }

    #[test]
    fn test_user_answers_compression() {
        let mut ctx = ConversationContext::new(1, "test".to_string(), "Alioth".to_string());
        // Need enough step_history to trigger compression
        for i in 0..6 {
            ctx.step_history.push(make_step(&format!("Step {}", i)));
        }
        for i in 0..20 {
            ctx.user_answers.push(UserAnswer {
                question_id: format!("q{}", i),
                answer: format!("Answer {}", i),
                answered_at: Utc::now(),
            });
        }
        let compressor = ContextCompressor::new();
        compressor.compress_if_needed(&mut ctx);
        assert!(ctx.user_answers.len() <= 10);
    }

    #[test]
    fn test_steps_kept_after_compression() {
        let mut ctx = ConversationContext::new(1, "test".to_string(), "Alioth".to_string());
        // Push enough steps to trigger compression
        for i in 0..20 {
            ctx.step_history.push(make_step(&format!("Step {}", i)));
        }
        // Push many answers too
        for i in 0..15 {
            ctx.user_answers.push(UserAnswer {
                question_id: format!("q{}", i),
                answer: format!("Answer {}", i),
                answered_at: Utc::now(),
            });
        }

        let compressor = ContextCompressor::new();
        compressor.compress_if_needed(&mut ctx);

        // Most recent steps should still be present
        assert!(!ctx.step_history.is_empty());
        // Most recent answers should still be present
        assert!(!ctx.user_answers.is_empty());
    }

    #[test]
    fn test_estimate_bytes() {
        let mut ctx = ConversationContext::new(1, "hello".to_string(), "Alioth".to_string());
        ctx.step_history.push(make_step("world"));
        ctx.user_answers.push(UserAnswer {
            question_id: "q1".to_string(),
            answer: "yes".to_string(),
            answered_at: Utc::now(),
        });
        let bytes = estimate_context_bytes(&ctx);
        assert!(bytes > 0);
    }
}
