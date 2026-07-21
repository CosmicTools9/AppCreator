//! Ontology-based Permission System (Phase 25)
//!
//! 实现基于本体语义的细粒度权限控制模型
//! 支持本体级、属性级和实例级权限

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 权限动作类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum PermissionAction {
    /// 创建权限
    Create,
    /// 读取权限
    Read,
    /// 更新权限
    Update,
    /// 删除权限
    Delete,
    /// 列表查询权限
    List,
    /// 搜索权限
    Search,
    /// 执行权限（用于方法/操作）
    Execute,
    /// 通配符 - 所有权限
    All,
}

impl PermissionAction {
    /// 将字符串转换为权限动作
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "create" => Some(Self::Create),
            "read" => Some(Self::Read),
            "update" => Some(Self::Update),
            "delete" => Some(Self::Delete),
            "list" => Some(Self::List),
            "search" => Some(Self::Search),
            "execute" => Some(Self::Execute),
            "*" | "all" => Some(Self::All),
            _ => None,
        }
    }
}

impl std::str::FromStr for PermissionAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or_else(|| format!("unknown permission action: {}", s))
    }
}

impl PermissionAction {
    /// 转换为字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Read => "read",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::List => "list",
            Self::Search => "search",
            Self::Execute => "execute",
            Self::All => "*",
        }
    }

    /// 检查此动作是否包含另一个动作
    /// 例如：All 包含所有其他动作，Read 只包含自身
    pub fn contains(&self, other: &PermissionAction) -> bool {
        matches!(self, Self::All) || self == other
    }
}

impl std::fmt::Display for PermissionAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 本体级权限定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyPermission {
    /// 岗位视角名称
    pub position_view: String,
    /// 允许的动作列表
    pub actions: Vec<PermissionAction>,
    /// 权限条件表达式（可选）
    #[serde(default)]
    pub condition: Option<String>,
    /// 优先级（用于冲突解决）
    #[serde(default)]
    pub priority: i32,
    /// 是否启用
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

impl OntologyPermission {
    /// 创建新的本体权限
    pub fn new(position_view: impl Into<String>, actions: Vec<PermissionAction>) -> Self {
        Self {
            position_view: position_view.into(),
            actions,
            condition: None,
            priority: 0,
            enabled: true,
        }
    }

    /// 检查是否允许指定动作
    pub fn allows_action(&self, action: &PermissionAction) -> bool {
        if !self.enabled {
            return false;
        }
        self.actions.iter().any(|a| a.contains(action))
    }

    /// 添加条件表达式
    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// 属性级权限定义
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FieldPermission {
    /// 可读取此字段的岗位视角列表
    #[serde(default)]
    pub read_positions: Vec<String>,
    /// 可写入此字段的岗位视角列表
    #[serde(default)]
    pub write_positions: Vec<String>,
    /// 字段级条件表达式
    #[serde(default)]
    pub condition: Option<String>,
}

impl FieldPermission {
    /// 创建空的字段权限
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加读取岗位视角
    pub fn add_read_position(&mut self, position_view: impl Into<String>) {
        self.read_positions.push(position_view.into());
    }

    /// 添加写入岗位视角
    pub fn add_write_position(&mut self, position_view: impl Into<String>) {
        self.write_positions.push(position_view.into());
    }

    /// 检查岗位视角是否可以读取
    pub fn can_read(&self, position_view: &str) -> bool {
        self.read_positions.contains(&"*".to_string()) // 通配符表示所有岗位视角
            || self.read_positions.contains(&position_view.to_string())
    }

    /// 检查岗位视角是否可以写入
    pub fn can_write(&self, position_view: &str) -> bool {
        self.write_positions.contains(&"*".to_string()) // 通配符表示所有岗位视角
            || self.write_positions.contains(&position_view.to_string())
    }
}

/// 实例级权限表达式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstancePermission {
    /// 权限表达式字符串
    pub expression: String,
    /// 表达式类型
    #[serde(default)]
    pub expr_type: ExpressionType,
    /// 描述信息
    #[serde(default)]
    pub description: Option<String>,
}

/// 表达式类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExpressionType {
    /// 简单比较表达式
    #[default]
    Simple,
    /// SpEL/Spring 表达式
    Spel,
    /// JavaScript 表达式
    JavaScript,
    /// SQL WHERE 子句风格
    Sql,
}

