//! OntologyModel → LLM Prompt 结构化转换器
//!
//! 将本体语义模型转换为结构化 LLM prompt，支持以下生成目标：
//! - Rust 后端 handlers (Actix-web + SQLx)
//! - TypeScript 前端组件 (React + Jotai + React Query)
//!
//! 本模块是 CODE_GEN_PIPELINE_SPEC.md §3.3 "LLM Prompt 工程规范" 的 MVP 实现。

use meta_model::ontology::{
    ConstraintOntology, DomainKind, DomainOntology, OntologyModel, RelationOntology,
    TransactionLifecycle,
};

/// 生成目标语言
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetLang {
    Rust,
    TypeScript,
}

/// Prompt 构建器
pub struct PromptBuilder {
    lang: TargetLang,
    module_name: String,
    user_intent: String,
}

impl PromptBuilder {
    pub fn new(
        lang: TargetLang,
        module_name: impl Into<String>,
        user_intent: impl Into<String>,
    ) -> Self {
        Self {
            lang,
            module_name: module_name.into(),
            user_intent: user_intent.into(),
        }
    }

    /// 构建完整的 LLM prompt
    pub fn build(&self, model: &OntologyModel) -> String {
        let mut prompt = String::new();

        // ── 系统指令 ──
        prompt.push_str(&self.system_instruction());
        prompt.push_str("\n\n");

        // ── 领域实体 ──
        prompt.push_str(&self.format_domains(&model.domains));
        prompt.push('\n');

        // ── 关系定义 ──
        if !model.relations.is_empty() {
            prompt.push_str(&self.format_relations(&model.relations));
            prompt.push('\n');
        }

        // ── 交易生命周期 ──
        if let Some(ref lifecycle) = model.transaction_lifecycle {
            prompt.push_str(&self.format_lifecycle(lifecycle));
            prompt.push('\n');
        }

        // ── 约束 ──
        if !model.constraints.is_empty() {
            prompt.push_str(&self.format_constraints(&model.constraints));
            prompt.push('\n');
        }

        // ── 生成指令 ──
        prompt.push_str(&self.generation_instruction());

        prompt
    }

