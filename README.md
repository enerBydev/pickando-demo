# Pickando — Same-Direction Local Mobility Demo

> Demo funcional en Rust puro: Dioxus 0.7 + Axum 0.8 + WebAssembly.
> Matching inteligente, WebSocket en vivo, multi-plataforma desde un solo codebase.

---

## Stack

| Componente | Tecnología |
|-----------|-----------|
| Lenguaje | Rust 1.96 |
| Frontend | Dioxus 0.7 (WASM) |
| Backend | Axum 0.8 + Tokio |
| Matching | Geohash + Haversine (Rust puro) |
| Real-time | WebSocket bidireccional |
| Deploy | Railway + GitHub Actions |

---

## Arquitectura

```
pickando-demo/
├── crates/
│   ├── shared/       # Modelos + motor de matching
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
[Browser/WASM] ↔ WebSocket ↔ [Axum Server] ↔ Echo + Live ticks
```

---

## Compilación rápida

### Requisitos
- Rust 1.96+ (`rustup.rs`)
- Dioxus CLI 0.7.9 (`cargo install dioxus-cli --version 0.7.9 --locked`)

### Backend (API REST + WebSocket)

```bash
cargo run -p pickando-backend
# → http://localhost:3000
```

### Frontend Web (WASM)

```bash
cd crates/frontend
dx serve
# → http://localhost:8080
```

### Docker (todo en uno)

```bash
docker build -t pickando-demo .
docker run -p 3000:3000 pickando-demo
# → http://localhost:3000
```

### Railway

1. Conectar el repo en Railway
2. Railway detecta el `Dockerfile` automáticamente
3. Deploy automático en cada push a `main`
4. URL pública generada por Railway

---

## Endpoints

| Método | Ruta | Descripción |
|--------|------|-------------|
| GET | `/api/v1/health` | Health check con uptime y rutas activas |
| GET | `/api/v1/routes` | Listar rutas (sembradas + creadas en runtime) |
| POST | `/api/v1/routes` | Crear nueva ruta (persiste en memoria) |
| POST | `/api/v1/match` | Buscar rutas compatibles por ubicación |
| GET | `/ws` | WebSocket bidireccional con echo + live ticks |

### Ejemplo: Health Check

```bash
curl https://pickando-demo.up.railway.app/api/v1/health
```

```json
{
  "status": "ok",
  "service": "pickando-backend",
  "version": "0.1.0",
  "stack": "Rust + Axum 0.8 + Tokio 1.52",
  "uptime_seconds": 42.5,
  "routes_count": 6
}
```

### Ejemplo: Matching

```bash
curl -X POST https://pickando-demo.up.railway.app/api/v1/match \
  -H "Content-Type: application/json" \
  -d '{"lat": 19.4326, "lng": -99.1332, "radius_km": 5}'
```

### Ejemplo: Crear ruta

```bash
curl -X POST https://pickando-demo.up.railway.app/api/v1/routes \
  -H "Content-Type: application/json" \
  -d '{
    "origin_address": "Zócalo, CDMX",
    "dest_address": "Polanco, CDMX",
    "departure_time": "08:00",
    "seats_available": 3
  }'
```

### Ejemplo: WebSocket

```bash
wscat -c wss://pickando-demo.up.railway.app/ws
# Recibirás: welcome, tick cada 5s, y echo de todo lo que envíes
```

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

- **CI** (`ci.yml`): Formato, lint, tests, build backend + frontend WASM en cada push
- **Release** (`release.yml`): Build multi-plataforma + APK Android en cada tag `v*`
- **Railway**: Deploy automático desde `main`

---

## Demo en vivo

La demo incluye datos sembrados (6 rutas en CDMX y Monterrey) para que se sienta viva desde el primer momento. La UI:

- **Landing page**: hero, features, stack, CTAs — separada de la plataforma
- **Plataforma**: navbar + 4 secciones (Inicio, Conductor, Pasajero, Acerca de)
- **Conductor**: formulario que llama `POST /api/v1/routes` + lista de rutas en vivo
- **Pasajero**: matching con geohash+Haversine, lista de rutas, WebSocket visual, status
- **WebSocket en vivo**: conexión real con el backend, mensajes en tiempo real

---

## Licencia

MIT — Demo sin costo, sin compromiso.