impl InstancePermission {
    /// 创建新的实例权限
    pub fn new(expression: impl Into<String>) -> Self {
        Self {
            expression: expression.into(),
            expr_type: ExpressionType::Simple,
            description: None,
        }
    }

    /// 设置表达式类型
    pub fn with_type(mut self, expr_type: ExpressionType) -> Self {
        self.expr_type = expr_type;
        self
    }

    /// 设置描述
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// 解析表达式中的变量引用
    /// 例如: "owner == currentUser" -> ["owner", "currentUser"]
    pub fn extract_variables(&self) -> Vec<String> {
        let mut vars = Vec::new();
        // 简单解析：提取标识符
        let words: Vec<&str> = self.expression.split_whitespace().collect();
        for word in words {
            let cleaned =
                word.trim_matches(|c| c == '(' || c == ')' || c == '"' || c == '\'' || c == ',');
            if !cleaned.is_empty()
                && !matches!(
                    cleaned,
                    "==" | "!=" | ">" | "<" | ">=" | "<=" | "&&" | "||" | "AND" | "OR"
                )
                && cleaned.parse::<f64>().is_err()
            {
                vars.push(cleaned.to_string());
            }
        }
        vars
    }
}

/// 权限继承策略
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InheritanceStrategy {
    /// 继承所有父类权限
    InheritAll,
    /// 只继承读取权限
    InheritReadOnly,
    /// 不继承任何权限
    NoInheritance,
    /// 父类权限与子类权限合并
    #[default]
    Merge,
    /// 子类权限覆盖父类权限
    Override,
}

/// 权限冲突解决策略
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    /// 拒绝优先（默认安全策略）
    #[default]
    DenyOverrides,
    /// 允许优先
    PermitOverrides,
    /// 先定义优先
    FirstApplicable,
    /// 顺序优先级
    Ordered,
    /// 显式指定优先级
    ExplicitPriority,
}

/// 实体权限配置（聚合所有权限类型）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntityPermissionConfig {
    /// 本体级权限列表
    #[serde(default)]
    pub ontology_permissions: Vec<OntologyPermission>,
    /// 实例级权限表达式
    #[serde(default)]
    pub instance_permission: Option<InstancePermission>,
    /// 权限继承策略
    #[serde(default)]
    pub inheritance_strategy: InheritanceStrategy,
    /// 冲突解决策略
    #[serde(default)]
    pub conflict_resolution: ConflictResolution,
    /// 是否启用行级安全
    #[serde(default)]
    pub row_level_security: bool,
}

impl EntityPermissionConfig {
    /// 创建空的权限配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加本体权限
    pub fn add_permission(&mut self, permission: OntologyPermission) {
        self.ontology_permissions.push(permission);
    }

    /// 设置实例权限
    pub fn set_instance_permission(&mut self, permission: InstancePermission) {
        self.instance_permission = Some(permission);
    }

    /// 检查岗位视角是否对实体有指定动作权限
    pub fn check_permission(&self, position_view: &str, action: &PermissionAction) -> bool {
        for perm in &self.ontology_permissions {
            if perm.position_view == position_view && perm.allows_action(action) {
                return true;
            }
        }
        false
    }

    /// 获取岗位视角拥有的所有动作
    pub fn get_position_actions(&self, position_view: &str) -> Vec<PermissionAction> {
        let mut actions = Vec::new();
        for perm in &self.ontology_permissions {
            if perm.position_view == position_view {
                actions.extend(perm.actions.clone());
            }
        }
        // 去重
        actions.sort_by_key(|a| format!("{:?}", a));
        actions.dedup_by_key(|a| format!("{:?}", a));
        actions
    }
}

