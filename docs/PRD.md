# AppCreator — Product Requirements Document / 产品需求文档

---

## Problem Statement / 问题描述

**EN**
Enterprise teams need custom management applications — admin panels, approval workflows, ERP modules, HR systems, data dashboards — but traditional development is slow and expensive. Each app requires a full cycle: requirements gathering, design, frontend/backend development, database schema, authentication integration, deployment. A single mid-size admin panel can take weeks.

Small and mid-size enterprises cannot afford dedicated product teams, and no-code/low-code platforms produce rigid, vendor-locked outputs that don't fit enterprise compliance or deployment requirements.

**CN**
企业团队经常需要定制化的管理应用——管理后台、审批流程、ERP 模块、HR 系统、数据看板——但传统开发周期长、成本高。每个应用都要经历完整的需求分析、设计、前后端开发、数据库建模、认证集成和部署上线，一个中型管理后台动辄数周。

中小型企业无法负担独立的产品团队，而现有的无代码/低代码平台输出的产物灵活性差、供应商锁定严重，难以满足企业合规和部署要求。

---

## Solution / 解决方案

**EN**
AppCreator is an open-source AI-powered enterprise management application generator. Users describe requirements in natural language; the AI drives a multi-stage pipeline — semantic analysis, planning, ontology analysis, composing, verification, publishing — to produce an interactive single-HTML prototype.

Core value propositions:

- **Conversation-driven**: Describe what you need; the AI asks clarifying questions, iterates, and builds.
- **Enterprise-grade service**: AppCreator itself follows SSO authentication, namespace isolation, session ownership enforcement, and PostgreSQL persistence. The generated output is an interactive HTML prototype.
- **Instant preview**: Every session produces a downloadable, interactive HTML prototype.
- **Open-source, self-hosted**: No vendor lock-in. Full control over code and data.
- **B2B-only**: Management applications only.
- **Landing page → one-click start**: First-time visitors see a marketing landing page with feature overview, templates, and quick-start. Single click to create an account (standalone) or SSO redirect.

**CN**
AppCreator 是一个开源、AI 驱动的企业管理应用生成器。用户通过自然语言对话描述需求，AI 驱动多阶段管道——语义分析、规划、本体分析、组合、验证、发布——最终产出可交付的 HTML 原型。

核心价值：

- **对话式驱动**：描述需求，AI 引导细化、迭代、构建。
- **企业级服务架构**：AppCreator 自身采用 SSO 认证、namespace 隔离、会话归属强校验、PostgreSQL 持久化。生成物为交互式 HTML 原型。
- **即时预览**：每次会话产出可下载的交互式 HTML 原型。
- **开源自托管**：无供应商锁定，完全掌控代码和数据。
- **仅限 B2B**：专注企业管理应用。
- **落地页 → 一键开始**：首次访问者看到营销落地页，含功能介绍、模板展示、快速启动入口。一键注册（standalone）或 SSO 跳转。

---

## User Stories / 用户故事

1. As a first-time visitor, I want to see a landing page that explains AppCreator's value proposition and capabilities, so that I understand what the tool does before signing up.

   > 作为首次访问者，我想看到清晰的落地页，了解 AppCreator 的价值主张和能力范围，在注册前理解这个工具能做什么。

2. As a first-time visitor, I want to choose between logging in and trying a demo, so that I can evaluate the tool before committing to use it.

   > 作为首次访问者，我想选择登录直接使用或先浏览示例，以便在决定使用前评估工具。

3. As a business user, I want to describe my management application needs in natural language, so that I don't need programming skills to build an app.

   > 作为业务用户，我想用自然语言描述管理应用需求，无需编程即可创建应用。

4. As a business user, I want the AI to ask clarifying questions when my requirements are vague, so that the generated app matches my actual needs.

   > 作为业务用户，我需要 AI 在需求模糊时主动追问，以确保生成的应用符合实际业务。

5. As a business user, I want to see real-time progress (pipeline stage + percentage) while the app is being generated, so that I know the process is advancing.

   > 作为业务用户，我想在生成过程中看到实时的进度指示（当前阶段 + 百分比），了解管道推进状态。

