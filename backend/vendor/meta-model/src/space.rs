//! 4D Space (Quaternions Space) - 四维空间定位系统
//!
//! 提供本体4元数空间定位能力：时间(T) | 场景(S) | 要素(Fa) | 职能(Fu)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::ontology::Position4D;

/// 维度枚举
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Dimension {
    Temporal, // T - 时间
    Scene,    // S - 场景
    Factor,   // Fa - 要素
    Function, // Fu - 职能
}

impl std::fmt::Display for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Dimension::Temporal => write!(f, "T"),
            Dimension::Scene => write!(f, "S"),
            Dimension::Factor => write!(f, "Fa"),
            Dimension::Function => write!(f, "Fu"),
        }
    }
}

/// 时间维度上下文 (IR-1)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemporalContext {
    /// 版本标识，如 "v1", "v2"
    #[serde(default)]
    pub version: Option<String>,
    /// 有效期开始时间
    #[serde(default)]
    pub valid_from: Option<String>,
    /// 有效期结束时间
    #[serde(default)]
    pub valid_to: Option<String>,
    /// 是否启用历史记录追踪
    #[serde(default)]
    pub history_enabled: bool,
    /// 时间戳字段名（用于版本控制）
    #[serde(default)]
    pub timestamp_field: Option<String>,
}

/// 场景维度上下文 (IR-1)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SceneContext {
    /// 业务域定义，如 "sales", "inventory"
    #[serde(default)]
    pub domain: Option<String>,
    /// 业务上下文标识
    #[serde(default)]
    pub business_context: Option<String>,
    /// 环境配置，如 "production", "staging", "development"
    #[serde(default)]
    pub environment: Option<String>,
    /// 扩展属性
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

/// 要素维度上下文 (IR-1)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FactorContext {
    /// 要素类型，如 "transactional", "master", "reference"
    #[serde(default)]
    pub factor_type: Option<String>,
    /// 分类标识
    #[serde(default)]
    pub category: Option<String>,
    /// 维度定义列表
    #[serde(default)]
    pub dimensions: Vec<String>,
    /// 要素属性
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

/// 职能维度上下文 (IR-1)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FunctionContext {
    /// 角色职能，如 "manager", "operator", "viewer"
    #[serde(default)]
    pub role: Option<String>,
    /// 职责定义列表
    #[serde(default)]
    pub responsibilities: Vec<String>,
    /// 能力模型标识
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// 职能属性
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

/// 4D空间上下文 (IR-1) - 聚合所有四维信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpaceContext4D {
    /// 时间维度
    #[serde(default)]
    pub temporal: TemporalContext,
    /// 场景维度
    #[serde(default)]
    pub scene: SceneContext,
    /// 要素维度
    #[serde(default)]
    pub factor: FactorContext,
    /// 职能维度
    #[serde(default)]
    pub function: FunctionContext,
    /// 完整4D坐标（可由各维度推导）
    #[serde(default)]
    pub position: Option<Position4D>,
}

// ============== IR-2 Generator Types ==============

/// 生成器时间上下文 (IR-2)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneratorTemporalContext {
    pub version: Option<String>,
    pub version_snake: String,
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
    pub history_enabled: bool,
    pub timestamp_field: String,
    /// 生成的版本控制表名
    pub history_table_name: Option<String>,
}

/// 生成器场景上下文 (IR-2)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneratorSceneContext {
    pub domain: Option<String>,
    pub domain_snake: String,
    pub business_context: Option<String>,
    pub environment: Option<String>,
    /// 生成的路由前缀
    pub route_prefix: String,
    /// API版本路径
    pub api_version_path: String,
}

/// 生成器要素上下文 (IR-2)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneratorFactorContext {
    pub factor_type: Option<String>,
    pub factor_type_snake: String,
    pub category: Option<String>,
    pub category_snake: String,
    pub dimensions: Vec<String>,
    /// 生成的分类查询字段
    pub category_field_name: String,
}

/// 生成器职能上下文 (IR-2)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneratorFunctionContext {
    pub role: Option<String>,
    pub role_snake: String,
    pub responsibilities: Vec<String>,
    pub capabilities: Vec<String>,
    /// 生成的权限检查函数名
    pub permission_check_fn: String,
    /// 角色相关的NGAC属性名
    pub ngac_attribute_name: Option<String>,
}

