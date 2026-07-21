# AppCreator Runtime Dependencies — vendoring inventory

AppCreator is independently open-sourced from
[AliothStudio](https://github.com/CosmicTools9/AliothStudio).
This file tracks which main-repo dependencies have been vendored and
which remain for future activation.

## Vendored (present in this repo)

| Asset | Repo path | Source (main repo) |
|---|---|---|
| `guard-mise-task.sh` | `scripts/lib/` | `scripts/lib/` |
| `guard-database-tier.sh` | `scripts/lib/` | `scripts/lib/` |
| `decrypt-env.sh` | `scripts/env/` | `scripts/env/` |
| `ensure-schema.sh` | `scripts/db/` | new — delegates to `AliothStudio/scripts/db/reset-db.sh` |

### Cargo workspace

`AppCreator/Cargo.toml` includes a self-contained `[workspace.dependencies]`
and the following vendored crates under `backend/vendor/`:

| Crate | Source | Used by |
|---|---|---|
| `common` | `Framework/backend/common` | `app-creator` backend (auth, errors, telemetry, DB testing) |
| `llm` | `Framework/backend/llm` | `EnvLlmAdapter` → AppAgent LLM facade |
| `runtime-contract` | `Framework/backend/runtime-contract` | transitive via `app-agent` |
| `runtime-engine` | `Framework/backend/runtime-engine` | transitive via `app-agent` / `runtime-contract` |
| `meta-common` | `Meta/backend/common` | transitive via `app-agent` / `alioth-gen` |
| `meta-model` | `Meta/backend/meta-model` | transitive via `app-agent` / `alioth-gen` |
| `alioth-gen` | `Meta/backend/alioth-gen` | `app-agent` IR / Visualizer (CLI codegen **not** supported) |
| `ontology-mapping` | `Meta/backend/ontology-mapping` | transitive via `app-agent` |
| `app-agent` | `Meta/backend/app-agent` | core AppAgent orchestrator, state machine, skills |

## Not yet vendored

These are **compile-time transparent** (code compiles without them) but
become **runtime 404s** if the corresponding code path is exercised.

| Asset | Referenced from | Status | Action needed |
|---|---|---|---|
| `Framework/backend/crud` | `alioth-gen` CLI module/backend codegen templates | Not vendored by design | Vendor if AppCreator needs to run `alioth-gen` CLI to generate new module crates |
| `scripts/gateway/build-docker.sh` | `docker.rs::build_images()` | Not wired into any route yet | Rewrite with `app-creator` image names + Dockerfiles when service-mode lands |
| `Pre-Proc/` + `framework.proto` | `docker.rs::build()` / `generate_compose()` | Not wired into any route yet | Vendor with sync-framework.sh when compose generation is activated |
| `@alioth/{api,components,hooks}` | `frontend/package.json` (removed) | Declared but unused (0 imports in `src/`) | Vendor when frontend consumes shared components; add `pnpm-workspace.yaml` |

## `alioth-gen` CLI limitation

`backend/vendor/alioth-gen` is vendored for its IR and ontology visualizer
runtime used by `app-agent`. The CLI module/backend code-generation path
references `Framework/backend/crud`, which is **not** vendored. AppCreator
cannot generate new module/backend `Cargo.toml` files through `alioth-gen` CLI
until `crud` is also vendored.

## Ignored tests

Two tests in `docker.rs::tests` are annotated with `#[ignore]`:

- `test_compose_references_gateway_image` — expects `Pre-Proc/` in project root, not present in standalone repo

Run ignored tests in a monorepo checkout:
```bash
cargo test -- --ignored
```

## Re-vendoring workflow

```bash
# Dev scripts (always safe to refresh):
bash scripts/sync-framework.sh /path/to/AliothStudio

# @alioth/* packages (when frontend needs them):
mkdir -p frontend/packages
for pkg in api components hooks; do
  cp -r "/path/to/AliothStudio/Framework/frontend/$pkg" "frontend/packages/$pkg"
done
# Then create pnpm-workspace.yaml + update package.json refs
```

## Upstream alignment

AppCreator tracks AliothStudio `main` branch. `sync-framework.sh` is a
best-effort developer tool (not a CI gate). No formal version lock.