/// 字段权限配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldPermissionConfig {
    /// 字段名称
    pub field_name: String,
    /// 字段权限
    pub permission: FieldPermission,
}

/// 权限推理结果
#[derive(Debug, Clone, Default)]
pub struct PermissionInferenceResult {
    /// 从父类继承的权限
    pub inherited_permissions: Vec<InheritedPermission>,
    /// 推导出的隐式权限
    pub implied_permissions: Vec<ImpliedPermission>,
    /// 检测到的权限冲突
    pub conflicts: Vec<PermissionConflict>,
}

/// 继承的权限
#[derive(Debug, Clone)]
pub struct InheritedPermission {
    /// 权限定义
    pub permission: OntologyPermission,
    /// 来源父类
    pub source_class: String,
    /// 继承路径
    pub inheritance_path: Vec<String>,
}

/// 隐式权限（通过推理得出）
#[derive(Debug, Clone)]
pub struct ImpliedPermission {
    /// 岗位视角
    pub position_view: String,
    /// 动作
    pub action: PermissionAction,
    /// 推理依据
    pub reason: String,
}

/// 权限冲突
#[derive(Debug, Clone)]
pub struct PermissionConflict {
    /// 岗位视角
    pub position_view: String,
    /// 动作
    pub action: PermissionAction,
    /// 冲突的权限定义1
    pub permission1: OntologyPermission,
    /// 冲突的权限定义2
    pub permission2: OntologyPermission,
    /// 冲突类型
    pub conflict_type: ConflictType,
}

/// 冲突类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictType {
    /// 允许 vs 拒绝
    AllowDeny,
    /// 条件冲突
    ConditionConflict,
    /// 优先级冲突
    PriorityConflict,
}

/// 权限推理器
pub struct PermissionReasoner;

impl PermissionReasoner {
    /// 推理实体的有效权限
    pub fn infer_permissions(
        entity_name: &str,
        direct_permissions: &[OntologyPermission],
        parent_configs: &[EntityPermissionConfig],
        strategy: InheritanceStrategy,
    ) -> PermissionInferenceResult {
        let mut result = PermissionInferenceResult::default();

        // 根据继承策略处理父类权限
        match strategy {
            InheritanceStrategy::InheritAll | InheritanceStrategy::Merge => {
                for (idx, parent) in parent_configs.iter().enumerate() {
                    for perm in &parent.ontology_permissions {
                        let inherited = InheritedPermission {
                            permission: perm.clone(),
                            source_class: format!("parent_{}", idx),
                            inheritance_path: vec![entity_name.to_string()],
                        };
                        result.inherited_permissions.push(inherited);
                    }
                }
            }
            InheritanceStrategy::InheritReadOnly => {
                for (idx, parent) in parent_configs.iter().enumerate() {
                    for perm in &parent.ontology_permissions {
                        // 只继承读取相关权限
                        let read_actions: Vec<_> = perm
                            .actions
                            .iter()
                            .filter(|a| {
                                matches!(
                                    a,
                                    PermissionAction::Read
                                        | PermissionAction::List
                                        | PermissionAction::Search
                                )
                            })
                            .cloned()
                            .collect();
                        if !read_actions.is_empty() {
                            let mut inherited_perm = perm.clone();
                            inherited_perm.actions = read_actions;
                            let inherited = InheritedPermission {
                                permission: inherited_perm,
                                source_class: format!("parent_{}", idx),
                                inheritance_path: vec![entity_name.to_string()],
                            };
                            result.inherited_permissions.push(inherited);
                        }
                    }
                }
            }
            InheritanceStrategy::NoInheritance | InheritanceStrategy::Override => {
                // 不继承或使用覆盖模式，直接权限优先
            }
        }

        // 检测冲突
        result.conflicts =
            Self::detect_conflicts(direct_permissions, &result.inherited_permissions);

        result
    }

