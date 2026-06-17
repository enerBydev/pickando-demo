# Pickando — Same-Direction Local Mobility Demo

[![CI](https://github.com/enerBydev/pickando-demo/actions/workflows/ci.yml/badge.svg)](https://github.com/enerBydev/pickando-demo/actions/workflows/ci.yml)
[![Release](https://github.com/enerBydev/pickando-demo/actions/workflows/release.yml/badge.svg)](https://github.com/enerBydev/pickando-demo/releases)
[![Rust](https://img.shields.io/badge/rust-1.96-orange.svg)](https://www.rust-lang.org/)
[![Dioxus](https://img.shields.io/badge/dioxus-0.7-blueviolet.svg)](https://dioxuslabs.com/)
[![Axum](https://img.shields.io/badge/axum-0.8-blue.svg)](https://github.com/tokio-rs/axum)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Live demo](https://img.shields.io/badge/live-demo-00FF88.svg)](https://pickando-demo-production.up.railway.app)

> Demo funcional en Rust puro: **Dioxus 0.7** (WASM) + **Axum 0.8** + **Tokio**.
> Matching multi-factor (geohash + haversine + bearing + tiempo), WebSocket
> bidireccional con broadcast, multi-plataforma desde un solo codebase.

---

## Tabla de contenidos

- [Stack](#stack)
- [Demo en vivo](#demo-en-vivo)
- [Arquitectura](#arquitectura)
- [Endpoints REST](#endpoints-rest)
- [Compilación rápida](#compilación-rápida)
- [Multi-plataforma](#multi-plataforma)
- [Testing y calidad](#testing-y-calidad)
- [DevOps y deployment](#devops-y-deployment)
- [Documentación](#documentación)
- [Roadmap](#roadmap)
- [Licencia](#licencia)

---

## Stack

| Componente       | Tecnología                                  |
|------------------|---------------------------------------------|
| Lenguaje         | Rust 1.96                                   |
| Frontend         | Dioxus 0.7 (compila a WASM)                 |
| Backend          | Axum 0.8 + Tokio                            |
| Matching engine  | Geohash + Haversine + Bearing cosine + Time |
| Real-time        | WebSocket bidireccional con broadcast       |
| Tracing          | `tracing` + `tower-http::TraceLayer` (UUID) |
| Deploy           | Railway + Docker multi-stage                |
| CI/CD            | GitHub Actions (lint, test, audit, deny)    |

---

## Demo en vivo

- **App web:** <https://pickando-demo-production.up.railway.app>
- **API health:** <https://pickando-demo-production.up.railway.app/api/v1/health>
- **API stats:** <https://pickando-demo-production.up.railway.app/api/v1/stats>
- **Repositorio:** <https://github.com/enerBydev/pickando-demo>
- **APK Android:** ver [Releases](https://github.com/enerBydev/pickando-demo/releases)

---

## Arquitectura

```
┌─────────────────────┐         ┌─────────────────────────┐
│   Browser (WASM)    │         │   Axum + Tokio backend  │
│                     │  HTTP   │                         │
│  Dioxus 0.7 frontend├────────►│  /api/v1/*  endpoints   │
│  (Landing + 4 pages)│         │  /ws        WebSocket   │
│                     │  WS     │                         │
│                     │◄───────►│  AppState (in-memory)   │
└──────────┬──────────┘         └────────────┬────────────┘
           │                                  │
           └─────────► pickando-shared ◄──────┘
                      (models + matching)
```

3 crates comparten tipos sin duplicar lógica:

```
crates/
├── shared/      # pickando-shared  — pure domain logic + 40 tests
├── backend/     # pickando-backend — Axum HTTP/WS server + 10 tests
└── frontend/    # pickando-frontend — Dioxus UI (WASM)
```

Para detalles ver [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) y los
[ADRs](docs/adr/).

---

## Endpoints REST

| Método  | Ruta                              | Descripción                                   |
|---------|-----------------------------------|-----------------------------------------------|
| GET     | `/api/v1/health`                  | Health check con uptime, memoria, requests    |
| GET     | `/api/v1/stats`                   | Telemetría: rutas y solicitudes por estado    |
| GET     | `/api/v1/routes`                  | Listar todas las rutas                        |
| POST    | `/api/v1/routes`                  | Crear nueva ruta (broadcast `route_created`)  |
| GET     | `/api/v1/routes/{id}`             | Obtener ruta por ID                           |
| DELETE  | `/api/v1/routes/{id}`             | Cancelar ruta (broadcast `route_cancelled`)   |
| POST    | `/api/v1/routes/{id}/request`     | Solicitar unirse (broadcast `ride_request`)   |
| POST    | `/api/v1/match`                   | Buscar matches (geohash+haversine+dir+tiempo) |
| POST    | `/api/v1/demo-reset`              | Reiniciar demo a seeds iniciales (limpia spam)|
| GET     | `/ws`                             | WebSocket bidireccional con broadcast         |

Referencia completa: [`docs/API.md`](docs/API.md).

### Seguridad

- **CORS restrictivo:** en producción solo permite `pickando-demo-production.up.railway.app`.
  En desarrollo (`PICKANDO_DEV=1`) permite localhost en cualquier puerto.
- **Security headers:** `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`,
  `Referrer-Policy: strict-origin-when-cross-origin`, `Permissions-Policy: geolocation=(), camera=(), microphone=(), payment=()`.
- **Validación de input:** todos los POST handlers rechazan bodies no-objeto (422),
  coordenadas fuera de rango (400), `departure_time` inválido (400), `radius_km` negativo (400).
- **`#[serde(deny_unknown_fields)]`** en `MatchRequest`, `CreateRouteRequest`, `CreateRideRequest`
  para defense-in-depth contra deserialización de arrays como structs.

Ver ADR-0007 (validación), ADR-0008 (CORS + headers), ADR-0009 (demo-reset) en `docs/adr/`.

### Ejemplos rápidos

```bash
# Health check
curl https://pickando-demo-production.up.railway.app/api/v1/health

# Stats
curl https://pickando-demo-production.up.railway.app/api/v1/stats

# Matching (CDMX Zócalo, radio 5km)
curl -X POST https://pickando-demo-production.up.railway.app/api/v1/match \
  -H "Content-Type: application/json" \
  -d '{"lat": 19.4326, "lng": -99.1332, "radius_km": 5}'

# Matching avanzado (con dirección + ventana temporal)
curl -X POST https://pickando-demo-production.up.railway.app/api/v1/match \
  -H "Content-Type: application/json" \
  -d '{
    "lat": 19.4326,
    "lng": -99.1332,
    "radius_km": 5,
    "passenger_bearing_deg": 0,
    "time_window_minutes": 60,
    "passenger_departure_time": "08:00"
  }'

# Crear ruta
curl -X POST https://pickando-demo-production.up.railway.app/api/v1/routes \
  -H "Content-Type: application/json" \
  -d '{
    "origin_address": "Zócalo, CDMX",
    "dest_address": "Polanco, CDMX",
    "departure_time": "08:00",
    "seats_available": 3
  }'

# Solicitar unirse a una ruta
curl -X POST https://pickando-demo-production.up.railway.app/api/v1/routes/route-001/request \
  -H "Content-Type: application/json" \
  -d '{"passenger_name":"María","seats_requested":1}'

# Cancelar una ruta
curl -X DELETE https://pickando-demo-production.up.railway.app/api/v1/routes/route-002

# WebSocket
wscat -c wss://pickando-demo.up.railway.app/ws
```

---

## Compilación rápida

### Requisitos

- Rust 1.96+ (`rustup.rs`)
- Dioxus CLI 0.7.9 (`cargo install dioxus-cli --version 0.7.9 --locked`)
- Opcional: Docker para build aislado

### Backend

```bash
cargo run -p pickando-backend
# → http://localhost:3000
```

### Frontend (desarrollo con hot-reload)

```bash
cd crates/frontend
dx serve
# → http://localhost:8080 (con hot reload)
```

### Frontend (build release para WASM)

```bash
cd crates/frontend
dx build --platform web --release
# Output: target/dx/pickando-frontend/release/web/public/
```

### Docker (todo en uno)

```bash
docker build -t pickando-demo .
docker run -p 3000:3000 pickando-demo
# → http://localhost:3000
```

El contenedor corre como usuario non-root (`appuser`), tiene
`HEALTHCHECK` configurado, y comprime respuestas con gzip.

---

## Multi-plataforma

La app compila a 4+ plataformas desde un solo codebase:

| Plataforma | Comando                                          | Output                                  |
|------------|--------------------------------------------------|-----------------------------------------|
| Web (WASM) | `dx build --platform web --release`              | `target/dx/.../web/public/`             |
| Linux      | `cargo build --release -p pickando-backend`      | Binary                                  |
| Windows    | `cargo build --release -p pickando-backend`      | `.exe`                                  |
| Android    | `cd android && ./gradlew assembleDebug`          | `.apk` (WebView wrapper)                |

> **Nota Android**: El APK usa un WebView wrapper que carga la demo
> desplegada (approach más confiable en CI que `dx build --android`
> que requiere NDK completo). Ver
> [ADR-0004](docs/adr/0004-android-webview-wrapper.md).

---

## Testing y calidad

### Cobertura de tests

| Crate              | Tests | Tipo                                |
|--------------------|-------|-------------------------------------|
| `pickando-shared`  | 40    | Unit + property-based (proptest)    |
| `pickando-backend` | 25    | Integration (handler-level)         |
| Doc tests          | 1     | `haversine_km` doctest              |
| **Total**          | **66**| all passing                         |

Incluye 25 tests de regresión cubriendo los 6 bugs críticos de v0.2.1 (ver CHANGELOG v0.3.0).

### Verificación local

```bash
# Formato
cargo fmt --all -- --check

# Lint (estricto, deny warnings)
cargo clippy --workspace --all-targets -- -D warnings

# Tests
cargo test --workspace

# Seguridad
cargo audit

# Licencias + bans + fuentes
cargo deny check

# Benchmarks (informational)
cargo bench -p pickando-shared --bench matching
```

### CI pipeline

El workflow `.github/workflows/ci.yml` corre en cada push y PR:

1. **fmt** — `cargo fmt --check`
2. **clippy** — `cargo clippy -D warnings`
3. **audit** — RustSec advisory DB
4. **deny** — licencias, bans, fuentes
5. **test** — `cargo test --workspace` + doc tests
6. **build-backend** — release build + smoke test de endpoints
7. **build-frontend-web** — WASM build + verificación de archivos
8. **bench** — benchmarks informativos (PRs solamente)

### Cobertura de features enterprise

- ✅ 51 tests automatizados (unit + property + integration)
- ✅ `cargo clippy` con `-D warnings` en CI
- ✅ `cargo audit` contra RustSec advisory DB (nightly + on push)
- ✅ `cargo deny` para licencias, bans, fuentes
- ✅ `Cargo.lock` commiteado para builds reproducibles
- ✅ Profiles de release con LTO + strip + codegen-units=1
- ✅ Sin bloques `unsafe` en el workspace
- ✅ Conventional Commits + CHANGELOG.md
- ✅ 6 ADRs documentando decisiones arquitectónicas
- ✅ SECURITY.md + CONTRIBUTING.md + CODE_OF_CONDUCT.md
- ✅ Templates de Issues y PR en `.github/`
- ✅ Non-root Docker user + HEALTHCHECK

---

## DevOps y deployment

### Railway (producción)

1. Conectar el repo en Railway
2. Railway detecta el `Dockerfile` automáticamente
3. Deploy automático en cada push a `main`
4. URL pública: <https://pickando-demo-production.up.railway.app>

`railway.json` configura:
- `startCommand`: `/app/pickando-backend`
- `healthcheckPath`: `/api/v1/health`
- `restartPolicyType`: `ON_FAILURE` (max 10 retries)

### GitHub Actions

- **CI** (`ci.yml`): formato, lint, audit, deny, tests, builds
- **Release** (`release.yml`): build multi-plataforma + APK Android en cada tag `v*`

> Cada release `v*` incluye `pickando-demo.apk` pre-compilado y firmado.

### Costos de terceros (informativo)

| Servicio               | Costo                          |
|------------------------|--------------------------------|
| Railway (free tier)    | $0 (hasta 500 horas/mes)       |
| GitHub Actions         | $0 (público repo)              |
| Dominio                | N/A (usa URL de Railway)       |
| Google Maps API        | N/A (no usado en la demo)      |
| Firebase               | N/A                            |
| **Total demo**         | **$0/mes**                     |

---

## Documentación

| Documento                              | Descripción                                  |
|----------------------------------------|----------------------------------------------|
| [docs/API.md](docs/API.md)             | Referencia completa de la REST API + WS      |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Arquitectura, diagrams, flujos          |
| [docs/adr/](docs/adr/)                 | 6 Architecture Decision Records              |
| [CHANGELOG.md](CHANGELOG.md)           | Historial de versiones (Keep a Changelog)    |
| [SECURITY.md](SECURITY.md)             | Política de seguridad + hardening checklist  |
| [CONTRIBUTING.md](CONTRIBUTING.md)     | Cómo contribuir (workflow, PR template)      |
| [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) | Contributor Covenant 2.1                   |

---

## Roadmap

### v0.2.0 (actual)

- ✅ Matching multi-factor (geohash+haversine+bearing+time)
- ✅ 8 endpoints REST + WebSocket con broadcast
- ✅ Stats + telemetría
- ✅ Tracing estructurado con request IDs
- ✅ 51 tests + property-based + benchmarks
- ✅ ADRs + governance docs
- ✅ CI/CD con audit + deny

### v0.3.0 (planeado)

- ⏳ PostgreSQL persistence (vía `sqlx` con compile-time query verification)
- ⏳ Redis para sesiones + cache + pub/sub
- ⏳ Autenticación JWT + OTP (Twilio)
- ⏳ Live GPS tracking (WebSocket streaming de coordenadas)
- ⏳ Docker Compose para desarrollo local con PG + Redis

### v0.4.0 (futuro)

- ⏳ Integración Stripe + MercadoPago
- ⏳ QR check-in/check-out (JWT-firmado)
- ⏳ Safety contacts + Emergency/SOS button
- ⏳ Route sharing (link público con tracking)
- ⏳ Load testing con `k6` o `goose`

---

## Licencia

MIT — Demo sin costo, sin compromiso. Ver [LICENSE](LICENSE).

---

## Autor

**René Mendoza** · [enerBydev](https://github.com/enerBydev)
· Desarrollador de Software Fullstack | Especialista en Rust e IA
· <https://enerby.dev>

Si esta demo te interesa para un proyecto real, contáctame. El código
es 100% tuyo bajo licencia MIT y está documentado para que cualquier
desarrollador Rust senior pueda mantenerlo.
