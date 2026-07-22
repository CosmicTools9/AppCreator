//! Agent 状态机与对话上下文

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// App Agent 业务编排状态机（7 状态：Initializing→Planning→Extending→Composing→Verifying→Presenting）
///
/// 此状态机**仅**管理 App Agent 的应用生成编排逻辑。
/// 与 `r-queen::SessionState` 是**完全独立的分层**：
/// - `AgentState`（此枚举）: 业务层 — LLM 驱动的应用组装管道
/// - `SessionState`: 传输层 — 网络连接（TCP/TLS）生命周期
///
/// 两者不共享状态转换；`AgentState` 的推进由 `Orchestrator::step()` 驱动，
/// `SessionState` 由 r-queen 的 `Session` 异步 I/O 引擎驱动。
///
pub mod progress_event {
    pub const PLANNING_START: &str = "planning_start";
    pub const ONTOLOGY_CONTEXT_QUERIED: &str = "ontology_context_queried";
    pub const LLM_ROUTE_SELECTED: &str = "llm_route_selected";
    pub const ONTOLOGY_PARSED: &str = "ontology_parsed";
    pub const PLAN_VIOLATIONS_FOUND: &str = "plan_violations_found";
    pub const CLARIFICATION_QUESTIONS: &str = "clarification_questions";
    pub const GAP_ANALYSIS_DONE: &str = "gap_analysis_done";
    pub const COMPOSING_START: &str = "composing_start";
    pub const ARTIFACT_WRITTEN: &str = "artifact_written";
    pub const GATEWAY_RESTART_TRIGGERED: &str = "gateway_restart_triggered";
    pub const GATEWAY_HEALTH_CHECK: &str = "gateway_health_check";
    pub const VERIFICATION_ERROR: &str = "verification_error";
    pub const AUTO_FIX_ATTEMPTED: &str = "auto_fix_attempted";
    pub const STEP_STARTED: &str = "step_started";
    pub const STEP_COMPLETED: &str = "step_completed";
    pub const COMPLETED: &str = "completed";
    pub const EXECUTION_LOG: &str = "execution_log";
}
/// 进度事件，通过 WebSocket 实时推送至前端
#[derive(Debug, Clone, Serialize)]
pub struct AgentProgress {
    pub state: String,
    pub percent: u8,
    pub message: String,
    pub event_kind: String,
    pub payload: Option<serde_json::Value>,
}

