# Stage 1: Build
FROM rust:1.80-slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src/ ./src/
COPY frontend/ ./frontend/
COPY static/ ./static/
COPY icon.ico ./

# Build backend (console mode, no tray)
RUN cargo build --release --features console

# Build frontend
RUN apt-get update && apt-get install -y curl && \
    curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
    apt-get install -y nodejs && rm -rf /var/lib/apt/lists/*
RUN cd frontend && npm install && npm run build

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/workload-tool /app/
COPY --from=builder /app/static/ /app/static/
COPY --from=builder /app/icon.ico /app/

RUN mkdir -p /app/data

ENV WORKLOAD_DATA_DIR=/app/data
EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8000/api/health || exit 1

CMD ["./workload-tool"]
