# =====================================================================
# Pickando Demo — Multi-stage Dockerfile
# Builds: backend (Linux binary) + frontend (WASM) → minimal runtime image
# =====================================================================

# ---------- Stage 1: Build everything ----------
FROM rust:1.96-bookworm AS builder

WORKDIR /app

# System deps for OpenSSL + dioxus-cli
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# ---------- Pre-install dioxus-cli as a separate cached layer ----------
# This is a 30+ minute compile, so we cache it independently of source code.
# Source: https://github.com/DioxusLabs/dioxus/releases (v0.7.9)
RUN cargo install dioxus-cli --version 0.7.9 --locked

# ---------- Cache cargo deps separately ----------
# Copy only manifests so we can cache the dependency build layer
COPY Cargo.toml Cargo.lock* ./
COPY crates/shared/Cargo.toml crates/shared/Cargo.toml
COPY crates/backend/Cargo.toml crates/backend/Cargo.toml
COPY crates/frontend/Cargo.toml crates/frontend/Cargo.toml
COPY crates/frontend/Dioxus.toml crates/frontend/Dioxus.toml

# Create dummy sources for dependency caching
RUN mkdir -p crates/shared/src && echo "pub fn dummy() {}" > crates/shared/src/lib.rs && \
    mkdir -p crates/backend/src && echo "fn main() {}" > crates/backend/src/main.rs && \
    mkdir -p crates/frontend/src && echo "fn main() {}" > crates/frontend/src/main.rs && \
    mkdir -p crates/frontend/assets

# Build dependencies only (cached layer)
RUN cargo build --release -p pickando-shared 2>/dev/null || true

# ---------- Copy real source ----------
COPY crates/shared/src/ crates/shared/src/
COPY crates/backend/src/ crates/backend/src/
COPY crates/frontend/src/ crates/frontend/src/
COPY crates/frontend/assets/ crates/frontend/assets/
COPY crates/frontend/index.html crates/frontend/index.html

# Touch source files to invalidate cache for our crates only
RUN find crates -name "*.rs" -exec touch {} +

# ---------- Build backend ----------
RUN cargo build --release -p pickando-backend

# ---------- Build frontend WASM ----------
RUN rustup target add wasm32-unknown-unknown

# Build the WASM bundle. If this fails, the Docker build fails loudly.
RUN cd crates/frontend && dx build --platform web --release

# Dioxus 0.7 outputs to target/dx/<crate>/release/web/public/
# CRITICAL: Dioxus 0.7 does NOT copy main.css from the asset_dir to the build
# output even though it injects a <link rel="stylesheet" href="/assets/main.css">
# tag into the built index.html. We must copy it manually or the browser gets
# a 404 for /assets/main.css and the page renders unstyled (and may appear
# stuck on the loading screen if the WASM also fails to mount).
RUN mkdir -p /app/target/dx/pickando-frontend/release/web/public/assets && \
    cp /app/crates/frontend/assets/main.css \
       /app/target/dx/pickando-frontend/release/web/public/assets/main.css && \
    cp /app/crates/frontend/assets/favicon.svg \
       /app/target/dx/pickando-frontend/release/web/public/assets/favicon.svg

# Verify the expected output files exist — fail loudly if missing
RUN test -f /app/target/dx/pickando-frontend/release/web/public/index.html && \
    test -f /app/target/dx/pickando-frontend/release/web/public/assets/main.css && \
    echo "[OK] index.html + main.css present" && \
    ls -la /app/target/dx/pickando-frontend/release/web/public/ && \
    ls -la /app/target/dx/pickando-frontend/release/web/public/assets/

# ---------- Stage 2: Runtime ----------
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy backend binary
COPY --from=builder /app/target/release/pickando-backend /app/pickando-backend

# Copy frontend static files (index.html + assets + WASM bundle)
COPY --from=builder /app/target/dx/pickando-frontend/release/web/public /app/static

# Make sure index.html exists at /app/static/index.html for the fallback service
RUN test -f /app/static/index.html || \
    (echo "ERROR: static/index.html missing" && exit 1)

ENV PORT=3000
ENV RUST_LOG=pickando=info

EXPOSE 3000

CMD ["/app/pickando-backend"]
