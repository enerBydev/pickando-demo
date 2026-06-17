# ADR-0007: Request Body Validation Strategy

- **Estado:** Accepted
- **Fecha:** 2026-06-17
- **Deciders:** René Mendoza (enerBydev)
- **Tags:** backend, security, validation, serde

## Contexto

During the forensic audit of v0.2.1 (see `hallazgos_2.md`), we discovered that
`POST /api/v1/match -d '[1,2,3]'` returned `200 []` instead of `422 Unprocessable Entity`.

**Root cause:** serde deserializes arrays as the seq representation of structs by default.
`[1,2,3]` was being silently deserialized as `MatchRequest { lat: 1, lng: 2, radius_km: 3 }`,
which then ran the matching engine with bogus values and returned an empty array.

This is a known serde foot-gun: without `#[serde(deny_unknown_fields)]` and without
explicit `is_object()` validation, any JSON array of the right length can be deserialized
into a struct, bypassing all field-level validation.

Additionally, `departure_time: "not-a-time"` was accepted because the field was typed
as `String` without any format validation, and `radius_km: -5` was silently clamped to
`0.1` by `sanitized().clamp()` instead of being rejected.

## Decisión

We adopt a **defense-in-depth validation strategy** with three layers:

### Layer 1: `#[serde(deny_unknown_fields)]` on all request types

Applied to `MatchRequest`, `CreateRouteRequest`, `CreateRideRequest`. This prevents
serde from accepting arrays as seq representation and rejects unknown fields.

### Layer 2: Explicit `is_object()` check in handlers

All POST handlers accept `Json<serde_json::Value>` (not `Json<T>`) and validate
`value.is_object()` before deserializing. This catches the array case even if
Layer 1 fails (e.g., for new request types added in the future).

```rust
pub async fn create_route(
    State(state): State<Arc<AppState>>,
    Json(value): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<Route>), (StatusCode, String)> {
    if !value.is_object() {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "request body must be a JSON object".into()));
    }
    let body: CreateRouteRequest = serde_json::from_value(value).map_err(|e| {
        (StatusCode::UNPROCESSABLE_ENTITY, format!("invalid CreateRouteRequest: {e}"))
    })?;
    // ... field-level validation follows
}
```

### Layer 3: Field-level validation in the handler

Each field is validated for range, format, or business rules:
- `origin_lat`/`origin_lng`/`dest_lat`/`dest_lng`: must be in `[-90, 90]` / `[-180, 180]`
- `seats_available`: must be in `[1, 6]`
- `departure_time`: must be HH:MM, HH:MM:SS, or ISO-8601 (via `validate_departure_time()`)
- `radius_km`: must be finite, `> 0`, `<= 200`
- `passenger_name`: must not be empty
- `passenger_bearing_deg`: must be finite

### Layer 4 (future): Declarative validation with `garde` crate

Deferred to a future version. The `garde` crate would allow declarative validation
via `#[garde(range(min = -90.0, max = 90.0))]` attributes on struct fields, eliminating
the need for manual validation in handlers. Not adopted in v0.3.0 to avoid adding a
new dependency mid-sprint.

## Consecuencias

### Positivas
- All non-object JSON bodies are rejected with `422 Unprocessable Entity`.
- All out-of-range values are rejected with `400 Bad Request` (no silent clamping).
- Defense-in-depth: even if one layer fails, the others catch the issue.
- 11 new regression tests cover all validation paths.

### Negativas
- Slightly more boilerplate in handlers (`Json<serde_json::Value>` + `from_value`).
- Tests must serialize structs to `serde_json::Value` before passing to handlers.
- Manual validation is repetitive — `garde` would be cleaner (Layer 4).

### Neutrales
- `serde_json::Value` intermediate step adds negligible overhead (<1μs per request).
- Error messages are user-friendly and include the field name.

## Alternativas consideradas

### A: `axum::extract::rejection::JsonRejection` handling
Rejected: doesn't solve the array-as-struct problem. serde happily deserializes
`[1,2,3]` into a struct without triggering a rejection.

### B: Custom `FromRequest` extractor
Rejected: too much boilerplate for the demo. The `Json<Value>` + `is_object()` pattern
is simpler and equally effective.

### C: `garde` crate (Layer 4)
Deferred: would be cleaner but adds a new dependency. Will revisit in a future version
when the validation rules become more complex.

## Referencias

- `hallazgos_2.md` §6 — BUG #5: array body accepted
- `crates/shared/src/models.rs` — `#[serde(deny_unknown_fields)]` attributes
- `crates/backend/src/routes.rs` — `Json<serde_json::Value>` handlers
- `crates/backend/src/routes.rs::validate_departure_time` — field-level validation
- serde docs: <https://serde.rs/attr-deny-unknown-fields.html>
