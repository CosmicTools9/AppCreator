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
# 退出码: 0 = 同步完成 / 无 drift; 1 = 源路径无效; 2 = 检测到 drift;
#          3 = MANIFEST 记录的 commit 与 source HEAD 不匹配

set -u
SOURCE="${1:-}"
MODE="${2:-sync}"

# 标准化 MODE：去除 -- 前缀
case "$MODE" in
    --check) MODE=check ;;
    --dry-run) MODE=dry-run ;;
esac

if [ -z "$SOURCE" ] || [ ! -d "$SOURCE" ]; then
    echo "Usage: bash scripts/sync-framework.sh <aliothstudio-root> [--check|--dry-run]"
    exit 1
fi

SOURCE="$(cd "$SOURCE" && pwd)"
DEST="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="$DEST/backend/vendor/MANIFEST"

echo "=== sync-framework ==="
echo "src:  $SOURCE"
echo "dst:  $DEST"
echo "mode: $MODE"
echo ""

FW="$SOURCE/Framework/backend"
MT="$SOURCE/Meta/backend"
DRIFT=""

# ---------------------------------------------------------------------------
# 在 sync/check 模式中，先检查被同步路径的 dirtiness
# （Framework/backend/, Meta/backend/, scripts/lib/, scripts/env/）
# 不检查 AppCreator/ 或 openspec/ 的改动。
# ---------------------------------------------------------------------------
SYNCED_PATHS="Framework/backend/ Meta/backend/ scripts/lib/ scripts/env/"
DIRTY=$(cd "$SOURCE" && git status --porcelain -- $SYNCED_PATHS 2>/dev/null || true)
if [ -n "$DIRTY" ]; then
    echo "⚠️  Source checkout has uncommitted changes in synced paths:"
    echo "$DIRTY"
    case "$MODE" in
        sync)
            echo "❌ Aborting. Commit or stash changes in synced paths first."
            exit 1
            ;;
        check)
            echo "❌ CI check: source checkout should be clean."
            exit 1
            ;;
        dry-run)
            echo "⚠️  Dry-run: dirty source — drift report may include uncommitted changes."
            ;;
    esac
fi

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

    # workspace dep → version pin（AppCreator workspace 不含上游的集中依赖）
    sed -i '' \
        -e 's|yaml_serde = { workspace = true }|yaml_serde = "0.10"|' \
        -e 's|similar = { workspace = true }|similar = "3.1"|' \
        -e 's|hex = { workspace = true }|hex = "0.4"|' \
        -e 's|md5 = { workspace = true }|md5 = "0.8"|' \
        -e 's|prometheus = { workspace = true }|prometheus = "0.14"|' \
        -e 's|log = { workspace = true }|log = "0.4"|' \
        -e 's|async-trait = { workspace = true }|async-trait = "0.1"|' \
        -e 's|json5 = { workspace = true }|json5 = "1.3"|' \
        -e 's|sha2 = { workspace = true }|sha2 = "0.11"|' \
        -e 's|actix-rt = { workspace = true }|actix-rt = "2"|' \
        -e 's|chrono = { workspace = true }|chrono = { version = "0.4", features = ["serde"] }|' \
        -e 's|rust_decimal = { workspace = true }|rust_decimal = { version = "1.35", features = ["serde"] }|' \
        "$cargo"
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
        count=$(echo "$out" | grep -c '^Files|^Only in')
        DRIFT="$DRIFT  * $label ($count changes)\n"
    fi
}

# check_drift_with_adaptations — 对 upstream source 应用 ADAPTATIONS 后再 diff vendor
# 用于 adapter crate（如 app-agent），确保 --check 不会将已知 adaptation 误报为 drift
check_drift_with_adaptations() {
    local crate="$1" src="$2" dst="$3"
    local adapt_file="$DEST/backend/vendor/$crate/ADAPTATIONS"
    if [ ! -f "$adapt_file" ]; then
        check_drift "$crate" "$src" "$dst"
        return
    fi
    if [ ! -d "$dst" ]; then
        DRIFT="$DRIFT  [NEW] $crate\n"
        return
    fi
    # temp dir: upstream source + adaptations → compare vs vendor
    local tmp_dir
    tmp_dir=$(mktemp -d)
    # 保留 src/ 层，确保 ADAPTATIONS 中的 src/xxx 路径正确
    mkdir -p "$tmp_dir/src"
    cp -R "$src/." "$tmp_dir/src/"
    while IFS= read -r line || [ -n "$line" ]; do
        case "$line" in
            ''|'#'*) continue ;;
        esac
        local file expr
        file=$(echo "$line" | awk '{print $1}')
        expr=$(echo "$line" | awk '{$1=""; print $0}' | sed 's/^ //')
        if [ -f "$tmp_dir/$file" ]; then
            sed -i '' -e "$expr" "$tmp_dir/$file"
        fi
    done < "$adapt_file"
    local out
    out=$(diff -rq "$tmp_dir/src/" "$dst" 2>&1) || true
    rm -rf "$tmp_dir"
    if [ -n "$out" ]; then
        local count
        count=$(echo "$out" | grep -c -E '^Files|^Only in')
        if [ "$count" -gt 0 ]; then
            DRIFT="$DRIFT  * $crate ($count changes)\n"
        fi
    fi
}

