# Pickando Demo — Architecture

> This document explains the runtime architecture of the demo.
> For *decisions* and rationale, see [`adr/`](adr/).

## High-level diagram

```
                            ┌──────────────────────────────────┐
                            │           Browser (WASM)         │
                            │  ┌────────────────────────────┐  │
                            │  │   Dioxus 0.7 frontend      │  │
                            │  │   (pickando-frontend)      │  │
                            │  │                            │  │
                            │  │  • Landing page            │  │
                            │  │  • Driver panel            │  │
                            │  │  • Passenger panel         │  │
                            │  │  • About / endpoints       │  │
                            │  │  • WebSocket client        │  │
                            │  └──────────┬─────────────────┘  │
                            └─────────────┼────────────────────┘
                                          │
                              HTTP /api/v1/* │ ws://.../ws
                                          │
                            ┌──────────────▼──────────────────┐
                            │      Axum 0.8 + Tokio backend   │
                            │      (pickando-backend)         │
                            │                                 │
                            │  ┌───────────────────────────┐  │
                            │  │     HTTP routes           │  │
                            │  │  /health  /stats          │  │
                            │  │  /routes  /routes/{id}    │  │
                            │  │  /routes/{id}/request     │  │
                            │  │  /match                   │  │
                            │  └────────────┬──────────────┘  │
                            │               │                 │
                            │  ┌────────────▼──────────────┐  │
                            │  │    AppState (Arc)         │  │
                            │  │  • routes: RwLock<Vec>    │  │
                            │  │  • ride_requests          │  │
                            │  │  • ws_broadcaster         │  │
                            │  │  • request_counter        │  │
                            │  │  • route_counter          │  │
                            │  └────────────┬──────────────┘  │
                            │               │                 │
                            │  ┌────────────▼──────────────┐  │
                            │  │   WebSocket handler       │  │
                            │  │   /ws                      │  │
                            │  │  • connected welcome      │  │
                            │  │  • live_tick (5s)         │  │
                            │  │  • broadcast fan-out      │  │
                            │  │  • echo                   │  │
                            │  └───────────────────────────┘  │
                            └──────────────┬──────────────────┘
                                           │
                            ┌──────────────▼──────────────────┐
                            │  pickando-shared (library)      │
                            │  • models: Route, User,         │
                            │    RideRequest, WsMessage       │
                            │  • matching: geohash, haversine,│
                            │    bearing, time compatibility  │
                            │  • Pure functions, no I/O       │
                            └─────────────────────────────────┘
```

## Crate dependency graph

```
                  pickando-shared
                  /              \
                 v                v
       pickando-backend    pickando-frontend
              │                    │
              v                    v
       (Linux binary)         (WASM module)
              │                    │
              └─── deployed ───────┘
                   on Railway
```

`pickando-shared` is the **single source of truth** for the data
contract between backend and frontend. Backend and frontend never
import from each other directly.

## Request lifecycle

1. **Browser** sends `POST /api/v1/match` with `MatchRequest` JSON.
2. **Axum** deserializes the body via `serde_json` (types from
   `pickando-shared`).
3. **TraceLayer** creates a span with a UUID `request_id` for logs.
4. **Handler** (`routes::find_matches`):
   - Calls `state.record_request()` (atomic counter).
   - Sanitizes the input (`MatchRequest::sanitized`).
   - Acquires a read lock on `state.routes`.
   - Calls `find_matching_routes(lat, lng, &routes, radius)` from
     `pickando-shared::matching`.
   - Returns `Json<Vec<MatchResult>>`.
5. **CompressionLayer** gzips the response if the client sent
   `Accept-Encoding: gzip`.
6. **Browser** receives JSON, `serde_json::from_str` into typed
   structs, Dioxus re-renders the match cards.

End-to-end latency: <50 ms on the demo's dataset (6 routes seeded).

## WebSocket fan-out

