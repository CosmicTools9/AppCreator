//! Module DSL 扩展示例
//!
//! 支持定义业务模块，包括实体、页面、权限配置。

/// Module DSL 示例
///
/// ```dsl
/// module orders {
///     entity Order {
///         field order_number: String
///         field customer: Customer
///         field items: [OrderItem]
///         field total_amount: Money
///         
///         rule total_must_match: "sum(items.subtotal) == total_amount"
///         
///         statemachine OrderStatus {
///             Draft -> Submitted -> Paid -> Shipped -> Delivered
///             Paid -> Cancelled
///         }
///         
///         permission {
///             sales: [create, read, update]
///             finance: [read, update_status]
///         }
///     }
///     
///     page OrderList {
///         type: list
///         entity: Order
///         columns: [order_number, customer.name, total_amount, status]
///         filters: [status, date_range]
///     }
///     
///     page OrderDetail {
///         type: detail
///         entity: Order
///         sections: [header, items, payment, shipping]
///     }
/// }
/// ```
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::exception::{MetaException, MetaExceptionHandler, MetaThrowsClause};
use crate::permission::{
    ConflictResolution, EntityPermissionConfig, FieldPermission, InheritanceStrategy,
};
use crate::quality::MetaQualityRule;

/// IR-1: 从 DSL 解析的原始元模型
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetaModel {
    pub entities: Vec<MetaEntity>,
    pub enums: Vec<MetaEnum>,
    /// Phase 26: 异常定义列表
    #[serde(default)]
    pub exceptions: Vec<MetaException>,
    /// Phase 26: 全局异常处理器
    #[serde(default)]
    pub exception_handlers: Vec<MetaExceptionHandler>,
}

/// State machine definition (IR-1)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetaStateMachine {
    /// Whether this entity has a state machine
    #[serde(default)]
    pub enabled: bool,
    /// List of all possible states
    #[serde(default)]
    pub states: Vec<String>,
    /// Initial state when entity is created
    #[serde(default)]
    pub initial_state: Option<String>,
    /// State field name (e.g., "status")
    #[serde(default)]
    pub state_field: Option<String>,
}

/// State transition definition (IR-1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaTransition {
    /// Event name that triggers this transition
    pub event: String,
    /// Source state(s)
    pub from: Vec<String>,
    /// Target state
    pub to: String,
    /// Guard condition (optional function name or expression)
    #[serde(default)]
    pub guard: Option<String>,
    /// Action to execute during transition
    #[serde(default)]
    pub action: Option<String>,
}

/// Lifecycle hook definition (IR-1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaLifecycleHook {
    /// The lifecycle event type: onCreate, onUpdate, onDelete, onTransition
    pub event: String,
    /// Hook function name
    pub function_name: String,
    /// For onTransition: source state (optional)
    #[serde(default)]
    pub from_state: Option<String>,
    /// For onTransition: target state (optional)
    #[serde(default)]
    pub to_state: Option<String>,
    /// Execution order
    #[serde(default)]
    pub order: i32,
}

/// Business rule definition (IR-1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaBusinessRule {
    /// Rule name (unique identifier)
    pub name: String,
    /// Condition expression or function name
    pub condition: String,
    /// Action to execute when condition is met
    #[serde(default)]
    pub action: Option<String>,
    /// Error message when condition is not met
    #[serde(default)]
    pub error_message: Option<String>,
    /// Rule priority
    #[serde(default)]
    pub priority: i32,
    /// When the rule should be evaluated: onCreate, onUpdate, onDelete, onTransition, always
    #[serde(default)]
    pub trigger: String,
}

