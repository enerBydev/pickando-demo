# Pickando Demo — REST API Reference

> Base URL (production): `https://pickando-demo-production.up.railway.app`
> Base URL (local): `http://localhost:3000`

All endpoints accept and return `application/json` unless otherwise noted.
WebSocket endpoint is `ws://` (or `wss://` in production) at `/ws`.

---

## Health & Telemetry

### `GET /api/v1/health`

Health check. Used by Railway and Docker `HEALTHCHECK` to verify the
backend is alive.

**Response 200:**

```json
{
  "status": "ok",
  "service": "pickando-backend",
  "version": "0.5.4",
  "stack": "Rust + Axum + Tokio (rustc 1.96)",
  "uptime_seconds": 42.51,
  "routes_count": 6,
  "memory_rss_mb": 3.82,
  "requests_served": 1024
}
```

---

### `GET /api/v1/stats`

Platform telemetry — counts of routes and ride requests by status,
uptime, and total requests served.

**Response 200:**

```json
{
  "routes_total": 7,
  "routes_published": 5,
  "routes_requested": 1,
  "routes_accepted": 0,
  "routes_started": 0,
  "routes_completed": 0,
  "routes_cancelled": 1,
  "ride_requests_total": 1,
  "ride_requests_pending": 1,
  "ride_requests_accepted": 0,
  "ride_requests_rejected": 0,
  "uptime_seconds": 120.45,
  "requests_served": 12,
  "avg_relevance_score": null
}
```

---

## Routes

### `GET /api/v1/routes`

List all routes currently in the in-memory store. Seeded at startup
with 6 CDMX/Monterrey routes; new routes can be added via `POST`.

**Response 200:** `Route[]`

```json
[
  {
    "id": "route-001",
    "driver_id": "driver-001",
    "origin_lat": 19.4326,
    "origin_lng": -99.1332,
    "dest_lat": 19.4512,
    "dest_lng": -99.1100,
    "origin_address": "Zócalo, CDMX",
    "dest_address": "Polanco, CDMX",
    "departure_time": "08:00",
    "seats_available": 3,
    "status": "published",
    "geohash": "9g3w81",
    "created_at_ms": 1781670333000
  }
]
```

---

### `POST /api/v1/routes`

Create a new route. Broadcasts a `route_created` WebSocket event.

**Request body:** `CreateRouteRequest`

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `origin_address` | string | ✅ | Non-empty |
| `dest_address` | string | ✅ | Non-empty |
| `departure_time` | string | ✅ | `HH:MM` or ISO-8601 |
| `seats_available` | u32 | ✅ | 1..=6 |
| `driver_id` | string? | ❌ | Defaults to `"demo-driver"` |
| `origin_lat` | f64? | ❌ | Defaults to 19.4326 (CDMX) |
| `origin_lng` | f64? | ❌ | Defaults to -99.1332 |
| `dest_lat` | f64? | ❌ | Defaults to 19.4512 |
| `dest_lng` | f64? | ❌ | Defaults to -99.1100 |

**Response 201:** `Route`

**Errors:**
- `400 Bad Request` — empty address, invalid seats, invalid coordinates
- `500 Internal Server Error` — server-side failure

---

### `GET /api/v1/routes/{id}`

Get a single route by ID.

**Path params:**
- `id` (string) — the route identifier (e.g. `route-001`)

**Response 200:** `Route`

**Errors:**
- `404 Not Found` — route with that ID does not exist

---

### `DELETE /api/v1/routes/{id}`

Cancel a route. Marks the route's status as `Cancelled`. Does not
remove it from the store (history is preserved for the demo).
Broadcasts a `route_cancelled` WebSocket event.

**Path params:**
- `id` (string) — the route identifier

**Response 200:** `WsMessage` (type: `route_cancelled`)

**Errors:**
- `404 Not Found` — route does not exist
- `409 Conflict` — route is already cancelled or already completed

---

## Ride Requests

### `POST /api/v1/routes/{id}/request`

A passenger requests to join a published route. Validates seat
availability, marks the route's status as `Requested`, creates a
`RideRequest` record, and broadcasts a `ride_request` WebSocket event
to any subscribed drivers.

**Path params:**
- `id` (string) — the route identifier

**Request body:** `CreateRideRequest`

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `passenger_name` | string | ✅ | Non-empty |
| `seats_requested` | u32 | ✅ | 1..=6 |
| `passenger_id` | string? | ❌ | Auto-generated UUID if omitted |

**Response 201:** `RideRequest`

```json
{
  "id": "req-abc123...",
  "route_id": "route-001",
  "passenger_id": "passenger-xyz789...",
  "passenger_name": "María",
  "seats_requested": 1,
  "status": "pending",
  "created_at_ms": 1781670333111
}
```

**Errors:**
- `400 Bad Request` — empty passenger_name, invalid seats
- `404 Not Found` — route does not exist
- `409 Conflict` — route is not in `Published` status, or not enough seats

---

## Matching

### `POST /api/v1/match`

Find routes matching a passenger's location. The core feature of
Pickando.

Uses geohash prefix filtering, then Haversine refinement, then
direction similarity (cosine of bearing vectors) and time-window
compatibility if those optional fields are provided.

**Request body:** `MatchRequest`

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `lat` | f64 | ✅ | -90..=90 |
| `lng` | f64 | ✅ | -180..=180 |
| `radius_km` | f64? | ❌ | 0.1..=200, defaults to 5.0 |
| `passenger_bearing_deg` | f64? | ❌ | 0..=360 (compass direction) |
| `time_window_minutes` | i64? | ❌ | 1..=480 (± around passenger time) |
| `passenger_departure_time` | string? | ❌ | `HH:MM` or ISO-8601 |

**Response 200:** `MatchResult[]`

```json
[
  {
    "route": { /* Route object */ },
    "distance_km": 0.0,
    "direction_similarity": 1.0,
    "time_compatibility": 1.0,
    "relevance_score": 1.0
  }
]
```

The relevance score is a weighted blend:
```
relevance = 0.5 * distance_score + 0.3 * direction_score + 0.2 * time_score
```

Results are sorted by relevance descending.

**Errors:**
- `400 Bad Request` — lat/lng out of range, radius out of range

---

## WebSocket

### `GET /ws`

Bidirectional WebSocket connection. The server:

1. Sends a `connected` welcome on open.
2. Sends a `live_tick` event every 5 seconds with uptime and active
   routes count.
3. Forwards all broadcast events from `AppState.ws_broadcaster`:
   - `route_created` — when a new route is POSTed
   - `route_cancelled` — when a route is DELETEd
   - `ride_request` — when a passenger requests to join a route
4. Echoes any text message the client sends as an `echo` event.

**Wire format:** JSON envelope `WsMessage`

```json
{
  "type": "live_tick",
  "message": "Tick #42s — servidor activo",
  "data": {
    "uptime_seconds": 42,
    "server_time": 1781670375,
    "active_routes": 6
  }
}
```

**Connect with `wscat`:**

```bash
npm install -g wscat
wscat -c wss://pickando-demo.up.railway.app/ws
```

---

## Status lifecycle

```
Published → Requested → Accepted → Started → Completed
                ↓           ↓
             Cancelled   Cancelled
```

All transitions are validated server-side. Clients cannot skip states.

---

## Error format

All errors return a plain-text body with a human-readable message:

```
HTTP 400 Bad Request
origin_address and dest_address must not be empty
```

The HTTP status code is the authoritative signal — the body is for
humans and debugging.
