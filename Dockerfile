# ── Stage 1: Build frontend ──────────────────────────
FROM node:20-alpine AS frontend
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# ── Stage 2: Build backend ───────────────────────────
FROM rust:1.86-slim-bookworm AS backend

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app/backend
COPY backend/Cargo.toml backend/Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src

COPY backend/src ./src
COPY backend/migrations ./migrations
RUN cargo build --release

# ── Stage 3: Runtime ─────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=backend /app/backend/target/release/sobrou-nada-pro-bet .
COPY --from=frontend /app/frontend/dist ./dist

EXPOSE 3000

CMD ["./sobrou-nada-pro-bet"]