    fn system_instruction(&self) -> String {
        let lang_str = match self.lang {
            TargetLang::Rust => "Rust (Actix-web + SQLx)",
            TargetLang::TypeScript => "TypeScript (React + Jotai v2 + @tanstack/react-query)",
        };

        format!(
            r#"你是一个企业级数据管理应用的代码生成专家。
你的任务是根据给定的本体语义模型和预制件接口契约，生成可直接编译运行的 {lang} 代码。

## Alioth 核心约束（必须遵守）

### 数据库与命名

1. **物理列名一致性**：后端 Rust 字段名必须与数据库物理列名完全一致。
   - `notice` 而非 `name`（名称/描述字段）
   - `code` 而非 `product_id`（编码字段，仅面向编码体系/SKU/ISBN 等场景使用）
   - `fk_source` 而非 `source_id`
   - `fc_`/`qk_`/`sk_`/`dk_`/`ck_`/`tk_`/`ik_`/`rk_`/`ak_` 前缀分别对应外键/计量键/标量键/维度键/分类键/标签键/等级键/角色键/数组键
   - 连字符列使用 `#[sqlx(rename = "exact-column")]`，如 `_f_`、`_t_`
   - 前端 TypeScript 允许业务概念命名

2. **类型规范**：
   - 金额/数量使用 `rust_decimal::Decimal`（Rust）或 `number`（TypeScript），禁止 `f64`/`double`
   - 文本统一使用 `text` 类型，禁止 `varchar(n)`
   - 布尔标志使用 `boolean`
   - 数组使用 `bigint[]`/`text[]`

3. **主键策略**：
   - `isahl.zc_id_lifecycle` 及其子表：`BIGINT DEFAULT isahl.gen_next_zuid()`
   - `isahl.zc_id_object` 及非 lifecycle 子表：`BIGINT DEFAULT isahl.gen_next_uid(table_code)`
   - `isahl_meta` 表：`BIGSERIAL`
   - **前端 zuid 处理**（强制）：zuid 值远超 JS Number 安全整数上限 `2^53-1`。
     TypeScript 中所有 id/zuid/主键/外键变量类型必须声明为 `string | number`，禁止窄化为 `number`。
     ID 比较统一使用 `String(a) === String(b)`。

### 触发型字段（前端禁止使用/写入，由 DB Trigger 自动填充）

4. 以下字段**绝不能**出现在前端创建/更新请求体中：
   - `notice` — 对象类型名（Trigger 自动填充）
   - `o_number` — 业务编号（Trigger 自动生成）

5. **_f_ / _t_ 自动派生**：由 `dk_function`（通过 `zc_id_function.code` 前缀）
   **自动派生**（Trigger Registry 的 LifecycleBizTemplate），INSERT/UPDATE 时若显式传入非空值则保留，
   否则自动计算。映射规则：`!.`→(创意,范例) `!_`→(创意,实例) `↑.`→(设计,范例) `↑_`→(设计,实例)
   `↓.`→(实现,范例) `↓_`→(实现,实例)。

### 字段映射（恒定规则）

6. **前端 `name` ↔ 后端 `notice`**：适用于 `zc_ad_object` 继承体系（分类、状态、标签等标量/维度表）。
   后端 Rust 模型输入时，`notice` 字段承载名称语义。

7. **`code` vs `notice` 优先级**：`notice` 是优先使用的内容字段；`code` 仅面向编码体系（SKU、ISBN 等）
   使用。除非明确属于编码场景，否则使用 `notice` 承载名称/描述类内容。

### 表约束

8. **禁止非 Alioth 表**：所有数据库表必须命名为 `zc_id_*` 或 `zc_ad_*`，
   继承自 Alioth 核心基类（`zc_id_entity`、`zc_id_bill`、`zc_id_event`、`zc_id_lifecycle` 等）。
   禁止创建独立自定义表名（如 `products`、`orders`、`warehouses`）。

9. **继承要求**：业务模块表继承适当地基类：
   - 单据 → `zc_id_bill`
   - 实体 → `zc_id_entity`
   - 事件 → `zc_id_event`
   - 协议 → `zc_id_agreement`
   - 关系表 → `zc_id_lifecycle_rr_non_self`（M:N）或 `zc_id_lifecycle_r_*`（1:N）

### 前端架构

10. **状态管理**：Jotai v2 + @tanstack/react-query。禁止 Zustand/Redux/Recoil/MobX。
11. **路由守卫**：必须等待 `isLoading === false` 后再判断 `isAuthenticated`，禁止在加载期间跳转。
12. **DTO 映射**：前端发送请求时，DTO 字段映射到后端物理列：
    - `name` → `notice`
    - `code` → `code`
    - `customer_id` → `fk_client` 等

### 工程质量

13. **错误处理**：所有代码必须包含基础错误处理和日志（`common::telemetry` crate / `console.error`）。
14. **NGAC 权限**：资源操作需接入 NGAC 权限检查（`isahl_auth` schema），禁止裸查询无权限过滤。

## 生成模块：{module}

## 用户意图
{intent}
"#,
            lang = lang_str,
            module = self.module_name,
            intent = self.user_intent,
        )
    }