6. As a business user, I want to interrupt an ongoing generation if I need to refine my requirements, so that I don't waste time on a wrong direction.

   > 作为业务用户，我想在生成过程中随时中断（如果需求需要调整），避免浪费时间在错误方向上。

7. As a business user, I want to preview the generated prototype as an interactive HTML page, so that I can evaluate the result before committing to deployment.

   > 作为业务用户，我想预览生成的原型（交互式 HTML 页面），以便在决定部署前评估效果。

8. As a business user, I want to start from a template (customer management, approval workflow, ERP module, data dashboard), so that I don't have to describe everything from scratch.

   > 作为业务用户，我想从模板（客户管理、审批流程、ERP 模块、数据看板）开始，避免从零描述。

9. As a business user, I want to see my session history in a sidebar, so that I can return to previous conversations and continue where I left off.

   > 作为业务用户，我想在侧边栏看到历史会话列表，方便回到之前的对话继续。

10. As a business user, I want my session state to survive page refresh, so that I don't lose progress if I close the tab accidentally.

    > 作为用户，我希望会话状态在页面刷新后保留，避免意外关闭标签页导致进度丢失。

11. As a developer, I want the generated prototype to follow consistent conventions (data table layout, form patterns, navigation structure), so that the HTML output is navigable and well-structured.

    > 作为开发者，我希望生成的原型遵循一致的约定（数据表格布局、表单模式、导航结构），确保 HTML 产出可导航、结构良好。

12. As an IT administrator, I want each user and their apps isolated in their own namespace, so that data and configurations don't leak across tenants.

    > 作为运维管理员，我希望每个用户/应用拥有独立的 namespace 隔离，防止跨租户数据泄漏。

13. As an IT administrator, I want to deploy AppCreator behind my existing SSO, so that my team uses existing credentials without managing another user database.

    > 作为运维管理员，我想把 AppCreator 部署在已有 SSO 后方，团队成员使用现有凭据登录。

14. As an IT administrator, I want the open-source version to be self-hosted with minimal infrastructure dependencies (PostgreSQL only), so that I can get it running quickly without complex infrastructure. (Requires network access to an LLM API provider.)

    > 作为运维管理员，我希望开源版本基础设施依赖极简（仅 PostgreSQL），快速部署无须复杂基础设施。需要 LLM API 的网络访问。

15. As a developer evaluating the tool, I want to try the full generation pipeline without paying, so that I can assess the quality before purchasing deployable source code.

    > 作为评估工具的开发者，我想免费试用完整的生成流程，在购买可部署源码前评估质量。

16. As a user, I want the chat session pipeline to auto-advance through all AppAgent stages (SemanticAnalysis → Planning → OntologyAnalysis → Composing → Verifying → Publishing) without requiring manual prompting at each step, so that the experience is fluid.

    > 作为用户，我希望聊天管道自动推进所有 AppAgent 阶段（语义分析→规划→本体分析→组合→验证→发布），无需手动触发每一步。

17. As a user, I want my login session to persist via refresh token, so that I don't have to re-authenticate during a long working session.

    > 作为用户，我希望登录状态通过 refresh token 延续，避免长时间工作中断后需要重新认证。

18. As a user, I want to see my generated apps in a list, so that I can browse, preview, or delete them.

    > 作为用户，我想以列表形式查看已生成的应用，以便浏览、预览或删除。

19. As a user, I want to download the generated prototype HTML, so that I can share it or host it independently.
    > 作为用户，我想下载生成的原型 HTML，以便分享给团队或独立部署。

---

## Implementation Decisions / 实现决策

### Architecture / 架构

**EN**

