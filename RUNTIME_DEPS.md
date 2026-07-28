# AppCreator Runtime Dependencies — vendoring inventory

AppCreator is independently open-sourced from
[AliothStudio](https://github.com/CosmicTools9/AliothStudio).
This file tracks which main-repo dependencies have been vendored and
which remain for future activation.

## Vendored (present in this repo)

### Dev scripts

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
| `ontology-gen-bridge` | `Meta/backend/ontology-gen-bridge` | transitive via `app-agent` |

### Frontend packages (`frontend/packages/`)

| Package | Source | Files |
|---|---|---|
| `@alioth/api` | `Framework/frontend/api` | 16 TS files, 140K |
| `@alioth/hooks` | `Framework/frontend/hooks` | 26 TS files, 116K |
| `@alioth/components` | `Framework/frontend/components` | 200 TS/CSS files, 1.2M |
| `@alioth/i18n` | `Framework/frontend/i18n` | 7 TS files, 36K |
| `@alioth/types` | `Framework/frontend/types` | 5 TS files, 20K |
| `@alioth/utils` | `Framework/frontend/utils` | 12 TS files, 48K |
| `@alioth/config` | `Framework/frontend/config` | vite/tsconfig presets |
| `@alioth/composables` | `Framework/frontend/composables` | 23 TS files, 168K |
| `@alioth/ontology` | `Framework/frontend/ontology` | 7 TS files, 52K |

### Build scripts + references

| Asset | Path | Source |
|---|---|---|
| GatewayShell component | `references/gateway-shell.tsx` (823 lines) | `.agents/skills/alioth-design/references/gateway-shell.tsx` |
| Skill definitions | `skill-adapters/*.yaml` (9 files) | `skill-adapters/*.yaml` |
| Icon pool | `references/icon-pool.js` | `.agents/skills/alioth-design/references/icon-pool.js` |
| Base CSS | `references/prototype-base.css` | `.agents/skills/alioth-design/references/prototype-base.css` |
| Shell templates | `references/shells/*.tsx` (5 files) | `.agents/skills/alioth-design/references/shells/` |
| CDN vendor UMDs | `references/vendor/*` (7 files) | `.agents/skills/alioth-design/references/vendor/` |
| Prototype build script | `scripts/prototype-tool.js` (2062 lines) | `scripts/prototype-tool.js` |
| Sync script | `scripts/sync-prototype.sh` (253 lines) | `scripts/sync-prototype.sh` |
| CSS audit | `scripts/check/audit-css-framework.mjs` | `scripts/check/audit-css-framework.mjs` |
| Prototype eval | `scripts/eval/evaluate-prototype-reference.ts` | `scripts/eval/evaluate-prototype-reference.ts` |
| CSS utilities data | `Framework/frontend/components/utilities.json` | `Framework/frontend/components/utilities.json` (symlink) |

### Path compatibility symlinks

| Symlink | Target |
|---|---|
| `.agents/skills/alioth-design/references` | `../../references` |
| `Framework/frontend/components` | `../frontend/packages/components` |

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

AppCreator tracks AliothStudio `main` branch. The upstream commit is recorded in
`backend/vendor/MANIFEST` after each `sync-framework.sh` run.

### Sync flow

```bash
# Full sync: copy sources → pin workspace deps → apply ADAPTATIONS → write MANIFEST
bash scripts/sync-framework.sh /path/to/AliothStudio

# CI gate: validates vendor matches expected sync + adaptations result, plus commit provenance
bash scripts/sync-framework.sh /path/to/AliothStudio --check

# Preview drift without writing
bash scripts/sync-framework.sh /path/to/AliothStudio --dry-run
```

Exit codes: 0 = clean, 1 = source invalid/dirty, 2 = content drift (run sync), 3 = commit mismatch.

### Per-crate adaptations

Each vendored crate may include an `ADAPTATIONS` file with post-sync sed commands.
These encode AppCreator-specific changes that must survive future syncs.

| Crate | Adaptations |
|---|---|
| `app-agent` | `orchestrator.rs`: `pub fn state_name`/`progress_percent`; `lib.rs`: grouped re-export |

### Cargo.toml version pinning

`sync-framework.sh`'s `rewrite_cargo()` automatically converts upstream `workspace = true`
deps to pinned versions for deps not defined in AppCreator's workspace.
See the `sed -i '' -e 's|... = { workspace = true }|... = "version"|'` patterns in the script.

### Sync protocol for future maintainers

1. Run `bash scripts/sync-framework.sh /path/to/AliothStudio .`
2. Verify `--check` passes (exit 0)
3. Verify `cargo check -p app-creator` passes
4. If `cargo check` fails, fix compilation then update ADAPTATIONS for the affected crate

### Vendor drift (2026-07-25)

_This section is historical. Current drift can be checked with `sync-framework.sh --check`._

`diff -rq` between vendored crates and main-repo source reveals:

| Crate | Differing files |
|---|---|
| `app-agent` | 9 |
| `alioth-gen` | 5 |
| `common` | 1 |
| `llm` | 3 |
| `meta-common` | 0 (sync) |
| `meta-model` | 2 |
| `ontology-mapping` | 5 |
| `runtime-contract` | 1 |
| `runtime-engine` | 0 (sync) |
| `ontology-gen-bridge` | 3 |

**Decision (2026-07-25)**: Not all differences are drift — some reflect AppCreator-specific adaptations (e.g., mock paths, removed test dependencies). Before synchronizing, diff each crate manually and decide whether to port changes or keep vendored versions. See the plan in `.planning/appcreator-drift-diagnosis-2026-07-25.md §Ch8`.
