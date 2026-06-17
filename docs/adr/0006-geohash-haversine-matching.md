# ADR-0006: Geohash + Haversine for the matching engine

- Status: Accepted
- Date: 2026-06-13
- Deciders: René Mendoza
- Tags: algorithm, matching, geospatial, performance

## Context

The core feature of Pickando is **matching passengers with drivers going
in the same direction**. Given:

- A passenger at `(lat, lng)` looking within `radius_km`.
- A list of driver routes with origin, destination, departure time, seats.

We need to return routes that are:

1. **Geographically near** the passenger (within `radius_km` of the route's
   origin).
2. **Going in a compatible direction** (the route's bearing roughly matches
   the passenger's intended direction).
3. **Time-compatible** (departure within an acceptable window).
4. **Have seats available**.
5. **Sorted by relevance** (closest + best direction match first).

This must run **fast** — sub-50 ms for 1k routes — to keep the demo feeling
instant.

## Decision

Implement matching in pure Rust (no PostGIS, no external geospatial DB):

### Step 1 — Geohash filter

Encode each route's origin as a geohash of length 6 (~0.6 km × 0.6 km cell).
Encode the passenger's location the same way. Compare shared prefix length
to quickly eliminate routes that are too far:

```
common_prefix | approx_distance
6 chars       | ~0.6 km
5 chars       | ~2.4 km
4 chars       | ~20 km
3 chars       | ~156 km
< 3           | too far — reject
```

This is O(1) per route — a string prefix comparison.

### Step 2 — Haversine refinement

For routes that pass the geohash filter, compute the great-circle distance
via Haversine:

```rust
fn haversine_km(lat1, lng1, lat2, lng2) -> f64 {
    let r = 6371.0; // Earth mean radius in km
    let dlat = (lat2 - lat1).to_radians();
    let dlng = (lng2 - lng1).to_radians();
    let a = (dlat / 2).sin().powi(2)
          + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlng / 2).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r * c
}
```

Reject routes with `haversine_km > radius_km`.

### Step 3 — Direction similarity (planned v0.2)

Compute the **bearing** of the route (origin → destination) and the
passenger's intended bearing (passenger → destination proxy, or just
passenger → route.origin for now). Cosine similarity between the two
unit-bearing vectors gives a score in `[-1, 1]`:

- `1.0` = identical direction
- `0.0` = perpendicular
- `-1.0` = opposite direction

### Step 4 — Relevance score

```
relevance = 1.0 / (distance_km + 1.0)
```

Distance-weighted inverse — closer routes score higher. Direction and time
scores will be combined as a weighted average in v0.2.

## Alternatives Considered

### PostGIS `ST_DWithin` + `ST_Distance`

- ✅ Battle-tested, accurate.
- ❌ Requires PostgreSQL + PostGIS extension — heavy for a demo.
- ❌ Cannot run in `pickando-shared` (no DB in that crate by design).
- ❌ Latency: every match request becomes a DB round-trip.

### R-tree (rstar crate)

- ✅ Faster than geohash for static datasets.
- ❌ Build complexity O(n log n) on every state mutation.
- ❌ Less intuitive to debug than a string prefix.
- ❌ YAGNI for the demo's dataset size (~100 routes max).

### KD-tree on lat/lng directly

- ❌ Euclidean distance on lat/lng is wrong — Earth is a sphere.
- ❌ Same re-build problem as R-tree.

### H3 hexagonal indexing (Uber's library)

- ✅ Excellent for ride-share.
- ❌ Adds `h3o` crate with C dependency (binds to H3 C lib).
- ❌ Overkill for the demo.

## Consequences

**Positive:**
- Zero external geospatial dependencies — `pickando-shared` stays pure.
- Sub-millisecond matching for the demo's dataset size.
- Algorithm is unit-testable in isolation (no DB, no network).
- Geohash is human-debuggable — you can eyeball `"9ed9j"` for CDMX.
- Haversine is the industry standard for great-circle distance.

**Negative:**
- Geohash prefix matching has **edge cases at cell boundaries**: two
  points 100m apart but in different 6-char cells get filtered out.
  Mitigated: we use 6-char geohash (~0.6 km cells) and a generous radius
  filter, then let Haversine make the final call.
- Haversine assumes a spherical Earth — actual error up to 0.5%. For a
  demo: irrelevant. For surveying: would need Vincenty's formulae.
- Direction similarity is currently a simple heuristic. The full
  implementation (cosine of bearings) is tracked in CHANGELOG for v0.2.

**Neutral:**
- The matching module is pure — it takes `&[Route]` and returns
  `Vec<MatchResult>`. Swapping in a PostGIS-backed version is a drop-in
  replacement at the call site.

## Compliance

- All geospatial code lives in `crates/shared/src/matching.rs`.
- No `unsafe` blocks — pure safe Rust.
- Property-based tests (`proptest`) verify: haversine symmetry,
  triangle inequality, and that identical inputs produce `0.0` distance.
- Benchmarks (`criterion`) measure throughput at 100/1k/10k routes.