impl AgentProgress {
    pub fn new(
        state: impl Into<String>,
        percent: u8,
        message: impl Into<String>,
        event_kind: impl Into<String>,
        payload: Option<serde_json::Value>,
    ) -> Self {
        Self {
            state: state.into(),
            percent,
            message: message.into(),
            event_kind: event_kind.into(),
            payload,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    // ── 新版 7 阶段流水线 ──
    /// 1. 语义分析：解析用户自然语言输入，提取业务意图和关键概念
    SemanticAnalysis,
    /// 2. 功能拆解：将业务意图分解为功能单元（模块→场景→因子）
    FunctionDecomposition,
    /// 3. 本体分析：映射到 Alioth 本体论，识别实体/关系/坐标
    OntologyAnalysis {
        ontology_round: u32,
    },
    /// 4. 模块创建/组装：创建新模块或复用已有模块
    ModuleCreation,
    /// 5. 区块创建/组装：为模块创建业务区块流程骨架
    #[serde(alias = "SceneCreation")]
    BlockCreation,
    /// 6. 本体转移：将本体分析结果转移到 Factor 层
    OntologyTransfer,
    /// 7. Service API 生成：生成业务语义 DTO 和 Service API
    #[serde(alias = "factor_api", alias = "FactorAPI")]
    ServiceAPI,
    /// 自动发布：编译验证 + 发布到 Gateway
    Publishing {
        publish_attempt: u32,
        last_error: Option<String>,
    },
    Published {
        result: Box<BuildResult>,
    },

    // ── 向后兼容（旧 session 反序列化）──
    #[serde(alias = "Initializing")]
    Initializing,
    Planning {
        revision_round: u32,
        needs_clarification: Option<Vec<Question>>,
    },
    /// 缺口分析（记录未覆盖需求到 request-no-impl/）
    Extending,
    #[serde(alias = "Generating")]
    Generating,
    /// 从 Scenes/Modules 原型生成前端代码（React/TS）
    GeneratingFrontend {
        /// 本轮生成的模块数
        modules_generated: u32,
        /// 验证结果（tsc 输出摘要）
        verification_log: Option<String>,
    },
    Composing,
    /// 验证交付产物（app.json + extensions/*.yaml 格式校验）
    Verifying {
        verification_round: u32,
    },
    /// 向用户展示构建结果
    Presenting {
        result: Box<BuildResult>,
    },
    /// 等待人工干预：评估环达到上限 `MAX_EVAL_ITERATIONS` 仍不达标，自动收敛失败，
    /// 暂停并请求用户审查 `eval-report.json` / `eval-trajectory.jsonl`，调整需求/目标后重试，
    /// 或显式确认强制发布（避免静默发布低质量产物）。
    AwaitingUserInput {
        reason: String,
    },
    /// 执行 Skill 工作流
    ExecutingSkill {
        skill_name: String,
        track_index: usize,
        step_index: usize,
        /// 第几次执行尝试
        attempt: u32,
        /// 模板变量（{ns}、{module}、{block} 等），由 caller 在进入 ExecutingSkill 时填入
        context: HashMap<String, String>,
        /// 全部 Track 完成后跳回的状态（替代旧硬编码 Planning）
        #[serde(default = "default_return_state")]
        return_state: Box<AgentState>,
    },
    /// 终态：门禁重试耗尽 / Skill step 必失败时进入，等待人工干预
    Failed {
        error: String,
    },
}

fn default_return_state() -> Box<AgentState> {
    Box::new(AgentState::Planning {
        revision_round: 0,
        needs_clarification: None,
    })
}

/// 单步执行结果（可中断/恢复模式）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepResult {
    /// 执行前的状态
    pub state_before: AgentState,
    /// 执行后的状态
    pub state_after: AgentState,
    /// 渲染给用户的消息
    pub message: String,
    /// 是否为终止状态（AwaitingUserInput / Presenting / Failed）
    pub is_terminal: bool,
    /// 本步执行耗时（毫秒）
    pub elapsed_ms: u64,
}

/// 步骤执行详情（含完整 prompt/response，用于可观测和重放）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepDetail {
    /// 步骤索引（对应 step_history 中的序号）
    pub index: usize,
    /// 状态转移前
    pub state_before: AgentState,
    /// 状态转移后
    pub state_after: AgentState,
    /// 耗时（ms）
    pub elapsed_ms: u64,
    /// 是否为终止状态
    pub is_terminal: bool,
    /// LLM 调用 system prompt（仅 LLM 调用步骤有值）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm_system_prompt: Option<String>,
    /// LLM 调用 user prompt（仅 LLM 调用步骤有值）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm_user_prompt: Option<String>,
    /// LLM 原始响应（仅 LLM 调用步骤有值）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm_response: Option<String>,
    /// 规约验证违规列表（Planning 步骤有值）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_violations: Option<Vec<PlanViolation>>,
    /// 步骤消息摘要
    pub message: String,
}

