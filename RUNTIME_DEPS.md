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

### Cargo workspace

`AppCreator/Cargo.toml` provides a self-contained `[workspace.dependencies]`
with the 6 shared crates that `backend/Cargo.toml` references via
`workspace = true`. All other deps are hardcoded versions.

### Frontend packages

`package.json` declares only OSS npm dependencies (`react`, `vite`, etc).
No `@alioth/*` packages — they were declared but unused (0 imports in `src/`).

## Not yet vendored

These are **compile-time transparent** (code compiles without them) but
become **runtime 404s** if the corresponding code path is exercised.

| Asset | Referenced from | Status | Action needed |
|---|---|---|---|
| `scripts/gateway/build-docker.sh` | `docker.rs::build_images()` | Not wired into any route yet | Rewrite with `app-creator` image names + Dockerfiles when service-mode lands |
| `Pre-Proc/` + `framework.proto` | `docker.rs::build()` / `generate_compose()` | Not wired into any route yet | Vendor with sync-framework.sh when compose generation is activated |
| `common` crate | `backend/Cargo.toml` (removed) | Declared but unused (0 `use common::*` in `src/`) | Add via sync-framework.sh when DB pool / JWT middleware / error mapping is wired |
| `@alioth/{api,components,hooks}` | `frontend/package.json` (removed) | Declared but unused (0 imports in `src/`) | Vendor when frontend consumes shared components; add `pnpm-workspace.yaml` |

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

# common crate (when service-mode needs it):
cp -r /path/to/AliothStudio/Framework/backend/common backend/vendor/common
# Then add to backend/Cargo.toml: common = { path = "vendor/common" }

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
