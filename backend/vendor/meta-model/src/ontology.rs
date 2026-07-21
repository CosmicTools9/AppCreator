//! IR Ontology Semantic Model
//!
//! 本体语义表达模型 - 描述领域本体、关系、交易生命周期
//! 用于可视化推演和 LLM 代码生成指导
//!
//! 设计原则：
//! - 描述"是什么"（本体），而非"怎么做"（实现）
//! - 支持本体关系推理和可视化
//! - 与预制件通过接口契约关联

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 本体模型根节点
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OntologyModel {
    /// 本体标识
    #[serde(default)]
    pub id: String,
    /// 本体名称
    #[serde(default)]
    pub name: String,
    /// 本体描述
    pub description: Option<String>,
    /// 版本
    #[serde(default)]
    pub version: String,

    /// 领域本体集合
    pub domains: Vec<DomainOntology>,
    /// 交易生命周期定义
    pub transaction_lifecycle: Option<TransactionLifecycle>,
    /// 关系本体定义
    pub relations: Vec<RelationOntology>,
    /// 约束本体
    pub constraints: Vec<ConstraintOntology>,
    /// 计算本体
    pub computations: Vec<ComputationOntology>,

    /// 命名空间定义
    #[serde(default)]
    pub namespaces: HashMap<String, String>,
    /// 元数据
    #[serde(default)]
    pub metadata: OntologyMetadata,
}

/// 领域本体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DomainOntology {
    /// 本体标识
    #[serde(default)]
    pub id: String,
    /// 本体名称
    #[serde(default)]
    pub name: String,
    /// 本体描述
    pub description: Option<String>,
    /// 本体类型
    pub kind: DomainKind,
    /// 父本体（继承关系）
    pub parent_ids: Vec<String>,
    /// 等价本体
    pub equivalent_ids: Vec<String>,
    /// 互斥本体
    pub disjoint_ids: Vec<String>,
    /// 本体属性
    pub properties: Vec<OntologyProperty>,
    /// 预制件接口契约
    pub prefab_contract: Option<PrefabContract>,
}

/// 领域类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DomainKind {
    /// 实体本体 - 业务实体（如订单、客户）
    #[default]
    #[serde(alias = "Entity")]
    Entity,
    /// 值对象本体 - 无标识的业务概念（如地址、金额）
    #[serde(alias = "ValueObject")]
    ValueObject,
    /// 聚合根本体 - 一致性边界
    #[serde(alias = "AggregateRoot")]
    AggregateRoot,
    /// 领域服务本体 - 跨实体的业务操作
    #[serde(alias = "DomainService")]
    DomainService,
    /// 事件本体 - 领域事件
    #[serde(alias = "DomainEvent")]
    DomainEvent,
    /// 枚举本体 - 有限值集合
    #[serde(alias = "Enumeration")]
    Enumeration,
}

/// 本体属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyProperty {
    /// 属性标识
    #[serde(default)]
    pub id: String,
    /// 属性名称
    #[serde(default)]
    pub name: String,
    /// 属性类型
    pub property_type: PropertyType,
    /// 是否必需
    pub required: bool,
    /// 基数约束
    pub cardinality: Cardinality,
    /// 定义域（所属本体）
    pub domain: String,
    /// 值域（值类型）
    pub range: String,
    /// 是否为函数属性（单值）
    pub is_functional: bool,
    /// 是否为传递属性
    pub is_transitive: bool,
    /// 是否为对称属性
    pub is_symmetric: bool,
    /// 属性约束
    pub constraints: Vec<PropertyConstraint>,
    /// 语义描述
    pub semantic_description: Option<String>,
}

/// 属性类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    /// 数据属性 - 字面量值
    #[serde(alias = "DataProperty")]
    DataProperty,
    /// 对象属性 - 关联其他本体
    #[serde(alias = "ObjectProperty")]
    ObjectProperty,
    /// 注解属性 - 元数据
    #[serde(alias = "AnnotationProperty")]
    AnnotationProperty,
}

