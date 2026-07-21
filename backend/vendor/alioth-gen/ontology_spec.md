# Alioth Ontology Spec

Alioth 平台本体规约文件。描述平台已有本体结构，供 AI Planner 在构建 OntologyModel 时参考。

> **唯一真相源**: `docs/specs/ALIOTH_ONTOLOGY_SPEC.md`
>
> 本文件由 `alioth-gen` crate 在编译时嵌入，通过 `ALIOTH_ONTOLOGY_SPEC` 常量暴露。
> Planner 应首选编译时嵌入的 `ALIOTH_ONTOLOGY_SPEC` 常量（即本文件内容），
> 获取补充细节时再参考 `docs/specs/ALIOTH_ONTOLOGY_SPEC.md`。
>
> 两个文件**内容应保持同步**。若本文件与 `docs/specs/ALIOTH_ONTOLOGY_SPEC.md` 不一致，
> 以 `docs/specs/ALIOTH_ONTOLOGY_SPEC.md` 为准。

## 一、实体→表映射

### 1.1 映射流程

```
JSON 实体 → 语义分析 → 查现有 lifecycle 叶表 → 匹配/无匹配 → 字段映射
```

**铁则**: `isahl` schema **禁止** CREATE TABLE / ALTER TABLE / 加列。
只允许 CREATE VIEW / MATERIALIZED VIEW / FUNCTION / INDEX。
新增实体类型通过 Meta 平台 `meta_collections` / `meta_fields` 扩展。

### 1.2 现有 lifecycle 叶表速查

以下为 `isahl.zc_id_lifecycle` 继承体系中常用叶表，映射时优先匹配：

| 语义域 | 建议匹配的叶表 | 继承链 |
|--------|---------------|--------|
| 交易记录（订单/单据） | `zc_id_stat-trade_order` 或其子表 | `zc_id_lifecycle` → ... |
| 合同/协议 | `zc_id_contract` 或其子表 | `zc_id_lifecycle` → ... |
| 明细/行项 | `zc_id_deta-trade_order` 或其子表 | `zc_id_lifecycle` → ... |
| 主体/参与方（组织） | `zc_id_subj-org` | `zc_id_subj-org` → `zc_id_position` → `zc_id_lifecycle` |
| 主体/参与方（自然人） | `zc_id_empl-natural` | `zc_id_lifecycle` → ... |
| 产品/物料 | `zc_id_production` 继承链 | `zc_id_lifecycle` → ... |
| 事件/跟进 | `zc_id_event` | `zc_id_lifecycle` → ... |
| 评估/评分 | `zc_id_eval-calculable` | `zc_id_lifecycle` → ... |
| 进度/计划 | `zc_id_progress` 继承链 | `zc_id_lifecycle` → ... |
| 存储/账户 | `zc_id_stor-account` 继承链 | `zc_id_lifecycle` → ... |
| 通用分类 | `zc_id_cate-general` | `zc_id_lifecycle` → ... |
| 通用标签 | `zc_id_tags` | `zc_id_lifecycle` → ... |
| 通用状态 | `zc_id_status` | `zc_id_lifecycle` → ... |

> **无匹配叶表时**: 选择语义最接近的父表，通过 `_f_` / `_t_` / `dk_function` 区分实体类型，标注"待确认"。

### 1.3 ID 生成函数

| 表类型 | 函数 | 说明 |
|--------|------|------|
| `zc_id_lifecycle` 及其所有子表 | `isahl.gen_next_zuid()` | 业务生命周期实体 |
| `zc_ad_object` 根表 | `isahl.gen_next_uid(0)` | 仅根表自身 |
| `isahl_meta` 表 | `BIGSERIAL` | 除 `meta_collections`/`meta_fields` |
| `isahl_auth` / `isahl_audit` | `isahl.gen_next_zuid()` | 认证/审计实体 |
| 非 lifecycle 业务表 | `isahl.gen_next_uid(table_code)` | 确定性 ID |

---

## 二、嵌套分解规则

### 规则 1: 内嵌对象 → belongsTo

```json
{ "order": { "customer": { "id": 1, "name": "ACME" } } }
```
→ `Order.fk_customer = Customer.id`。若 Customer 已存在 → 仅存 FK；若新建 → 先 INSERT Customer。

