#!/bin/bash
# sync-framework.sh — 从 AliothStudio 主仓库同步 vendored 依赖
#
# AppCreator 独立开源后，将 Framework/backend/* 和 Meta/backend/* 的 crate
# 以及开发脚本 vendor 到本仓库。此脚本用于从主仓库同步最新代码。
#
# 用法:
#   bash scripts/sync-framework.sh <aliothstudio-root>           # 执行同步
#   bash scripts/sync-framework.sh <aliothstudio-root> --check   # 检查 drift（CI 模式）
#   bash scripts/sync-framework.sh <aliothstudio-root> --dry-run # 预览变更
#
# 退出码: 0 = 同步完成 / 无 drift; 1 = 源路径无效; 2 = 检测到 drift

set -u
SOURCE="${1:-}"
MODE="${2:-sync}"

if [ -z "$SOURCE" ] || [ ! -d "$SOURCE" ]; then
    echo "Usage: bash scripts/sync-framework.sh <aliothstudio-root> [--check|--dry-run]"
    exit 1
fi

SOURCE="$(cd "$SOURCE" && pwd)"
DEST="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "=== sync-framework ==="
echo "src:  $SOURCE"
echo "dst:  $DEST"
echo "mode: $MODE"
echo ""

FW="$SOURCE/Framework/backend"
MT="$SOURCE/Meta/backend"
DRIFT=""

# 源映射: AppCreator vendor 名 → 主仓库路径
src_of() {
    case "$1" in
        meta-common) echo "$MT/common" ;;
        common|llm|runtime-contract|runtime-engine) echo "$FW/$1" ;;
        *) echo "$MT/$1" ;;
    esac
}

rewrite_cargo() {
    local cargo="$1"
    # ../../../Framework/backend/xxx → ../xxx
    sed -i '' -e 's|../../../Framework/backend/|../|g' "$cargo"
}

check_drift() {
    local label="$1" src="$2" dst="$3"
    if [ ! -e "$dst" ]; then
        DRIFT="$DRIFT  [NEW] $label\n"
        return 0
    fi
    local out
    out=$(diff -rq "$src" "$dst" 2>&1) || true
    if [ -n "$out" ]; then
        local count
        count=$(echo "$out" | grep -c '^Files\|^Only in')
        DRIFT="$DRIFT  * $label ($count changes)\n"
    fi
}

sync_crate() {
    local crate="$1" dir="$2"
    local src_dir="$dir/src"
    local dst_dir="$DEST/backend/vendor/$crate/src"
    local src_cargo="$dir/Cargo.toml"
    local dst_cargo="$DEST/backend/vendor/$crate/Cargo.toml"

    check_drift "$crate" "$src_dir" "$dst_dir"
    [ "$MODE" != "sync" ] && return

    rm -rf "$dst_dir"
    mkdir -p "$dst_dir"
    cp -R "$src_dir/." "$dst_dir/"

    # Copy + rewrite Cargo.toml
    if [ -f "$src_cargo" ]; then
        cp "$src_cargo" "$dst_cargo"
        rewrite_cargo "$dst_cargo"
    fi

    # tests/
    if [ -d "$dir/tests" ]; then
        mkdir -p "$DEST/backend/vendor/$crate/tests"
        cp -R "$dir/tests/." "$DEST/backend/vendor/$crate/tests/"
    fi

    # Special: app-agent's meta-common path
    if [ "$crate" = "app-agent" ]; then
        sed -i '' 's|^meta-common = { path = "../common" }$|meta-common = { path = "../meta-common" }|' "$dst_cargo"
    fi
}

echo "--- Framework/backend crates ---"
for crate in common llm runtime-contract runtime-engine; do
    sync_crate "$crate" "$FW/$crate"
done

echo "--- Meta/backend crates ---"
for crate in app-agent alioth-gen meta-common meta-model ontology-mapping ontology-gen-bridge; do
    sync_crate "$crate" "$(src_of "$crate")"
done

echo "--- 开发脚本 ---"
for f in "lib/guard-mise-task.sh" "lib/guard-database-tier.sh" "env/decrypt-env.sh"; do
    check_drift "scripts/$f" "$SOURCE/scripts/$f" "$DEST/scripts/$f"
    if [ "$MODE" = "sync" ] && [ -f "$SOURCE/scripts/$f" ]; then
        mkdir -p "$(dirname "$DEST/scripts/$f")"
        cp "$SOURCE/scripts/$f" "$DEST/scripts/$f"
    fi
done

echo ""
echo "=== 结果 ==="
if [ -n "$DRIFT" ]; then
    printf "Drift items:\n$DRIFT"
    if [ "$MODE" = "check" ]; then
        echo "❌ CI FAILED: vendored code differs from upstream." >&2
        exit 2
    fi
fi
[ "$MODE" != "check" ] && echo "✅ Done."
