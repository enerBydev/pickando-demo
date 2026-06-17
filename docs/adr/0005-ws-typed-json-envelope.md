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
