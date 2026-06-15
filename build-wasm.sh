#!/usr/bin/env bash
# =============================================================================
# build-wasm.sh — Build the Pickando frontend as WASM for web deployment
#
# This script replaces `dx build --platform web` with a manual WASM build
# using cargo + wasm-bindgen-cli. This is more reliable in CI environments
# where installing dioxus-cli can be slow or fail.
#
# Usage:
#   ./build-wasm.sh              # Build to crates/frontend/dist/
#   ./build-wasm.sh /path/out    # Build to custom output directory
#
# Prerequisites:
#   - Rust toolchain with wasm32-unknown-unknown target
#   - wasm-bindgen-cli (installed automatically if missing)
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

OUTPUT_DIR="${1:-crates/frontend/dist}"
echo "=== Pickando WASM Build ==="
echo "Output directory: $OUTPUT_DIR"

# Step 1: Ensure WASM target is installed
echo "[1/5] Checking WASM target..."
rustup target add wasm32-unknown-unknown 2>/dev/null || true

# Step 2: Build the WASM binary
echo "[2/5] Compiling frontend to WASM..."
cargo build --release --target wasm32-unknown-unknown -p pickando-frontend

# Step 3: Find the WASM file
WASM_FILE=$(find target/wasm32-unknown-unknown/release/deps -name "pickando_frontend*.wasm" ! -name "*_*" 2>/dev/null | head -1)
if [ -z "$WASM_FILE" ]; then
    WASM_FILE=$(find target/wasm32-unknown-unknown/release/deps -name "pickando_frontend-*.wasm" 2>/dev/null | head -1)
fi

if [ -z "$WASM_FILE" ]; then
    echo "ERROR: No WASM file found in target/wasm32-unknown-unknown/release/deps/"
    exit 1
fi
echo "  Found WASM: $WASM_FILE ($(du -h "$WASM_FILE" | cut -f1))"

# Step 4: Install wasm-bindgen-cli if needed
echo "[3/5] Checking wasm-bindgen-cli..."
WASM_BINDGEN_VER=$(grep -A1 'name = "wasm-bindgen"' Cargo.lock | grep version | head -1 | sed 's/.*"\([^"]*\)".*/\1/')
echo "  Required version: $WASM_BINDGEN_VER"

if ! command -v wasm-bindgen &>/dev/null; then
    echo "  Installing wasm-bindgen-cli v$WASM_BINDGEN_VER..."
    cargo install wasm-bindgen-cli --version "$WASM_BINDGEN_VER"
else
    INSTALLED_VER=$(wasm-bindgen --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "unknown")
    if [ "$INSTALLED_VER" != "$WASM_BINDGEN_VER" ]; then
        echo "  Version mismatch ($INSTALLED_VER vs $WASM_BINDGEN_VER), updating..."
        cargo install wasm-bindgen-cli --version "$WASM_BINDGEN_VER"
    else
        echo "  wasm-bindgen-cli v$INSTALLED_VER already installed"
    fi
fi

# Step 5: Process WASM with bindgen
echo "[4/5] Running wasm-bindgen..."
mkdir -p "$OUTPUT_DIR/assets"
wasm-bindgen \
    --target web \
    --out-dir "$OUTPUT_DIR/assets" \
    --out-name pickando \
    "$WASM_FILE"

# Step 6: Copy static assets and create index.html
echo "[5/5] Packaging output..."
cp crates/frontend/assets/main.css "$OUTPUT_DIR/assets/"

cat > "$OUTPUT_DIR/index.html" << 'HTMLEOF'
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
            background: #0D0D11;
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
            document.getElementById('main').innerHTML = `
                <div style="text-align:center; padding:2rem;">
                    <h2 style="color:#FF4466;">Error al cargar la aplicación</h2>
                    <p style="color:#aaa;">${e.message || 'Unknown error'}</p>
                    <button onclick="location.reload()" style="margin-top:1rem; padding:0.5rem 1rem; background:#00FF88; border:none; border-radius:6px; cursor:pointer; color:#000;">Reintentar</button>
                </div>
            `;
        });
    </script>
</body>
</html>
HTMLEOF

# Verify output
echo ""
echo "=== Build Verification ==="
for f in index.html assets/pickando.js assets/pickando_bg.wasm assets/main.css; do
    if [ -f "$OUTPUT_DIR/$f" ]; then
        SIZE=$(du -h "$OUTPUT_DIR/$f" | cut -f1)
        echo "  ✓ $f ($SIZE)"
    else
        echo "  ✗ $f MISSING!"
        exit 1
    fi
done

echo ""
echo "✅ WASM build complete! Output in $OUTPUT_DIR/"
echo "   Serve with: STATIC_DIR=$OUTPUT_DIR ./target/release/pickando-backend"