/// 断点恢复配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResumeConfig {
    /// 目标状态（重置后从此状态开始）
    pub target_state: AgentState,
    /// 是否保留本体模型
    pub preserve_ontology: bool,
    /// 是否保留 flow_plan
    pub preserve_flow_plan: bool,
    /// 是否保留 compose_scratch（后端/前端产物缓存）
    pub preserve_scratch: bool,
    /// 是否保留 yaml_operations 队列
    pub preserve_yaml_ops: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Question {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub category: MissingInfoCategory,
    #[serde(default)]
    pub question: String,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MissingInfoCategory {
    #[default]
    SceneAmbiguity,
    EntityExtension,
    StatusLifecycle,
    FieldExtension,
    RelationAmbiguity,
    ModuleDependency,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissingInfo {
    #[serde(default)]
    pub category: MissingInfoCategory,
    /// 场景条件：用户当前处于什么业务场景/上下文
    #[serde(default)]
    pub scene_condition: String,
    /// 决策要素：需要用户做出决策的核心要素/选项
    #[serde(default)]
    pub decision_elements: String,
    /// 判断标准：做出决策时应依据的规则/约束/标准
    #[serde(default)]
    pub judgment_criteria: String,
    /// 判断结果：系统基于当前信息给出的倾向性结论或建议
    #[serde(default)]
    pub judgment_result: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Extension {
    pub extension_type: ExtensionType,
    pub name: String,
    pub parent: String,
    pub reason: String,
    pub fields: Vec<ExtensionField>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionType {
    Entity,
    Scene,
    Status,
    Field,
    Relation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensionField {
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub description: Option<String>,
}

// ─── App 级元数据(LLM 输出,驱动 app.json 17 字段中非结构化部分) ────────────
//
// 对齐 Pre-Proc/Alioth/_schema/app.schema.json 的可选字段。
// composer.rs::AppJson 从此结构读取 brand/navigation/routing/permissions 等。

/// LLM 输出的 App 级元数据(brand/navigation/routing 等)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppMeta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// [development, staging, production]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    /// [standalone, embedded] — null 表示自动
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "deploymentMode"
    )]
    pub deployment_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<PermissionsMeta>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routing: Option<RoutingMeta>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brand: Option<BrandMeta>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub navigation: Option<Vec<NavGroupMeta>>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "non_scope")]
    pub non_scope: Option<Vec<String>>,
    /// App 核心目标描述(对齐 app.schema.json goal 字段)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsMeta {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_roles: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub public_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub admin_roles: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RoutingMeta {
    /// pattern ^/
    pub base: String,
    /// pattern ^/
    pub default_route: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BrandMeta {
    /// HSL "H S% L%"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NavGroupMeta {
    pub group: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FunctionalUnit {
    pub name: String,
    pub description: String,
    pub entities: Vec<String>,
    pub suggested_module: Option<String>,
    #[serde(alias = "suggested_scenes")]
    pub suggested_blocks: Vec<String>,
    #[serde(alias = "suggested_factors")]
    pub suggested_services: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowPlan {
    pub used_modules: Vec<String>,
    pub namespace: String,
    pub known_entities: Vec<String>,
    pub workflow_steps: Vec<String>,
    pub missing_info: Vec<MissingInfo>,
    /// 7 阶段流水线产出：创建的模块 ID 列表
    #[serde(default)]
    pub created_modules: Vec<String>,
    /// 7 阶段流水线产出：创建的 block ID 列表
    #[serde(default, alias = "created_scenes")]
    pub created_blocks: Vec<String>,
    /// 7 阶段流水线产出：创建的 Service API 列表
    #[serde(default, alias = "created_factors")]
    pub created_services: Vec<String>,
    /// 本体分析结果（OntologyModel JSON）
    #[serde(default)]
    pub ontology_model_json: Option<String>,
    /// 功能拆解结果：功能单元列表
    #[serde(default)]
    pub functional_units: Vec<FunctionalUnit>,
    /// 语义分析结果：提取的关键概念
    #[serde(default)]
    pub semantic_concepts: Vec<String>,
    /// 计算逻辑规划：目标字段、公式、依赖字段、触发时机
    #[serde(default)]
    pub computations: Vec<ComputationPlan>,
    /// 约束验证规划：字段级与跨字段约束
    #[serde(default)]
    pub constraints: Vec<ConstraintPlan>,
    /// 业务规则规划：条件-动作规则
    #[serde(default)]
    pub business_rules: Vec<BusinessRulePlan>,
    /// LLM 输出的 App 级元数据(brand/navigation/routing 等,驱动 app.json 17 字段)
    #[serde(default)]
    pub app_meta: Option<AppMeta>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanViolation {
    pub kind: PlanViolationKind,
    pub detail: String,
    /// 是否可通过 LLM 自动修复（false = 需用户输入）
    pub fixable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanViolationKind {
    /// 引用的 domain/relation 在 PlatformCatalog 和 OntologyModel 中都不存在
    UndefinedReference,
    /// 跨层依赖（高层模块引用低层模块的实体）
    LayerViolation,
    /// 继承链冲突（试图在非中间节点上建子表）
    InheritanceConflict,
    /// 循环依赖
    CircularDependency,
    /// Domain 语义模糊（多可能解释）
    SemanticAmbiguity,
    /// 缺少关键信息
    MissingCriticalInfo,
}

// ─── Extension & Generation ─────────────────────────────────────────────────

/// 需要 Meta 扩展的缺口
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensionGap {
    pub domain_id: String,
    pub parent_table: String,
    pub proposed_table_name: String,
    pub new_fields: Vec<FieldInfo>,
    pub status: ExtensionGapStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionGapStatus {
    Pending,
    Creating,
    Synced,
    Failed { error: String },
}

/// 字段映射条目（OntologyMapper 产出的精简形态，session 序列化稳定）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappedField {
    pub json_path: String,
    /// 映射到的物理列（None = 未命中，Unclear）
    pub column: Option<String>,
    pub scalar_table: Option<String>,
    /// safe / suggest / unclear
    pub tier: String,
}

/// 本体转移产物：gap 域经 discovery 匹配到 DB 叶表 + 坐标推断结果。
///
/// 坐标纪律（与 alioth-ontology G3.5 门禁一致）：
/// - `function_code` 由 rules.yaml 实体名规则推断（可为空 = 未命中，Unclear）；
/// - `scene_code` / `factor_code` 恒为空——层2 坐标适配需原型意图上下文，
///   禁止在自动管线中猜测，留待 Composing 后的确认环节填充。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappedEntity {
    pub domain_id: String,
    /// 匹配到的 DB 叶表（isahl.zc_id_xxx）
    pub table: String,
    /// discovery 综合评分（名称 40% + 字段覆盖 60%）
    pub score: f64,
    pub name_score: f64,
    pub field_score: f64,
    /// 坐标：scene/factor 待确认（None），function 规则推断（None = 未命中规则）
    pub scene_code: Option<String>,
    pub factor_code: Option<String>,
    pub function_code: Option<String>,
    pub function_confidence: f64,
    /// 字段映射（OntologyMapper.map 产出，含 tier）
    #[serde(default)]
    pub field_mappings: Vec<MappedField>,
}

// ─── Ontology Alignment Graph ────────────────────────────────────────────
///
/// biz-ontology 到 alioth-ontology 的语义对齐图。
/// 一对一或一对多的映射，组合阿里奥斯实体、关系机制与证据。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlignmentGraph {
    /// biz-domain → alioth 实体绑定的集合
    #[serde(default)]
    pub nodes: Vec<AlignmentNode>,
    /// biz-relation → alioth 物理机制的集合
    #[serde(default)]
    pub edges: Vec<AlignmentEdge>,
    /// 未覆盖的 biz 语义
    #[serde(default)]
    pub gaps: Vec<AlignmentGap>,
}

/// 单个 biz domain 到 alioth 实体的绑定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentNode {
    /// biz domain id（对应 OntologyModel.domains[].id）
    pub biz_domain: String,
    /// AggregateRoot / Entity / ValueObject
    pub biz_kind: String,
    /// 1..N alioth 实体（一个 biz 聚合可由多个 alioth 表拼合）
    pub alioth_entities: Vec<AliothBinding>,
    #[serde(default)]
    pub evidence: String,
    #[serde(default)]
    pub confidence: f64,
}

/// 单个 Alioth 表绑定 + 角色与坐标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliothBinding {
    /// isahl.zc_id_xxx
    pub table: String,
    /// "aggregate_root" / "子实体" / "标量值"
    pub role: String,
    /// 坐标（可为空，待 LLM 推理）
    pub coordinates: Option<CoordinatesSnapshot>,
    #[serde(default)]
    pub field_mappings: Vec<MappedField>,
    /// 过滤约束（如 "code = IN_TRANSIT"），复合概念多表绑定时 [1..] 使用。
    /// 格式规约：每项为 "<列名> = <值>"，列名为 status 表字段，值为枚举字面量。
    #[serde(default)]
    pub constraints: Vec<String>,
}

/// 可序列化的坐标快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinatesSnapshot {
    pub scene: Option<String>,
    pub factor: Option<String>,
    pub function: Option<String>,
    pub function_confidence: f64,
}