/// 基数约束
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Cardinality {
    /// 最小基数
    pub min: Option<u32>,
    /// 最大基数
    pub max: Option<u32>,
    /// 精确基数
    pub exact: Option<u32>,
}

/// 属性约束
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyConstraint {
    /// 约束类型
    pub constraint_type: ConstraintType,
    /// 约束值
    pub value: String,
    /// 约束描述
    pub description: Option<String>,
}

/// 约束类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintType {
    /// 值范围约束
    #[serde(alias = "Range")]
    Range,
    /// 模式匹配约束
    #[serde(alias = "Pattern")]
    Pattern,
    /// 枚举值约束
    #[serde(alias = "Enum")]
    Enum,
    /// 唯一性约束
    #[serde(alias = "Unique")]
    Unique,
    /// 自定义约束
    #[serde(alias = "Custom")]
    Custom,
}

/// 交易生命周期本体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLifecycle {
    /// 生命周期标识
    #[serde(default)]
    pub id: String,
    /// 生命周期名称
    #[serde(default)]
    pub name: String,
    /// 交易类型
    #[serde(default)]
    pub transaction_type: TransactionType,
    /// 阶段定义
    #[serde(default)]
    pub phases: Vec<TransactionPhase>,
    /// 阶段转换规则
    #[serde(default)]
    pub transitions: Vec<PhaseTransition>,
    /// 对称性定义（交易双方的对称关系）
    #[serde(default)]
    pub symmetry: TransactionSymmetry,
    /// 生命周期约束
    #[serde(default)]
    pub constraints: Vec<LifecycleConstraint>,
}

/// 交易类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    #[default]
    /// 单向交易 - 单向流动（如采购）
    #[serde(alias = "Unidirectional")]
    Unidirectional,
    /// 双向交易 - 双向流动（如转账）
    #[serde(alias = "Bidirectional")]
    Bidirectional,
    /// 循环交易 - 闭环流动（如库存调拨）
    #[serde(alias = "Cyclic")]
    Cyclic,
    /// 复合交易 - 包含子交易
    #[serde(alias = "Composite")]
    Composite,
}

/// 交易阶段
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransactionPhase {
    /// 阶段标识
    #[serde(default)]
    pub id: String,
    /// 阶段名称
    #[serde(default)]
    pub name: String,
    /// 阶段类型
    #[serde(default)]
    pub phase_type: PhaseType,
    /// 阶段顺序
    #[serde(default)]
    pub order: u32,
    /// 是否为终止阶段
    #[serde(default)]
    pub is_terminal: bool,
    /// 进入条件
    #[serde(default)]
    pub entry_conditions: Vec<String>,
    /// 退出条件
    #[serde(default)]
    pub exit_conditions: Vec<String>,
    /// 阶段内约束
    #[serde(default)]
    pub invariants: Vec<String>,
    /// 关联本体
    #[serde(default)]
    pub related_ontologies: Vec<String>,
}

/// 阶段类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PhaseType {
    #[default]
    /// 创建阶段
    #[serde(alias = "Creation")]
    Creation,
    /// 验证阶段
    #[serde(alias = "Validation")]
    Validation,
    /// 确认阶段
    #[serde(alias = "Confirmation")]
    Confirmation,
    /// 执行阶段
    #[serde(alias = "Execution")]
    Execution,
    /// 结算阶段
    #[serde(alias = "Settlement")]
    Settlement,
    /// 归档阶段
    #[serde(alias = "Archival")]
    Archival,
    /// 取消阶段
    #[serde(alias = "Cancellation")]
    Cancellation,
    /// 自定义阶段
    Custom(String),
}