/// 生成器4D坐标 (IR-2)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct GeneratorPosition4D {
    pub temporal: Option<String>,
    pub temporal_snake: String,
    pub scene: Option<String>,
    pub scene_snake: String,
    pub factor: Option<String>,
    pub factor_snake: String,
    pub function: Option<String>,
    pub function_snake: String,
    /// 完整的坐标字符串（用于索引）
    pub coordinate_string: String,
}

/// 生成器4D空间上下文 (IR-2)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneratorSpaceContext4D {
    pub temporal: GeneratorTemporalContext,
    pub scene: GeneratorSceneContext,
    pub factor: GeneratorFactorContext,
    pub function: GeneratorFunctionContext,
    pub position: Option<GeneratorPosition4D>,
    /// 是否启用4D空间特性
    pub enabled: bool,
}

// ============== Space Query Engine ==============

/// 空间查询过滤器
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpaceQueryFilter {
    /// 时间维度过滤
    pub temporal: Option<String>,
    /// 场景维度过滤
    pub scene: Option<String>,
    /// 要素维度过滤
    pub factor: Option<String>,
    /// 职能维度过滤
    pub function: Option<String>,
    /// 是否启用模糊匹配
    pub fuzzy_match: bool,
    /// 时间范围查询（开始）
    pub temporal_from: Option<String>,
    /// 时间范围查询（结束）
    pub temporal_to: Option<String>,
}

impl SpaceQueryFilter {
    /// 创建空过滤器
    pub fn new() -> Self {
        Self::default()
    }

    /// 按时间维度过滤
    pub fn with_temporal(mut self, t: impl Into<String>) -> Self {
        self.temporal = Some(t.into());
        self
    }

    /// 按场景维度过滤
    pub fn with_scene(mut self, s: impl Into<String>) -> Self {
        self.scene = Some(s.into());
        self
    }

    /// 按要素维度过滤
    pub fn with_factor(mut self, fa: impl Into<String>) -> Self {
        self.factor = Some(fa.into());
        self
    }

    /// 按职能维度过滤
    pub fn with_function(mut self, fu: impl Into<String>) -> Self {
        self.function = Some(fu.into());
        self
    }

