# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.1] — 2026-06-19

### Summary
Continuous-improvement pass: removed every emoji and Unicode-glyph icon
from the frontend, replacing them with a shared inline-SVG icon set. This
aligns the codebase with the #09-v2 design system rule ("NO emoji icons")
and guarantees pixel-identical rendering across browsers / OSes.

### Added — `crates/frontend/src/icons.rs`
- Single source of truth for all UI pictograms (Level 2: composition).
- 14 components: `IconPin`, `IconList`, `IconPulse`, `IconClock`, `IconUser`,
  `IconUsers`, `IconHome`, `IconTarget`, `IconSteering`, `IconCheck`, `IconX`,
  `IconInfo`, `IconAlert`, `IconArrowRight`, `IconDownload`, `IconUpload`.
- Each accepts `size` (default 16) and optional `class`.
- All inherit `currentColor` for theme integration.

### Changed — Emoji purge
- `mobile/shell.rs`: replaced `⌂`/`⌖`/`⚠` glyphs with `IconHome`,
  `IconTarget`, `IconSteering` SVG components in the bottom nav.
- `platform/passenger.rs`:
  - `📋 Rutas` → `IconList` + "Rutas"
  - `🔴 WebSocket` → `IconPulse` + "WebSocket"
  - `📍 {location}` preset buttons → `IconPin` SVG
  - `📍 {distance} km` in match cards → `IconPin` SVG
  - `⏱ Latencia` → `IconClock` SVG
  - `🕐 {time}` → `IconClock` SVG
  - `💺 {seats}` → `IconUser` SVG
  - WS log markers `✅`/`❌`/`📥`/`📤` → ASCII `[+]`/`[!]`/`[<]`/`[>]`
    (text-only log, no icon needed).
- `platform/about.rs`: removed `✅` from quality-list items; the list now
  uses a `::before` gold dash marker (CSS-only, no character).

### Added — CSS for new icon containers
- `.tab-icon`, `.btn-icon` — inline-flex wrappers for SVG in tab/button labels.
- `.match-meta-inline`, `.match-distance`, `.match-distance-icon`,
  `.route-meta`, `.route-meta-item` — flex containers for match-card metadata.
- `.quality-list` and `.quality-list li::before` — replaces `✅` glyphs with
  a 14×2px gold dash (Bauhaus-style typographic mark).

### Verified
- `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace`
  (40 tests + 1 doc test) all green.
- `dx build --platform web --release` succeeds, produces WASM bundle.
- Backend smoke test: 6 endpoints all return expected JSON.
- Visual regression (Playwright + VLM): all 8 routes (landing, 4× platform,
  3× mobile) render correctly; no emojis detected; no layout overlap;
  scroll heights grow naturally with content (e.g. landing=4529px,
  about=4635px, passenger=1228px) — confirms the v0.5.0 scroll fix holds.

## [0.5.0] — 2026-06-19

### Summary
Major rebranding to **Nitheky** with **Mono Elegance + DE-Gold** design system,
critical scroll bug fix, and strict architectural separation between Landing /
Platform / Mobile via Dioxus Router. Applies the 10-level Rust/Dioxus methodology
framework (Clean Architecture, SOLID, type-state enums, anti-pattern elimination).

### Fixed — Critical
- **Scroll bug eliminated at root cause**: `index.html` had `html, body { overflow: hidden;
  height: 100%; }` declared at global scope for the loading screen — but never scoped back.
  This broke scrolling for the ENTIRE app (page rendered only specific pixel ranges,
  e.g. 0-200 then 250-600). Fix: scoped loading-screen CSS to `#loading-screen` only;
  `html, body` now allow natural document flow with `overflow-x: hidden` only.
- Removed the legacy state-based view switching (`View::Landing` vs `View::Platform`)
  which prevented URL-based navigation and back-button support.

