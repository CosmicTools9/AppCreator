# AppCreator 设计意图

> 版本: v1 | 2026-07-27

---

## 一句话定位

AppCreator 是 **Alioth 模型的开源独立消费者入口**——通过对话调用 Alioth 模型管道创建企业管理应用，不具备 AliothStudio 对 Alioth 模型的完整管理和开发能力。

## 与 AliothStudio 的关系

```
Alioth 模型 (alioth-gen v10) — 代码生成引擎
    ├── AliothStudio (完整平台) — 拥有 Alioth 模型的完整管理和开发能力
    │   ├── AppAgent 对话创建应用
    │   ├── Meta 元数据管理 UI（collection 编辑器、字段编辑器、版本管理、数据字典、血缘追踪）
    │   ├── 开发组件原型管理（Prototype/Block/Module 设计器、视觉验证工作台）
    │   ├── Gateway 运行时（多应用发现、路由、NGAC 权限）
    │   ├── SSO 认证体系（OAuth2、企业目录集成）
    │   └── 规约审计层（45+ 规约 + spec-audit + 检查门禁）
    │
    └── AppCreator (开源独立入口) — 受限的 Alioth 模型消费者
        ├── AppAgent 对话创建应用 ✅
        ├── 元数据管理 UI             ❌
        ├── 开发组件原型管理           ❌
        ├── Gateway 运行时             ❌（但产物兼容）
        ├── SSO 认证                   ❌（可选集成，非内建）
        └── 规约审计层                 ❌
```

**AliothStudio 拥有 Alioth 模型的完整管理和开发能力**——它可以管理元数据（实体定义、字段配置、生命周期绑定）、管理开发组件（Prototype/Block/Module 设计、视觉验证）、控制权限和发布流程。AppCreator 不具备这些能力，它只能通过对话接口调用 Alioth 模型管道的子集。

## 共享管道

AppCreator 和 AliothStudio 共享同一套 AppAgent 管道代码（通过 vendor crate）：

| 组件 | 来源 | AppCreator | AliothStudio |
|------|------|-----------|--------------|
| AppAgent 状态机 | `vendor/app-agent` | ✅ | ✅ |
| alioth-gen 引擎（IR/常量/可视化） | `vendor/alioth-gen` | ✅ | ✅ |
| ontology-mapping | `vendor/ontology-mapping` | ✅ | ✅ |
| runtime-engine | `vendor/runtime-engine` | ✅ | ✅ |
| alioth-gen CLI 完整代码生成（module 级 crate） | 主仓 `Meta/backend/alioth-gen` | ❌ 未 vendor `crud` crate | ✅ |
| `isahl_meta` 元数据表 | 共享 PostgreSQL | ✅ 完整访问（AppAgent 管道需要的所有表：`meta_collections`、`meta_fields`、`devv_inherits_union` + `meta_chat_sessions/messages`；standalone 模式启动时自愈建表） | ✅ 完整访问 |

AppCreator 能生成与 AliothStudio 完全相同的产物格式（`app.json` + extensions + HTML 原型），但其 Alioth 模型能力受限于已 vendored 的 crate 子集——缺少 `crud` crate 意味着无法通过 `alioth-gen` CLI 生成新模块后端的完整 Rust crate。

## 为什么独立开源

1. **Alioth 模型的开源入口**：让任何人无需部署完整 AliothStudio 即可体验 Alioth 模型的对话创建应用能力。
2. **低门槛**：仅需 PostgreSQL + LLM API key，不需要 SSO 集群、Gateway 编排、Docker。
3. **自托管隐私**：代码和数据完全由用户控制，无供应商锁定。
4. **产物兼容**：用 AppCreator 创建的应用可导入 AliothStudio 进行完整开发和管理。

## 适用范围

| 场景 | 适用产品 |
|------|---------|
| 评估 Alioth 模型能力、快速原型验证 | AppCreator |
| 生产级多模块应用的完整开发和管理 | AliothStudio |
| 企业内部需要元数据管理、权限控制、审计合规 | AliothStudio |
| 个人自托管、最小依赖、对话式应用创建 | AppCreator |

## 生成物

AppCreator 的产物格式与 AliothStudio 一致：

```
Pre-Proc/{namespace}/Apps/{app}/
├── app.json              ← 应用配置（模块、路由、权限、品牌）
├── extensions/           ← YAML 扩展（约束、规则、状态机、工作流）
│   ├── constraints.yaml
│   ├── rules.yaml
│   ├── statemachines.yaml
│   └── workflows.yaml
├── prototype.html        ← 交互式 HTML 原型
├── gateway_design.md     ← 前端设计方案
└── Sources/              ← 模块/Service/Block 源码骨架
    ├── Modules/{id}/
    └── Services/{name}/
```

这些产物可直接导入 AliothStudio 做进一步开发和发布，也可由 Gateway 发现加载。