    /// 检查位置是否匹配过滤器
    pub fn matches(&self, pos: &Position4D) -> bool {
        if let Some(ref t) = self.temporal {
            if !self.fuzzy_match {
                if pos.temporal.as_ref() != Some(t) {
                    return false;
                }
            } else if let Some(ref pos_t) = pos.temporal {
                if !pos_t.contains(t) {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let Some(ref s) = self.scene {
            if !self.fuzzy_match {
                if pos.scene.as_ref() != Some(s) {
                    return false;
                }
            } else if let Some(ref pos_s) = pos.scene {
                if !pos_s.contains(s) {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let Some(ref fa) = self.factor {
            if !self.fuzzy_match {
                if pos.factor.as_ref() != Some(fa) {
                    return false;
                }
            } else if let Some(ref pos_fa) = pos.factor {
                if !pos_fa.contains(fa) {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let Some(ref fu) = self.function {
            if !self.fuzzy_match {
                if pos.function.as_ref() != Some(fu) {
                    return false;
                }
            } else if let Some(ref pos_fu) = pos.function {
                if !pos_fu.contains(fu) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

/// 空间索引项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceIndexEntry {
    /// 实体名称
    pub entity_name: String,
    /// 4D坐标
    pub position: Position4D,
    /// 额外元数据
    pub metadata: HashMap<String, String>,
}

/// 4D空间索引引擎
#[derive(Debug, Clone, Default)]
pub struct SpaceIndex {
    entries: Vec<SpaceIndexEntry>,
}

impl SpaceIndex {
    /// 创建新的空间索引
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加条目到索引
    pub fn add(&mut self, entry: SpaceIndexEntry) {
        self.entries.push(entry);
    }

    /// 根据过滤器查询条目
    pub fn query(&self, filter: &SpaceQueryFilter) -> Vec<&SpaceIndexEntry> {
        self.entries
            .iter()
            .filter(|e| filter.matches(&e.position))
            .collect()
    }

    /// 按单一维度投影查询
    pub fn project(&self, dim: Dimension, value: &str) -> Vec<&SpaceIndexEntry> {
        let filter = match dim {
            Dimension::Temporal => SpaceQueryFilter::new().with_temporal(value),
            Dimension::Scene => SpaceQueryFilter::new().with_scene(value),
            Dimension::Factor => SpaceQueryFilter::new().with_factor(value),
            Dimension::Function => SpaceQueryFilter::new().with_function(value),
        };
        self.query(&filter)
    }

    /// 按时间维度投影
    pub fn project_temporal(&self, t: &str) -> Vec<&SpaceIndexEntry> {
        self.project(Dimension::Temporal, t)
    }

    /// 按场景维度投影
    pub fn project_scene(&self, s: &str) -> Vec<&SpaceIndexEntry> {
        self.project(Dimension::Scene, s)
    }

    /// 按要素维度投影
    pub fn project_factor(&self, fa: &str) -> Vec<&SpaceIndexEntry> {
        self.project(Dimension::Factor, fa)
    }

    /// 按职能维度投影
    pub fn project_function(&self, fu: &str) -> Vec<&SpaceIndexEntry> {
        self.project(Dimension::Function, fu)
    }

    /// 获取所有条目
    pub fn all(&self) -> &[SpaceIndexEntry] {
        &self.entries
    }

    /// 清空索引
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// 从模型构建空间索引
    pub fn from_models(models: &[super::ir2::GeneratorEntity]) -> Self {
        let mut index = Self::new();
        for model in models {
            if let Some(ref space_ctx) = model.space_4d {
                if let Some(ref pos) = space_ctx.position {
                    let entry = SpaceIndexEntry {
                        entity_name: model.name.raw.clone(),
                        position: Position4D {
                            temporal: pos.temporal.clone(),
                            scene: pos.scene.clone(),
                            factor: pos.factor.clone(),
                            function: pos.function.clone(),
                        },
                        metadata: {
                            let mut m = HashMap::new();
                            m.insert("pascal_name".to_string(), model.name.pascal.clone());
                            m.insert("snake_name".to_string(), model.name.snake.clone());
                            m
                        },
                    };
                    index.add(entry);
                }
            }
        }
        index
    }
}

/// 多维视图定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiDimensionalView {
    /// 视图名称
    pub name: String,
    /// 视图维度（哪些维度被选中）
    pub dimensions: Vec<Dimension>,
    /// 过滤条件
    pub filter: SpaceQueryFilter,
    /// 包含的实体名称列表
    pub entities: Vec<String>,
}

/// 多维视图生成器
#[derive(Debug, Clone, Default)]
pub struct ViewGenerator;

impl ViewGenerator {
    /// 生成单一维度视图
    pub fn generate_single_dimension_view(
        &self,
        index: &SpaceIndex,
        dim: Dimension,
        value: &str,
    ) -> MultiDimensionalView {
        let entries = index.project(dim, value);
        MultiDimensionalView {
            name: format!("{}_{}_view", dim, value),
            dimensions: vec![dim],
            filter: match dim {
                Dimension::Temporal => SpaceQueryFilter::new().with_temporal(value),
                Dimension::Scene => SpaceQueryFilter::new().with_scene(value),
                Dimension::Factor => SpaceQueryFilter::new().with_factor(value),
                Dimension::Function => SpaceQueryFilter::new().with_function(value),
            },
            entities: entries.iter().map(|e| e.entity_name.clone()).collect(),
        }
    }

    /// 生成跨维度视图
    pub fn generate_cross_dimension_view(
        &self,
        index: &SpaceIndex,
        filter: &SpaceQueryFilter,
        name: impl Into<String>,
    ) -> MultiDimensionalView {
        let entries = index.query(filter);
        let mut dimensions = Vec::new();
        if filter.temporal.is_some() {
            dimensions.push(Dimension::Temporal);
        }
        if filter.scene.is_some() {
            dimensions.push(Dimension::Scene);
        }
        if filter.factor.is_some() {
            dimensions.push(Dimension::Factor);
        }
        if filter.function.is_some() {
            dimensions.push(Dimension::Function);
        }

        MultiDimensionalView {
            name: name.into(),
            dimensions,
            filter: filter.clone(),
            entities: entries.iter().map(|e| e.entity_name.clone()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_4d_creation() {
        let pos = Position4D::new("2024-Q1", "sales", "order", "manager");
        assert_eq!(pos.temporal, Some("2024-Q1".to_string()));
        assert_eq!(pos.scene, Some("sales".to_string()));
        assert_eq!(pos.factor, Some("order".to_string()));
        assert_eq!(pos.function, Some("manager".to_string()));
        assert!(pos.is_complete());
    }

    #[test]
    fn test_position_4d_partial() {
        let pos = Position4D::from_temporal("v2");
        assert_eq!(pos.temporal, Some("v2".to_string()));
        assert!(!pos.is_complete());
    }

    #[test]
    fn test_position_4d_string_rep() {
        let pos = Position4D::new("v1", "inventory", "product", "operator");
        assert_eq!(
            pos.to_string_rep(),
            "T:v1|S:inventory|Fa:product|Fu:operator"
        );

        let partial = Position4D::from_scene("sales");
        assert_eq!(partial.to_string_rep(), "T:*|S:sales|Fa:*|Fu:*");
    }

    #[test]
    fn test_space_query_filter() {
        let filter = SpaceQueryFilter::new()
            .with_scene("sales")
            .with_factor("order");

        let pos_match = Position4D::new("2024", "sales", "order", "manager");
        let pos_no_match = Position4D::new("2024", "inventory", "order", "manager");

        assert!(filter.matches(&pos_match));
        assert!(!filter.matches(&pos_no_match));
    }

    #[test]
    fn test_space_index_query() {
        let mut index = SpaceIndex::new();

        index.add(SpaceIndexEntry {
            entity_name: "Order".to_string(),
            position: Position4D::new("2024-Q1", "sales", "transactional", "manager"),
            metadata: HashMap::new(),
        });

        index.add(SpaceIndexEntry {
            entity_name: "Product".to_string(),
            position: Position4D::new("2024-Q1", "inventory", "master", "operator"),
            metadata: HashMap::new(),
        });

        index.add(SpaceIndexEntry {
            entity_name: "Customer".to_string(),
            position: Position4D::new("2024-Q2", "sales", "master", "viewer"),
            metadata: HashMap::new(),
        });

        // Test temporal projection
        let temporal_results = index.project_temporal("2024-Q1");
        assert_eq!(temporal_results.len(), 2);

        // Test scene projection
        let scene_results = index.project_scene("sales");
        assert_eq!(scene_results.len(), 2);

        // Test cross-dimensional filter
        let filter = SpaceQueryFilter::new()
            .with_scene("sales")
            .with_factor("master");
        let cross_results = index.query(&filter);
        assert_eq!(cross_results.len(), 1);
        assert_eq!(cross_results[0].entity_name, "Customer");
    }

    #[test]
    fn test_dimension_display() {
        assert_eq!(Dimension::Temporal.to_string(), "T");
        assert_eq!(Dimension::Scene.to_string(), "S");
        assert_eq!(Dimension::Factor.to_string(), "Fa");
        assert_eq!(Dimension::Function.to_string(), "Fu");
    }

    #[test]
    fn test_view_generator() {
        let mut index = SpaceIndex::new();

        index.add(SpaceIndexEntry {
            entity_name: "Order".to_string(),
            position: Position4D::new("2024", "sales", "transactional", "manager"),
            metadata: HashMap::new(),
        });

        index.add(SpaceIndexEntry {
            entity_name: "Invoice".to_string(),
            position: Position4D::new("2024", "sales", "transactional", "operator"),
            metadata: HashMap::new(),
        });

        let generator = ViewGenerator;
        let view = generator.generate_single_dimension_view(&index, Dimension::Scene, "sales");

        assert_eq!(view.name, "S_sales_view");
        assert_eq!(view.dimensions, vec![Dimension::Scene]);
        assert_eq!(view.entities.len(), 2);
        assert!(view.entities.contains(&"Order".to_string()));
        assert!(view.entities.contains(&"Invoice".to_string()));
    }
}