### 规则 2: 内嵌数组 → hasMany

```json
{ "order": { "items": [{ "product": "...", "qty": 2 }] } }
```
→ `OrderItem` 独立表，含 `fk_order → Order.id`。

**深度优先插入**: `INSERT Order` → 获取 `order_id` → `INSERT OrderItem (fk_order=order_id)` × N。

### 规则 3: 枚举/状态值 → 标量表引用

```json
{ "status": "pending" }
```
→ 查状态定义表 `"zc_id_stus-*"` 获取状态 ID → INSERT INTO `"zc_id_lifecycle_r_primary-status"` (ref_left, ref_right)。
若状态记录不存在 → 先 INSERT 到状态定义表。

### 规则 4: ID 数组 → belongsToMany（桥接表）

```json
{ "order": { "tagIds": [101, 102] } }
```
→ `INSERT INTO zc_id_lifecycle_r_tags (ref_left, ref_right) VALUES (order_id, 101), (order_id, 102)`。

### 规则 5: 多层嵌套 → 递归展开

```json
{ "order": { "items": [{ "batches": [{ "lotNo": "L1", "qty": 3 }] }] } }
```
→ 三层: Order（根）→ OrderItem（子，fk_order）→ OrderItemBatch（孙，fk_order_item）。
插入顺序: Order → OrderItem → OrderItemBatch（深度优先）。

---

## 三、关系类型决策

**核心判据**: 子实体是否与父实体同生命周期？

```
JSON 中有数组或嵌套对象 →
  子实体独立于父实体存在（有自己的生命周期）？
    → 是 → 关系表（r_* 表: 1:N，rr_* 表: M:N）
    → 否 → FK 列（子表持 fk_parent 列）
  是否是单一状态/属性值？
    → 是 → 标量引用（qk_* → zc_id_scal-*）
```

| JSON 结构 | 关系类型 | Alioth 实现 | 生命周期耦合 |
|-----------|----------|-------------|-------------|
| `{ parent: { child } }` 内嵌单对象 | belongsTo | `fk_*` BIGINT 列 → 父表 | ❌ 松耦合 |
| `{ parent: { children: [...] } }` 内嵌数组，子归属父生命周期 | hasMany（FK 列） | 子表持 `fk_parent` 列 | ✅ 紧耦合 |
| `{ parent: { children: [...] } }` 内嵌数组，子可独立存在 | hasMany（关系表） | `r_*` 关系表 | ❌ 松耦合 |
| `{ parent: { single } }` | hasOne | `fk_*` 列或 `r_*` 表（带 unique） | 视选择 |
| `{ items: [id1, id2] }` ID 数组 | belongsToMany（M:N） | `rr_*` 桥接表 | ❌ 松耦合 |
| `{ amount: 100, date: "..." }` 数值/日期 | 标量引用 | `qk_*` → `zc_id_scal-*` | ❌ 独立 |

**判定铁则**:

```text
FK 列 vs 关系表:
  子实体创建/删除与父实体同生命周期？ → 子表持 fk_parent（如 OrderItem.fk_list）
  子实体可独立存在？ → r_* 关系表（如 zc_id_lifecycle_r_file）

r_*（1:N）vs rr_*（M:N）:
  一个子只能属于一个父？ → r_*（ref_left=父, ref_right=子，ref_right 唯一）
  一个子可属于多个父？ → rr_*（ref_left, ref_right 均可重复）
```

---

## 四、字段选择决策树

### 4.1 `number` vs `code` vs `notice`

```text
JSON 中的"编号/名称"字段 →
|  系统自动分配的流水号（如 ORD-20260001）？ → number（TEXT，始终 🔒 维度派生）
|  用户手动输入的编码（如 客户订单号）？ → code（TEXT，✅ 用户）
|  显示名称/标题（如"华东区销售订单"）？ → notice（TEXT，✅ 用户）
|  描述/备注？ → comments（TEXT，✅ 用户）
```

**检查**: 同一记录中，一个字段不应同时承载"编号"和"名称"两种语义。

### 4.2 状态关联机制

