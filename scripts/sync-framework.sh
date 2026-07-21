#!/bin/bash
# sync-framework.sh — 从 AliothStudio 主仓库同步 vendored Framework 文件
#
# AppCreator 独立开源后，部分基础设施文件（开发脚本、common crate、@alioth 包）
# 从主仓库 vendor 到本目录。此脚本在主仓库有更新时供开发者手动同步。
#
# 用法:
#   bash scripts/sync-framework.sh <aliothstudio-root>
#
# 示例:
#   bash scripts/sync-framework.sh ../AliothStudio
#
# 依赖:
#   - 需要阿里猿Studio 主仓库已克隆到本地
#   - 仅同步已 vendor 的文件，不会引入新依赖
#
# 退出码: 0 = 同步成功; 1 = 源路径无效

set -euo pipefail

SOURCE="${1:-}"
if [[ -z "$SOURCE" || ! -d "$SOURCE" ]]; then
    echo "用法: bash scripts/sync-framework.sh <aliothstudio-root>"
    echo "错误: 无效的源路径: ${SOURCE:-<空>}"
    exit 1
fi

SOURCE="$(cd "$SOURCE" && pwd)"
DEST="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "=== sync-framework ==="
echo "源:   $SOURCE"
echo "目标: $DEST"

# ── 开发脚本 ──
echo "--- 同步 scripts/ 开发脚本 ---"
cp "$SOURCE/scripts/lib/guard-mise-task.sh"     "$DEST/scripts/lib/guard-mise-task.sh"
cp "$SOURCE/scripts/lib/guard-database-tier.sh"  "$DEST/scripts/lib/guard-database-tier.sh"
cp "$SOURCE/scripts/env/decrypt-env.sh"           "$DEST/scripts/env/decrypt-env.sh"
echo "  ✓ scripts/"

# ── common crate（暂未 vendor，占位）──
# 当 production service-mode 接入时需要同步:
#   cp -r "$SOURCE/Framework/backend/common" "$DEST/backend/vendor/common"
# 并更新 backend/Cargo.toml 添加:
#   common = { path = "vendor/common" }
echo "  ⚠ common crate 尚未 vendor — 接入 service-mode 时执行:"
echo "    cp -r \"\$SOURCE/Framework/backend/common\" \"\$DEST/backend/vendor/common\""

# ── @alioth/* packages（暂未 vendor，占位）──
echo "  ⚠ @alioth/* packages 尚未 vendor — 前端接入时执行:"
echo "    mkdir -p $DEST/frontend/packages"
echo "    for pkg in api components hooks; do"
echo "      cp -r \"\$SOURCE/Framework/frontend/\$pkg\" \"\$DEST/frontend/packages/\$pkg\""
echo "    done"

echo "=== 同步完成 ==="