/// 阶段转换
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PhaseTransition {
    /// 转换标识
    #[serde(default)]
    pub id: String,
    /// 源阶段
    #[serde(default)]
    pub from_phase: String,
    /// 目标阶段
    #[serde(default)]
    pub to_phase: String,
    /// 触发事件
    #[serde(default)]
    pub trigger_event: String,
    /// 守卫条件
    #[serde(default)]
    pub guard_conditions: Vec<String>,
    /// 转换动作
    #[serde(default)]
    pub actions: Vec<TransitionAction>,
    /// 是否为自动转换
    #[serde(default)]
    pub is_automatic: bool,
    /// 超时设置
    pub timeout: Option<TransitionTimeout>,
}

/// 转换动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionAction {
    /// 动作类型
    pub action_type: ActionType,
    /// 动作描述
    pub description: String,
    /// 关联本体
    pub target_ontology: Option<String>,
    /// 动作参数
    pub parameters: HashMap<String, String>,
}

/// 动作类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    /// 状态变更
    #[serde(alias = "StateChange")]
    StateChange,
    /// 数据创建
    #[serde(alias = "DataCreation")]
    DataCreation,
    /// 数据更新
    #[serde(alias = "DataUpdate")]
    DataUpdate,
    /// 数据删除
    #[serde(alias = "DataDeletion")]
    DataDeletion,
    /// 事件发布
    #[serde(alias = "EventPublish")]
    EventPublish,
    /// 通知发送
    #[serde(alias = "Notification")]
    Notification,
    /// 外部调用
    #[serde(alias = "ExternalCall")]
    ExternalCall,
    /// 自定义动作
    Custom(String),
}

/// 转换超时
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionTimeout {
    /// 超时时间（秒）
    pub duration: u64,
    /// 超时后的目标阶段
    pub fallback_phase: String,
    /// 超时动作
    pub timeout_action: Option<TransitionAction>,
}

/// 交易对称性
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransactionSymmetry {
    /// 是否对称
    pub is_symmetric: bool,
    /// 对称类型
    pub symmetry_type: SymmetryType,
    /// 交易方定义
    pub parties: Vec<TransactionParty>,
    /// 对称约束
    pub symmetry_constraints: Vec<String>,
}

/// 对称类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SymmetryType {
    #[default]
    /// 完全对称 - 双方行为完全一致
    #[serde(alias = "Full")]
    Full,
    /// 镜像对称 - 双方行为镜像
    #[serde(alias = "Mirror")]
    Mirror,
    /// 互补对称 - 双方行为互补
    #[serde(alias = "Complementary")]
    Complementary,
    /// 非对称 - 无对称关系
    #[serde(alias = "Asymmetric")]
    Asymmetric,
}

/// 交易方
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionParty {
    /// 方标识
    #[serde(default)]
    pub id: String,
    /// 方名称
    pub name: String,
    /// 方角色
    pub role: String,
    /// 关联本体
    pub ontology_id: String,
}

/// 生命周期约束
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleConstraint {
    /// 约束标识
    #[serde(default)]
    pub id: String,
    /// 约束类型
    pub constraint_type: LifecycleConstraintType,
    /// 约束描述
    pub description: String,
    /// 约束表达式
    pub expression: String,
    /// 错误信息
    pub error_message: Option<String>,
}

/// 生命周期约束类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleConstraintType {
    /// 前置条件
    #[serde(alias = "Precondition")]
    Precondition,
    /// 后置条件
    #[serde(alias = "Postcondition")]
    Postcondition,
    /// 不变式
    #[serde(alias = "Invariant")]
    Invariant,
    /// 时序约束
    #[serde(alias = "Temporal")]
    Temporal,
    /// 数量约束
    #[serde(alias = "Quantitative")]
    Quantitative,
}

/// 关系本体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RelationOntology {
    /// 关系标识
    #[serde(default)]
    pub id: String,
    /// 关系名称
    #[serde(default)]
    pub name: String,
    /// 关系类型
    #[serde(default)]
    pub relation_type: RelationType,
    /// 源本体
    #[serde(default)]
    pub source_ontology: String,
    /// 目标本体
    #[serde(default)]
    pub target_ontology: String,
    /// 是否为双向关系
    #[serde(default)]
    pub is_bidirectional: bool,
    /// 关系属性
    #[serde(default)]
    pub properties: Vec<OntologyProperty>,
    /// 关系约束
    #[serde(default)]
    pub constraints: Vec<RelationConstraint>,
    /// 语义描述
    pub semantic_description: Option<String>,
}

