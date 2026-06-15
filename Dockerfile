# Build stage — compile both frontend (WASM) and backend
FROM rust:1.96-bookworm AS builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies
COPY Cargo.toml Cargo.lock* ./
COPY crates/shared/Cargo.toml crates/shared/Cargo.toml
COPY crates/backend/Cargo.toml crates/backend/Cargo.toml
COPY crates/frontend/Cargo.toml crates/frontend/Cargo.toml

# Create dummy sources for dependency caching
RUN mkdir -p crates/shared/src && echo "pub fn dummy() {}" > crates/shared/src/lib.rs && \
    mkdir -p crates/backend/src && echo "fn main() {}" > crates/backend/src/main.rs && \
    mkdir -p crates/frontend/src && echo "fn main() {}" > crates/frontend/src/main.rs

# Build dependencies only (cached layer)
RUN cargo build --release -p pickando-shared 2>/dev/null || true

# Copy real source code
COPY crates/shared/src/ crates/shared/src/
COPY crates/backend/src/ crates/backend/src/
COPY crates/frontend/ crates/frontend/

# Touch source files to invalidate cache
RUN find crates -name "*.rs" -exec touch {} +

# Build backend binary
RUN cargo build --release -p pickando-backend

# Build frontend WASM
RUN rustup target add wasm32-unknown-unknown
RUN cargo install dioxus-cli --version 0.7.9
RUN cd crates/frontend && dx build --platform web --release || \
    echo "WASM build failed, serving API-only mode"

# Runtime stage — minimal image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy backend binary
COPY --from=builder /app/target/release/pickando-backend /app/pickando-backend

# Copy frontend static files (if built)
COPY --from=builder /app/crates/frontend/dist /app/static

# Set environment
ENV PORT=3000
ENV RUST_LOG=pickando=info

EXPOSE 3000

CMD ["/app/pickando-backend"]