- **Self-contained binary**: Single Rust binary serves both REST API and static frontend. No reverse proxy required.
- **Rust + PostgreSQL backend**: `actix-web` HTTP framework, `sqlx` async DB access, ES256 JWT via `jsonwebtoken`.
- **React + TypeScript + Vite frontend**: Jotai v2 state management, REST client with JWT bearer header.
- **Dual auth mode**: SSO mode (validates external ES256 JWT) and Standalone mode (self-signed ES256 JWT, passwordless login, refresh token rotation).
- **DB schema boundary**: Chat sessions in `isahl_meta` (shared); users, session ownership, refresh tokens in `app_creator` (self-owned). No Meta HTTP calls.
- **Startup self-healing**: Idempotent `ALTER TYPE ... ADD VALUE IF NOT EXISTS` and `CREATE TABLE IF NOT EXISTS` at startup.
- **Atomic generation claim**: `UPDATE ... WHERE status NOT IN ('generating','completed','abandoned')` prevents concurrent generation on same session.
- **Session ownership**: `check_session_owner()` on every handler; session list filtered by JOIN against `session_owners` table.
- **AppAgent pipeline**: 6-stage state machine (`app_creating` → `semantic_analysis` → `planning` → `ontology_analysis` → `composing` → `verifying` → `publishing` → `completed`). Persisted as `ConversationContext` JSON.
- **Prototype serving**: `GET /sessions/{id}/prototype` resolves namespace+app_name from context, reads `prototype.html` from `Pre-Proc/` paths.
- **FS app repository**: Apps stored at `Pre-Proc/{namespace}/Apps/{code}/`, CRUD directly on filesystem.

**CN**

- **单体二进制**：单一 Rust 二进制同时提供 REST API 和静态前端，基础部署无需反向代理。
- **Rust + PostgreSQL 后端**：`actix-web` HTTP 框架，`sqlx` 异步 DB，ES256 JWT。
- **React + TypeScript + Vite 前端**：Jotai v2 状态管理，REST 客户端自动注入 JWT header。
- **双认证模式**：SSO 模式验证外部 ES256 JWT；Standalone 模式自签 ES256 JWT、免密码登录、refresh token 轮换。
- **DB schema 边界**：聊天会话在 `isahl_meta`（共享）；用户/会话归属/refresh token 在 `app_creator`（自有）。零 HTTP 调用 Meta。
- **启动自愈**：幂等创建 `chat_session_status` 枚举值和 `app_creator` 表。
- **原子生成锁**：`UPDATE ... WHERE status` 条件防止同一会话并发生成。
- **会话归属校验**：每个 handler 执行归属检查，列表查询通过 JOIN 过滤。
- **AppAgent 管道**：6 阶段状态机，持久化为 `ConversationContext` JSON。
- **原型提供**：从 `ConversationContext` 解析坐标，读取 `Pre-Proc/` 下 HTML 文件返回。
- **文件系统应用仓库**：`Pre-Proc/{namespace}/Apps/{code}/` 下 CRUD 操作文件系统。

### Auth & Security / 认证与安全

**EN**

- **ES256 mandatory**: Both modes use ES256 (ECDSA P-256). HS256 rejected. SSO mode verifies with external public key; standalone mode signs with local ES256 key.
- **Production key guard**: `ENV=production` without `APP_CREATOR_JWT_PRIVATE_KEY` panics at startup (embedded dev key refused).
- **Passwordless standalone**: Username → normalized lookup → create-or-return. No password, no Argon2. Intranet/dev/self-hosted only.
- **Rate limiting**: Per-IP rate limit on login endpoints via `common::RateLimitMiddleware`.

**CN**

- **ES256 强制**：双模式均使用 ES256（ECDSA P-256）。SSO 模式用外部公钥验证，Standalone 模式用本地私钥签名。
- **生产环境密钥保护**：`ENV=production` 且缺失 `APP_CREATOR_JWT_PRIVATE_KEY` 时启动 panic，拒绝使用内嵌开发密钥。
- **免密码登录**：用户名 → 归一化查询 → 创建或返回已有用户。无密码、无 Argon2。仅适用于内网/开发/自托管。
- **限流**：登录端点 per-IP 速率限制。

### Chat Session API / 聊天会话 API

**EN**
All under `/api/creator/sessions`:

| Method | Path                               | Description                               |
| ------ | ---------------------------------- | ----------------------------------------- |
| POST   | `/sessions`                        | Create session + init ConversationContext |
| GET    | `/sessions/{id}`                   | Get session with all messages             |
| POST   | `/sessions/{id}/messages`          | Append user message                       |
| POST   | `/sessions/{id}/generate-response` | Run one AppAgent step                     |
| POST   | `/sessions/{id}/interrupt`         | Request graceful interruption             |
| POST   | `/sessions/{id}/resume`            | Resume from interruption                  |
| POST   | `/sessions/{id}/reset-state`       | Reset state machine                       |
| GET    | `/sessions/{id}/prototype`         | Serve generated prototype HTML            |
| GET    | `/sessions/{id}/progress`          | Read pipeline progress snapshot           |

