use crate::matcher::tier_from_confidence;
use crate::output::{Coordinates, MappingInput, Tier, TieredValue};
use crate::rules::CoordinateInference;

pub struct CoordinateInferrer<'a> {
    rules: &'a CoordinateInference,
}

impl<'a> CoordinateInferrer<'a> {
    pub fn new(rules: &'a CoordinateInference) -> Self {
        Self { rules }
    }

    /// 坐标推理：将 LLM 推演阶段 (alioth-ontology Phase 2/3b) 产出的
    /// scene_code / factor_ids 和 entity_name 进行坐标组合。
    ///
    /// # Input 来源
    /// - `input.scene_code` / `input.factor_ids`: **MUST** 来自 Phase 2 LLM 语义对齐阶段
    /// - `entity_name`: Phase 2 产出的 Alioth 模型实体名
    ///
    /// # 职责
    /// 本方法执行坐标组合 + 子串匹配退避 (function 仅当 LLM 未指定时)，
    /// **不承担原型意图→坐标的语义推演**——那由 Phase 3b Step 1 (LLM 坐标推演) 完成。
    /// 生成的坐标应被 Phase 3b Step 2 (规则引擎 DB 校验) 验证存在性。
    pub fn infer(&self, entity_name: &str, input: &MappingInput) -> Coordinates {
        let entity_lower = entity_name.to_lowercase();

        Coordinates {
            scene: TieredValue {
                value: input.scene_code.clone(),
                tier: tier_from_confidence(self.rules.scene.confidence),
                confidence: self.rules.scene.confidence,
                source: "input".into(),
            },
            factor: TieredValue {
                value: input.factor_ids.first().cloned().unwrap_or_default(),
                tier: tier_from_confidence(self.rules.factor.confidence),
                confidence: self.rules.factor.confidence,
                source: "input".into(),
            },
            function: self.infer_function(&entity_lower),
        }
    }

    fn infer_function(&self, entity_lower: &str) -> TieredValue {
        for rule in &self.rules.function.rules {
            if rule
                .entity_types
                .iter()
                .any(|t| entity_lower.contains(t.as_str()))
            {
                return TieredValue {
                    value: rule.default.clone(),
                    tier: tier_from_confidence(rule.confidence),
                    confidence: rule.confidence,
                    source: "semantic_inference".into(),
                };
            }
        }
        // No match — unclear
        TieredValue {
            value: String::new(),
            tier: Tier::Unclear,
            confidence: 0.0,
            source: "no_match".into(),
        }
    }
}
