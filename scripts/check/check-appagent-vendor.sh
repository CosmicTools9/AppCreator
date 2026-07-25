#!/usr/bin/env bash
# check-appagent-vendor.sh — verify AppAgent vendor completeness.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
errors=0

# Expected vendored crates
EXPECTED=(
  "app-agent"
  "common"
  "llm"
  "runtime-contract"
  "runtime-engine"
  "meta-common"
  "meta-model"
  "alioth-gen"
  "ontology-mapping"
  "ontology-gen-bridge"
)

echo "=== Checking vendored crates ==="
for crate in "${EXPECTED[@]}"; do
  dir="$ROOT/backend/vendor/$crate"
  if [ -d "$dir" ]; then
    if [ -f "$dir/Cargo.toml" ]; then
      echo "  [OK] $crate (Cargo.toml present)"
    else
      echo "  [FAIL] $crate (missing Cargo.toml)"
      errors=$((errors + 1))
    fi
  else
    echo "  [FAIL] $crate (directory missing)"
    errors=$((errors + 1))
  fi
done

echo ""
echo "=== Checking app-creator crate depends on vendored crates ==="
BACKEND_CARGO="$ROOT/backend/Cargo.toml"
for dep in app-agent common llm; do
  if grep -q "vendor/$dep" "$BACKEND_CARGO" 2>/dev/null; then
    echo "  [OK] backend depends on $dep"
  else
    echo "  [FAIL] backend missing dependency on $dep"
    errors=$((errors + 1))
  fi
done

echo ""
echo "=== Checking workspace glob includes all vendored crates ==="
WORKSPACE_CARGO="$ROOT/Cargo.toml"
if grep -q 'vendor/\*' "$WORKSPACE_CARGO" 2>/dev/null; then
  echo "  [OK] workspace uses members glob: backend/vendor/*"
  for dep in runtime-contract runtime-engine meta-common meta-model alioth-gen ontology-mapping; do
    dir="$ROOT/backend/vendor/$dep"
    if [ -d "$dir" ] && [ -f "$dir/Cargo.toml" ]; then
      echo "    [OK] $dep resolved via glob"
    else
      echo "    [FAIL] $dep not found under vendor/"
      errors=$((errors + 1))
    fi
  done
else
  echo "  [FAIL] workspace missing vendor glob"
  errors=$((errors + 1))
fi

echo ""
echo "=== Checking auth middleware exists ==="
MIDDLEWARE="$ROOT/backend/src/middleware.rs"
if [ -f "$MIDDLEWARE" ] && grep -q "SsoAuthMiddleware" "$MIDDLEWARE" 2>/dev/null; then
  echo "  [OK] middleware.rs with SsoAuthMiddleware"
else
  echo "  [FAIL] middleware.rs missing or incomplete"
  errors=$((errors + 1))
fi

echo ""
echo "=== Checking ensure-schema.sh exists and is valid ==="
SCHEMA_SCRIPT="$ROOT/scripts/db/ensure-schema.sh"
if [ -f "$SCHEMA_SCRIPT" ]; then
  if grep -q "ALIOTH_STUDIO_ROOT" "$SCHEMA_SCRIPT" 2>/dev/null; then
    echo "  [OK] ensure-schema.sh uses ALIOTH_STUDIO_ROOT"
  else
    echo "  [FAIL] ensure-schema.sh missing ALIOTH_STUDIO_ROOT reference"
    errors=$((errors + 1))
  fi
else
  echo "  [FAIL] ensure-schema.sh missing"
  errors=$((errors + 1))
fi

echo ""
if [ "$errors" -eq 0 ]; then
  echo "Result: PASS (0 errors)"
else
  echo "Result: $errors FAILURE(s)"
fi
exit "$errors"