# ---------------------------------------------------------------------------
# apply_adaptations — sync 后重新应用 AppCreator 特有修改
# 每个 crate 可包含 ADAPTATIONS 文件，格式：
#   <rel-file> s/<pat>/<rep>/
#   或 sed append: <rel-file> /<addr>/a\\new-line\\<new-line>
# ---------------------------------------------------------------------------
apply_adaptations() {
    local crate="$1"
    local adapt_file="$DEST/backend/vendor/$crate/ADAPTATIONS"
    [ "$MODE" != "sync" ] && return
    [ ! -f "$adapt_file" ] && return
    local crate_dir="$DEST/backend/vendor/$crate"
    echo "  applying adaptations: $crate"
    while IFS= read -r line || [ -n "$line" ]; do
        case "$line" in
            ''|'#'*) continue ;;
        esac
        local file expr
        file=$(echo "$line" | awk '{print $1}')
        expr=$(echo "$line" | awk '{$1=""; print $0}' | sed 's/^ //')
        if [ -f "$crate_dir/$file" ]; then
            sed -i '' -e "$expr" "$crate_dir/$file"
        else
            echo "  ⚠️  ADAPTATIONS: $file not found"
        fi
    done < "$adapt_file"
}

sync_crate() {
    local crate="$1" dir="$2"
    local src_dir="$dir/src"
    local dst_dir="$DEST/backend/vendor/$crate/src"
    local src_cargo="$dir/Cargo.toml"
    local dst_cargo="$DEST/backend/vendor/$crate/Cargo.toml"

    check_drift_with_adaptations "$crate" "$src_dir" "$dst_dir"
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

    apply_adaptations "$crate"
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

# ---------------------------------------------------------------------------
# Sync 模式: MANIFEST 写入
# ---------------------------------------------------------------------------
if [ "$MODE" = "sync" ]; then
    UPSTREAM_COMMIT=$(cd "$SOURCE" && git rev-parse HEAD)
    echo "UPSTREAM_COMMIT=$UPSTREAM_COMMIT" > "$MANIFEST"
    echo ""
    echo "--- MANIFEST ---"
    echo "Wrote $MANIFEST"
    echo "UPSTREAM_COMMIT=$UPSTREAM_COMMIT"
fi

# ---------------------------------------------------------------------------
# Check 模式: MANIFEST commit 校验（来源锚点，非内容证明）
# ---------------------------------------------------------------------------
PROVENANCE_ERROR=0
if [ "$MODE" = "check" ] && [ -f "$MANIFEST" ]; then
    read -r first_line < "$MANIFEST"
    line="${first_line#UPSTREAM_COMMIT=}"
    # 校验格式：恰好 40 位 hex
    if [ "${#line}" -eq 40 ]; then
        case "$line" in
            *[!0-9a-f]*)
                echo "  [SKIP] MANIFEST: UPSTREAM_COMMIT value contains non-hex characters"
                ;;
            *)
                recorded_commit="$line"
                actual_commit=$(cd "$SOURCE" && git rev-parse HEAD)
                if [ "$actual_commit" != "$recorded_commit" ]; then
                    echo "  [PROVENANCE] MANIFEST records $recorded_commit, HEAD is $actual_commit"
                    PROVENANCE_ERROR=1
                else
                    echo "  [PROVENANCE] HEAD matches MANIFEST: $actual_commit"
                fi
                ;;
        esac
    else
        echo "  [SKIP] MANIFEST: UPSTREAM_COMMIT value length != 40"
    fi
elif [ "$MODE" = "check" ] && [ ! -f "$MANIFEST" ]; then
    echo "  [SKIP] MANIFEST: file not found at $MANIFEST"
fi

echo "=== 结果 ==="
if [ -n "$DRIFT" ]; then
    printf "Drift items:\n$DRIFT"
fi

if [ "$MODE" = "check" ]; then
    if [ -n "$DRIFT" ]; then
        echo "❌ CI FAILED: vendored code differs from upstream." >&2
        exit 2
    fi
    if [ "$PROVENANCE_ERROR" -eq 1 ]; then
        echo "❌ PROVENANCE FAILED: MANIFEST commit does not match HEAD." >&2
        exit 3
    fi
    echo "✅ All checks passed."
else
    echo "✅ Done."
fi
