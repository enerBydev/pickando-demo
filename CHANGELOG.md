# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] — 2026-06-17

### Summary
Critical bug fixes + UI/UX redesign + robustness improvements.
This release fixes 6 blocker bugs that prevented the demo from working in the browser,
adds a storytelling-driven landing page redesign, hardens security (CORS + headers),
and introduces a demo-reset endpoint for keeping the public demo clean.

### Added — Endpoints
- `POST /api/v1/demo-reset` — clears all routes, ride requests, and relevance scores,
  then re-seeds with the 6 initial sample routes. Useful for keeping the public demo clean.
- `avg_relevance_score` in `/api/v1/stats` now returns a real rolling average
  (ring buffer of last 100 match scores) instead of always `null`.

### Added — UI/UX
- Storytelling section "Una historia Pickando" with María & Antonio narrative
  (concrete numbers: $800/mes ahorro, $40 vs $120 Uber, 2.3 t CO₂/año).
- Trust signals in hero: "Sin registro / Sin costo / 70% ahorro vs Uber / Reduce CO₂".
- Demo transparency banner in passenger + driver pages
  ("Demo sin autenticación. Cualquier dato que ingreses es público").
- Footnote under stats bar with methodology for CO₂ savings estimate.
- CSS for `.demo-banner`, `.landing-story`, `.story-card`, `.story-actors`,
  `.story-avatar`, `.story-connector` (animated), `.story-narrative`, mobile responsive.

### Added — Security
- `X-Content-Type-Options: nosniff` header on all responses.
- `X-Frame-Options: DENY` header on all responses.
- `Referrer-Policy: strict-origin-when-cross-origin` header.
- `Permissions-Policy: geolocation=(), camera=(), microphone=(), payment=()`.
- `set-header` feature added to `tower-http` in workspace Cargo.toml.
- `#[serde(deny_unknown_fields)]` added to `MatchRequest`, `CreateRouteRequest`,
  `CreateRideRequest` for defense-in-depth against array-as-struct deserialization.

### Added — Tests (25 new)
- `validate_departure_time_accepts_hh_mm`, `_hh_mm_ss`, `_iso8601`, `_rejects_garbage`.
- `create_route_rejects_invalid_departure_time`, `create_route_accepts_iso8601_departure_time`.
- `create_route_rejects_out_of_range_coordinates`.
- `find_matches_rejects_negative_radius`, `_zero_radius`, `_huge_radius`.
- `create_route_rejects_array_body`, `find_matches_rejects_array_body`, `request_ride_rejects_array_body`.
- `demo_reset_clears_state_and_reseeds`, `demo_reset_clears_relevance_scores`.

### Changed
- **CORS**: replaced `CorsLayer::permissive()` with `build_cors_layer()`.
  Production allows only `pickando-demo-production.up.railway.app`.
  Dev mode (`PICKANDO_DEV=1`) allows localhost on any port.
- **Hero copy**: removed "DEMO EN VIVO · RUST + DIOXUS + AXUM" badge.
  New headline: "Hoy, alguien va por tu mismo camino".
  New subtitle: "Conduce o comparte. Sin desvíos, sin esperas, sin Uber."
- **CTA buttons**: "Buscar viaje" → "Buscar viaje cerca de ti",
  "Publicar ruta" → "Tengo asientos libres", "Entrar a la plataforma" →
  now goes to `Page::Home` (dashboard) instead of `Page::Passenger`.
- **Cómo funciona**: rewritten in human language.
  No more references to geohash, haversine, websocket, axum.
  New tags: "30 segundos · gratis", "matching por cercanía + dirección + horario",
  "costo compartido justo".
- **Stats bar**: replaced technical metrics (100% Rust, 4 plataformas, <50ms, 51 tests)
  with human metrics (70% ahorro, 2.3 t CO₂, 1-2 km radio, $0, 6 rutas activas).
- **Footer tagline**: "Same-direction local mobility · Demo en Rust"
  → "Comparte el viaje, no el taxi · Demo funcional en Rust".
- **WebSocket copy**: "tracking GPS, estado del viaje, mensajes"
  → "broadcast de eventos en tiempo real (rutas creadas, canceladas,
  solicitudes de pasajeros)".
- **Backend handlers** (`create_route`, `find_matches`, `request_ride`):
  changed from `Json<T>` to `Json<serde_json::Value>` + explicit `is_object()`
  validation + `serde_json::from_value`. Prevents serde from deserializing
  arrays as seq representation of structs.
