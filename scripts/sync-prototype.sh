#!/usr/bin/env bash
# sync-prototype.sh — 通用原型同步脚本
#
# 功能：验证原型 + 同步到 Sources + 清理非原型数据。
# 原型文件仅以 v{N}.html 版本管理，不创建副本。
# 替代 OMP hook ~/.omp/extensions/prototype-jsx-validate.ts 的核心逻辑。
# 在任何环境中均可调用（不依赖 OMP）。
#
# 用法:
#   bash scripts/sync-prototype.sh Pre-Proc/{ns}/Prototypes/Modules/{name}/v{N}.html   # 模块原型
#   bash scripts/sync-prototype.sh Pre-Proc/{ns}/Prototypes/Blocks/{id}/v{N}.html      # 场景原型
#   bash scripts/sync-prototype.sh Pre-Proc/{ns}/Prototypes/Apps/{id}/v{N}.html        # 应用原型（预留）
#   bash scripts/sync-prototype.sh --check-only <same_path>                            # 仅验证，不同步
#
# 返回码:
#   0 = 全部通过 (且已同步)
#   1 = 验证失败

set -euo pipefail

SRC="${1:-}"
CHECK_ONLY=false

if [ "$SRC" = "--check-only" ]; then
    CHECK_ONLY=true
    SRC="${2:-}"
fi

if [ -z "$SRC" ] || [ ! -f "$SRC" ]; then
    echo "用法: $0 [--check-only] <prototype.html>" >&2
    exit 1
fi

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$(cd "$(dirname "$SRC")" && pwd)/$(basename "$SRC")"

# ── 1. 检测类型 ──────────────────────────────────────────────
MODULE_RE='Pre-Proc/([^/]+)/Prototypes/Modules/([^/]+)/(m-)?v[0-9]+\.html$'
SCENE_RE='Pre-Proc/([^/]+)/Prototypes/Blocks/([^/]+)/(s-|b-)?v[0-9]+\.html$'
APP_RE='Pre-Proc/([^/]+)/Prototypes/Apps/([^/]+)/(a-)?v[0-9]+\.html$'

TYPE=""
NAMESPACE=""
NAME=""
DEST=""

if [[ "$SRC" =~ $MODULE_RE ]]; then
    TYPE="module"
    NAMESPACE="${BASH_REMATCH[1]}"
    NAME="${BASH_REMATCH[2]}"
    DEST="$PROJECT_ROOT/Pre-Proc/$NAMESPACE/Sources/Modules/$NAME/prototype.html"
elif [[ "$SRC" =~ $SCENE_RE ]]; then
    TYPE="scene"
    NAMESPACE="${BASH_REMATCH[1]}"
    NAME="${BASH_REMATCH[2]}"
    DEST="$PROJECT_ROOT/Pre-Proc/$NAMESPACE/Sources/Blocks/$NAME/prototype.html"
elif [[ "$SRC" =~ $APP_RE ]]; then
    TYPE="app"
    NAMESPACE="${BASH_REMATCH[1]}"
    NAME="${BASH_REMATCH[2]}"
    DEST="$PROJECT_ROOT/Pre-Proc/$NAMESPACE/Apps/$NAME/prototype.html"
else
    echo "    用法: bash scripts/sync-prototype.sh Pre-Proc/{namespace}/Prototypes/{Modules|Blocks|Apps}/{name}/v{N}.html"
    exit 0
fi

# 提取版本号：从 v{N}.html 中取 N
VER=""
if [[ "$SRC" =~ v([0-9]+)\.html$ ]] || [[ "$SRC" =~ [msa]-v([0-9]+)\.html$ ]]; then
    VER="${BASH_REMATCH[1]}"
fi

echo "[sync-prototype] $TYPE: $NAME"
echo "  源文件: $SRC"

# ── 2. 验证 ──────────────────────────────────────────────────
FAILED=0
SCRIPT_DIR="$PROJECT_ROOT/.agents/skills"

# 通用合规
STANDALONE_BIN="$PROJECT_ROOT/target/debug/ontology-mapping"
if [ -f "$STANDALONE_BIN" ]; then
    STANDALONE_RC=0
    "$STANDALONE_BIN" prototype-check "$SRC" >/dev/null 2>&1 || STANDALONE_RC=$?
    if [ "$STANDALONE_RC" -eq 0 ]; then
        echo "  ✓ prototype-check"
    elif [ "$STANDALONE_RC" -eq 2 ]; then
        echo "  △ prototype-check（warnings only，非阻断）"
    else
        "$STANDALONE_BIN" prototype-check "$SRC" 2>&1
        echo "  ✗ prototype-check"
        FAILED=1
    fi
fi