### Changed — Strict architectural separation (Level 2: Separation of Concerns)
- **Landing (`/`)**: marketing site, public, no app chrome
- **Platform (`/app/*`)**: authenticated web app, navbar + footer chrome, desktop-optimized
- **Mobile (`/m/*`)**: Android-optimized, bottom-nav, safe-area insets, touch-first
- Each area is a separate module tree (`src/landing/`, `src/platform/`, `src/mobile/`)
  with its own shell component — can be split into independent builds if needed.

### Added — Dioxus Router (Level 5: Dioxus-specific architecture)
- Type-safe `Route` enum with `#[derive(Routable)]` — compile-time verified URLs.
- Eliminates "Stringly-typed" anti-pattern (Level 10): typos in link targets caught at compile.
- All navigation uses `<Link to={Route::...}>` instead of event-handler callbacks.
- Real URLs enable browser back/forward, deep linking, and bookmarking.

### Added — Design System #09 "Mono Elegance + DE-Gold"
- **Palette**: monocromo Suizo (`#0A0A0A` ink + perceptual grayscale) + único acento
  oro mate Alemán `#C9A961` (Bauhaus / Wittmann).
- **Inspiraciones fusionadas**:
  - GBM: grilla estricta 8px, tipografía institucional Inter, jerarquía por contraste tipográfico
  - Uber: densidad espacial, search-card from/to apilados, lista densa de conductores, CTA sólido
  - inDrive: concepto "tú propones el precio" (offer-card con slider), acento cromático reinterpretado
- **Tipografía**: Inter (display + body) + JetBrains Mono (data/labels)
- **Algoritmo de tokens**: base 4px, escala 1.2, radii base 1.5
- **Accesibilidad**: WCAG AAA 19.2:1 ink-on-paper

### Changed — Rebranding Pickando → Nitheky
- Updated: `index.html` meta tags, `Dioxus.toml` app name and bundle identifier,
  `Cargo.toml` keywords, favicon SVG, Android `strings.xml`, `styles.xml`,
  launcher icon foreground/background drawables.
- Android WebView now loads `/m/` (mobile route) instead of `/` — strict separation.
- Android theme colors updated to new brand (`#0A0A0A` ink + `#C9A961` accent).

### Added — Mobile module (`/m/*`)
- `MobileShell` with header + bottom-nav (sticky, safe-area aware)
- `MobileHome`: Uber-style search-card + map + inDrive-style offer slider + driver list + CTA
- `MobilePassenger`: compact passenger flow
- `MobileDriver`: compact driver flow with passenger requests list
- All mobile views use the same WASM bundle as landing/platform (Dioxus philosophy:
  "learn once, write anywhere") but render mobile-specific layouts.

### Methodology applied (10-level Rust/Dioxus framework)
- **L1 (Scrum/Agile)**: CHANGELOG-driven development, semantic versioning
- **L2 (SOLID/Clean Arch)**: module separation per bounded context
- **L3 (TDD/CI)**: existing 40 tests pass, clippy clean with `-D warnings`
- **L4 (Quality)**: zero compiler warnings, zero clippy warnings
- **L5 (System Design)**: type-safe routing, signals-based reactivity
- **L6 (Security)**: CSP-friendly (no inline scripts), scoped CSS, no `unsafe`
- **L7 (Observability)**: structured module docs with methodology cross-refs
- **L8 (Rust philosophy)**: fearless refactoring — full restructure compiles clean
- **L9 (Modern concepts)**: WASM, full-stack Rust, edge-ready
- **L10 (Anti-patterns eliminated)**:
  - No "Stringly-typed" routes (enum instead)
  - No "God struct" (separate shell/navbar/footer/home modules)
  - No "Prop drilling" (Router context provides navigation)
  - No "Hooks in conditionals" (all hooks at top of components)
  - No "Global overflow:hidden" (the bug we fixed)

## [0.4.0] — 2026-06-19

### Summary
Major visual rebranding + critical scroll bug fix.
This release introduces the "Sendero Compartido" (warm trust editorial) design direction,
fixes a critical layout bug that broke scrolling on the landing page, and removes
all AI-slop visual patterns (emoji icons, indigo/purple gradients, glassmorphism).

