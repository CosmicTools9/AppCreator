//! IR-2: 生成器模型 - 生成器无关的表示

// use crate::ALIOTH_STUDIO_VERSION; // removed: meta-model has no access to alioth-gen constants
use serde::{Deserialize, Serialize};

// Phase 24: Import 4D Space types

// Phase 26: Import Exception types
use crate::exception::{GeneratorException, GeneratorExceptionHandler, GeneratorThrowsClause};

// Phase 27: Import Quality types
use crate::quality::{GeneratorQualityConfig, GeneratorQualityRule};

/// IR-2: 用于代码生成的规范化模型
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneratorModel {
    pub entities: Vec<GeneratorEntity>,
    pub enums: Vec<GeneratorEnum>,
    pub metadata: ModelMetadata,
    /// Phase 26: 异常定义列表
    #[serde(default)]
    pub exceptions: Vec<GeneratorException>,
    /// Phase 26: 全局异常处理器
    #[serde(default)]
    pub exception_handlers: Vec<GeneratorExceptionHandler>,
    /// Phase 28: i18n configuration
    #[serde(default)]
    pub i18n_config: Option<GeneratorI18nConfig>,
    /// External module dependencies for multi-module code generation
    #[serde(default)]
    pub external_dependencies: Vec<ModuleDependency>,
}

/// External module dependency for multi-module code generation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModuleDependency {
    pub module_id: String,
    pub crate_name: String,
    pub path: String,
    #[serde(default)]
    pub exported_tables: Vec<String>,
    #[serde(default)]
    pub exported_events: Vec<String>,
    #[serde(default)]
    pub extension_points: Vec<String>,
}

/// State machine definition (IR-2)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneratorStateMachine {
    /// Whether this entity has a state machine
    #[serde(default)]
    pub enabled: bool,
    /// List of all possible states
    #[serde(default)]
    pub states: Vec<GeneratorState>,
    /// Initial state when entity is created
    #[serde(default)]
    pub initial_state: Option<String>,
    /// State field name
    #[serde(default)]
    pub state_field: String,
    /// State enum type name
    #[serde(default)]
    pub state_enum_name: String,
}

/// State definition (IR-2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorState {
    /// State name (e.g., "Pending")
    pub name: String,
    /// PascalCase state name for enum variant
    pub pascal_name: String,
    /// Snake_case state name for functions
    pub snake_name: String,
    /// Whether this is a terminal/final state
    #[serde(default)]
    pub is_final: bool,
}

/// State transition definition (IR-2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorTransition {
    /// Event name that triggers this transition
    pub event: String,
    /// Snake_case event name for function
    pub event_snake: String,
    /// Source state(s)
    pub from: Vec<String>,
    /// Target state
    pub to: String,
    /// Guard condition function name
    #[serde(default)]
    pub guard: Option<String>,
    /// Action function name
    #[serde(default)]
    pub action: Option<String>,
}

/// Lifecycle hook definition (IR-2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorLifecycleHook {
    /// The lifecycle event type
    pub event: String,
    /// Hook function name
    pub function_name: String,
    /// Snake_case function name
    pub function_name_snake: String,
    /// For onTransition: source state (optional)
    #[serde(default)]
    pub from_state: Option<String>,
    /// For onTransition: target state (optional)
    #[serde(default)]
    pub to_state: Option<String>,
    /// Execution order
    #[serde(default)]
    pub order: i32,
    /// Whether this hook is async
    #[serde(default)]
    pub is_async: bool,
}

/// Business rule definition (IR-2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorBusinessRule {
    /// Rule name (unique identifier)
    pub name: String,
    /// Snake_case rule name
    pub name_snake: String,
    /// Condition expression or function name
    pub condition: String,
    /// Action to execute when condition is met
    #[serde(default)]
    pub action: Option<String>,
    /// Error message when condition is not met
    #[serde(default)]
    pub error_message: Option<String>,
    /// Error code
    #[serde(default)]
    pub error_code: Option<String>,
    /// Rule priority
    #[serde(default)]
    pub priority: i32,
    /// When the rule should be evaluated
    #[serde(default)]
    pub trigger: String,
}

/// SWRL-style rule definition (IR-2) - Phase 23
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorSwrlRule {
    /// Rule name (unique identifier)
    pub name: String,
    /// Snake_case rule name
    pub name_snake: String,
    /// Human-readable description
    #[serde(default)]
    pub description: Option<String>,
    /// Rule body (IF part) - conditions as string
    pub body: String,
    /// Rule head (THEN part) - conclusions as string
    pub head: String,
    /// Rule priority
    #[serde(default)]
    pub priority: i32,
    /// Whether this rule is active
    #[serde(default)]
    pub active: bool,
}

/// Constraint definition (IR-2) - Phase 23
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorConstraint {
    /// Constraint name (optional)
    #[serde(default)]
    pub name: Option<String>,
    /// Constraint expression
    pub expression: String,
    /// Constraint level
    pub level: GeneratorConstraintLevel,
    /// Human-readable error message
    #[serde(default)]
    pub error_message: Option<String>,
    /// Error code
    #[serde(default)]
    pub error_code: Option<String>,
    /// Whether this constraint is active
    #[serde(default)]
    pub active: bool,
    /// Whether violation is blocking
    #[serde(default)]
    pub blocking: bool,
    /// Field name (for field-level constraints)
    #[serde(default)]
    pub field_name: Option<String>,
}

/// Constraint level (IR-2)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum GeneratorConstraintLevel {
    #[default]
    Field,
    Entity,
}