/// biz 关系到 alioth 物理机制的映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbRelationEvidence {
    pub query_kind: String,              // "fk_column" | "r_table" | "rr_table"
    pub schema: String,                  // "isahl" | "isahl_meta"
    pub relation_table: String,          // 具体关系表或源表名
    pub relation_column: Option<String>, // FK 列名
    pub target_table: String,
}

/// biz 关系到 alioth 物理机制的映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentEdge {
    pub biz_rel_id: String,
    pub biz_rel_type: String,
    pub alioth_mechanism: AliothRelationMechanism,
    #[serde(default)]
    pub evidence: Option<DbRelationEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AliothRelationMechanism {
    #[serde(rename = "fk")]
    FK {
        column: String,
        target_table: String,
    },
    #[serde(rename = "r")]
    RTable { table: String },
    #[serde(rename = "rr")]
    RRTable { table: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentGap {
    pub biz_element: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub suggested_alioth_entities: Vec<String>,
}

/// 生成产物清单（已废弃：Generating 阶段移除后不再填充，保留结构用于旧 session 反序列化兼容）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneratedManifest {
    pub crate_name: String,
    pub output_dir: String,
    pub file_count: usize,
    /// external_dependencies 填充结果
    pub module_deps_loaded: usize,
}

/// 计算逻辑规划
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComputationPlan {
    pub entity: String,
    pub target_field: String,
    pub formula: String,
    pub depends_on: Vec<String>,
    pub trigger: String, // onCreate, onUpdate, onCreateOrUpdate
}

/// 约束验证规划
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstraintPlan {
    pub entity: String,
    pub field: Option<String>,
    pub expression: String,
    pub level: String, // error, warning
    pub message: String,
}

/// 业务规则规划
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BusinessRulePlan {
    pub entity: String,
    pub rule_name: String,
    pub trigger: String, // onCreate, onUpdate, onTransition, always
    pub condition: String,
    pub action: String,
    pub priority: i32,
    pub error_message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunValidationResult {
    pub health_ok: bool,
    pub health_url: String,
    pub stdout: String,
    pub stderr: String,
    pub retry_count: u8,
}
fn default_max_repair_count() -> u8 {
    8
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BuildResult {
    pub app_name: String,
    pub output_path: String,
    pub used_modules: Vec<ModuleUsage>,
    pub extensions: Vec<ExtensionResult>,
    pub generated_files: Vec<String>,
    pub pending_confirmations: Vec<String>,
    pub endpoint_url: Option<String>,
    /// Prototype preview URL (e.g., /apps/{namespace}/{code}/prototype.html)
    #[serde(default)]
    pub preview_url: Option<String>,
    #[serde(default)]
    pub runtime_validation: Option<RunValidationResult>,
    #[serde(default)]
    pub has_runtime_error: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleUsage {
    pub module_id: String,
    pub module_name: String,
    pub collections: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensionResult {
    pub extension: Extension,
    pub ddl: String,
    pub validated: bool,
}

/// 构建过程中的中间缓存
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeScratch {
    pub app_name: String,
    pub output_path: String,
    pub files_written: usize,
    pub module_count: usize,
    pub gateway_design_content: Option<String>,
}

/// YAML 结构化 Patch 操作
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YamlPatch {
    /// 路径表达式，如 "constraints[0].expression" 或 "rules/{name=check_price}"
    pub path: String,
    /// 新值（null/None 表示删除）
    pub value: Option<serde_json::Value>,
}

/// YAML 文件操作指令（由 AI 输出或用户提交）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum YamlOperation {
    /// 读取现有 YAML 文件内容
    Read { file: String },
    /// 覆盖写入 YAML 文件（完整内容替换）
    Write { file: String, content: String },
    /// 结构化 Patch
    Patch {
        file: String,
        patches: Vec<YamlPatch>,
    },
}

/// AppAgent 应用层 tool_call(不依赖 LLM API 原生 function calling)
///
/// LLM 在 OntologyOutput JSON 输出中包含 `tool_calls` 数组,
/// harness.rs 解析后由 orchestrator.rs 填充到 ConversationContext。
///
/// 序列化格式(标准 tool_call):
/// ```json
/// {"name": "write_gateway_design", "arguments": {"content": "..."}}
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "name", content = "arguments", rename_all = "snake_case")]
pub enum AgentToolCall {
    /// 写入 gateway_design.md(激活 ctx.compose_scratch.gateway_design_content)
    WriteGatewayDesign {
        /// markdown 内容(遵循 GATEWAY_DESIGN 规约)
        content: String,
    },
    /// 覆盖写入 extension YAML 文件(激活 ctx.yaml_operations)
    WriteExtensionYaml {
        /// 文件名(如 "constraints.yaml")
        file: String,
        /// YAML 完整内容
        content: String,
    },
    /// 结构化 Patch extension YAML(激活 ctx.yaml_operations)
    PatchExtensionYaml {
        /// 文件名
        file: String,
        /// patch 列表
        patches: Vec<YamlPatch>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    pub id: String,
    pub session_id: i64,
    pub user_description: String,
    /// 派生的应用名（在 Generating 阶段计算一次并缓存，保证 Generating/Composing 一致）
    #[serde(default)]
    pub app_name: Option<String>,
    /// 应用所在 namespace（如 AVIC-CAASEC、WZ、Cosmic-Tools）
    /// 通过前端会话或用户描述确定，Composing 阶段写入目录结构。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    pub state: AgentState,
    pub platform_catalog: Option<PlatformCatalog>,
    pub flow_plan: Option<FlowPlan>,
    /// 本体模型（方案 A1：Planner 主输出）
    pub ontology_model: Option<alioth_gen::generator::ir::ontology::OntologyModel>,
    pub user_answers: Vec<UserAnswer>,
    pub extensions: Vec<ExtensionResult>,
    pub generated_files: Vec<String>,
    pub pending_questions: Vec<Question>,
    /// 最近一次规划验证发现的违规（供 LLM 修正循环使用）
    pub plan_violations: Option<Vec<PlanViolation>>,
    /// 扩展缺口（OntologyModel 中有但 DB 中缺少的表）
    #[serde(default)]
    pub extension_gaps: Vec<ExtensionGap>,
    /// 生成产物清单（已废弃，保留用于旧 session 反序列化兼容）
    #[serde(default)]
    pub generated_manifest: Option<GeneratedManifest>,
    /// Meta 扩展操作的追踪集合（collection_id → sync status）
    #[serde(default)]
    pub extension_tracking: std::collections::HashMap<String, ExtensionGapStatus>,
    /// 本体转移产物（OntologyTransfer 阶段：gap 域 → DB 叶表 + 坐标）
    #[serde(default)]
    pub mapped_entities: Vec<MappedEntity>,
    /// 本体对齐图（biz-ontology ↔ alioth-ontology）
    #[serde(default)]
    pub alignment_graph: Option<AlignmentGraph>,
    /// 构建中间状态缓存（在 ComposingBackend ~ Verifying 之间传递）
    pub compose_scratch: Option<ComposeScratch>,
    /// Planning 阶段 LLM tool_call 输出的 gateway_design.md 内容(暂存,Composing 阶段转移到 compose_scratch)
    #[serde(default)]
    pub pending_gateway_design: Option<String>,
    /// 验证阶段发现的错误（用于 Building 阶段修复后重试）
    pub verification_error: Option<String>,
    /// YAML 操作队列（由 AI Planner 或用户前端提交）
    #[serde(default)]
    pub yaml_operations: Vec<YamlOperation>,
    /// YAML 操作执行结果日志
    #[serde(default)]
    pub yaml_operation_log: Vec<String>,
    /// 执行流水日志（ExecutionEvent 序列化条目，最近 500 条缓冲）
    /// 完整历史由磁盘 execution.log 维护
    #[serde(default)]
    pub execution_log: Vec<crate::execution_log::ExecutionLogEntry>,
    /// 执行历史（单步模式下每次 step 后追加）
    #[serde(default)]
    pub step_history: Vec<StepResult>,
    /// 最近一次成功到达的断点状态（用于快速恢复）
    #[serde(default)]
    pub last_checkpoint: Option<AgentState>,
    /// 步骤执行详情（含完整 prompt/response，用于重放调试）
    /// 仅保留最近 10 条，完整历史通过 execution.log + step_history 重建
    #[serde(default)]
    pub step_details: Vec<StepDetail>,
    /// 中断信号（用户请求停止执行）
    #[serde(default)]
    pub interrupt_requested: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// 用户在 Presenting 之后提出的变更请求
    #[serde(default)]
    pub change_requests: Vec<String>,
    /// 上一次成功生成的 app.json 内容摘要（用于再规划时保留上下文）
    #[serde(default)]
    pub last_built_app: Option<String>,
    /// 最近一次运行时验证结果
    #[serde(default)]
    pub runtime_validation: Option<RunValidationResult>,
    /// 本次 Verifying 阶段最大自动修复次数
    #[serde(default = "default_max_repair_count")]
    pub max_repair_count: u8,
    /// 当前自动修复尝试计数
    #[serde(default)]
    pub repair_attempt: u8,
    /// 快速草稿模式：跳过复杂扩展配置，只生成最小可运行 app.json
    #[serde(default)]
    pub draft_mode: bool,
    /// 评估环当前迭代计数（Verifying rubric 未通过时回流 Composing，上限见 evaluate::MAX_EVAL_ITERATIONS）
    #[serde(default)]
    pub eval_iteration: u32,
    /// 最近一次 rubric 评估的 critique（JSON 序列化），回流 Composing 时用于派生改进（如缺失 goal）
    #[serde(default)]
    pub eval_feedback: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAnswer {
    pub question_id: String,
    pub answer: String,
    pub answered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlatformCatalog {
    pub modules: Vec<ModuleInfo>,
    pub collections: Vec<CollectionInfo>,
    pub scenes: Vec<SceneInfo>,
    pub factors: Vec<FactorInfo>,
    pub functions: Vec<FunctionInfo>,
    pub status_bases: Vec<StatusBaseInfo>,
    pub lifecycle_entities: Vec<String>,
    pub inheritance: Vec<InheritanceEntry>,
}

/// 继承关系条目（来自 devv_inherits_union 视图）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritanceEntry {
    pub depth: i32,
    pub child_table: String,
    pub parent_table: String,
    /// 继承路径（如 "zc_ad_object → zc_ad_variable → zc_id_production"）
    pub path: String,
    /// 已有子表列表（同 parent 的直接子表）
    #[serde(default)]
    pub children: Vec<String>,
}

///
/// Collection 信息（扩展了继承元数据）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CollectionInfo {
    pub id: i64,
    pub name: String,
    pub table_name: String,
    pub fields: Vec<FieldInfo>,
    /// 直接父表名
    #[serde(default)]
    pub parent_table: Option<String>,
    /// 继承深度（0 = 根表）
    #[serde(default)]
    pub inheritance_depth: i32,
    /// 已有子表列表（非空 = 可扩展的父表）
    #[serde(default)]
    pub child_tables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub id: String,
    pub name: String,
    pub collections: Vec<CollectionInfo>,
    /// 扩展点声明（来自 module.json）
    #[serde(default, skip_serializing)]
    pub extension_points: Vec<runtime_engine::ModuleExtensionPoints>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldInfo {
    pub name: String,
    pub field_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneInfo {
    pub id: i64,
    pub code: String,
    pub notice: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorInfo {
    pub id: i64,
    pub code: String,
    pub notice: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub id: i64,
    pub code: String,
    pub notice: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBaseInfo {
    pub id: i64,
    pub code: String,
    pub notice: String,
    pub leaf_tables: Vec<LeafStatusTable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeafStatusTable {
    pub table_name: String,
    pub notice: String,
}

impl ConversationContext {
    pub fn new(session_id: i64, user_description: String, namespace: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            session_id,
            user_description,
            app_name: None,
            namespace: Some(namespace),
            state: AgentState::Initializing,
            platform_catalog: None,
            flow_plan: None,
            ontology_model: None,
            user_answers: Vec::new(),
            extensions: Vec::new(),
            generated_files: Vec::new(),
            pending_questions: Vec::new(),
            plan_violations: None,
            extension_gaps: Vec::new(),
            #[allow(deprecated)]
            generated_manifest: None,
            extension_tracking: std::collections::HashMap::new(),
            mapped_entities: Vec::new(),
            alignment_graph: None,
            compose_scratch: None,
            pending_gateway_design: None,
            verification_error: None,
            yaml_operations: Vec::new(),
            yaml_operation_log: Vec::new(),
            execution_log: Vec::new(),
            step_history: Vec::new(),
            step_details: Vec::new(),
            last_checkpoint: None,
            interrupt_requested: false,
            change_requests: Vec::new(),
            last_built_app: None,
            runtime_validation: None,
            max_repair_count: default_max_repair_count(),
            repair_attempt: 0,
            draft_mode: false,
            eval_iteration: 0,
            eval_feedback: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executing_skill_default_return_state() {
        let state = AgentState::ExecutingSkill {
            skill_name: "test".into(),
            track_index: 0,
            step_index: 0,
            attempt: 0,
            context: HashMap::new(),
            return_state: Box::new(AgentState::Planning {
                revision_round: 0,
                needs_clarification: None,
            }),
        };
        match &state {
            AgentState::ExecutingSkill { return_state, .. } => {
                assert!(matches!(**return_state, AgentState::Planning { .. }));
            }
            _ => panic!("expected ExecutingSkill"),
        }
    }

    #[test]
    fn test_failed_state_roundtrip() {
        let state = AgentState::Failed {
            error: "test error".into(),
        };
        let json = serde_json::to_string(&state).unwrap();
        let deser: AgentState = serde_json::from_str(&json).unwrap();
        match &deser {
            AgentState::Failed { error } => assert_eq!(error, "test error"),
            _ => panic!("expected Failed"),
        }
    }

    #[test]
    fn test_module_creation_delegates_to_alioth_module() {
        // 验证 ModuleCreation 后转移边指向 alioth-module skill
        let ctx_map = {
            let mut m = HashMap::new();
            m.insert("ns".to_string(), "Test".into());
            m.insert("module".to_string(), "inventory-app".into());
            m
        };
        let next = AgentState::ExecutingSkill {
            skill_name: "alioth-module".into(),
            track_index: 0,
            step_index: 0,
            attempt: 0,
            context: ctx_map,
            return_state: Box::new(AgentState::BlockCreation),
        };
        match &next {
            AgentState::ExecutingSkill {
                skill_name,
                return_state,
                context,
                ..
            } => {
                assert_eq!(skill_name, "alioth-module");
                assert_eq!(context.get("ns").unwrap(), "Test");
                assert!(matches!(**return_state, AgentState::BlockCreation));
            }
            _ => panic!("expected ExecutingSkill"),
        }
    }

    #[test]
    fn test_service_api_delegates_to_alioth_service() {
        let ctx_map = {
            let mut m = HashMap::new();
            m.insert("ns".to_string(), "WZ".into());
            m.insert("service".to_string(), "order-service".into());
            m
        };
        let next = AgentState::ExecutingSkill {
            skill_name: "alioth-service".into(),
            track_index: 0,
            step_index: 0,
            attempt: 0,
            context: ctx_map,
            return_state: Box::new(AgentState::Publishing {
                publish_attempt: 0,
                last_error: None,
            }),
        };
        match &next {
            AgentState::ExecutingSkill {
                skill_name,
                return_state,
                context,
                ..
            } => {
                assert_eq!(skill_name, "alioth-service");
                assert_eq!(context.get("ns").unwrap(), "WZ");
                assert!(matches!(**return_state, AgentState::Publishing { .. }));
            }
            _ => panic!("expected ExecutingSkill"),
        }
    }
}
