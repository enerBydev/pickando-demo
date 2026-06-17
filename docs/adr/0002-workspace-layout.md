# ADR-0002: Workspace layout — 3 crates (shared, backend, frontend)

- Status: Accepted
- Date: 2026-06-13
- Deciders: René Mendoza
- Tags: architecture, workspace, crates

## Context

We have three distinct concerns:

1. **Domain models and pure algorithms** (Route, User, matching engine,
   haversine, geohash encoding). These have no I/O dependencies and must be
   usable by every other crate.
2. **The HTTP/WebSocket server** (Axum + Tokio + tower). I/O-bound,
   async-heavy, talks to the network.
3. **The user interface** (Dioxus). Renders state, calls the backend over
   HTTP/WebSocket, compiles to WASM.

We need to organize these so that:

- Domain logic is reused, not duplicated.
- Backend and frontend can be built and deployed independently.
- Tests for each layer run in isolation.
- Compilation of one layer doesn't force recompilation of unrelated layers.

## Decision

Use a **Cargo workspace** with three crates:

```
pickando-demo/
├── Cargo.toml              # [workspace] with members
└── crates/
    ├── shared/             # pickando-shared — pure domain logic
    │   └── src/{lib,models,matching}.rs
    ├── backend/            # pickando-backend — Axum HTTP/WS server
    │   └── src/{main,routes,ws,state}.rs
    └── frontend/           # pickando-frontend — Dioxus UI
        └── src/{main,api,components/*}.rs
```

Dependency graph:

```
shared   ←   backend
shared   ←   frontend
backend  ↔   frontend   (only at runtime, via HTTP/WS)
```

`backend` and `frontend` never import from each other directly — they share
types only via `shared`.

## Alternatives Considered

### Single crate

- ❌ Forces recompiling everything on any change.
- ❌ No way to enforce layer boundaries via the type system.
- ❌ WASM target conflicts with backend's Tokio runtime deps.

### Two crates (server + client)

- ❌ No clean home for domain models — they end up duplicated in both.
- ❌ Or in one of the two, creating an artificial dependency.

### Hexagonal / ports-and-adapters with 5+ crates

- ✅ Maximum isolation.
- ❌ Overkill for a demo. Premature abstraction — YAGNI.
- ❌ Slower iteration for a 3-person-equivalent project.

## Consequences

**Positive:**
- `shared` compiles in <1s — fast inner loop for matching algorithm work.
- Backend and frontend can be built in parallel in CI.
- Clear ownership: each crate has one responsibility.
- `shared` is publishable to crates.io if we ever want to extract it.
- Frontend's WASM build is not contaminated by Tokio/axum deps.

**Negative:**
- Some boilerplate: structs in `shared` must be `Serialize + Deserialize`
  and `Clone`, even if only one side uses them.
- Refactoring an API contract touches 3 crates.

**Neutral:**
- The `shared` crate is the contract surface — changes there are the most
  impactful and deserve the most review.

## Compliance

- `Cargo.toml` at workspace root lists exactly 3 members.
- `cargo tree -p pickando-backend` shows no path dep on `pickando-frontend`.
- `cargo tree -p pickando-frontend` shows no path dep on `pickando-backend`.