/// 关系类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    #[default]
    /// 关联关系
    #[serde(alias = "Association")]
    Association,
    /// 聚合关系
    #[serde(alias = "Aggregation")]
    Aggregation,
    /// 组合关系
    #[serde(alias = "Composition")]
    Composition,
    /// 继承关系
    #[serde(alias = "Inheritance")]
    Inheritance,
    /// 依赖关系
    #[serde(alias = "Dependency")]
    Dependency,
    /// 实现关系
    #[serde(alias = "Realization")]
    Realization,
    /// 自定义关系
    Custom(String),
}

/// 关系约束
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationConstraint {
    /// 约束类型
    pub constraint_type: RelationConstraintType,
    /// 约束值
    pub value: String,
    /// 约束描述
    pub description: Option<String>,
}

/// 关系约束类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RelationConstraintType {
    /// 基数约束
    #[serde(alias = "Cardinality")]
    Cardinality,
    /// 唯一性约束
    #[serde(alias = "Uniqueness")]
    Uniqueness,
    /// 引用完整性
    #[serde(alias = "ReferentialIntegrity")]
    ReferentialIntegrity,
    /// 级联约束
    #[serde(alias = "Cascade")]
    Cascade,
    /// 自定义约束
    #[serde(alias = "Custom")]
    Custom,
}

/// 约束本体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConstraintOntology {
    /// 约束标识
    #[serde(default)]
    pub id: String,
    /// 约束名称
    #[serde(default)]
    pub name: String,
    /// 约束类型
    pub constraint_type: ConstraintOntologyType,
    /// 约束范围
    pub scope: ConstraintScope,
    /// 约束表达式
    pub expression: String,
    /// 约束描述
    pub description: Option<String>,
    /// 错误信息模板
    pub error_message_template: Option<String>,
    /// 严重程度
    pub severity: ConstraintSeverity,
}

/// 约束本体类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintOntologyType {
    #[default]
    /// 结构约束
    #[serde(alias = "Structural")]
    Structural,
    /// 业务规则约束
    #[serde(alias = "BusinessRule")]
    BusinessRule,
    /// 数据质量约束
    #[serde(alias = "DataQuality")]
    DataQuality,
    /// 安全约束
    #[serde(alias = "Security")]
    Security,
    /// 性能约束
    #[serde(alias = "Performance")]
    Performance,
    /// 自定义约束
    Custom(String),
}

/// 约束范围
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConstraintScope {
    /// 目标本体
    #[serde(default)]
    pub target_ontology: String,
    /// 目标属性（可选）
    pub target_property: Option<String>,
    /// 约束上下文
    #[serde(default)]
    pub context: Vec<String>,
}

/// 约束严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintSeverity {
    #[default]
    /// 错误 - 必须满足
    #[serde(alias = "Error")]
    Error,
    /// 警告 - 建议满足
    #[serde(alias = "Warning")]
    Warning,
    /// 信息 - 提示性
    #[serde(alias = "Info")]
    Info,
}

/// 计算本体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComputationOntology {
    /// 计算标识
    #[serde(default)]
    pub id: String,
    /// 计算名称
    #[serde(default)]
    pub name: String,
    /// 计算类型
    #[serde(default)]
    pub computation_type: ComputationType,
    /// 输入本体
    #[serde(default)]
    pub inputs: Vec<ComputationInput>,
    /// 输出本体
    #[serde(default)]
    pub outputs: Vec<ComputationOutput>,
    /// 计算公式
    #[serde(default)]
    pub formula: String,
    /// 计算描述
    pub description: Option<String>,
    /// 触发条件
    #[serde(default)]
    pub trigger_conditions: Vec<String>,
    /// 刷新策略
    #[serde(default)]
    pub refresh_strategy: RefreshStrategy,
}