```text
实体的主状态用什么机制 →
|  状态值来自 "zc_id_stus-*" 继承链（领域特化状态表）？
|    → zc_id_lifecycle_r_primary-status（ref_right → stus 叶子表）
|  状态值来自 "zc_id_status" 继承链（flag: start/doing/end 三值）？
|    → zc_id_lifecycle_r_status
|  状态仅为实体表上的标志位（不需要独立状态表）？
|    → 实体表上的 flag 列（如 booleans）
|  实体没有状态，仅有生命周期？
|    → 不声明状态关联（deleted_at 即可）
```

> **`r_primary-status` 是主状态，`r_status` 是通用状态，`flag` 是内置枚举。**
> 一个实体最多一个主状态，可有多个通用状态。

### 4.3 `_f_` / `_t_` 判断

`_f_` 和 `_t_` 由 `dk_function.code` 前缀自动计算。
**在任何模块的 DTO 中都不应出现。**

```text
原型字段名为 type / category / form →
|  正交分类（来自 zc_id_category 的 FK）？ → ck_category（✅ 用户）
|  标签（来自 zc_id_tags 的 FK）？ → tk_*（✅ 用户）
|  自由文本分类？ → x_* 扩展列（✅ 用户）
|  是 _f_/_t_？ → ❌ 禁止映射到 DTO
```

> **例外**: `zc_id_contact_infos` 中的 `_f_` 用于区分联系方式类型（phone/email/website），
> 该表未参与触发器自动计算。

### 4.4 `domain_` / `public`

| 字段 | 可写性 | 判定依据 |
|------|--------|----------|
| `domain_` | 🔒 维度派生 | **始终不可写**，不允许例外 |
| `public` | ⚠️ 条件 | DDL `DEFAULT true`，创建时可覆盖，编辑时不可写 |
| `number` | 🔒 维度派生 | **始终不可写**（系统自动生成/触发器分配） |

> **检查**: 所有模块的字段映射中 `domain_` 必须标记 🔒，不得出现 ✅。
>

---

## 五、标量引用模型

### 5.1 硬约束

所有 `qk_*` 字段在 DDL 中为 `bigint`，Rust 模型为 `Option<i64>`，
**绝对禁止**定义为 `DateTime<Utc>`、`Decimal`、`String` 等实际值类型。

### 5.2 标量表清单

| 前缀 | 标量表 | 实际值字段 | 场景 |
|------|--------|-----------|------|
| `qk_date` | `zc_id_scal-date` | `date` (timestamptz) | 日期/时间 |
| `qk_amount` | `zc_id_scal-amount` | `mark` (numeric) | 金额/财务数值 |
| `qk_price` | `zc_id_scal-price` | `mark` (numeric) | 价格/单价 |
| `qk_qty` | `zc_id_scal-common` | `mark` (numeric) | 数量/通用数值 |
| 其他 `qk_*` | `zc_id_scal` 继承体系 | `mark` (numeric) | 通用刻度 |

### 5.3 分层责任

| 层级 | 职责 | 规则 |
|------|------|------|
| DB | 存储标量引用 ID | `qk_*` 列统一为 `bigint` |
| Rust 应用层 | 实际值 ↔ 标量 ID | DTO 可收实际值，服务层通过 `ScalarService` 查找/创建后存 ID |
| 前端 | 只处理实际值 | 表单输入实际值；列表通过 `_refs` 显示解析后的值 |

**禁止**:
- SQL 中直接对 `qk_*` 列进行算术运算
- Rust 模型中直接对 `qk_*`（`i64` 标量 ID）进行算术运算
- 前端直接显示 raw ID

---

## 六、列可写性速查表（映射时的唯一真相源）

以下表格按物理列名索引，映射字段可写性时**直接对照，禁止凭记忆推理**。