    /// 检测权限冲突
    fn detect_conflicts(
        direct: &[OntologyPermission],
        inherited: &[InheritedPermission],
    ) -> Vec<PermissionConflict> {
        let mut conflicts = Vec::new();
        let all_perms: Vec<_> = direct
            .iter()
            .map(|p| (p, "direct"))
            .chain(inherited.iter().map(|i| (&i.permission, "inherited")))
            .collect();

        for (i, (perm1, _source1)) in all_perms.iter().enumerate() {
            for (perm2, _source2) in all_perms.iter().skip(i + 1) {
                // 检查角色和动作重叠
                if perm1.position_view == perm2.position_view {
                    let overlapping: Vec<_> = perm1
                        .actions
                        .iter()
                        .filter(|a1| perm2.actions.iter().any(|a2| a1 == &a2))
                        .cloned()
                        .collect();

                    for action in overlapping {
                        // 检查是否有真正的冲突（优先级不同或条件互斥）
                        if perm1.priority != perm2.priority
                            || Self::conditions_conflict(&perm1.condition, &perm2.condition)
                        {
                            conflicts.push(PermissionConflict {
                                position_view: perm1.position_view.clone(),
                                action,
                                permission1: (*perm1).clone(),
                                permission2: (*perm2).clone(),
                                conflict_type: ConflictType::PriorityConflict,
                            });
                        }
                    }
                }
            }
        }

        conflicts
    }

    /// 检查两个条件是否冲突
    fn conditions_conflict(cond1: &Option<String>, cond2: &Option<String>) -> bool {
        match (cond1, cond2) {
            (None, None) => false,
            (Some(_), None) | (None, Some(_)) => false,
            (Some(c1), Some(c2)) => {
                // 简单检查：如果条件字符串不同且不是子串关系，可能冲突
                c1 != c2 && !c1.contains(c2) && !c2.contains(c1)
            }
        }
    }

    /// 解决权限冲突
    pub fn resolve_conflicts(
        conflicts: &[PermissionConflict],
        resolution: ConflictResolution,
    ) -> HashMap<(String, PermissionAction), OntologyPermission> {
        let mut resolved = HashMap::new();

        for conflict in conflicts {
            let winner = match resolution {
                ConflictResolution::DenyOverrides => {
                    // 检查哪个是拒绝权限（通常空动作列表或特定标记）
                    // 这里简化处理：优先级高的胜出
                    if conflict.permission1.priority >= conflict.permission2.priority {
                        &conflict.permission1
                    } else {
                        &conflict.permission2
                    }
                }
                ConflictResolution::PermitOverrides => {
                    // 允许优先，选择优先级高的
                    if conflict.permission1.priority >= conflict.permission2.priority {
                        &conflict.permission1
                    } else {
                        &conflict.permission2
                    }
                }
                ConflictResolution::FirstApplicable => &conflict.permission1,
                ConflictResolution::Ordered | ConflictResolution::ExplicitPriority => {
                    if conflict.permission1.priority >= conflict.permission2.priority {
                        &conflict.permission1
                    } else {
                        &conflict.permission2
                    }
                }
            };

            resolved.insert(
                (conflict.position_view.clone(), conflict.action),
                winner.clone(),
            );
        }

        resolved
    }
}

/// NGAC 权限映射配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NgacMappingConfig {
    /// 策略类名称
    pub policy_class: String,
    /// 实体到对象属性的映射前缀
    pub entity_oa_prefix: String,
    /// 岗位视角到用户属性的映射前缀
    pub position_ua_prefix: String,
    /// 是否生成容器属性
    pub generate_container_oas: bool,
    /// 字段级OA生成策略
    pub field_oa_strategy: FieldOaStrategy,
}

impl Default for NgacMappingConfig {
    fn default() -> Self {
        Self {
            policy_class: "ontology_policy".to_string(),
            entity_oa_prefix: "entity".to_string(),
            position_ua_prefix: "position".to_string(),
            generate_container_oas: true,
            field_oa_strategy: FieldOaStrategy::Separate,
        }
    }
}

/// 字段OA生成策略
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FieldOaStrategy {
    /// 为每个字段创建单独的OA
    Separate,
    /// 将字段分组到单个OA
    Grouped,
    /// 不为字段创建OA（仅实体级）
    None,
}

