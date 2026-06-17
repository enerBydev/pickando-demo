# ADR-0003: In-memory state store for the demo

- Status: Accepted
- Date: 2026-06-13
- Deciders: René Mendoza
- Tags: backend, state, persistence, demo

## Context

The Pickando demo needs to:

- Show seeded sample routes from the first request (no setup required).
- Persist routes created by visitors during a demo session.
- Be deployable on Railway's free tier without external services.
- Reset to a clean state on redeploy (no migration headaches).

A production system would use PostgreSQL for durability and Redis for
real-time session/cache. But for a *demo* whose purpose is to prove the
Rust + Dioxus + Axum stack works end-to-end, those dependencies add:

- ~30 seconds of cold-start time.
- A Railway paid tier for managed Postgres.
- Migration setup, connection pooling, retry logic.
- A "demo seed" script that needs to run before the demo is usable.

## Decision

Use an **in-memory store** for the demo:

```rust
pub struct AppState {
    pub routes: Arc<RwLock<Vec<Route>>>,
    pub start_time: Instant,
    pub route_counter: Arc<AtomicU64>,
}
```

- `tokio::sync::RwLock<Vec<Route>>` — concurrent reads, serialized writes.
- `AtomicU64` counter for monotonic route IDs.
- Seeded with 6 routes (CDMX + Monterrey) at `main()` startup.

The store is wrapped in `Arc` and injected into every Axum handler via
`State<Arc<AppState>>`.

## Alternatives Considered

### SQLite (embedded)

- ✅ Single-file, no external service.
- ✅ Persistent across redeploys if volume-mounted.
- ❌ Adds `rusqlite` dep with C bindings — complicates the WASM-friendly
  story.
- ❌ Requires migrations and schema versioning for a demo.
- ❌ Cold start slower (open file, run migrations).

### PostgreSQL (managed)

- ✅ Production-ready, durable.
- ❌ Requires Railway paid tier or external Neon/Supabase account.
- ❌ Connection pool, retry logic, migration framework — all extra code
  for a demo.
- ❌ Deploy review friction for the reviewer (Helder) — they have to
  set up a database before the demo works.

### Redis (managed)

- ✅ Fast, supports pub/sub for WebSocket fan-out.
- ❌ Same external-service concern as PostgreSQL.
- ❌ Adds an async Redis client dep that pulls in `tokio::net` features
  we don't otherwise need.

## Consequences

**Positive:**
- Zero external dependencies — `cargo run` and it works.
- Sub-second cold start on Railway.
- All demo data is fresh on each deploy — no stale state from yesterday's
  experiments.
- No credentials, no migration files, no connection pool tuning.
- Reviewer (Helder) can clone and run without configuring anything.

**Negative:**
- **Data is lost on every redeploy.** Acceptable for a demo; would be
  catastrophic for production.
- **No persistence across instances.** If Railway scales to 2 replicas,
  they won't share state. Acceptable for the demo's single-instance tier.
- **No crash recovery.** If the process dies, in-flight routes are gone.
- **Memory grows monotonically** with created routes. Mitigated by
  capping the seed list and the natural rate of demo usage.

**Neutral:**
- The `AppState` struct is designed to be swappable: replace
  `Arc<RwLock<Vec<Route>>>` with `PgPool` and the handler signatures
  barely change.

## Compliance

- `AppState` lives in `crates/backend/src/state.rs`.
- All handlers depend on `State<Arc<AppState>>` (or `AppState` methods),
  never on concrete collection types.
- When we promote to production in v0.3.0, this ADR will be superseded
  by an ADR-NNNN: "PostgreSQL via sqlx for production persistence".