| 物理列 | 可写性 | 判定依据 | DTO 形式 |
|--------|--------|----------|----------|
| `id` | 🚫 系统 | `gen_next_zuid()`/`gen_next_uid()` 自动生成 | 不在 DTO 出现 |
| `created_at` / `updated_at` / `deleted_at` | 🚫 系统 | 框架 DEFAULT / 软删除 | 不在 DTO 出现 |
| `created_by_id` / `updated_by_id` | 🚫 系统 | 认证中间件注入 | 不在 DTO 出现 |
| `notice` | ✅ 用户 | 本体核心名称 | `String`，必填 |
| `code` | ✅ 用户 | 本体核心编码 | `String`，必填 |
| `o_number` | 🔒 维度派生 | 系统分配声明编号 | 不在 DTO 出现 |
| `number` | 🔒 维度派生 | 系统分配流水号 | 不在 DTO 出现 |
| `domain_` | 🔒 维度派生 | 领域标记 | 不在 DTO 出现 |
| `_f_` | 🔒 维度派生 | `dk_function.code` 前缀自动计算 | 不在 DTO 出现 |
| `_t_` | 🔒 维度派生 | 同上 | 不在 DTO 出现 |
| `dk_scene` | 🔒 应用绑定 | 后端 repository 绑定 | 不在 DTO 出现 |
| `dk_factor` | 🔒 应用绑定 | 后端 repository 绑定 | 不在 DTO 出现 |
| `dk_function` | 🔒 应用绑定 | 后端 repository 绑定 | 不在 DTO 出现 |
| `t_color_` | ✅ 用户 | 颜色标记 | `Option<String>` |
| `public` | ⚠️ 条件 | DDL `DEFAULT true`，创建时可覆盖 | `Option<boolean>`，创建时可选 |
| `sort` | ✅ 用户 | 排序权重 | `Option<i32>` |
| `cron` | ✅ 用户 | Cron 表达式 | `Option<String>` |
| `progress_pct` / `schedule_pct` | ✅ 用户 | 进度百分比 | `Option<f64>` |
| `d_count` | 🔒 系统 | 触发器维护明细计数 | 不在 DTO 出现 |
| `c_count` | ⚠️ 条件 | 部分表 DEFAULT=0 可写入 | `Option<i64>`，仅创建时 |
| `ref_count` | 🔒 系统 | 触发器维护引用计数 | 不在 DTO 出现 |
| `ak_dimensions` / `ak_components` | 🔒 系统 | 触发器维护组合分量 | 不在 DTO 出现 |
| `r_number` | ✅ 用户 | 关系编号 | `Option<String>` |
| `r_notice` | ✅ 用户 | 关系名称 | `Option<String>` |
| `ref_left` / `ref_right` | ✅ 用户 | 关联实体 ID | `Option<i64>` |
| `action_type` | ✅ 用户 | 操作类型 | `Option<String>` |
| `majority` | 🔒 维度派生 | 多数标记 | 不在 DTO 出现 |
| `sprint` | 🔒 维度派生 | 冲刺/迭代标记 | 不在 DTO 出现 |
| `model` | 🔒 维度派生 | 模型标记 | 不在 DTO 出现 |
| `p_number` | 🔒 维度派生 | 父编号 | 不在 DTO 出现 |
| `paths` | 🔒 系统 | 层级路径 JSONB，子表路由器维护 | 不在 DTO 出现 |
| `qk_*`（所有标量引用）| ✅ 用户 | 结构化值对象（`ScalarPriceValue` 等）→ 服务层转 ID | 结构化值对象 |
| `sk_*`（所有单位引用）| ✅ 用户 | 选择器传单位 ID | `Option<i64>` |
| `fk_*`（所有生命体引用）| ✅ 用户 | 选择器传目标实体 ID | `Option<i64>` |
| `ck_*`（所有类目引用）| ✅ 用户 | 类目选择器传类目 ID | `Option<i64>` |
| `tk_*`（所有标签引用）| ✅ 用户 | 标签选择器传标签 ID（单选） | `Option<i64>` |
| `lk_*`（所有等级引用）| ✅ 用户 | 等级选择器传等级 ID | `Option<i64>` |

### 使用规则

1. **直接对照本表**，不凭记忆。
2. 物理列不在本表中 → 优先标记 🔒 维度派生，标注待确认。
3. `isahl_meta` / `isahl_auth` / `isahl_audit` schema 的表不受本表约束。

---

## 七、跨模块一致性

