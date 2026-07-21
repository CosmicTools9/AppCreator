# AppAgent Integration in AppCreator

AppCreator vendors the AliothStudio `app-agent` crate and exposes a chat-session API
that drives the AppAgent orchestrator. This document describes how to set up the
backend so that AppCreator can create Alioth apps through LLM dialogue.

## Architecture

```text
┌─────────────────┐      HTTP /api/creator/sessions      ┌──────────────────┐
│  AppCreator UI  │  ───────────────────────────────────►  │  AppCreator API  │
└─────────────────┘                                      └────────┬─────────┘
                                                                  │
                                                                  ▼
                                            ┌──────────────────────────────────┐
                                            │ chat.rs                            │
                                            │  • create / get / list sessions    │
                                            │  • add user message                │
                                            │  • generate-response (AppAgent)    │
                                            │  • interrupt / resume / reset      │
                                            └────────┬─────────────────────────┘
                                                     │
                                                     ▼
                                            ┌──────────────────────────────────┐
                                            │ app-agent orchestrator             │
                                            │  • SemanticAnalysis → Planning → │
                                            │    OntologyAnalysis → Composing →  │
                                            │    Verifying → Publishing          │
                                            └────────┬─────────────────────────┘
                                                     │
                                                     ▼
                                            ┌──────────────────────────────────┐
                                            │ isahl_meta / isahl (PostgreSQL)  │
                                            └──────────────────────────────────┘
```

## Prerequisites

- PostgreSQL running locally with a valid user role.
- An AliothStudio checkout available on disk (for `ensure-schema.sh`).
- A valid LLM API key.

## Database setup

AppCreator reuses the same `isahl_meta` and `isahl` schemas that AliothStudio Meta
uses. Per project rules, AppCreator does **not** maintain its own DDL copy; schema
initialization delegates to AliothStudio's canonical reset script.

### Allowed database tiers

| Tier | Allowed? | Notes |
|---|---|---|
| `aliothstudio` | ❌ | Production |
| `aliothstudio_pre` | ❌ | Pre-release |
| `aliothstudio_dev` | ✅ | Development |
| `aliothstudio_test` | ✅ | Automated tests |
| a dedicated DB | ✅ | Recommended for isolated deployments |

### Initialize schema

```bash
cd AppCreator
ALIOTH_STUDIO_ROOT=/path/to/AliothStudio \
DATABASE_URL=postgres://postgres@localhost:5432/aliothstudio_dev \
  bash scripts/db/ensure-schema.sh
```

To drop and recreate the database first:

```bash
RESET=true bash scripts/db/ensure-schema.sh
```

The script delegates to `AliothStudio/scripts/db/reset-db.sh`, which applies the
latest `Backup/latest/schema.sql` to the target database.

### Running tests

```bash
cd AppCreator
DATABASE_URL=postgres://postgres@localhost:5432/aliothstudio_test \
  cargo test -p app-creator --test chat_lifecycle
```

Tests must use `#[tokio::test]` + `common::testing::connect_test_db()`; `#[sqlx::test]`
is prohibited.

## Environment variables

Copy `backend/.env.example` to `backend/.env` and fill in at least:

```bash
DATABASE_URL=postgres://postgres@localhost:5432/aliothstudio_dev
LLM_PROVIDER=deepseek          # or kimi / minimax
LLM_API_KEY=your-api-key-here
LLM_MODEL=deepseek-v4-pro
```

`backend/.mise.toml` provides non-sensitive defaults for `LLM_PROVIDER`,
`LLM_MODEL`, `LLM_TIMEOUT_SECONDS`, etc.; the API key should be provided via
`.env` or the encrypted env source configured in `.mise.toml`.

## Chat session API

All endpoints are under `/api/creator/sessions` and require a valid JWT unless the
server is started without `SSO_JWT_PUBLIC_KEY` configured (not recommended for
production).

| Method | Path | Description |
|---|---|---|
| POST | `/api/creator/sessions` | Create a session |
| GET | `/api/creator/sessions/{id}` | Get session + messages |
| POST | `/api/creator/sessions/{id}/messages` | Append a user message |
| POST | `/api/creator/sessions/{id}/generate-response` | Run one AppAgent step and append assistant response |
| POST | `/api/creator/sessions/{id}/interrupt` | Request interruption |
| POST | `/api/creator/sessions/{id}/resume` | Resume from interruption |
| POST | `/api/creator/sessions/{id}/reset-state` | Reset state machine to a target state |

## Known limitations

- **Schema ownership**: AppCreator currently reads/writes `isahl_meta.meta_chat_sessions`
  and `isahl_meta.meta_chat_messages`. These tables are owned by AliothStudio Meta.
  Column changes in Meta will drift AppCreator's code. The recommended long-term fix is
  for AppCreator to own its own `migrations/` directory and its own chat-session tables.
- **alioth-gen CLI codegen**: `backend/vendor/alioth-gen` is vendored for the IR and
  ontology visualizer runtime used by `app-agent`. The CLI module/backend code-generation
  path requires `Framework/backend/crud`, which is not vendored. AppCreator cannot generate
  new module/backend crates via the `alioth-gen` CLI.
- **HTTP-level test coverage**: `generate-response` is tested at the repository and
  AppAgent level but not yet through the actix-web handler stack. A mockable `LlmService`
  backend-injection point is needed for full HTTP integration tests.

## Next steps

1. Design AppCreator-owned DB schema boundary and add migrations.
2. Implement a create-app endpoint that initializes a `ConversationContext` and starts
   AppAgent from a project/template selection.
3. Update the frontend to call the chat-session endpoints and render progress cards.
4. Add end-to-end validation with a real LLM backend.
