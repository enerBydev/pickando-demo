# ADR-0005: WebSocket protocol — typed JSON envelope

- Status: Accepted
- Date: 2026-06-17
- Deciders: René Mendoza
- Tags: backend, frontend, websocket, protocol

## Context

The WebSocket endpoint (`GET /ws`) needs to carry multiple kinds of
messages:

- `connected` — server greeting on connect.
- `live_tick` — periodic telemetry push (every 5 s).
- `echo` — server echo of client messages.
- `route_created` — broadcast when a new route is posted (planned).
- `route_cancelled` — broadcast when a route is removed (planned).
- `position_update` — live GPS position of a driver (planned for v0.3).
- `error` — protocol or validation errors.

We need a wire format that:

- Is debuggable in the browser devtools (human-readable).
- Carries typed payloads (the frontend should not parse JSON manually
  into `serde_json::Value` and hope).
- Supports forward evolution (new message types added without breaking
  old clients).
- Works with the same `serde` types we use for HTTP.

## Decision

Use a **typed JSON envelope** defined in `pickando-shared`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}
```

Wire format example:

```json
{
  "type": "live_tick",
  "message": "Tick #42s — servidor activo",
  "data": {
    "uptime_seconds": 42,
    "server_time": 1718600000,
    "active_routes": 6
  }
}
```

The `type` field is a string discriminant. The frontend matches on it
and deserializes `data` into the appropriate struct.

## Alternatives Considered

### Binary protocol (MessagePack / CBOR / Protobuf)

- ✅ Smaller wire size (~30-50% smaller).
- ✅ Faster deserialization.
- ❌ Not debuggable in devtools — kills the "look, it works" demo flow.
- ❌ Adds a binary dependency (`rmp-serde` or `prost`).
- ❌ Protobuf requires code generation — complicates the build.
- ❌ YAGNI for a demo whose payload is a few hundred bytes per message.

### Tagged enum serialization (`#[serde(tag = "type", content = "data")]`)

- ✅ Even more type-safe — the compiler enforces exhaustive matching.
- ❌ Less flexible: adding a variant is a breaking change for old
  clients (they get `unknown variant` errors).
- ❌ The envelope's `message` human-readable string is awkward to fit
  into a pure enum.

### Untyped `serde_json::Value` everywhere

- ✅ Maximum flexibility.
- ❌ Zero type safety — bugs only surface at runtime.
- ❌ Frontend has to repeat parsing logic for every message.

### Server-Sent Events (SSE) instead of WebSocket

- ✅ Simpler server, one-way is enough for ticks.
- ❌ Cannot receive client messages — kills the "bidirectional" demo.
- ❌ SSE has connection limits in older browsers.

## Consequences

**Positive:**
- Every WS message is debuggable with `wscat` or browser devtools — perfect
  for a demo where the reviewer is verifying "is this real?".
- Adding a new message type is additive — old clients ignore unknown
  `type` values (with a logged warning).
- `WsMessage` is shared between backend and frontend via `pickando-shared`.
- The `message` field doubles as a human-friendly log line.

**Negative:**
- Slightly larger payload than binary (a few extra bytes for the JSON
  keys). For a demo at 1 message/5s: negligible.
- The `data: Option<Value>` field forces a second deserialization step
  on the client. We mitigate by providing typed helper constructors
  (`WsMessage::live_tick(...)`, etc.) on the backend.

**Neutral:**
- The protocol is versioned via the `protocol` field in the `connected`
  welcome message (`"pickando-ws-v1"`). When v2 lands, clients can opt in.

## Compliance

- All WS messages are constructed via `WsMessage { msg_type, message, data }`.
- The `connected` welcome message includes `"protocol": "pickando-ws-v1"`.
- The frontend deserializes via `serde_json::from_str` into typed structs.
- No raw string manipulation on either side.

## Update — v0.5.4 (2026-06-20): event types actually observed in production

The original Context section listed `position_update` and `error` as
"planned for v0.3" / "validation errors". As of v0.5.4 the **actual
event types** sent by the backend (verified in
`crates/backend/src/ws.rs` and `crates/backend/src/routes.rs`) are:

| `type`             | Origin                       | Trigger                                           | Sent to              |
|--------------------|------------------------------|---------------------------------------------------|----------------------|
| `connected`        | `ws::ws_handler` (on open)   | New WS client connects                            | the new client only  |
| `live_tick`        | `ws::ws_handler` (5 s task)  | Periodic telemetry — uptime, server time, routes  | the new client only  |
| `echo`             | `ws::ws_handler`             | Client sends any text message                     | the same client      |
| `route_created`    | `routes::create_route`       | `POST /api/v1/routes` succeeds                    | **all** WS clients   |
| `route_cancelled`  | `routes::cancel_route`       | `DELETE /api/v1/routes/{id}` succeeds             | **all** WS clients   |
| `ride_request`     | `routes::request_ride`       | `POST /api/v1/routes/{id}/request` succeeds       | **all** WS clients   |

The `position_update` event (originally listed as "planned for v0.3")
**has not been implemented** — there is no live-GPS tracking in the
demo. The `error` event **has not been implemented** either; protocol
and validation errors are returned over HTTP (4xx) for the matching
endpoints, and the WS layer simply drops malformed inbound frames.

This is recorded here so future readers don't go hunting for a
`position_update` dispatcher that doesn't exist. When live-GPS is added
(in a real production version, not the demo), it should reuse the same
envelope (just add `position_update` to the dispatch table) — the wire
format itself does not need to change.

### Frontend consumption (`crates/frontend/src/platform/passenger.rs`)

The frontend's `WebSocketDemo` component opens the connection, sends the
`connected` event handler's `data.protocol` to a status signal, and
renders every subsequent `type` into a log line. The matcher uses
`type === "live_tick" | "route_created" | …` discriminant checks, never
raw `JSON.parse` into an untyped object.

### v0.5.5 — WASM callback fix (commit f8143aa)

A separate fix (commit `f8143aa`, shipped as part of v0.5.5) addressed
a `RuntimeError: unreachable` trap that fired once per inbound WS
message in the WASM build. Root cause: the four `wasm_bindgen::Closure`
callbacks (`onopen`, `onmessage`, `onclose`, `onerror`) called
`Signal::write()` directly from the JS event loop, with no Dioxus
runtime on the thread-local `RUNTIMES` stack — triggering the explicit
panic in `Runtime::current()` (dioxus-core 0.7.9, `runtime.rs:96`). The
fix wraps each callback body in `runtime.in_scope(scope, || { … })`,
capturing `runtime` + `scope` at the top of the `connect` handler. This
is the canonical pattern recommended by the panic message itself. The
wire format documented above is unchanged — the bug was purely in the
client-side callback marshalling, not in the protocol.
