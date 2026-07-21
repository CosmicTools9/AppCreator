#!/usr/bin/env bash
set -euo pipefail

# AppCreator 一键启动脚本
# 同时启动 backend (Rust :49495) + frontend (Vite :49496)

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BACKEND_PID=""
FRONTEND_PID=""

cleanup() {
    echo ""
    echo "🛑 停止所有服务..."
    [ -n "$BACKEND_PID" ] && kill "$BACKEND_PID" 2>/dev/null && wait "$BACKEND_PID" 2>/dev/null
    [ -n "$FRONTEND_PID" ] && kill "$FRONTEND_PID" 2>/dev/null && wait "$FRONTEND_PID" 2>/dev/null
    echo "✅ 已停止"
}
trap cleanup EXIT INT TERM

echo "=============================="
echo "  AppCreator 开发服务器"
echo "=============================="

# 1. 后端
echo ""
echo "[1/2] 启动 backend（Rust :49495）..."
cd "$APP_DIR/backend"
eval "$(mise env)" 2>/dev/null || true
cargo run &
BACKEND_PID=$!

# 等待 backend 就绪
echo "  等待 backend 就绪..."
for i in $(seq 1 30); do
    if curl -sf http://127.0.0.1:49495/health >/dev/null 2>&1; then
        echo "  ✅ backend 就绪 (PID $BACKEND_PID)"
        break
    fi
    if [ "$i" -eq 30 ]; then
        echo "  ⚠️  backend 未在 30s 内就绪，进程可能仍在编译"
    fi
    sleep 1
done

# 2. 前端
echo ""
echo "[2/2] 启动 frontend（Vite :49496）..."
cd "$APP_DIR/frontend"
eval "$(mise env)" 2>/dev/null || true
bun run dev &
FRONTEND_PID=$!

echo ""
echo "=============================="
echo "  ✅ 服务已启动"
echo "  Frontend : http://localhost:49496"
echo "  Backend  : http://localhost:49495/health"
echo "  API      : http://localhost:49495/api/creator/status"
echo "=============================="
echo ""
echo "按 Ctrl+C 停止全部服务"

wait
