//! ModelRegistry — 模型注册表与别名解析
//!
//! ## 核心能力
//! - 规范模型名 + 别名映射
//! - 能力标记（supports_tools, supports_reasoning）
//! - 回退链（用户输入 → 最接近的可用模型）
//!
//! 用于 AppAgent 在生成过程中统一模型选择，
//! 确保 `model_override` 中写 `deepseek-chat` 也能自动解析到 `deepseek-v4-flash`。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 模型能力标记
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub supports_tools: bool,
    pub supports_reasoning: bool,
}

/// 注册表中的单条模型记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    /// 规范的模型标识符（用于 API 调用）
    pub id: String,
    /// 人类可读的名称
    pub name: String,
    /// 允许的别名（不区分大小写）
    pub aliases: Vec<String>,
    /// 能力标记
    pub capabilities: ModelCapabilities,
    /// 推理努力度建议（参考 oh-my-pi Effort 枚举）
    pub suggested_reasoning_effort: Option<&'static str>,
}

/// 模型注册表
#[derive(Debug, Clone)]
pub struct ModelRegistry {
    entries: Vec<ModelEntry>,
    /// 大小写不敏感的别名 → 条目索引映射
    lookup: HashMap<String, usize>,
}

impl ModelRegistry {
    /// 内置的 DeepSeek V4 模型列表
    pub fn builtin() -> Self {
        let entries = vec![
            ModelEntry {
                id: "deepseek-v4-pro".to_string(),
                name: "DeepSeek V4 Pro".to_string(),
                aliases: vec![
                    "deepseek-pro".to_string(),
                    "v4-pro".to_string(),
                    "ds-pro".to_string(),
                ],
                capabilities: ModelCapabilities {
                    supports_tools: true,
                    supports_reasoning: true,
                },
                suggested_reasoning_effort: Some("high"),
            },
            ModelEntry {
                id: "deepseek-v4-flash".to_string(),
                name: "DeepSeek V4 Flash".to_string(),
                aliases: vec![
                    "deepseek-chat".to_string(),
                    "deepseek-reasoner".to_string(),
                    "deepseek-r1".to_string(),
                    "deepseek-v3".to_string(),
                    "v4-flash".to_string(),
                    "ds-flash".to_string(),
                    "flash".to_string(),
                ],
                capabilities: ModelCapabilities {
                    supports_tools: true,
                    supports_reasoning: true,
                },
                suggested_reasoning_effort: Some("medium"),
            },
        ];

        let mut lookup = HashMap::new();
        for (idx, entry) in entries.iter().enumerate() {
            // 规范名
            lookup.insert(entry.id.to_lowercase(), idx);
            // 所有别名
            for alias in &entry.aliases {
                lookup.insert(alias.to_lowercase(), idx);
            }
        }

        Self { entries, lookup }
    }

    /// 根据用户输入解析模型名
    ///
    /// 返回:
    /// - `Some(entry)` — 找到匹配模型
    /// - `None` — 未知模型名
    pub fn resolve(&self, name: &str) -> Option<&ModelEntry> {
        let key = name.to_lowercase();
        self.lookup.get(&key).map(|&idx| &self.entries[idx])
    }

    /// 获取支持 tool calling 的默认模型
    pub fn default_tool_model(&self) -> &ModelEntry {
        // Flash 是默认的工具调用模型（快速、经济）
        self.resolve("deepseek-v4-flash")
            .or_else(|| self.entries.first())
            .expect("ModelRegistry: at least one entry")
    }

    /// 获取所有注册的模型
    pub fn all(&self) -> &[ModelEntry] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_canonical() {
        let reg = ModelRegistry::builtin();
        let entry = reg.resolve("deepseek-v4-flash").unwrap();
        assert_eq!(entry.id, "deepseek-v4-flash");
        assert!(entry.capabilities.supports_tools);
    }

    #[test]
    fn test_resolve_alias() {
        let reg = ModelRegistry::builtin();
        // "deepseek-chat" → "deepseek-v4-flash"
        let entry = reg.resolve("deepseek-chat").unwrap();
        assert_eq!(entry.id, "deepseek-v4-flash");
    }

    #[test]
    fn test_resolve_alias_case_insensitive() {
        let reg = ModelRegistry::builtin();
        let entry = reg.resolve("DeepSeek-Chat").unwrap();
        assert_eq!(entry.id, "deepseek-v4-flash");
    }

    #[test]
    fn test_resolve_deepseek_r1() {
        let reg = ModelRegistry::builtin();
        let entry = reg.resolve("deepseek-r1").unwrap();
        assert_eq!(entry.id, "deepseek-v4-flash");
    }

    #[test]
    fn test_resolve_unknown() {
        let reg = ModelRegistry::builtin();
        assert!(reg.resolve("nonexistent-model").is_none());
    }

    #[test]
    fn test_default_tool_model() {
        let reg = ModelRegistry::builtin();
        let default = reg.default_tool_model();
        assert_eq!(default.id, "deepseek-v4-flash");
    }

    #[test]
    fn test_suggested_reasoning_effort() {
        let reg = ModelRegistry::builtin();
        let pro = reg.resolve("deepseek-v4-pro").unwrap();
        assert_eq!(pro.suggested_reasoning_effort, Some("high"));
        let flash = reg.resolve("deepseek-v4-flash").unwrap();
        assert_eq!(flash.suggested_reasoning_effort, Some("medium"));
    }
}