```
POST /api/v1/routes         POST /api/v1/routes/{id}/request
         │                              │
         v                              v
   routes::create_route        routes::request_ride
         │                              │
         │  state.ws_broadcaster        │
         │       .send(WsMessage)       │
         │       .send(WsMessage)       │
         v                              v
   ┌─────────────────────────────────────────┐
   │  tokio::sync::broadcast channel          │
   │  (capacity 256)                          │
   └────┬─────────────────┬──────────────────┘
        │                 │
        v                 v
   ┌─────────┐       ┌─────────┐
   │  WS #1  │       │  WS #2  │   (any connected clients)
   └─────────┘       └─────────┘
```

Any client connected to `/ws` sees `route_created`,
`route_cancelled`, and `ride_request` events in real time.

## Build pipeline

```
┌─────────────┐
│  git push   │
└──────┬──────┘
       │
       v
┌─────────────────────────────────────────┐
│  GitHub Actions CI                       │
│  • fmt --check                           │
│  • clippy -D warnings                    │
│  • cargo audit (RustSec)                 │
│  • cargo deny (licenses/bans)            │
│  • cargo test --workspace                │
│  • build backend (release)               │
│  • smoke-test backend endpoints          │
│  • build frontend (WASM)                 │
└──────┬───────────────────────────────────┘
       │
       ├──────────────────┐
       v                  v
┌──────────────┐   ┌──────────────────┐
│  Railway     │   │  GitHub Release   │
│  auto-deploy │   │  on tag v*        │
│  via Docker  │   │  • Linux binary   │
│              │   │  • Android APK    │
└──────────────┘   └──────────────────┘
```

## Deployment topology (Railway)

```
┌──────────────────────────────────────────┐
│           Railway (single instance)        │
│                                            │
│  ┌──────────────────────────────────────┐ │
│  │  Docker container (Debian slim)      │ │
│  │  • pickando-backend binary           │ │
│  │  • /app/static/* (WASM + CSS + HTML) │ │
│  │  • Runs as appuser (non-root)        │ │
│  │  • HEALTHCHECK on /api/v1/health     │ │
│  └──────────────────────────────────────┘ │
│                                            │
│  Public URL:                               │
│  https://pickando-demo.up.railway.app      │
└──────────────────────────────────────────┘
```

Single instance is sufficient for the demo. For production, scale
horizontally with a sticky-session load balancer (or replace
in-memory state with PostgreSQL + Redis).

## Observability

| Signal       | Tool                    | Where                          |
|--------------|-------------------------|--------------------------------|
| Logs         | `tracing` + `tracing-subscriber` | stdout (Railway captures) |
| Request IDs  | `tower_http::TraceLayer` + UUID  | in every log span         |
| Metrics      | `/api/v1/stats` endpoint          | JSON via curl             |
| Health       | `/api/v1/health` endpoint         | Docker HEALTHCHECK + Railway |
| Memory       | `/proc/self/statm` (Linux)        | in health response        |

For production: add `tracing-subscriber` with `json` feature,
export to Datadog/Jaeger via OpenTelemetry.

## Security model (demo)

| Threat                      | Mitigation                                       |
|-----------------------------|--------------------------------------------------|
| SQL injection               | N/A — no SQL (in-memory)                         |
| XSS                         | Dioxus escapes all interpolated values           |
| CSRF                        | N/A — no auth                                    |
| DOS                         | `tower::limit` (planned for production)          |
| Supply-chain                | `cargo audit` + `cargo deny` in CI               |
| Memory safety               | Rust's borrow checker (no `unsafe` in workspace) |
| Secrets in repo             | None — `/.secrets` is gitignored                 |

## Performance characteristics

Benchmarked with `criterion` on a single core:

| Operation                  | Throughput          | Notes                                  |
|----------------------------|---------------------|----------------------------------------|
| `haversine_km`             | ~50 ns / call       | Single calculation                     |
| `find_matching_routes` (10 routes) | ~5 µs        | Includes geohash filter + sort         |
| `find_matching_routes` (100 routes) | ~50 µs       | Linear in input size                   |
| `find_matching_routes` (1000 routes) | ~500 µs     | Still sub-millisecond                  |
| `find_matching_routes` (10000 routes) | ~5 ms      | Acceptable for a demo with 10k routes  |

Cold start: ~3 ms (in-memory store init). Binary size: 4.2 MB
(stripped, LTO).
