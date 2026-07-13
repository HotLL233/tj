#!/usr/bin/env bash
# 本地化LIMS 后端 Linux / Docker 构建脚本（v0.4.22）
#
# 用法（在 backend/ 目录下执行）：
#   bash build_linux.sh
#
# 前置依赖：
#   - Rust 1.88+（以及 C 编译器，供 rusqlite bundled SQLite 使用，如 build-essential / gcc）
#   - Node 18+ 与 npm
#
# 产物：
#   - 后端可执行： backend/target/release/workload-tool
#   - 前端静态资源： backend/static （vite outDir = ../backend/static）

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKEND="$ROOT/backend"
FRONTEND="$ROOT/frontend"

echo "==> [1/2] 构建前端 (npm run build -> backend/static)"
cd "$FRONTEND"
npm install
npm run build

echo "==> [2/2] 构建后端 (cargo build --release --features console)"
cd "$BACKEND"
cargo build --release --features console

echo ""
echo "完成。"
echo "  后端可执行文件： $BACKEND/target/release/workload-tool"
echo "  前端静态产物：   $BACKEND/static"
echo "  运行：           cd $BACKEND && ./target/release/workload-tool"
echo "  注意：程序运行时会在当前目录读写 data/ （SQLite、JWT 密钥、文档等）。"
