# Pickando Demo — Same-Direction Local Mobility

> **Proof of execution** — Demo funcional que demuestra Rust + Dioxus + Axum + Matching Engine.
> No es MVP, no es M1 — es el esqueleto andando, listo para revisión.

---

## URLs de la Demo

| Recurso | URL |
|---------|-----|
| **Frontend + Backend** | https://pickando-demo-production.up.railway.app/ |
| **Health Check** | https://pickando-demo-production.up.railway.app/api/v1/health |
| **Repositorio** | https://github.com/enerbydev/pickando-demo |
| **APK Android** | Disponible en [GitHub Releases](https://github.com/enerbydev/pickando-demo/releases) (tag `v*`) |

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
├── scripts/          # E2E tests + visual validation
├── .github/workflows/ # CI/CD (build + test + APK)
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

## Instrucciones de Ejecución

### Requisitos
- Rust 1.96+ (`rustup.rs`)
- wasm-bindgen-cli (para build WASM manual)

### Opción 1: Docker (recomendado)

```bash
docker build -t pickando-demo .
docker run -p 3000:3000 pickando-demo
# → http://localhost:3000 (frontend + backend juntos)
```

### Opción 2: Desarrollo local (backend + frontend separados)

```bash
# Terminal 1 — Backend
cd crates/backend
cargo run
# → http://localhost:3000

# Terminal 2 — Frontend (requiere Dioxus CLI)
cd crates/frontend
dx serve
# → http://localhost:8080
```

### Opción 3: Build WASM manual (sin Dioxus CLI)

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
bash build-wasm.sh
# Output en crates/frontend/dist/
```

### Opción 4: Android APK

```bash
# Requisitos previos
cargo install dioxus-cli --version 0.7.9
dx android init  # primera vez — configura NDK
dx android build --release
# Output: target/android/app/release/app-release.apk
```

### Railway (deploy automático)

1. Conectar repo `enerbydev/pickando-demo` en Railway
2. Railway detecta `Dockerfile` automáticamente
3. Deploy automático en cada push a `main`
4. URL pública: `pickando-demo-production.up.railway.app`

---

## Endpoints Disponibles

| Método | Ruta | Descripción | Ejemplo |
|--------|------|-------------|---------|
| GET | `/api/v1/health` | Health check con uptime y metadata | `curl .../api/v1/health` |
| GET | `/api/v1/routes` | Listar todas las rutas publicadas | `curl .../api/v1/routes` |
| POST | `/api/v1/routes` | Crear nueva ruta (persiste en memoria) | Ver ejemplo abajo |
| POST | `/api/v1/routes/{id}/join` | Unirse a una ruta existente | Ver ejemplo abajo |
| POST | `/api/v1/match` | Buscar rutas compatibles por ubicación | Ver ejemplo abajo |
| GET | `/ws` | WebSocket bidireccional (echo server) | `wscat -c wss://.../ws` |

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

### Ejemplo: Crear Ruta

```bash
curl -X POST https://pickando-demo-production.up.railway.app/api/v1/routes \
  -H "Content-Type: application/json" \
  -d '{"origin_address":"Zocalo, CDMX","dest_address":"Polanco, CDMX","departure_time":"08:00","seats_available":3}'
```

### Ejemplo: Matching por Ubicación

```bash
curl -X POST https://pickando-demo-production.up.railway.app/api/v1/match \
  -H "Content-Type: application/json" \
  -d '{"lat": 19.4326, "lng": -99.1332, "radius_km": 5}'
```
```json
[
  {
    "route": { "id": "route-001", "origin_address": "Zocalo, CDMX", ... },
    "distance_km": 0.0,
    "direction_similarity": 0.0,
    "relevance_score": 1.0
  }
]
```

### Ejemplo: Unirse a una Ruta

```bash
curl -X POST https://pickando-demo-production.up.railway.app/api/v1/routes/route-001/join
```

### Ejemplo: WebSocket

```bash
wscat -c wss://pickando-demo-production.up.railway.app/ws
# Envia cualquier mensaje y recibirás echo + confirmación bidireccional
```

---

## Matching Engine

El motor de matching es el core de Pickando: conectar pasajeros con conductores que van en la misma dirección.

**Algoritmo actual (demo):**

1. **Geohash encoding**: Las coordenadas de origen del pasajero se codifican en un geohash de 6 caracteres (~0.6km de precisión)
2. **Prefix matching**: Se comparan los prefijos del geohash del pasajero con los de cada ruta publicada para estimar proximidad
3. **Haversine distance**: Se calcula la distancia real en km entre el pasajero y el origen de cada ruta candidata usando la fórmula de Haversine con el radio medio de la Tierra (6371 km)
4. **Radius filtering**: Solo se retornan rutas dentro del radio especificado (default: 5km)
5. **Seat filtering**: Solo rutas con asientos disponibles (`seats_available > 0`)
6. **Relevance scoring**: `score = 100 / (distance + 1)` — más cerca = más relevante

**Próximo en M2:**
- Direction similarity (ángulo entre vectores origen→destino)
- Temporal window matching (compatibilidad de horarios)
- Route overlap analysis (porcentaje de trayecto compartido)
- PostgreSQL spatial indexing con PostGIS

**Test de prueba:**
- Coordenadas CDMX (19.4326, -99.1332) con radio 5km → encuentra rutas 001, 002, 003
- Coordenadas Monterrey (25.6487, -100.4412) → no encuentra rutas CDMX (correcto)

---

## WebSocket

El endpoint `/ws` establece una conexión bidireccional en tiempo real.

**Comportamiento actual (demo):**
1. Al conectar, el servidor envía un mensaje de bienvenida JSON:
   ```json
   {"type": "connected", "message": "Pickando WebSocket live"}
   ```
2. Por cada mensaje del cliente, el servidor responde con un echo:
   ```json
   {"type": "echo", "message": "WebSocket bidireccional funcional", "data": {"received": "..."}}
   ```

**Próximo en M2:**
- GPS coordinate streaming para tracking en vivo
- Ride status updates (Requested → Accepted → Started → Completed)
- Driver-passenger chat relay
- Notification push (pasajero encontrado, ruta cancelada, etc.)

---

## Partes Demo / Placeholder

| Componente | Estado | Detalle |
|-----------|--------|---------|
| Base de datos | Placeholder | Datos en memoria (Vec<Route>), se pierde al reiniciar. PostgreSQL en M2 |
| Autenticación | Demo | Login sin credenciales — click "Iniciar Sesión" entra al dashboard. JWT real en M2 |
| Matching por dirección | Placeholder | Solo proximidad geohash. Dirección similar en M2 |
| GPS tracking | Placeholder | WebSocket hace echo. GPS streaming en M2 |
| Pagos | No implementado | Stripe/MercadoPago en M3 |
| Coordenadas de rutas | Hardcoded | Al publicar ruta, se usan coordenadas default CDMX. Geocoding en M2 |
| Dashboard datos | Demo | Stats hardcodeados (3 rutas, 12 viajes, etc.). Datos reales en M2 |
| APK Android | Debug-signed | Firmado con debug key. Release signing en producción |

---

## Código Reutilizable vs Demo

| Componente | Reutilizable | Se reemplaza en |
|-----------|-------------|-----------------|
| `shared/models.rs` | Sí — tipos core | Se amplía en M2 |
| `shared/matching.rs` | Sí — motor de matching | Se amplía en M2 |
| `backend/routes.rs` | Sí — handlers Axum | Se amplía en M2 |
| `backend/ws.rs` | Sí — WebSocket handler | Se amplía en M2 |
| `backend/state.rs` | Parcial — RwLock | PgPool en M2 |
| `frontend/components` | Sí — UI Dioxus | Se amplía en M3 |
| `frontend/pages` | Parcial — layout ok | Lógica real en M2 |
| Datos de prueba | No — sample data | PostgreSQL en M2 |
| CSS | Parcial — funcional | Design system en M3 |
| Dockerfile | Sí — multi-stage build | Se optimiza en M5 |
| CI/CD | Sí — GitHub Actions | Se amplía en M5 |

---

## Multi-Plataforma

La app compila a 4 plataformas desde un solo codebase Rust:

| Plataforma | Comando | Output |
|-----------|---------|--------|
| Web (WASM) | `bash build-wasm.sh` | `dist/` |
| Backend (Linux) | `cargo build --release -p pickando-backend` | Binary |
| Backend (Windows) | `cargo build --release -p pickando-backend` | `.exe` |
| Android | `dx android build --release` | `.apk` |

**Nota sobre APK Android:** El APK se genera automáticamente en GitHub Actions al crear un tag `v*`. El APK se firma con una debug key por defecto. Para producción, se requiere configurar signing keys.

---

## CI/CD

- **CI** (`.github/workflows/ci.yml`): Formato (rustfmt), lint (clippy), tests, build backend + WASM en cada push/PR a `main`
- **Release** (`.github/workflows/release.yml`): Build Linux binary + WASM + Android APK en cada tag `v*`. Crea GitHub Release con todos los artefactos.
- **Railway**: Deploy automático desde `main` branch via Dockerfile

---

## Tests

```bash
# Tests unitarios + integración
cargo test --workspace

# E2E smoke tests (requiere Railway deploy)
node scripts/e2e-smoke-test.js

# Visual validation con VLM
node scripts/visual-validate.js --analyze
```

---

## Licencia

MIT — Demo sin costo, sin compromiso. Built by René Mendoza (enerBydev).