- **`init_sample_routes()`**: changed from `fn` to `pub fn` so `demo_reset` can call it.
- **`AppState`**: added `recent_relevance_scores: Arc<RwLock<VecDeque<f64>>>`
  with `record_relevance_scores()` and `avg_relevance_score()` methods.

### Fixed — Critical (6 blocker bugs)
- **BUG-FE-001**: Frontend `driver.rs` expected `WsMessage` from `POST /api/v1/routes`
  but backend returns `Route`. Fixed type parameter to `Route`. This was the root
  cause of "No se pudo publicar la ruta: parse JSON: missing field `type`" error.
- **BUG-BE-002**: Coordinates out of range (lat=999) were already rejected by
  existing validation. Added regression test `create_route_rejects_out_of_range_coordinates`.
- **BUG-BE-003**: `departure_time` accepted any string ("not-a-time", "banana", "").
  Added `validate_departure_time()` function accepting HH:MM, HH:MM:SS, ISO-8601.
  Integrated into `create_route` handler.
- **BUG-BE-004**: `radius_km: -5` was silently clamped to 0.1 by `sanitized().clamp()`.
  Added explicit validation in `find_matches` rejecting <=0, >200, NaN, infinity.
- **BUG-BE-005**: `POST /match -d '[1,2,3]'` returned `200 []` because serde
  deserializes arrays as seq representation of structs. Fixed by switching to
  `Json<serde_json::Value>` + `is_object()` check + `serde_json::from_value`.
  Also added `#[serde(deny_unknown_fields)]` to all request types.
- **BUG-DEPLOY-006**: `index.html` had hardcoded stale JS hash
  `dxh768cde497d9597ed.js` (404). Dioxus CLI was injecting a second script tag
  without removing the stale one. Removed the hardcoded script and preload tags.
  Dioxus CLI now manages script tags cleanly during `dx build`.
  This was the root cause of the "loading screen eterno" bug in production.

### Test Suite
- 66 tests passing (25 backend + 40 shared + 1 doctest).
- 0 regressions in existing tests.
- 25 new regression tests covering all P1-P4 fixes.

## [0.2.1] — 2026-06-17

### Changed
- Bumped version to `0.2.1`.
- Fixed homepage URL in `Cargo.toml`.

## [0.2.0] — 2026-06-17

### Added
- Workspace-level `Cargo.toml` with `release` and WASM-optimized profiles.
- `deny.toml` for license, advisory, ban, and source policy enforcement.
- `clippy.toml` and `rustfmt.toml` for consistent style and lint strictness.
- `SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`.
- Architecture Decision Records under `docs/adr/`.
- New `/api/v1/stats` endpoint for platform telemetry.
- New `POST /api/v1/routes/{id}/request` endpoint for passenger ride requests.
- New `DELETE /api/v1/routes/{id}` endpoint for cancelling routes.
- Direction similarity matching using cosine of bearing vectors.
- Time compatibility scoring using departure-time window overlap.
- WebSocket broadcast of `route_created` and `route_cancelled` events.
- Structured JSON logs with request IDs via `tower-http::trace`.
- Property-based tests for the matching engine using `proptest`.
- Benchmarks for `haversine_km` and `find_matching_routes` using `criterion`.
- Integration tests for the Axum backend (`tests/`).

### Changed
- Bumped version to `0.2.0`.
- `RouteStatus` now derives `Eq` for use in pattern matching.
- Health check now reports memory usage and total requests served.
- README rewritten with badges, architecture diagrams, and quickstart.

### Fixed
- WebSocket live-tick `active_routes` was hardcoded to `6`; now reads live state.

## [0.1.0] — 2026-06-14

### Added
- Initial release: Rust 1.96 + Dioxus 0.7 + Axum 0.8 demo.
- Workspace with 3 crates: `shared`, `backend`, `frontend`.
- `GET /api/v1/health`, `GET /api/v1/routes`, `POST /api/v1/routes`, `POST /api/v1/match`, `GET /ws`.
- 6 seeded routes across CDMX and Monterrey.
- Landing page + platform shell with Driver / Passenger / About pages.
- Multi-stage Dockerfile and Railway deployment.
- GitHub Actions CI: lint, format, tests, backend build, WASM build.
- GitHub Actions Release: builds Linux binary + Android APK on tags.

[Unreleased]: https://github.com/enerBydev/pickando-demo/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/enerBydev/pickando-demo/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/enerBydev/pickando-demo/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/enerBydev/pickando-demo/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/enerBydev/pickando-demo/releases/tag/v0.1.0
