# ADR-0001: Use Rust + Dioxus + Axum as the technology stack

- Status: Accepted
- Date: 2026-06-13
- Deciders: René Mendoza (enerBydev), Helder D. (client)
- Tags: architecture, stack, rust, dioxus, axum

## Context

Pickando is a same-direction local mobility platform. It requires:

- Real-time GPS tracking via WebSocket.
- Concurrent matching of routes under load.
- Multi-platform output: web + iOS + Android + desktop from one codebase.
- Payment integration (Stripe, MercadoPago) — class-critical reliability.
- Safety features (SOS, safety contacts) — must not crash under stress.

The client initially proposed Flutter + PHP/Node. The original proposal
matched that. After clarification of the concept (same-direction mobility,
not Uber-like taxi dispatch), we re-evaluated the stack.

## Decision

Use **Rust 1.96** as the single language, with:

- **Dioxus 0.7** for the frontend (compiles to WASM for web, native for
  desktop and mobile).
- **Axum 0.8** + **Tokio** for the backend.
- **PostgreSQL** + **Redis** for production persistence (planned for v0.3+).

## Alternatives Considered

### Option A — Flutter + NestJS

- ✅ Massive talent pool.
- ✅ Faster prototyping for juniors.
- ❌ Two codebases (Dart + TypeScript) — duplicate logic, double bugs.
- ❌ Flutter Web is heavy and limited.
- ❌ GC pauses perceptible in real-time UX.
- ❌ Two runtimes (Dart VM + V8) to reason about.
- ❌ No compile-time memory safety — null pointers, data races reach production.

### Option B — React Native + Node.js

- ✅ Largest ecosystem.
- ❌ JavaScript runtime errors are common.
- ❌ Native modules for WebSocket + GPS are fragile.
- ❌ Two languages (TS + native iOS/Android bridge for some features).
- ❌ Not viable for desktop without Electron bloat.

### Option C — Kotlin Multiplatform + Ktor

- ✅ Single language (Kotlin).
- ✅ JVM ecosystem.
- ❌ Kotlin/Native WASM target is still experimental.
- ❌ Compose Multiplatform mobile is mature but web is alpha.
- ❌ JVM startup time and memory footprint.

### Why Rust + Dioxus + Axum wins

- ✅ **One language, one type system, one compiler.** Matching logic,
  models, and validation live in `shared` and are reused verbatim by
  backend and all frontend targets.
- ✅ **Compile-time memory safety.** No null derefs, no data races, no
  use-after-free — verified by the borrow checker before the binary exists.
- ✅ **Fearless concurrency.** Tokio + `tokio::sync::RwLock` give us
  concurrent reads without explicit locking.
- ✅ **Zero-cost abstractions.** No GC pauses. Predictable latency for
  WebSocket streaming and GPS tracking.
- ✅ **6 platforms from 1 codebase.** WASM (web), Linux, Windows, macOS,
  Android (via Tauri/Dioxus mobile), iOS — all from the same Rust source.
- ✅ **Binaries are tiny.** 3-8 MB vs Flutter's 15-20 MB. Better UX on
  mobile data plans.

## Consequences

**Positive:**
- Massive reduction in duplication — `shared` crate used by every target.
- Compiler catches most bugs at build time — fewer production incidents.
- Type-safe API contract between frontend and backend (serde + shared types).
- Industry-leading runtime performance for the matching engine.

**Negative:**
- Smaller talent pool for future maintenance (mitigated by the borrow
  checker serving as a "second reviewer" and Rust's high learnability
  for senior engineers).
- Compile times are higher than JavaScript (mitigated by `sccache`,
  incremental builds, and CI caching).
- Dioxus 0.7 is younger than React/Flutter — some patterns evolve.

**Neutral:**
- Forces discipline: every refactor is intentional, not casual.

## Compliance

- All commits must build with `cargo build --workspace` clean.
- All PRs must pass `cargo clippy --workspace --all-targets -- -D warnings`.
- No `unsafe` blocks in workspace crates (verified by `cargo-geiger` in CI).