# JSX 平衡
# JSX 平衡（跳过 App 原型 — 使用 ESM build 而非 inline babel）
JSX_SCRIPT="$SCRIPT_DIR/alioth-design/scripts/jsx-balance.py"
if [ "$TYPE" != "app" ] && [ -f "$JSX_SCRIPT" ]; then
    if python3 "$JSX_SCRIPT" "$SRC" >/dev/null 2>&1; then
        echo "  ✓ jsx-balance.py"
    else
        echo "  ✗ jsx-balance.py"
        FAILED=1
    fi
fi
# Scene 专有契约（alioth-block §9 门禁）
if [ "$TYPE" = "scene" ]; then
    # Scene 原型：Scene 集成完整性（alioth-block §9 门禁）
    SCENE_INTEGRATION_SCRIPT="$SCRIPT_DIR/alioth-block/scripts/audit-block-integration.py"
    if [ -f "$SCENE_INTEGRATION_SCRIPT" ]; then
        if python3 "$SCENE_INTEGRATION_SCRIPT" "$SRC" >/dev/null 2>&1; then
            echo "  ✓ audit-block-integration.py"
        else
            echo "  ✗ audit-block-integration.py (§9.5 — 嵌套壳/重复 createRoot)"
            FAILED=1
        fi
    fi
fi

# Module CSS：变量滥用检测（--card 裸用而无 hsl() 包裹）
if [ "$TYPE" = "module" ]; then
    CSS_SCRIPT="$PROJECT_ROOT/scripts/check/check-css-compliance.mjs"
    THEME_CSS="$PROJECT_ROOT/Pre-Proc/$NAMESPACE/Sources/Modules/$NAME/frontend/src/theme.css.txt"
    if [ -f "$CSS_SCRIPT" ] && [ -f "$THEME_CSS" ]; then
        if bun "$CSS_SCRIPT" audit-css-vars "$THEME_CSS" >/dev/null 2>&1; then
            echo "  ✓ check-css-compliance.mjs audit-css-vars（CSS 变量用法正确）"
        else
            bun "$CSS_SCRIPT" audit-css-vars "$THEME_CSS" 2>&1
            echo "  ✗ check-css-compliance.mjs — CSS 变量裸用 HSL 分量（需用 var(--color-*) 或 hsl() 包裹）"
            FAILED=1
        fi
    fi
fi

# Module 原型：CSS 语法健壮性 + ICONS 图标尺寸兜底（alioth-design §9.6）
if [ "$TYPE" = "module" ]; then
    CSS_ROBUST_SCRIPT="$SCRIPT_DIR/alioth-block/scripts/audit-css-robustness.py"
    if [ -f "$CSS_ROBUST_SCRIPT" ]; then
        if python3 "$CSS_ROBUST_SCRIPT" "$SRC" >/dev/null 2>&1; then
            echo "  ✓ audit-css-robustness.py"
        else
            python3 "$CSS_ROBUST_SCRIPT" "$SRC" 2>&1 | head -20
            echo "  ✗ audit-css-robustness.py (§9.6 — CSS 语法错误/ICONS 缺尺寸兜底)"
            FAILED=1
        fi
    fi
fi

# 字段覆盖率/漂移检测（所有类型）
COVERAGE_SCRIPT="$PROJECT_ROOT/scripts/check/audit-field-coverage.py"
if [ -f "$COVERAGE_SCRIPT" ]; then
    COVERAGE_OUTPUT=$(python3 "$COVERAGE_SCRIPT" "$SRC" 2>&1)
    COVERAGE_JSON=$(echo "$COVERAGE_OUTPUT" | python3 -c "import sys,json; print(next((l for l in sys.stdin if l.strip().startswith('{')), '{}'))" 2>/dev/null || echo '{}')
    COV_STATUS=$(echo "$COVERAGE_JSON" | python3 -c "import sys,json; print(json.load(sys.stdin).get('status','error'))" 2>/dev/null || echo "error")
    if [ "$COV_STATUS" = "fail" ]; then
        COV_PCT=$(echo "$COVERAGE_JSON" | python3 -c "import sys,json; print(json.load(sys.stdin).get('coverage','?'))" 2>/dev/null || echo "?")
        echo "  △ audit-field-coverage.py — 覆盖率 ${COV_PCT}%（< 80%）"
        echo "    （字段覆盖率不足 ⇒ 原型与前端代码有差异，建议检查，当前不阻断）"
    elif [ "$COV_STATUS" = "skipped" ] || [ "$COV_STATUS" = "error" ]; then
        # 类型目录不存在或解析失败 → 非阻断
        echo "  △ audit-field-coverage.py — 跳过（无前端 types 目录）"
    else
        echo "  ✓ audit-field-coverage.py"
    fi
fi

# 原型：视觉验证报告检查（Track 2 HARD GATE）
# 逻辑：报告不存在 → △ 非阻断（Track 1 场景）