    fn format_domains(&self, domains: &[DomainOntology]) -> String {
        if domains.is_empty() {
            return String::new();
        }

        let mut out = String::from("## 领域实体\n\n");
        for d in domains {
            out.push_str(&format!(
                "### {} ({})\n",
                d.name,
                domain_kind_label(&d.kind)
            ));
            out.push_str(&format!("- ID: `{}`\n", d.id));
            if let Some(ref desc) = d.description {
                out.push_str(&format!("- 描述: {}\n", desc));
            }
            if !d.parent_ids.is_empty() {
                out.push_str(&format!(
                    "- 继承自: {}\n",
                    d.parent_ids
                        .iter()
                        .map(|s| format!("`{}`", s))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            // 属性列表
            if !d.properties.is_empty() {
                out.push_str("- 属性:\n");
                for prop in &d.properties {
                    let required = if prop.required { " (必需)" } else { "" };
                    out.push_str(&format!(
                        "  - `{}`{} — {}\n",
                        prop.name,
                        required,
                        prop.semantic_description.as_deref().unwrap_or(""),
                    ));
                }
            }

            // 预制件契约
            if let Some(ref contract) = d.prefab_contract {
                out.push_str(&format!(
                    "- 预制件契约: `{}` (version {})\n",
                    contract.prefab_id, contract.interface_version,
                ));
            }

            // 等价/互斥关系
            if !d.equivalent_ids.is_empty() {
                out.push_str(&format!(
                    "- 等价于: {}\n",
                    d.equivalent_ids
                        .iter()
                        .map(|s| format!("`{}`", s))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            if !d.disjoint_ids.is_empty() {
                out.push_str(&format!(
                    "- 互斥于: {}\n",
                    d.disjoint_ids
                        .iter()
                        .map(|s| format!("`{}`", s))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            out.push('\n');
        }
        out
    }

    fn format_relations(&self, relations: &[RelationOntology]) -> String {
        let mut out = String::from("## 关系定义\n\n");
        for r in relations {
            out.push_str(&format!(
                "- `{}` — {} → {}\n",
                r.name, r.source_ontology, r.target_ontology,
            ));
            if let Some(ref desc) = r.semantic_description {
                out.push_str(&format!("  - 描述: {}\n", desc));
            }
            out.push_str(&format!("  - 关系类型: {:?}\n", r.relation_type));
        }
        out
    }

    fn format_lifecycle(&self, lifecycle: &TransactionLifecycle) -> String {
        let mut out = String::from("## 交易生命周期\n\n");
        out.push_str(&format!("- ID: `{}`\n", lifecycle.id));
        out.push_str(&format!("- 交易类型: {:?}\n", lifecycle.transaction_type));
        if !lifecycle.phases.is_empty() {
            out.push_str("- 阶段:\n");
            for phase in &lifecycle.phases {
                out.push_str(&format!("  - {} ({})\n", phase.name, phase.id));
            }
        }
        if !lifecycle.transitions.is_empty() {
            out.push_str("- 状态转换:\n");
            for t in &lifecycle.transitions {
                out.push_str(&format!(
                    "  - {} → {} ({})\n",
                    t.from_phase, t.to_phase, t.trigger_event
                ));
            }
        }
        out
    }

    fn format_constraints(&self, constraints: &[ConstraintOntology]) -> String {
        let mut out = String::from("## 约束条件\n\n");
        for c in constraints {
            out.push_str(&format!(
                "- {}: {}\n",
                c.name,
                c.description.as_deref().unwrap_or("")
            ));
        }
        out
    }

    fn generation_instruction(&self) -> String {
        match self.lang {
            TargetLang::Rust => self.rust_generation_instruction(),
            TargetLang::TypeScript => self.ts_generation_instruction(),
        }
    }

    fn rust_generation_instruction(&self) -> String {
        let mut s = String::from(
            r#"
## 生成要求（Rust 后端）

请生成以下文件：

### 1. `handlers.rs` — Actix-web handler 层

- 标准 CRUD 使用 `Framework/backend/crud` 的泛型 handler：
  `crud_list::<E, C, U, R, Err>` / `crud_get` / `crud_create` / `crud_update` / `crud_delete`
- 一键路由注册：`crud_routes::<E, C, U, R, Err>(path)` 生成 5 条标准路由
- 若模块需接入应用级扩展（约束/规则/状态机），替换为
  `crud_routes_with_extensions::<E, C, U, R, Err>(path)`
- 需要自定义 list 过滤时，使用独立 handler + `QueryBuilder::<E>::from_list_query(&pool, &query)`
- 所有金额/数量字段使用 `rust_decimal::Decimal`

### 2. `models.rs` — 数据库行结构体

- 实现 `AliothDbEntity` trait：
  ```rust
  impl AliothDbEntity for MyEntity {
      fn table_name() -> &'static str { "isahl.zc_id_xxx" }
      const SELECT_FIELDS: &'static str = "id, notice, code, ...";  // 禁止 SELECT *
      const SOFT_DELETE: bool = true;
      const HAS_AUDIT: bool = false;
  }
  ```
- 派生 `sqlx::FromRow`，所有字段使用 `#[sqlx(rename = "exact-column")]`
  （连字符列如 `_f_`、`_t_` 必须 rename）
- 字段名必须与数据库物理列名一致（`notice`/`code`/`fk_source`，禁止业务别名）
- 触发型字段（`notice`/`o_number`）不包含在 Rust struct 中
- **若实体有关联字段（belongsTo / hasMany / belongsToMany 等）**，必须：
  1. 添加 `_refs: Option<serde_json::Value>` 字段（`#[sqlx(default)]` + `#[serde(skip_serializing_if = "Option::is_none")]`）
  2. 实现 `HasReferenceJoins` trait 声明所有关联关系
  3. repositorie 中 `list`/`get` 委托 `self.generic.list_refs()` / `self.generic.get_refs()`

### 3. `repositories.rs` — 持久层

- 实现 `AliothRepository<E, C, U, Err>` trait：
  ```rust
  async fn list(&self, query: &ListQuery) -> Result<PaginatedResponse<E>, Err>;
  async fn get(&self, id: i64) -> Result<Option<E>, Err>;
  async fn create(&self, req: C, user_id: i64) -> Result<E, Err>;
  async fn update(&self, id: i64, req: U, user_id: i64) -> Result<Option<E>, Err>;
  async fn delete(&self, id: i64, user_id: i64) -> Result<(), Err>;
  ```
- 标准读操作可委托 `QueryBuilder::<E>::from_list_query(&pool, &list_query).fetch(page, page_size)`
- 若实体需子表路由（根据 discriminator 路由到不同物理子表），实现 `SubtableRouter` trait

### 4. `services.rs` — 业务逻辑层

- 封装跨 repository 的业务编排、校验、审计日志写入
- 调用 `isahl.gen_next_zuid()` 或 `isahl.gen_next_uid(table_code)` 获取主键

### 5. `mod.rs` — 模块注册

- 在 Gateway 的 `config_fn` 中注册路由

"#,
        );
        s.push_str(&self.output_format_spec());
        s
    }

    fn ts_generation_instruction(&self) -> String {
        let mut s = String::from(
            r#"
## 生成要求（TypeScript 前端）

### 技术栈（强制）

- React 18+ + TypeScript 5+
- Jotai v2（唯一客户端状态管理，禁止 Zustand/Redux/Recoil/MobX）
- @tanstack/react-query（服务端状态缓存，禁用 mock 数据做功能验证）
- Tailwind CSS（样式方案）
- react-hook-form + zod（表单验证）
- React Router v7（路由）

### zuid / 大整数处理（强制）

所有 id/zuid/主键/外键变量的 TypeScript 类型声明必须为 `string | number`，禁止窄化为 `number`。
```tsx
// ✅ 正确
const [editingRowId, setEditingRowId] = useState<string | number | null>(null);
const isSame = String(row.id) === String(editingRowId);
await api.updateRecord(tableName, String(id), payload);

// ❌ 错误
const [editingRowId, setEditingRowId] = useState<number | null>(null);
const isSame = row.id === editingRowId;
await api.updateRecord(tableName, Number(id), payload);
```

### 触发型字段禁止（强制）

以下字段**禁止**出现在任何前端创建/更新请求体中，即使后端模型暴露了也不得发送：
`notice`, `o_number`

### DTO 字段映射（强制）

前端 DTO → 后端物理列映射表：

| 前端字段 | 后端物理列 | 适用场景 |
|---------|-----------|---------|
| `name` | `notice` | zc_ad_object 继承体系（分类/状态/标签等） |
| `code` | `code` | 编码字段（SKU/ISBN 等特定领域） |
| `customerId` | `fk_client` | 客户外键 |
| `vendorId` | `fk_vendor` | 供应商外键 |

### 路由守卫（强制）

路由守卫必须等待认证加载完成后再判断：
```tsx
function AuthGuard() {
  const { isAuthenticated, isLoading } = useAuth();
  if (isLoading) return <LoadingSpinner />;  // 禁止在加载期间跳转
  if (!isAuthenticated) return <Navigate to="/login" />;
  return <Outlet />;
}
```

### 页面布局（两栏布局标准）

所有业务模块前端采用统一的两栏布局：

```
┌──────────────┬───────────────────────────────────┐
│  Sidebar     │  TopBar (h-16)                    │
│  (w-60/w-16) │  ├ 面包屑 / 全局搜索 / 公共按钮    │
│              ├───────────────────────────────────┤
│  Branding    │  ContentArea (bg-muted/30)        │
│  MainNav     │  ├ PageHeader (标题 + 新建按钮)    │
│  Collapse    │  ├ StatsGrid (统计卡片)           │
│              │  ├ SearchToolbar                  │
│              │  └ DataTable                      │
└──────────────┴───────────────────────────────────┘
```

- 外层容器：`flex h-screen overflow-hidden bg-background`
- 左侧栏展开 `w-60`，收起 `w-16`（仅图标 Tooltip）
- 列表页结构：`PageHeader → StatsGrid → SearchToolbar → DataTable`
- 使用 `Framework/frontend/components` 的共享组件

### 请生成以下文件

1. **API 层** (`api.ts`) — 使用 `Framework/frontend/api` 共享客户端，封装 CRUD 调用
2. **列表页** (`ListPage.tsx`) — DataTable + react-query 分页 + Jotai 选中状态
3. **表单页/抽屉** (`FormPage.tsx` 或 `FormDrawer.tsx`) — react-hook-form + zod schema 验证
4. **路由配置** — 在模块 `App.tsx` 中注册路由

"#,
        );
        s.push_str(&self.output_format_spec());
        s
    }

    /// 统一的输出格式规范（对齐 CODEGEN_PIPELINE_SPEC §3.3）
    fn output_format_spec(&self) -> String {
        r#"## 输出格式（必须严格遵守）

使用以下 JSON 结构输出，不要包含 markdown 代码块标记：

```json
{
  "generation_id": "<唯一标识>",
  "files": [
    {
      "path": "相对路径（如 src/handlers.rs）",
      "content": "文件完整内容",
      "file_type": "rust|typescript|tsx|sql|toml|json",
      "purpose": "此文件的生成理由（一句话）"
    }
  ],
  "metadata": {
    "total_files": 0,
    "language": "rust|typescript",
    "framework": "actix-web|react"
  },
  "rationale": "生成决策说明（2-3 句）"
}
```"#
            .to_string()
    }
}

/// 从 CodeGenerationRequest 的 ontology_model 字符串构建 prompt
///
/// 尝试将字符串反序列化为 OntologyModel，成功则使用结构化 prompt；
/// 失败则回退到原始字符串拼接（兼容旧接口）。
pub fn build_prompt_from_request(
    request: &crate::generator::ir::llm_contract::CodeGenerationRequest,
) -> String {
    // 尝试反序列化 ontology_model
    if let Ok(model) = serde_json::from_str::<OntologyModel>(&request.ontology_model) {
        let lang = match request.target.platform {
            crate::generator::ir::llm_contract::TargetPlatform::Rust => TargetLang::Rust,
            _ => TargetLang::TypeScript,
        };
        let builder = PromptBuilder::new(lang, &request.target.module_name, &request.user_intent);
        return builder.build(&model);
    }

    // 回退：原始字符串拼接（兼容旧接口）
    format!(
        r#"You are an expert software engineer. Generate code based on the following ontology model and user intent.

## Target
- Type: {:?}
- Platform: {:?}
- Module: {}

## User Intent
{}

## Ontology Model
{}

## Prefab Contracts
{}

Please generate the code files as a JSON array with objects containing "path" and "content" fields.
"#,
        request.target.target_type,
        request.target.platform,
        request.target.module_name,
        request.user_intent,
        request.ontology_model,
        serde_json::to_string(&request.prefab_contracts).unwrap_or_default(),
    )
}

fn domain_kind_label(kind: &DomainKind) -> &str {
    match kind {
        DomainKind::Entity => "实体",
        DomainKind::ValueObject => "值对象",
        DomainKind::AggregateRoot => "聚合根",
        DomainKind::DomainService => "领域服务",
        DomainKind::DomainEvent => "领域事件",
        DomainKind::Enumeration => "枚举",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use meta_model::ontology::{OntologyProperty, PropertyType};

    #[test]
    fn test_empty_model() {
        let model = OntologyModel::default();
        let builder = PromptBuilder::new(TargetLang::Rust, "test", "测试生成");
        let prompt = builder.build(&model);

        assert!(prompt.contains("Alioth 核心约束"));
        assert!(prompt.contains("test"));
        assert!(prompt.contains("测试生成"));
        assert!(prompt.contains("Rust (Actix-web + SQLx)"));
    }

    #[test]
    fn test_model_with_domains() {
        let model = OntologyModel {
            domains: vec![DomainOntology {
                id: "order".into(),
                name: "订单".into(),
                description: Some("销售订单实体".into()),
                kind: DomainKind::AggregateRoot,
                properties: vec![OntologyProperty {
                    id: "total".into(),
                    name: "total_amount".into(),
                    semantic_description: Some("订单总金额".into()),
                    property_type: PropertyType::DataProperty,
                    required: true,
                    cardinality: meta_model::ontology::Cardinality::default(),
                    domain: String::new(),
                    range: "decimal".into(),
                    is_functional: false,
                    is_transitive: false,
                    is_symmetric: false,
                    constraints: vec![],
                }],
                parent_ids: vec![],
                equivalent_ids: vec![],
                disjoint_ids: vec![],
                prefab_contract: None,
            }],
            ..Default::default()
        };

        let builder = PromptBuilder::new(TargetLang::Rust, "orders", "生成订单 CRUD");
        let prompt = builder.build(&model);

        assert!(prompt.contains("订单"));
        assert!(prompt.contains("聚合根"));
        assert!(prompt.contains("total_amount"));
        assert!(prompt.contains("必需"));
    }
}
