# AppCreator 开源版工具缺失清单

> 版本: v3 — 全部已修复 | 2026-07-27
> 范围：AppAgent 管道各阶段依赖的外部系统/表/文件

---

## 全部已修复 ✅

| 问题 | 修复 |
|---|---|
| F1 `isahl_meta` schema 缺失 | 种子 DDL + 启动自动检测 |
| F1 `meta_collections/meta_fields` seed 数据 | 807 collections + 23,101 fields，启动自动注入 |
| F1 `isahl_meta` 关键函数 | 7 个 `gf_*` 函数已追加到 schema DDL |
| B1 `compiled_module_ids()` 空集 | 添加 `Pre-Proc/*/Sources/Modules/` 扫描 fallback |
| B2 GatewayShell 路径硬编码 | 复制到 `AppCreator/references/gateway-shell.tsx` + 模板 import 更新 |
| D1 `agent_memory` | 运行时 `CREATE TABLE IF NOT EXISTS` 自动处理 |
| D2 `@alioth/*` 前端包 | 9 个包 vendor 到 `frontend/packages/` + pnpm workspace |
| D3 Skills YAML 文件 | 9 个 YAML 文件到 `skill-adapters/` |

## 管道流程（全部 ✅）

```
AppAgent 状态机
  │
  ├─ SemanticAnalysis           — ✅ 纯 LLM
  ├─ OntologyAnalysis           — ✅ isahl_meta seed 就绪 → PlatformCatalog 有上下文
  ├─ Planning                   — ✅ compiled_module_ids 有 Pre-Proc 扫描 fallback
  ├─ ModuleCreation             — ✅ 文件系统写 module.json scaffold
  ├─ BlockCreation              — ✅ 文件系统写 block.json scaffold
  ├─ OntologyTransfer           — ✅ aligner 读 devv_inherits_union + meta_collections
  ├─ Module/Service API 创建    — ✅ 文件系统写 service.json scaffold
  ├─ ExecutingSkill              — ✅ skills YAML 已 vendored
  ├─ Composing                  — ✅ GatewayShell 路径已修复；ModuleLayout 由原型构建流程处理
  ├─ Verifying                  — ✅ JSON/YAML 格式校验
  ├─ Publishing                 — ✅ 不依赖 Gateway（仅写产物 + cargo check）
  └─ Published / Presenting     — ✅
```

## 设计说明

### `compiled_module_ids()` 在 AppCreator 中的行为

1. 优先读 `compiled_modules.json`（Gateway 编译产物）→ 不存在则
2. 读 `Gateway/backend/Cargo.toml`（feature 清单）→ 不存在则
3. 扫描 `Pre-Proc/*/Sources/Modules/` 下已有的模块 ID

在首次运行的 AppCreator 中，三路全空 → 返回空集 → Planner 不注入模块白名单约束 → LLM 自由定义模块。这是正确行为——AppCreator 不是 Gateway，没有预编译模块概念。

### Publishing 阶段

不调用 Gateway 重启。只做：
1. 聚合产物
2. 可选 `cargo check`（backend 目录存在时）
3. 写入产物文件

### ModuleLayout / `_shared/lifecycle` import

由 `prototype-tool.js build` 在 Composing 阶段处理。模块原型在 Module/BlockCreation 阶段由对应 skill 构建，Composing 时已就绪。