/// SWRL-style rule definition (IR-1) - Phase 23
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaSwrlRule {
    /// Rule name (unique identifier)
    pub name: String,
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

/// Constraint definition (IR-1) - Phase 23
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaConstraint {
    /// Constraint name (optional)
    #[serde(default)]
    pub name: Option<String>,
    /// Constraint expression (e.g., "price > 0")
    pub expression: String,
    /// Constraint level: field or entity
    pub level: MetaConstraintLevel,
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

/// Constraint level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum MetaConstraintLevel {
    #[default]
    Field,
    Entity,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetaEntity {
    pub name: String,
    pub description: Option<String>,
    pub fields: Vec<MetaField>,
    pub relations: Vec<MetaRelation>,
    pub annotations: Vec<MetaAnnotation>,

    /// Physical table name with schema. Set by ontology-gen-bridge adapter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub table_name: Option<String>,

    // OWL Class Constraints - Phase 21
    /// 父类列表 - @extends(Parent1, Parent2)
    #[serde(default)]
    pub parent_classes: Vec<String>,
    /// 等价类列表 - @equivalentTo(OtherClass)
    #[serde(default)]
    pub equivalent_classes: Vec<String>,
    /// 互斥类列表 - @disjointWith(OtherClass)
    #[serde(default)]
    pub disjoint_classes: Vec<String>,
    /// 是否为抽象类 - @abstract
    #[serde(default)]
    pub is_abstract: bool,

    // Behavior & State Machine - Phase 22
    /// 状态机定义 - @statemachine, @states([...])
    #[serde(default)]
    pub state_machine: MetaStateMachine,
    /// 状态转换定义 - @transition(...)
    #[serde(default)]
    pub transitions: Vec<MetaTransition>,
    /// 生命周期钩子 - @onCreate, @onUpdate, @onDelete, @onTransition
    #[serde(default)]
    pub lifecycle_hooks: Vec<MetaLifecycleHook>,
    /// 业务规则 - @rule(...)
    #[serde(default)]
    pub business_rules: Vec<MetaBusinessRule>,

    // Rule Reasoning - Phase 23
    /// SWRL 风格规则 - @swrlRule(...)
    #[serde(default)]
    pub swrl_rules: Vec<MetaSwrlRule>,
    /// 约束定义 - @constraint(...)
    #[serde(default)]
    pub constraints: Vec<MetaConstraint>,

    // Ontology-based Permissions - Phase 25
    /// 实体权限配置 - 本体级/实例级权限
    #[serde(default)]
    pub permission_config: EntityPermissionConfig,
    /// 权限继承策略
    #[serde(default)]
    pub permission_inheritance: InheritanceStrategy,
    /// 权限冲突解决策略
    #[serde(default)]
    pub permission_conflict_resolution: ConflictResolution,

    // Phase 27: Quality Validation
    /// 实体级质量规则
    #[serde(default)]
    pub quality_rules: Vec<MetaQualityRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetaField {
    pub name: String,
    pub field_type: MetaFieldType,
    pub description: Option<String>,
    pub nullable: bool,
    pub unique: bool,
    pub indexed: bool,
    pub default_value: Option<String>,
    pub validations: Vec<MetaValidation>,
    pub annotations: Vec<MetaAnnotation>,

    // OWL Property Constraints - Phase 21
    /// 定义域 - @domain(ClassName)
    #[serde(default)]
    pub domain: Option<String>,
    /// 值域 - @range(TypeName)
    #[serde(default)]
    pub range: Option<String>,
    /// 最小基数 - @minCardinality(n)
    #[serde(default)]
    pub min_cardinality: Option<u32>,
    /// 最大基数 - @maxCardinality(n)
    #[serde(default)]
    pub max_cardinality: Option<u32>,
    /// 是否为函数属性 - @functional (等价于 maxCardinality(1))
    #[serde(default)]
    pub is_functional: bool,

    // Rule Constraints - Phase 23
    /// 字段级约束
    #[serde(default)]
    pub constraints: Vec<MetaConstraint>,

    // Field-level Permissions - Phase 25
    /// 字段权限配置
    #[serde(default)]
    pub field_permission: FieldPermission,

    // Phase 26: Exception Handling
    /// 字段可能抛出的异常
    #[serde(default)]
    pub throws_clauses: Vec<MetaThrowsClause>,

    // Phase 27: Quality Validation
    /// 字段级质量规则
    #[serde(default)]
    pub quality_rules: Vec<MetaQualityRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum MetaFieldType {
    #[default]
    String,
    Integer,
    Long,
    Decimal,
    Boolean,
    DateTime,
    Uuid,
    Json,
    Enum(String),
    Reference(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaRelation {
    pub name: String,
    pub target_entity: String,
    pub relation_type: MetaRelationType,
    pub nullable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetaRelationType {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
    ManyHasMany,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaEnum {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaValidation {
    pub validation_type: MetaValidationType,
    pub params: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetaValidationType {
    MinLength,
    MaxLength,
    Pattern,
    Min,
    Max,
    Email,
    Url,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetaAnnotation {
    pub name: String,
    pub params: HashMap<String, String>,
}

/// Page type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum PageType {
    List,
    Detail,
    Form,
    Dashboard,
}

/// Page layout configuration
#[derive(Debug, Clone)]
pub struct PageLayout {
    pub columns: Vec<String>,
    pub filters: Vec<String>,
    pub sections: Vec<String>,
}

/// Meta page definition
#[derive(Debug, Clone)]
pub struct MetaPage {
    pub name: String,
    pub page_type: PageType,
    pub entity: String,
    pub layout: PageLayout,
}

/// Meta permission definition
#[derive(Debug, Clone)]
pub struct MetaPermission {
    pub role: String,
    pub actions: Vec<String>,
}

/// MetaModule - Module definition extending MetaModel
///
/// A module is a collection of related entities, pages, and permissions
/// that together form a business application feature.
#[derive(Debug, Clone)]
pub struct MetaModule {
    pub name: String,
    pub entities: Vec<MetaEntity>,
    pub pages: Vec<MetaPage>,
    pub permissions: Vec<MetaPermission>,
    pub state_machines: Vec<MetaStateMachine>,
    pub business_rules: Vec<MetaBusinessRule>,
}

impl MetaModule {
    /// Create a new empty module
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            entities: Vec::new(),
            pages: Vec::new(),
            permissions: Vec::new(),
            state_machines: Vec::new(),
            business_rules: Vec::new(),
        }
    }

    /// Add an entity to the module
    pub fn add_entity(&mut self, entity: MetaEntity) {
        self.entities.push(entity);
    }

    /// Add a page to the module
    pub fn add_page(&mut self, page: MetaPage) {
        self.pages.push(page);
    }

    /// Add a permission to the module
    pub fn add_permission(&mut self, permission: MetaPermission) {
        self.permissions.push(permission);
    }

    /// Get entity by name
    pub fn get_entity(&self, name: &str) -> Option<&MetaEntity> {
        self.entities.iter().find(|e| e.name == name)
    }

    /// Get page by name
    pub fn get_page(&self, name: &str) -> Option<&MetaPage> {
        self.pages.iter().find(|p| p.name == name)
    }

    pub fn infer_pages(entities: &[MetaEntity]) -> Vec<MetaPage> {
        let mut pages = Vec::new();
        for entity in entities {
            let name = &entity.name;
            // List page
            pages.push(MetaPage {
                name: format!("{}List", name),
                page_type: PageType::List,
                entity: name.clone(),
                layout: PageLayout {
                    columns: entity.fields.iter().map(|f| f.name.clone()).collect(),
                    filters: vec!["search".to_string()],
                    sections: vec![],
                },
            });
            // Detail page
            pages.push(MetaPage {
                name: format!("{}Detail", name),
                page_type: PageType::Detail,
                entity: name.clone(),
                layout: PageLayout {
                    columns: vec![],
                    filters: vec![],
                    sections: vec!["header".to_string(), "details".to_string()],
                },
            });
            // Form page
            pages.push(MetaPage {
                name: format!("{}Form", name),
                page_type: PageType::Form,
                entity: name.clone(),
                layout: PageLayout {
                    columns: vec![],
                    filters: vec![],
                    sections: vec!["form".to_string()],
                },
            });
        }
        pages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_creation() {
        let module = MetaModule::new("orders");
        assert_eq!(module.name, "orders");
        assert!(module.entities.is_empty());
        assert!(module.pages.is_empty());
    }

    #[test]
    fn test_add_entity() {
        use crate::ir1::{MetaEntity, MetaStateMachine};
        use crate::permission::{ConflictResolution, EntityPermissionConfig, InheritanceStrategy};

        let mut module = MetaModule::new("test");
        let entity = MetaEntity {
            name: "Order".into(),
            description: None,
            fields: Vec::new(),
            relations: Vec::new(),
            annotations: Vec::new(),
            parent_classes: Vec::new(),
            equivalent_classes: Vec::new(),
            disjoint_classes: Vec::new(),
            is_abstract: false,
            state_machine: MetaStateMachine::default(),
            table_name: None,
            transitions: Vec::new(),
            lifecycle_hooks: Vec::new(),
            business_rules: Vec::new(),
            constraints: Vec::new(),
            swrl_rules: Vec::new(),
            permission_config: EntityPermissionConfig::default(),
            permission_inheritance: InheritanceStrategy::default(),
            permission_conflict_resolution: ConflictResolution::default(),
            quality_rules: Vec::new(),
        };
        module.add_entity(entity);
        assert_eq!(module.entities.len(), 1);
        assert_eq!(module.get_entity("Order").unwrap().name, "Order");
    }

    #[test]
    fn test_page_type() {
        assert_eq!(PageType::List, PageType::List);
        assert_eq!(PageType::Detail, PageType::Detail);
    }
}