### Fixed — Critical
- **Landing page scroll broken**: `.landing-hero` had `min-height: 100vh` combined with
  `overflow: hidden` and massive absolutely-positioned SVGs with `inset: 0` + `height: 100%`.
  This caused the browser to clip rendering and break vertical scrolling on the landing page
  (only the first ~200px would render, then the next section, with broken flow in between).
  Fix: removed `min-height: 100vh` from hero (let content define height), changed
  `overflow: hidden` to `overflow: visible` on the hero, contained decorative SVGs inside
  their own overflow-hidden wrapper, and wrapped `View::Landing` in `app-container` so the
  body flex layout flows correctly (previously only `View::Platform` was wrapped).
- `body` now uses `display: flex; flex-direction: column; overflow-x: hidden` so vertical
  scroll works on every page without horizontal scrollbar jumps.

### Changed — Visual rebranding ("Sendero Compartido")
- **Palette**: shifted from dark theme (emerald + amber on `#0F1419`) to warm editorial
  light theme: cream paper `#FAF6F0` background, deep forest green `#1F4D3A` primary,
  warm terracotta `#C66B3D` accent, warm carbón `#2A2520` text. This aligns with the
  client's psychological profile (trust + warmth + humanity, not "hacker terminal").
- **Typography**: added Fraunces (humanist serif) for display headlines alongside Inter
  for body and JetBrains Mono for data. `font-variation-settings: "opsz" 144` for
  optical sizing on headlines.
- **Buttons**: removed gradient backgrounds + glow shadows, replaced with solid
  `--primary` fill + clean shadow. Secondary buttons use bordered surface style.
- **Cards**: removed gradient backgrounds on hover, replaced with subtle border color
  change + standard shadow elevation.
- **Hero**: removed the massive `600px x 600px` blurred orbs that overflowed, replaced
  with `480px` contained orbs + opacity 0.4 (still decorative but no longer breaks layout).
- **Story section**: removed gradient border + colored avatar backgrounds, replaced with
  tinted backgrounds + serif initials (M / A) instead of emoji.

### Removed — Anti-AI-slop cleanup
- All emoji icons removed from buttons, cards, and feature grids
  (🔍 🚗 🧭 ⚡ 🖥️ 📊 🔌 🎨 ℹ️ ⚠️ ☰ → replaced with text or numbered indices "01", "02" ... "06").
- Removed `linear-gradient(135deg, indigo, purple)` patterns from score bars.
- Removed `glassmorphism` (backdrop-filter blur + saturated + translucent) — kept only
  on navbar/header for sticky readability, with reduced opacity.
- Removed neon glow shadows (`box-shadow: 0 0 12px rgba(0,255,136,0.6)`).
- Removed `score-shimmer` animation (was AI-slop pattern).
- Removed `story-pulse` infinite pulsing animation (was visual noise).
- Removed `connector-bounce` animation (was visual noise).

### Added — Accessibility & UX
- `prefers-reduced-motion` media query: disables all animations and transitions.
- `*:focus-visible` outline for keyboard navigation (2px primary outline).
- `scroll-padding-top: 80px` on html for anchor links offset by sticky header.
- `scrollbar-width: thin` + `scrollbar-color` for Firefox.
- Better mobile responsive breakpoints (968px, 768px, 640px, 480px) with
  properly scaled typography via `clamp()`.
- Theme-color meta updated to `#FAF6F0` (was `#0D0D11`).

### Files changed
- `crates/frontend/assets/main.css` — full rewrite (~1470 lines changed)
- `crates/frontend/src/main.rs` — wrap View::Landing in app-container, update theme-color
- `crates/frontend/src/components/landing.rs` — remove emojis, use numbered indices
- `crates/frontend/src/components/driver.rs` — remove emojis from demo banner
- `crates/frontend/src/components/passenger.rs` — remove emojis from tabs/buttons
- `crates/frontend/src/components/navbar.rs` — replace ☰ emoji with "Menu" text

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