if [ "$TYPE" = "module" ] || [ "$TYPE" = "scene" ]; then
    VISUAL_VERIFY_SCRIPT="$PROJECT_ROOT/scripts/check/check-visual-verify.sh"
    if [ "$TYPE" = "module" ]; then
        REPORT_PATH="$PROJECT_ROOT/Pre-Proc/$NAMESPACE/Prototypes/Modules/$NAME/v$VER-report.json"
    else
        REPORT_PATH="$PROJECT_ROOT/Pre-Proc/$NAMESPACE/Prototypes/Blocks/$NAME/v$VER-report.json"
    fi
    if [ -f "$VISUAL_VERIFY_SCRIPT" ]; then
        if [ -f "$REPORT_PATH" ]; then
            # 报告已存在 → 全量检查（6 维度评分 + Token 审计 + 像素叠加对比），阻断
            echo "  → 运行 check-visual-verify（6 维度评分 + Token 审计 + 像素叠加对比）..."
            if bash "$VISUAL_VERIFY_SCRIPT" "$SRC" 2>&1; then
                echo "  ✓ check-visual-verify.sh（全部门禁通过）"
            else
                echo "  ✗ check-visual-verify.sh — 视觉验证未通过"
                echo "    （Track 2 前端校准强制要求交付前通过）"
                FAILED=1
            fi
        else
            # 报告不存在 → △ 非阻断（Track 1 场景）
            # 但运行轻量 Token 审计作为信息
            bash "$VISUAL_VERIFY_SCRIPT" "$SRC" >/dev/null 2>&1 || true
            echo "  △ check-visual-verify.sh — 视觉验证报告未提交"
            echo "    （Track 2 前端校准强制要求交付前通过；Track 1 原型设计可跳过）"
            # Track 1 轻量 Token 信息
            TOKEN_SCRIPT="$PROJECT_ROOT/scripts/check/extract-design-tokens.mjs"
            if [ -f "$TOKEN_SCRIPT" ]; then
                TOKEN_OUTPUT=$(node "$TOKEN_SCRIPT" "$SRC" --json 2>/dev/null || true)
                if [ -n "$TOKEN_OUTPUT" ]; then
                    TOKEN_ISSUES=$(echo "$TOKEN_OUTPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('analysis',{}).get('total_issues',0))" 2>/dev/null || echo "0")
                    if [ "$TOKEN_ISSUES" -gt 0 ]; then
                        echo "  📐 提示: 原型中有 $TOKEN_ISSUES 处硬编码值偏离 Token 体系"
                        echo "    （交付前应修正: node scripts/check/extract-design-tokens.mjs $(basename "$SRC")）"
                    fi
                fi
            fi
        fi
    fi
fi

if [ "$FAILED" -ne 0 ]; then
    echo "[FAIL] 验证未通过，不同步。"
    exit 1
fi

# ── 3. 同步（仅非 --check-only）──────────────────────────────
if [ "$CHECK_ONLY" = "true" ]; then
    echo "[OK] 验证通过（--check-only，未同步）"
    exit 0
fi

# 复制主原型文件
mkdir -p "$(dirname "$DEST")"
cp "$SRC" "$DEST"
echo "  ✓ 已同步: $DEST"

# 复制同目录下的 *.bundle.js（ESM build 模式：原型 html 通过相对 src 引用该 bundle）
SRC_DIR="$(dirname "$SRC")"
SRC_BASE="$(basename "$SRC" .html)"
BUNDLE_SRC="$SRC_DIR/$SRC_BASE.bundle.js"
if [ -f "$BUNDLE_SRC" ]; then
    cp "$BUNDLE_SRC" "$(dirname "$DEST")/"
    echo "  ✓ 已同步 bundle: $(dirname "$DEST")/$SRC_BASE.bundle.js"
fi

# ── 版本对齐检查 + 自动修复 ──
if [ -f "$PROJECT_ROOT/scripts/check/check-version-alignment.sh" ] && [ -n "$NAMESPACE" ]; then
    echo "  → 版本对齐检查..."
    if bash "$PROJECT_ROOT/scripts/check/check-version-alignment.sh" >/dev/null 2>&1; then
        echo "  ✓ 版本对齐"
    else
        echo "  → 自动修复版本..."
        if bun "$PROJECT_ROOT/scripts/check/fix-version-alignment.ts" --ns "$NAMESPACE" >/dev/null 2>&1; then
            echo "  ✓ 版本已对齐"
        else
            echo "  ⚠️  版本修复失败，手动运行: bun scripts/check/fix-version-alignment.ts --ns $NAMESPACE"
        fi
    fi
fi

# ── 完成 ──
echo "[OK] 验证通过"
exit 0