| 违反模式 | 修复方式 |
|---------|---------|
| `CreateXxxRequest.qk_price: Option<String>` | → `Option<ScalarPriceValue>` |
| `CreateXxxRequest.qk_date: Option<String>` | → `Option<ScalarDateValue>` |
| `CreateXxxRequest.sk_unit: Option<String>` | → `Option<i64>` |
| `CreateXxxRequest.fk_xxx: Option<String>` | → `Option<i64>` |
| `CreateXxxRequest.ck_xxx: Option<String>` | → `Option<i64>` |
| `CreateXxxRequest.tk_xxx: Option<String>` | → `Option<i64>` |
| `CreateXxxRequest.lk_xxx: Option<String>` | → `Option<i64>` |
| SQL `NULLIF($n, '')::bigint` | → 移除转换，直接 bind `i64`/`Option<i64>` |

---

## 八、构造约束

### 8.1 `isahl` schema 禁止项

在 `isahl` schema 中：
- ❌ 创建新表（CREATE TABLE）
- ❌ 加列（ALTER TABLE ... ADD COLUMN）
- ❌ 修改已有字段
- ✅ CREATE VIEW / MATERIALIZED VIEW
- ✅ CREATE FUNCTION
- ✅ CREATE INDEX

`isahl_meta` / `isahl_auth` / `isahl_audit` 在 v1 前（至 2026-09-30）可自由变更。

### 8.2 实体扩展策略

| 策略 | 适用场景 | 操作 |
|------|---------|------|
| **复用现有表** | 语义与某 lifecycle 叶表匹配 | 直接映射，通过 `_f_`/`_t_`/`dk_function` 区分 |
| **VIEW 扩展** | 需定制投影或过滤 | `CREATE VIEW isahl.vw_{名称} AS SELECT ...` |
| **Meta 平台扩展** | 需新实体类型元数据定义 | 通过 `meta_collections`/`meta_fields` 管理 |

### 8.3 命名规范

三层命名空间：

| 层级 | 规则 |
|------|------|
| L1 物理列（DB） | Alioth 物理命名: `fk_*`, `qk_*`, `sk_*`, `code`, `number`, `notice`, `_f_`, `_t_`, `t_color_` 等 |
| L2 DTO 字段（Rust） | 业务语义命名（去物理前缀）：`notice`→`name`, `fk_country`→`country`, `qk_price`→`price` |
| L3 前端模型（TS） | 与 DTO 字段名 1:1 透传，不另起名 |

❌ 禁止: `customer_id`, `order_number`, `total_amount`, `inventory_status` 等业务别名在数据库层出现。

### 8.4 系统字段与 DTO 排除

| 类别 | 例 | 规则 |
|------|-----|------|
| 框架系统 | `id`, `created_at`, `updated_at`, `deleted_at`, `created_by_id`, `updated_by_id` | 🚫 全排除 |
| 维度派生 | `o_number`, `number`, `domain_`, `_f_`, `_t_`, `majority`, `sprint`, `model`, `p_number` | 🔒 全排除 |
| 应用绑定 | `dk_scene`, `dk_factor`, `dk_function` | 🔒 全排除 |
| 触发器维护 | `d_count`, `ref_count`, `ak_dimensions`, `ak_components`, `paths` | 🔒 全排除 |
| 用户可写 | `notice`, `code`, `comments`, `t_color_`, `sort`, `cron` | ✅ 包含 |
| 条件可写 | `public`, `c_count` | ⚠️ 按规则 |

---

## 九、歧义场景预设（禁止重复提问）

以下场景在已完成模块中反复出现，已规约化为确定性决策。**遇到相同场景直接按此执行，无需再次向用户提问。**

### A: 编号/名称字段（已决）

→ 见 §4.1 决策树。

### B: 状态关联机制（已决）

→ 见 §4.2 决策树。

### C: FK 列 vs 关系表（已决）

→ 见 §3 决策树。

### D: `domain_` / `public` 可写性（已决）

→ 见 §4.4。

### E: `_f_` / `_t_` 映射（已决）

→ 见 §4.3。

### F: 同表多模块映射

多个模块映射到同一物理表时：
- 通过角色视角标签区分 → 允许
- 同一列在不同模块中可写性不同 → **冲突，需人工仲裁**
- 已有模块标记某列为"已移除"但在当前模块 DTO 中有 → **残留，自动移除**