`generate-response` returns `StepResponse`:

```
state_before, state_after: string
is_terminal: boolean
progress_percent: 0–100
message: string
error: string | null
```

Auto-advance: frontend polls `generate-response` up to 30 iterations until `is_terminal: true`. Then `GET /sessions/{id}` as single truth source.

**CN**
全部位于 `/api/creator/sessions` 下。会话 API 覆盖创建、读取、消息追加、AppAgent 单步执行、中断/恢复/重置状态机、原型提供、进度查询。

`generate-response` 返回 `StepResponse`（状态前后、是否终端、进度百分比、消息内容、错误信息）。

前端循环轮询 `generate-response`（最多 30 步）直至 `is_terminal: true`，结束后通过 `GET /sessions/{id}` 同步作为唯一真相源。

### App Repository API / 应用仓库 API

**EN**
Under `/api/creator/apps`:

| Method | Path           | Description                   |
| ------ | -------------- | ----------------------------- |
| GET    | `/apps`        | List apps in user's namespace |
| GET    | `/apps/{code}` | Read app's `app.json`         |
| DELETE | `/apps/{code}` | Delete app directory from FS  |

**CN**
位于 `/api/creator/apps` 下，提供当前用户 namespace 下的应用列表、查看和删除操作，直接操作文件系统。

### Design / 设计

**EN**

- **Enterprise dark theme**: Deep neutrals (`#0A0A0F` → `#1A1A2E`), sapphire accent (`#2563EB`), system font + PingFang SC for Chinese.
- **Layout**: Fixed sidebar (session list + user info) + main chat area + always-visible input bar.
- **Empty state**: Four template cards (客户管理后台 / 审批流程 / ERP 模块 / 数据看板).
- **Progress card**: State name + percentage bar + stop button during generation.
- **Brand**: "对话创建企业应用 · 从需求到部署" / "Enterprise apps from conversation".

**CN**

- **企业暗色主题**：深中性色背景（`#0A0A0F` → `#1A1A2E`），蓝宝石色强调（`#2563EB`），系统字体 + PingFang SC。
- **布局**：固定侧边栏 + 主聊天区 + 常驻输入栏。
- **空状态**：四个模板卡片。
- **进度卡**：生成中的状态名 + 百分比条 + 停止按钮。
- **品牌**："对话创建企业应用 · 从需求到部署"。

### Data Model / 数据模型

**EN**

- **Users** (`app_creator.users`): `id`, `username`, `username_norm`, `namespace`, `created_at`
- **Refresh tokens** (`app_creator.refresh_tokens`): `id`, `user_id`, `token_hash` (SHA256), `expires_at`, `revoked_at`, `created_at`
- **Session owners** (`app_creator.session_owners`): `session_id`, `user_id`, `created_at`
- **Chat sessions** (`isahl_meta.meta_chat_sessions`): `id`, `title`, `app_instance_id`, `namespace`, `status` (enum), timestamps
- **Chat messages** (`isahl_meta.meta_chat_messages`): `id`, `session_id`, `role`, `content`, `context` (JSON — `ConversationContext`), `created_at`

**CN**

- **用户表**（`app_creator.users`）：用户名、归一化用户名、namespace、创建时间
- **Refresh token 表**（`app_creator.refresh_tokens`）：用户 ID、SHA256 哈希、过期时间、撤销时间
- **会话归属表**（`app_creator.session_owners`）：会话 ID → 用户 ID 映射
- **聊天会话**（`isahl_meta.meta_chat_sessions`）：标题、namespace、状态枚举、时间戳
- **聊天消息**（`isahl_meta.meta_chat_messages`）：角色、内容、`ConversationContext` JSON 上下文

---

## Testing Decisions / 测试决策

**EN**

**Principles**: Test external behavior through the API contract, not internal implementation. Repository and DB logic tested against real PostgreSQL (aliothstudio_test tier) — no mocking at this layer.

**Backend test infrastructure**:

