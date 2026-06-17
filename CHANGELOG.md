# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/enerBydev/pickando-demo/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/enerBydev/pickando-demo/releases/tag/v0.1.0
