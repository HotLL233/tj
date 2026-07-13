# 本地化LIMS 后端 Docker 构建（多阶段）
# 构建上下文（build context）必须为 v0.4.22 目录，例如：
#   cd v0.4.22
#   docker build -f backend/Dockerfile -t lims-local:v0.4.22 .

# ---------- Stage 1: 构建 ----------
FROM rust:1.88-slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev curl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 后端依赖与源码
COPY backend/Cargo.toml backend/Cargo.lock* ./backend/
COPY backend/src/ ./backend/src/
COPY backend/icon.ico ./backend/icon.ico

# 前端源码（构建后产物写到 backend/static）
COPY frontend/ ./frontend/

# 构建后端（console 模式：无托盘、日志输出到 stdout，适合容器）
RUN cd backend && cargo build --release --features console

# 构建前端（vite outDir = ../backend/static）
RUN cd frontend && npm install && npm run build

# ---------- Stage 2: 运行 ----------
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/backend/target/release/workload-tool /app/
COPY --from=builder /app/backend/static/ /app/static/
COPY --from=builder /app/backend/icon.ico /app/

RUN mkdir -p /app/data

ENV WORKLOAD_DATA_DIR=/app/data
EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8000/api/health || exit 1

CMD ["./workload-tool"]
