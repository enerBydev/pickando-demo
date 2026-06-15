# Pickando Demo — Same-Direction Local Mobility

> **Proof of execution** — no es MVP, no es M1, es el esqueleto andando.  
> Demuestra que Rust + Dioxus compila a WASM, Axum sirve API, y el matching funciona.

🌐 **Demo en vivo**: [pickando-demo-production.up.railway.app](https://pickando-demo-production.up.railway.app/)

---

## Stack Tecnológico

| Componente | Tecnología | Versión |
|-----------|-----------|---------|
| Lenguaje | Rust | 1.96.0 |
| Frontend | Dioxus | 0.7.9 |
| Backend | Axum | 0.8.9 |
| Runtime | Tokio | 1.52.3 |
| Serialización | Serde | 1.0.228 |
| Matching | Geohash + Haversine | Rust puro |
| Deploy | Railway + GitHub Actions | — |

---

## Arquitectura

```
pickando-demo/
├── crates/
│   ├── shared/       # Modelos + motor de matching (reutilizable)
│   ├── backend/      # API REST + WebSocket (Axum)
│   └── frontend/     # App Dioxus (Web WASM / Desktop / Android)
├── .github/workflows/ # CI/CD multi-plataforma
├── Dockerfile        # Build para Railway
└── railway.json      # Configuración Railway
```

**Flujo de datos:**
```
[Browser/WASM] → REST API → [Axum Server] → Matching Engine → [Result]
[Browser/WASM] ← JSON ←    [Axum Server] ← Geohash+Haversine ← [Result]
[Mobile/Desktop] → WebSocket → [Axum Server] → Echo/Tracking → [Result]
```

---

## Compilación Rápida

### Requisitos
- Rust 1.96+ (`rustup.rs`)
- Dioxus CLI 0.7.9 (`cargo install dioxus-cli --version 0.7.9`)

### Backend (API REST + WebSocket)

```bash
cd crates/backend
cargo run
# → http://localhost:3000
```

### Frontend Web (WASM)

```bash
cd crates/frontend
dx serve
# → http://localhost:8080
```

### Docker

```bash
docker build -t pickando-demo .
docker run -p 3000:3000 pickando-demo
# → http://localhost:3000
```

### Railway

1. Conectar repo `enerbydev/pickando-demo` en Railway
2. Railway detecta `Dockerfile` automáticamente
3. Deploy automático en cada push a `main`
4. URL pública generada por Railway

---

## Endpoints

| Método | Ruta | Descripción |
|--------|------|-------------|
| GET | `/api/v1/health` | Health check con uptime |
| GET | `/api/v1/routes` | Listar rutas de prueba |
| POST | `/api/v1/routes` | Crear ruta (placeholder) |
| POST | `/api/v1/match` | Buscar rutas compatibles |
| GET | `/ws` | WebSocket (echo server) |

### Ejemplo: Health Check

```bash
curl https://pickando-demo-production.up.railway.app/api/v1/health
```

```json
{
  "status": "ok",
  "service": "pickando-backend",
  "version": "0.1.0-proof",
  "stack": "Rust + Axum 0.8 + Tokio 1.52",
  "uptime_seconds": 42.5
}
```

### Ejemplo: Matching

```bash
curl -X POST https://pickando-demo-production.up.railway.app/api/v1/match \
  -H "Content-Type: application/json" \
  -d '{"lat": 19.4326, "lng": -99.1332, "radius_km": 5}'
```

### Ejemplo: WebSocket

```bash
wscat -c wss://pickando-demo-production.up.railway.app/ws
# Envia cualquier mensaje y recibirás echo + confirmación
```

---

## Código Reutilizable vs Demo

| Componente | Reutilizable | Se reemplaza en |
|-----------|-------------|-----------------|
| `shared/models.rs` | ✅ Sí | Se amplía en M2 |
| `shared/matching.rs` | ✅ Sí | Se amplía en M2 |
| `backend/routes.rs` | ✅ Sí | Se amplía en M2 |
| `backend/ws.rs` | ✅ Sí | Se amplía en M2 |
| `backend/state.rs` | ⚠️ Parcial | PgPool en M2 |
| `frontend/components` | ✅ Sí | Se amplía en M3 |
| `frontend/pages` | ⚠️ Parcial | Lógica real en M2 |
| Datos de prueba | ❌ No | PostgreSQL en M2 |
| CSS | ⚠️ Parcial | Design system en M3 |
| Dockerfile | ✅ Sí | Se optimiza en M5 |
| CI/CD | ✅ Sí | Se amplía en M5 |

---

## Multi-Plataforma

La app compila a 4 plataformas desde un solo codebase:

| Plataforma | Comando | Output |
|-----------|---------|--------|
| Web (WASM) | `dx build --platform web --release` | `dist/` |
| Linux | `cargo build --release -p pickando-backend` | Binary |
| Windows | `cargo build --release -p pickando-backend` | `.exe` |
| Android | `dx android build --release` | `.apk` |

---

## CI/CD

- **CI** (`ci.yml`): Formato, lint, tests, build backend + frontend web en cada push
- **Release** (`release.yml`): Build multi-plataforma (Linux, Windows, Web, Android) en cada tag `v*`
- **Railway**: Deploy automático desde `main` branch

---

## Licencia

MIT — Demo sin costo, sin compromiso. Built by René Mendoza (enerBydev).