/// 计算类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ComputationType {
    #[default]
    /// 聚合计算
    #[serde(alias = "Aggregation")]
    Aggregation,
    /// 派生计算
    #[serde(alias = "Derivation")]
    Derivation,
    /// 转换计算
    #[serde(alias = "Transformation")]
    Transformation,
    /// 验证计算
    #[serde(alias = "Validation")]
    Validation,
    /// 自定义计算
    Custom(String),
}

/// 计算输入
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComputationInput {
    /// 输入标识
    #[serde(default)]
    pub id: String,
    /// 输入名称
    pub name: String,
    /// 来源本体
    pub source_ontology: String,
    /// 来源属性
    pub source_property: Option<String>,
    /// 输入类型
    pub input_type: String,
}

/// 计算输出
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComputationOutput {
    /// 输出标识
    #[serde(default)]
    pub id: String,
    /// 输出名称
    pub name: String,
    /// 目标本体
    pub target_ontology: String,
    /// 目标属性
    pub target_property: Option<String>,
    /// 输出类型
    pub output_type: String,
}

/// 刷新策略
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RefreshStrategy {
    /// 刷新类型
    pub refresh_type: RefreshType,
    /// 刷新间隔（秒）
    pub interval: Option<u64>,
    /// 触发事件
    pub trigger_events: Vec<String>,
}

/// 刷新类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RefreshType {
    #[default]
    /// 实时刷新
    Realtime,
    /// 定时刷新
    Scheduled,
    /// 事件驱动刷新
    EventDriven,
    /// 手动刷新
    Manual,
}

/// 预制件接口契约
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabContract {
    /// 预制件类型
    pub prefab_type: PrefabType,
    /// 预制件标识
    pub prefab_id: String,
    /// 接口版本
    pub interface_version: String,
    /// 接口定义
    pub interfaces: Vec<InterfaceDefinition>,
    /// 配置参数
    pub configuration: HashMap<String, String>,
}

/// 预制件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrefabType {
    /// Framework 预制件
    Framework,
    /// Module 预制件
    Module,
    /// Gateway 预制件
    Gateway,
}

/// 接口定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceDefinition {
    /// 接口名称
    pub name: String,
    /// 接口类型
    pub interface_type: InterfaceType,
    /// 输入参数
    pub inputs: Vec<InterfaceParameter>,
    /// 输出参数
    pub outputs: Vec<InterfaceParameter>,
    /// 接口描述
    pub description: Option<String>,
}

/// 接口类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InterfaceType {
    /// 数据接口
    Data,
    /// 服务接口
    Service,
    /// 事件接口
    Event,
    /// UI 接口
    Ui,
    /// 配置接口
    Config,
}

/// 接口参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceParameter {
    /// 参数名称
    pub name: String,
    /// 参数类型
    pub parameter_type: String,
    /// 是否必需
    pub required: bool,
    /// 默认值
    pub default_value: Option<String>,
    /// 参数描述
    pub description: Option<String>,
}

/// 本体元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyMetadata {
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
    /// 作者
    pub author: Option<String>,
    /// 组织
    pub organization: Option<String>,
    /// 标签
    pub tags: Vec<String>,
    /// 文档链接
    pub documentation_url: Option<String>,
    /// 依赖模块标识列表
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// 模块版本要求（模块名 → 版本要求）
    #[serde(default)]
    pub version_requirements: HashMap<String, String>,
}

impl Default for OntologyMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            created_at: now.clone(),
            updated_at: now,
            author: None,
            organization: None,
            tags: vec![],
            documentation_url: None,
            dependencies: vec![],
            version_requirements: HashMap::new(),
        }
    }
}