/// NGAC 映射结果
#[derive(Debug, Clone)]
pub struct NgacMappingResult {
    /// 生成的用户属性
    pub user_attributes: Vec<NgacUserAttribute>,
    /// 生成的对象属性
    pub object_attributes: Vec<NgacObjectAttribute>,
    /// 生成的关联
    pub associations: Vec<NgacAssociation>,
}

/// NGAC 用户属性定义
#[derive(Debug, Clone)]
pub struct NgacUserAttribute {
    pub name: String,
    pub position_view: String,
    pub parent_ua: Option<String>,
    pub properties: HashMap<String, String>,
}

/// NGAC 对象属性定义
#[derive(Debug, Clone)]
pub struct NgacObjectAttribute {
    pub name: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub parent_oa: Option<String>,
    pub properties: HashMap<String, String>,
}

/// NGAC 关联定义
#[derive(Debug, Clone)]
pub struct NgacAssociation {
    pub ua_name: String,
    pub oa_name: String,
    pub access_rights: Vec<String>,
    pub condition: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_action_from_str() {
        assert_eq!(
            PermissionAction::parse("create"),
            Some(PermissionAction::Create)
        );
        assert_eq!(
            PermissionAction::parse("READ"),
            Some(PermissionAction::Read)
        );
        assert_eq!(PermissionAction::parse("*"), Some(PermissionAction::All));
        assert_eq!(PermissionAction::parse("unknown"), None);
    }

    #[test]
    fn test_permission_action_contains() {
        assert!(PermissionAction::All.contains(&PermissionAction::Read));
        assert!(PermissionAction::Read.contains(&PermissionAction::Read));
        assert!(!PermissionAction::Read.contains(&PermissionAction::Create));
    }

    #[test]
    fn test_ontology_permission_allows() {
        let perm = OntologyPermission::new(
            "admin",
            vec![PermissionAction::Read, PermissionAction::Create],
        );
        assert!(perm.allows_action(&PermissionAction::Read));
        assert!(perm.allows_action(&PermissionAction::Create));
        assert!(!perm.allows_action(&PermissionAction::Delete));
    }

    #[test]
    fn test_field_permission() {
        let mut fp = FieldPermission::new();
        fp.add_read_position("admin");
        fp.add_read_position("user");
        fp.add_write_position("admin");

        assert!(fp.can_read("admin"));
        assert!(fp.can_read("user"));
        assert!(!fp.can_read("guest"));
        assert!(fp.can_write("admin"));
        assert!(!fp.can_write("user"));
    }

    #[test]
    fn test_instance_permission_extract_vars() {
        let ip = InstancePermission::new("owner == currentUser OR isPublic == true");
        let vars = ip.extract_variables();
        assert!(vars.contains(&"owner".to_string()));
        assert!(vars.contains(&"currentUser".to_string()));
        assert!(vars.contains(&"isPublic".to_string()));
        assert!(vars.contains(&"true".to_string()));
    }

    #[test]
    fn test_entity_permission_config() {
        let mut config = EntityPermissionConfig::new();
        config.add_permission(OntologyPermission::new(
            "admin",
            vec![PermissionAction::All],
        ));
        config.add_permission(OntologyPermission::new(
            "user",
            vec![PermissionAction::Read, PermissionAction::List],
        ));

        assert!(config.check_permission("admin", &PermissionAction::Delete));
        assert!(config.check_permission("user", &PermissionAction::Read));
        assert!(!config.check_permission("user", &PermissionAction::Delete));
        assert!(!config.check_permission("guest", &PermissionAction::Read));
    }

    #[test]
    fn test_permission_reasoner() {
        let direct = vec![OntologyPermission::new(
            "user",
            vec![PermissionAction::Read],
        )];
        let result = PermissionReasoner::infer_permissions(
            "TestEntity",
            &direct,
            &[],
            InheritanceStrategy::InheritAll,
        );

        assert!(result.inherited_permissions.is_empty());
        assert!(result.conflicts.is_empty());
    }
}