- `#[tokio::test]` + `common::testing::connect_test_db()` against `aliothstudio_test` database.
- `#[sqlx::test]` prohibited (per project convention `TEST_INFRASTRUCTURE.md`).
- DDL applied via `ensure_chat_session_status_values()` and `ensure_app_creator_tables()` at test startup (idempotent, same as production).

**Current coverage**:

| Layer             | What is tested                                                    | Status                                                            |
| ----------------- | ----------------------------------------------------------------- | ----------------------------------------------------------------- |
| DB/repository     | Session CRUD, message append, generation claim, session ownership | ✅ integrated tests passing                                       |
| AppAgent pipeline | Single-step pipeline with `MockLlmService`                        | ✅ passing                                                        |
| HTTP handlers     | generate-response and other endpoints                             | ❌ not covered — needs mockable `AgentLlmService` trait injection |
| Frontend          | Components, stores, API client                                    | ❌ not covered — vitest in dependencies, no test files            |
| E2E (real LLM)    | Full pipeline end to end                                          | ❌ manual only                                                    |

**Prior art**: Repository-level integration tests in `chat.rs::tests` provide the testing pattern — real DB connection, async test functions, `sqlx::query_as` with `FromRow`. New tests should follow the same pattern.

**CN**

**原则**：通过 API 契约测试外部行为，不测试内部实现。仓库层和 DB 逻辑对真实 PostgreSQL（aliothstudio_test 库）测试，不做 mock。

**后端测试基础设施**：

- `#[tokio::test]` + `common::testing::connect_test_db()`，目标 `aliothstudio_test`。
- `#[sqlx::test]` 禁止使用。
- DDL 在测试启动时幂等应用（同生产环境）。

**当前覆盖**：

| 层            | 测试内容                              | 状态                                         |
| ------------- | ------------------------------------- | -------------------------------------------- |
| DB/仓库层     | 会话 CRUD、消息追加、生成锁、会话归属 | ✅ 集成测试通过                              |
| AppAgent 管道 | 与 `MockLlmService` 的单步管道        | ✅ 通过                                      |
| HTTP handler  | generate-response 等端点              | ❌ 未覆盖 — 需可注入 `AgentLlmService` trait |
| 前端          | 组件、store、API client               | ❌ 未覆盖 — vitest 在 deps 但未接入          |
| 真实 LLM E2E  | 完整管道                              | ❌ 仅手动                                    |

**已有模式参考**：`chat.rs::tests` 中的仓库层集成测试提供了可复用的模式（真实 DB、异步 test 函数、`sqlx::query_as` + `FromRow`），新测试应沿用。

---

## Out of Scope / 不在范围内

**EN**

| Item                            | Rationale                                                                                                                |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| Consumer-facing apps            | B2B enterprise management only (brand-spec)                                                                              |
| Mobile-native UI                | Desktop-first management panels                                                                                          |
| Docker/container deployment     | Deliberately excluded from open-source distribution                                                                      |
| Multi-tenant SaaS hosting       | Self-hosted standalone only                                                                                              |
| Marketplace / template store    | No community sharing; 4 built-in templates                                                                               |
| Offline mode                    | Requires LLM API access                                                                                                  |
| Real-time collaboration         | Single-user sessions                                                                                                     |
| Deployable source code purchase | Design intent per brand-spec; prototype generation is current scope, source code package delivery is not yet implemented |

**CN**

| 项               | 原因                                                              |
| ---------------- | ----------------------------------------------------------------- |
| 面向消费者的应用 | 仅限 B2B 企业管理                                                 |
| 移动端原生界面   | 桌面端管理后台优先                                                |
| Docker 容器部署  | 开源发行版已剥离                                                  |
| 多租户 SaaS 托管 | 仅自托管单机部署                                                  |
| 模板市场         | 仅有 4 个内置模板，无社区分享                                     |
| 离线模式         | 依赖 LLM API 网络访问                                             |
| 实时协作         | 单用户会话                                                        |
| 可部署源码购买   | brand-spec 的设计意图；当前范围到原型生成为止，源码包交付尚未实现 |

---

## Further Notes / 补充说明

### Known Limitations / 已知限制

**EN**

