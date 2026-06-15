# =============================================================================
# Pickando Demo — Multi-stage Dockerfile for Railway
# Builds: Backend (Axum) + Frontend (Dioxus WASM) → Single serving container
#
# Strategy: Uses wasm-bindgen-cli instead of dioxus-cli for WASM builds.
# This is faster (no dx compilation), more reliable in CI, and produces
# the same output. Dioxus compiles to WASM via standard Rust toolchain.
# =============================================================================

# ---------------------------------------------------------------------------
# Stage 1: Build shared crate (dependency caching layer)
# ---------------------------------------------------------------------------
FROM rust:1.96-bookworm AS deps

WORKDIR /app

# Install system build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/shared/Cargo.toml crates/shared/Cargo.toml
COPY crates/backend/Cargo.toml crates/backend/Cargo.toml
COPY crates/frontend/Cargo.toml crates/frontend/Cargo.toml

# Create dummy sources to build dependency cache
RUN mkdir -p crates/shared/src && echo "pub fn dummy() {}" > crates/shared/src/lib.rs && \
    mkdir -p crates/backend/src && echo "fn main() {}" > crates/backend/src/main.rs && \
    mkdir -p crates/frontend/src && echo "fn main() {}" > crates/frontend/src/main.rs

# Build dependencies only (this layer is cached unless Cargo.lock changes)
RUN cargo build --release --workspace 2>/dev/null || true

# ---------------------------------------------------------------------------
# Stage 2: Build backend binary
# ---------------------------------------------------------------------------
FROM deps AS backend-builder

# Copy real backend + shared source code
COPY crates/shared/src/ crates/shared/src/
COPY crates/backend/src/ crates/backend/src/

# Touch source files to invalidate the dummy cache
RUN find crates/shared/src crates/backend/src -name "*.rs" -exec touch {} +

# Build backend binary
RUN cargo build --release -p pickando-backend

# Verify the binary exists
RUN test -f /app/target/release/pickando-backend && echo "Backend build OK"

# ---------------------------------------------------------------------------
# Stage 3: Build frontend WASM
# ---------------------------------------------------------------------------
FROM deps AS frontend-builder

# Install WASM target
RUN rustup target add wasm32-unknown-unknown

# Install wasm-bindgen-cli — must match the wasm-bindgen crate version
# We detect the version from Cargo.lock to ensure compatibility
RUN WASM_BINDGEN_VER=$(grep -A1 'name = "wasm-bindgen"' Cargo.lock | grep version | head -1 | sed 's/.*"\([^"]*\)".*/\1/') && \
    echo "Installing wasm-bindgen-cli v${WASM_BINDGEN_VER}" && \
    cargo install wasm-bindgen-cli --version "${WASM_BINDGEN_VER}"

# Copy real frontend + shared source code + assets
COPY crates/shared/src/ crates/shared/src/
COPY crates/frontend/src/ crates/frontend/src/
COPY crates/frontend/assets/ crates/frontend/assets/

# Touch source files to invalidate the dummy cache
RUN find crates/shared/src crates/frontend/src -name "*.rs" -exec touch {} +

# Build frontend for WASM target
RUN cargo build --release --target wasm32-unknown-unknown -p pickando-frontend

# Find the WASM binary and run wasm-bindgen
RUN WASM_FILE=$(find target/wasm32-unknown-unknown/release/deps -name "pickando_frontend*.wasm" ! -name "*_*" | head -1) && \
    if [ -z "$WASM_FILE" ]; then \
        WASM_FILE=$(find target/wasm32-unknown-unknown/release/deps -name "pickando_frontend-*.wasm" | head -1); \
    fi && \
    echo "Processing WASM: $WASM_FILE" && \
    mkdir -p /app/dist/assets && \
    wasm-bindgen \
        --target web \
        --out-dir /app/dist/assets \
        --out-name pickando \
        "$WASM_FILE"

# Copy CSS and create index.html
RUN cp crates/frontend/assets/main.css /app/dist/assets/

# Create the index.html that loads the WASM app
RUN cat > /app/dist/index.html << 'HTMLEOF'
<!DOCTYPE html>
<html lang="es">
<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <meta name="description" content="Pickando — Same-direction local mobility" />
    <title>Pickando — Same-Direction Local Mobility</title>
    <link rel="stylesheet" href="/assets/main.css" />
    <style>
        .wasm-loading {
            display: flex;
            align-items: center;
            justify-content: center;
            height: 100vh;
            background: #0a0a0a;
            color: #e0e0e0;
            font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
        }
        .wasm-loading-spinner {
            width: 40px;
            height: 40px;
            border: 3px solid rgba(255,255,255,0.1);
            border-top: 3px solid #00FF88;
            border-radius: 50%;
            animation: spin 1s linear infinite;
            margin-right: 16px;
        }
        @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
        }
    </style>
</head>
<body>
    <div id="main" class="wasm-loading">
        <div class="wasm-loading-spinner"></div>
        <span>Cargando Pickando...</span>
    </div>
    <script type="module">
        import init from '/assets/pickando.js';
        init('/assets/pickando_bg.wasm').catch(e => {
            console.error('Failed to initialize Pickando:', e);
            document.getElementById('main').innerHTML = '<div style="text-align:center;padding:2rem;"><h2 style="color:#ff6b6b;">Error al cargar</h2><p style="color:#aaa;">' + (e.message || 'Unknown error') + '</p><button onclick="location.reload()" style="margin-top:1rem;padding:0.5rem 1rem;background:#00FF88;border:none;border-radius:6px;cursor:pointer;color:#000;">Reintentar</button></div>';
        });
    </script>
</body>
</html>
HTMLEOF

# Verify all output files exist
RUN test -f /app/dist/index.html && \
    test -f /app/dist/assets/pickando.js && \
    test -f /app/dist/assets/pickando_bg.wasm && \
    test -f /app/dist/assets/main.css && \
    echo "Frontend build OK — all static files present"

# ---------------------------------------------------------------------------
# Stage 4: Runtime — minimal production image
# ---------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy backend binary from builder
COPY --from=backend-builder /app/target/release/pickando-backend /app/pickando-backend

# Copy frontend static files from builder
COPY --from=frontend-builder /app/dist /app/static

# Verify both components are present
RUN test -f /app/pickando-backend && echo "Backend binary OK" && \
    test -f /app/static/index.html && echo "Frontend static OK"

# Environment configuration (Railway sets PORT automatically)
ENV PORT=3000
ENV RUST_LOG=pickando=info
ENV STATIC_DIR=/app/static

EXPOSE 3000

# Health check at startup — verifies backend can bind and serve
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/api/v1/health || exit 1

CMD ["/app/pickando-backend"]