/// 本体推理结果
#[derive(Debug, Clone, Default)]
pub struct OntologyInferenceResult {
    /// 推断出的新关系
    pub inferred_relations: Vec<InferredRelation>,
    /// 推断出的新属性
    pub inferred_properties: Vec<InferredProperty>,
    /// 推断出的约束
    pub inferred_constraints: Vec<InferredConstraint>,
    /// 冲突检测
    pub conflicts: Vec<OntologyConflict>,
    /// 应用的推理规则
    pub applied_rules: Vec<String>,
}

/// 推断关系
#[derive(Debug, Clone)]
pub struct InferredRelation {
    /// 源本体
    pub source: String,
    /// 目标本体
    pub target: String,
    /// 关系类型
    pub relation_type: RelationType,
    /// 推理来源
    pub inference_source: String,
}

/// 推断属性
#[derive(Debug, Clone)]
pub struct InferredProperty {
    /// 目标本体
    pub ontology: String,
    /// 属性
    pub property: OntologyProperty,
    /// 推理来源
    pub inference_source: String,
}

/// 推断约束
#[derive(Debug, Clone)]
pub struct InferredConstraint {
    /// 约束
    pub constraint: ConstraintOntology,
    /// 推理来源
    pub inference_source: String,
}

/// 本体冲突
#[derive(Debug, Clone)]
pub struct OntologyConflict {
    /// 冲突类型
    pub conflict_type: ConflictType,
    /// 冲突描述
    pub description: String,
    /// 涉及本体
    pub involved_ontologies: Vec<String>,
    /// 冲突严重程度
    pub severity: ConflictSeverity,
}

/// 冲突类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictType {
    /// 继承冲突
    InheritanceConflict,
    /// 互斥冲突
    DisjointConflict,
    /// 约束冲突
    ConstraintConflict,
    /// 关系冲突
    RelationConflict,
    /// 命名冲突
    NamingConflict,
}

/// 冲突严重程度
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictSeverity {
    /// 错误
    Error,
    /// 警告
    Warning,
    /// 信息
    Info,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ontology_model_creation() {
        let model = OntologyModel {
            id: "test-model".to_string(),
            name: "Test Model".to_string(),
            description: Some("A test ontology model".to_string()),
            version: "1.0.0".to_string(),
            domains: vec![],
            transaction_lifecycle: None,
            relations: vec![],
            constraints: vec![],
            computations: vec![],
            namespaces: HashMap::new(),
            metadata: OntologyMetadata::default(),
        };

        assert_eq!(model.id, "test-model");
        assert_eq!(model.name, "Test Model");
    }

    #[test]
    fn test_domain_ontology() {
        let domain = DomainOntology {
            id: "order".to_string(),
            name: "Order".to_string(),
            description: None,
            kind: DomainKind::AggregateRoot,
            parent_ids: vec![],
            equivalent_ids: vec![],
            disjoint_ids: vec![],
            properties: vec![],
            prefab_contract: None,
        };

        assert_eq!(domain.name, "Order");
        assert_eq!(domain.kind, DomainKind::AggregateRoot);
    }

    #[test]
    fn test_transaction_lifecycle() {
        let lifecycle = TransactionLifecycle {
            id: "order-lifecycle".to_string(),
            name: "Order Lifecycle".to_string(),
            transaction_type: TransactionType::Unidirectional,
            phases: vec![TransactionPhase {
                id: "creation".to_string(),
                name: "Creation".to_string(),
                phase_type: PhaseType::Creation,
                order: 1,
                is_terminal: false,
                entry_conditions: vec![],
                exit_conditions: vec![],
                invariants: vec![],
                related_ontologies: vec!["order".to_string()],
            }],
            transitions: vec![],
            symmetry: TransactionSymmetry {
                is_symmetric: false,
                symmetry_type: SymmetryType::Asymmetric,
                parties: vec![],
                symmetry_constraints: vec![],
            },
            constraints: vec![],
        };

        assert_eq!(lifecycle.phases.len(), 1);
        assert_eq!(lifecycle.phases[0].phase_type, PhaseType::Creation);
    }
}