- **Schema ownership**: Chat tables in `isahl_meta` are owned by Meta. Schema changes in Meta drift AppCreator. Long-term fix: AppCreator-owned migration directory.
- **Deliverable scope**: Current output is a single-file HTML prototype. Full Rust+React+PostgreSQL code generation requires the `crud` crate not yet vendored.
- **alioth-gen CLI codegen**: Cannot generate new module backend crates — only IR/ontology visualizer runtime available.
- **Vendor drift**: 10 vendored crates, no formal version lock or sync mechanism. Upstream changes require manual porting.
- **Frontend i18n**: Locale key sets exist but unused; all UI hardcoded in Chinese.
- **HTTP test coverage**: Generate-response tested at repository/agent level only. Handler-level tests need mockable `LlmService` injection.

**CN**

- **Schema 归属**：聊天表归 AliothStudio Meta 所有，Meta schema 变更会漂移 AppCreator。长期方案：AppCreator 自有 migration 目录。
- **交付范围**：当前产出为单文件 HTML 原型。完整 Rust+React+PostgreSQL 代码生成需要 `crud` crate vendor。
- **alioth-gen CLI 代码生成**：不能生成新模块后端 crate，仅 IR/本体可视化运行时可用。
- **Vendor 漂移**：10 个 vendor crate 无正式版本锁定或同步机制，上游变更需手动移植。
- **前端国际化**：locale 键集存在但未接入，全部 UI 硬编码为中文。
- **HTTP 测试覆盖**：generate-response 仅仓库层/agent 层测试，handler 层需可注入的 `LlmService` mock。

### No HTTP Coupling to Meta / 与 Meta 零 HTTP 耦合

**EN**: AppCreator accesses `isahl_meta` tables via direct SQL only — no HTTP calls to Meta API endpoints. Enforced at dependency level (`Cargo.toml` references no Meta client libraries).

**CN**: AppCreator 仅通过直接 SQL 访问 `isahl_meta` 表，不调用 Meta 的 HTTP API。依赖层面强制隔离（`Cargo.toml` 不引用任何 Meta 客户端库）。

### Environment Variables / 环境变量

**EN**

| Variable                      | Required | Purpose                                                 |
| ----------------------------- | -------- | ------------------------------------------------------- |
| `DATABASE_URL`                | Yes      | PostgreSQL connection                                   |
| `LLM_PROVIDER`                | Yes      | deepseek, kimi, minimax, or OpenAI                      |
| `LLM_API_KEY`                 | Yes      | Provider API key                                        |
| `LLM_MODEL`                   | No       | Model name                                              |
| `SSO_JWT_PUBLIC_KEY`          | No       | Enables SSO mode                                        |
| `APP_CREATOR_JWT_PRIVATE_KEY` | No       | Standalone signing key (required when `ENV=production`) |
| `SERVER_ADDR`                 | No       | Listen address (default `127.0.0.1:49495`)              |
| `ENV`                         | No       | `production` enables key guard                          |

**CN**

| 变量                          | 必需 | 用途                                           |
| ----------------------------- | ---- | ---------------------------------------------- |
| `DATABASE_URL`                | 是   | PostgreSQL 连接串                              |
| `LLM_PROVIDER`                | 是   | LLM 供应商                                     |
| `LLM_API_KEY`                 | 是   | 供应商 API key                                 |
| `LLM_MODEL`                   | 否   | 模型名                                         |
| `SSO_JWT_PUBLIC_KEY`          | 否   | 开启 SSO 模式                                  |
| `APP_CREATOR_JWT_PRIVATE_KEY` | 否   | Standalone 签名密钥（`ENV=production` 时必需） |
| `SERVER_ADDR`                 | 否   | 监听地址（默认 `127.0.0.1:49495`）             |
| `ENV`                         | 否   | `production` 触发密钥保护                      |

### Runtime Dependencies / 运行时依赖

**EN**

- Infrastructure (self-hosted): PostgreSQL 14+ (only infrastructure dependency)
- Service (external): LLM API endpoint with network access
- Build-time: Disk space for vendored crate compilation (~84K LOC, one-time Rust compile cost)

**CN**

- 基础设施（自托管）：PostgreSQL 14+（唯一基础设施依赖）
- 服务（外部）：LLM API 端点（需要网络访问）
- 构建时：vendor crate 编译磁盘空间（约 84K LOC，一次性的 Rust 编译成本）