/// 主键类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PrimaryKeyType {
    /// BigInt 主键（默认），使用 gen_next_zuid()
    #[default]
    BigInt,
    /// UUID 主键，使用 gen_random_uuid()
    Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneratorEntity {
    pub name: EntityName,
    pub description: Option<String>,
    pub fields: Vec<GeneratorField>,
    pub relations: Vec<GeneratorRelation>,
    pub annotations: Vec<GeneratorAnnotation>,
    /// 主键类型，默认为 BigInt
    #[serde(default)]
    pub primary_key_type: PrimaryKeyType,

    // OWL Class Constraints - Phase 21
    #[serde(default)]
    pub parent_classes: Vec<EntityName>,
    #[serde(default)]
    pub equivalent_classes: Vec<String>,
    #[serde(default)]
    pub disjoint_classes: Vec<String>,
    #[serde(default)]
    pub is_abstract: bool,
    /// 继承层次深度（用于排序生成）
    #[serde(default)]
    pub inheritance_depth: u32,

    // Behavior & State Machine - Phase 22
    #[serde(default)]
    pub state_machine: GeneratorStateMachine,
    #[serde(default)]
    pub transitions: Vec<GeneratorTransition>,
    #[serde(default)]
    pub lifecycle_hooks: Vec<GeneratorLifecycleHook>,
    #[serde(default)]
    pub business_rules: Vec<GeneratorBusinessRule>,

    // Rule Reasoning - Phase 23
    #[serde(default)]
    pub swrl_rules: Vec<GeneratorSwrlRule>,
    #[serde(default)]
    pub constraints: Vec<GeneratorConstraint>,

    // Phase 27: Quality Validation
    /// 质量规则配置
    #[serde(default)]
    pub quality_rules: Vec<GeneratorQualityRule>,
    /// 质量配置
    #[serde(default)]
    pub quality_config: GeneratorQualityConfig,

    /// 继承链父表名（从近到远），用于 DDL 关系表继承推断
    #[serde(default)]
    pub parent_tables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntityName {
    pub raw: String,
    pub snake: String,
    pub camel: String,
    pub pascal: String,
    pub kebab: String,
    pub screaming_snake: String,
    pub plural_snake: String,
    pub plural_pascal: String,
    pub plural_kebab: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneratorField {
    pub name: FieldName,
    pub field_type: GeneratorFieldType,
    pub description: Option<String>,
    pub nullable: bool,
    pub unique: bool,
    pub indexed: bool,
    pub default_value: Option<String>,
    pub validations: Vec<GeneratorValidation>,
    pub annotations: Vec<GeneratorAnnotation>,

    // OWL Property Constraints - Phase 21
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub range: Option<String>,
    #[serde(default)]
    pub min_cardinality: Option<u32>,
    #[serde(default)]
    pub max_cardinality: Option<u32>,
    #[serde(default)]
    pub is_functional: bool,

    // Rule Constraints - Phase 23
    #[serde(default)]
    pub constraints: Vec<GeneratorConstraint>,

    // Phase 26: Exception Handling
    /// 字段可能抛出的异常
    #[serde(default)]
    pub throws_clauses: Vec<GeneratorThrowsClause>,

    // Phase 27: Quality Validation
    /// 字段级质量规则
    #[serde(default)]
    pub quality_rules: Vec<GeneratorQualityRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FieldName {
    pub raw: String,
    pub snake: String,
    pub camel: String,
    pub pascal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum GeneratorFieldType {
    #[default]
    Text,
    Integer,
    BigInt,
    Decimal,
    Boolean,
    DateTime,
    Uuid,
    Json,
    Enum(String),
    Reference(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorRelation {
    pub name: String,
    pub target_entity: String,
    pub relation_type: GeneratorRelationType,
    pub nullable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeneratorRelationType {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
    ManyHasMany,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorEnum {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorValidation {
    pub validation_type: GeneratorValidationType,
    pub params: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeneratorValidationType {
    MinLength,
    MaxLength,
    Pattern,
    Min,
    Max,
    Email,
    Url,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorAnnotation {
    pub name: String,
    pub params: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub generated_at: String,
    pub generator_version: String,
}

impl Default for ModelMetadata {
    fn default() -> Self {
        Self {
            generated_at: chrono::Utc::now().to_rfc3339(),
            generator_version: "0.1.0".to_string(),
        }
    }
}

/// Constraint violation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintViolation {
    pub constraint_type: ConstraintType,
    pub message: String,
    pub entity: Option<String>,
    pub field: Option<String>,
}

/// Constraint violation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    CircularInheritance,
    UnknownParentClass,
    DisjointViolation,
    CardinalityConflict,
    EquivalentClassMismatch,
}

/// i18n configuration for generated code (Phase 28)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneratorI18nConfig {
    /// Default locale (e.g., "zh-CN")
    pub default_locale: String,
    /// Supported locales
    pub supported_locales: Vec<String>,
    /// Auto-extracted or manually defined i18n keys
    pub keys: Vec<GeneratorI18nKey>,
}

/// A single i18n key definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorI18nKey {
    /// Dotted key path, e.g. "entity.product.fields.name"
    pub key: String,
    /// Default message in the default locale
    pub default_message: String,
    /// Optional description for translators
    pub description: Option<String>,
    /// Interpolation parameters
    #[serde(default)]
    pub params: Vec<String>,
    /// Which generated file this key belongs to
    pub scope: I18nKeyScope,
}

/// Scope of an i18n key
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum I18nKeyScope {
    /// Entity-level labels (fields, states, actions)
    Entity,
    /// Exception/error messages
    Error,
    /// UI component text (buttons, placeholders, titles)
    #[default]
    Ui,
    /// Validation messages
    Validation,
}
