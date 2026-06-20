//! WebSocket handler.
//!
//! Each connected client:
//! 1. Receives a `connected` welcome on open.
//! 2. Receives periodic `live_tick` events (every 5 s) with active routes count.
//! 3. Receives every broadcast event from `AppState.ws_broadcaster`
//!    (`route_created`, `route_cancelled`, `ride_request`, etc.).
//! 4. Has its text messages echoed back via an `echo` message.
//!
//! ## Origin validation
//!
//! In production (`PICKANDO_DEV` unset or != "1"), the WebSocket upgrade is
//! rejected with HTTP 403 unless the request carries an `Origin` header
//! matching the same allow-list used by `build_cors_layer` in `main.rs`.
//! This prevents any malicious website from opening
//! `wss://pickando-demo-production.up.railway.app/ws` and observing all
//! `route_created` / `ride_request` / `live_tick` broadcasts (data
//! exfiltration vector per Security audit 8-a P2 / A01).
//!
//! In dev mode (`PICKANDO_DEV=1`), the check is skipped — this allows
//! curl-based smoke tests (which don't send `Origin`) and `ws://localhost:*`
//! connections from local frontends. See the README's WebSocket smoke-test
//! section.
//!
//! Browsers always send `Origin` on `WebSocket` upgrades; non-browser
//! clients (curl, server-to-server) typically do not. We therefore reject
//! no-origin requests in production.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use futures_util::{SinkExt, StreamExt};
use pickando_shared::models::WsMessage;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

use crate::state::AppState;

/// GET /ws — WebSocket endpoint.
///
/// Establishes a bidirectional WebSocket connection. Sends a welcome
/// message on connect, echoes incoming messages, emits periodic
/// "live" telemetry events, and fans out any broadcast events from
/// the app state (e.g. when a new route is created).
///
/// Returns 403 if the `Origin` header does not match the allow-list in
/// production. In dev mode (`PICKANDO_DEV=1`), the check is skipped.
pub async fn ws_handler(
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
    headers: HeaderMap,
) -> Response {
    let is_dev = is_dev_mode();
    if !is_origin_allowed(&headers, is_dev) {
        tracing::warn!(
            "WebSocket upgrade rejected: Origin header missing or not allowed (dev_mode={is_dev})"
        );
        return (StatusCode::FORBIDDEN, "origin not allowed").into_response();
    }
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Returns `true` when `PICKANDO_DEV=1` is set in the environment.
///
/// Extracted as a helper so tests can call [`is_origin_allowed`] with an
/// explicit `is_dev` argument rather than mutating process-wide env vars
/// (which would race with parallel test runs).
fn is_dev_mode() -> bool {
    std::env::var("PICKANDO_DEV").ok().as_deref() == Some("1")
}

/// Decide whether the request's `Origin` header is allowed to upgrade.
///
/// - In dev mode (`is_dev = true`), always returns `true`. This lets curl
///   and other non-browser clients (which don't send `Origin`) connect to
///   `ws://localhost:*` for smoke testing, and lets the WASM frontend
///   connect from any localhost port without enumerating them.
///
/// - In production (`is_dev = false`), returns `true` only when the
///   `Origin` header is present and matches one of [`crate::ALLOWED_ORIGINS`].
///   Missing `Origin` (curl, server-to-server) is rejected — browsers
///   always send `Origin` on WS upgrades, so a missing `Origin` in prod
///   is a strong signal of a non-browser client which should not be
///   observing the broadcast stream.
fn is_origin_allowed(headers: &HeaderMap, is_dev: bool) -> bool {
    if is_dev {
        return true;
    }
    match headers.get(axum::http::header::ORIGIN) {
        Some(origin) => origin
            .to_str()
            .map(|s| crate::ALLOWED_ORIGINS.contains(&s))
            .unwrap_or(false),
        None => false,
    }
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let connected_at = Instant::now();

    // Send welcome message on connection
    let welcome = WsMessage::connected();
    let mut socket = socket;
    if let Ok(json) = serde_json::to_string(&welcome) {
        let _ = socket.send(Message::Text(json.into())).await;
    }

    tracing::info!("WebSocket client connected");

    // Subscribe to broadcast events from the app state
    let mut rx = state.ws_broadcaster.subscribe();

    let (mut sender, mut receiver) = socket.split();

    // Background ticker: push a live telemetry event every 5 seconds
    let mut tick_interval = tokio::time::interval(Duration::from_secs(5));
    tick_interval.tick().await; // skip the first immediate tick

    loop {
        tokio::select! {
            // Periodic live telemetry push
            _ = tick_interval.tick() => {
                let uptime = connected_at.elapsed().as_secs();
                let active_routes = state.routes.read().await.len() as u32;
                let live = WsMessage::live_tick(uptime, active_routes);
                if let Ok(json) = serde_json::to_string(&live) {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
            }
            // Broadcast events from app state (route_created, route_cancelled, etc.)
            recv = rx.recv() => {
                match recv {
                    Ok(msg) => {
                        if let Ok(json) = serde_json::to_string(&msg) {
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("WebSocket client lagged by {n} messages");
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
            // Incoming messages from client
            msg = receiver.next() => {
                match msg {
                    Some(Ok(m)) => {
                        let text = m.to_text().unwrap_or("").to_string();
                        tracing::debug!("WS received: {}", text);

                        let echo = WsMessage::echo(&text);
                        if let Ok(json) = serde_json::to_string(&echo) {
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Some(Err(e)) => {
                        tracing::warn!("WebSocket error: {e}");
                        break;
                    }
                    None => break,
                }
            }
        }
    }

    tracing::info!("WebSocket client disconnected (uptime: {:?})", connected_at.elapsed());
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;
    use axum::http::HeaderValue;

    /// In dev mode, any request is allowed — including ones with no Origin
    /// header (e.g. curl-based smoke tests in the README).
    #[test]
    fn is_origin_allowed_dev_mode_accepts_missing_origin() {
        let headers = HeaderMap::new();
        assert!(is_origin_allowed(&headers, true));
    }

    /// In dev mode, any Origin is allowed (localhost, 127.0.0.1, anything).
    #[test]
    fn is_origin_allowed_dev_mode_accepts_any_origin() {
        let mut headers = HeaderMap::new();
        headers.insert("origin", HeaderValue::from_static("http://localhost:3000"));
        assert!(is_origin_allowed(&headers, true));
    }

    /// In production, requests with no Origin header are rejected —
    /// browsers always send Origin on WebSocket upgrades, so a missing
    /// Origin signals a non-browser client which should not be observing
    /// the broadcast stream.
    #[test]
    fn is_origin_allowed_prod_rejects_missing_origin() {
        let headers = HeaderMap::new();
        assert!(!is_origin_allowed(&headers, false));
    }

    /// In production, requests with an allow-listed Origin are accepted.
    #[test]
    fn is_origin_allowed_prod_accepts_allowlisted_origin() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "origin",
            HeaderValue::from_static("https://pickando-demo-production.up.railway.app"),
        );
        assert!(is_origin_allowed(&headers, false));
    }

    /// In production, requests with a foreign Origin are rejected —
    /// this is the data-exfiltration vector closed by Security audit 8-a P2.
    #[test]
    fn is_origin_allowed_prod_rejects_foreign_origin() {
        let mut headers = HeaderMap::new();
        headers.insert("origin", HeaderValue::from_static("https://evil.example.com"));
        assert!(!is_origin_allowed(&headers, false));
    }

    /// Malformed Origin header (non-ASCII bytes) is rejected, not panicked.
    #[test]
    fn is_origin_allowed_prod_rejects_malformed_origin() {
        let mut headers = HeaderMap::new();
        // Non-ASCII bytes cannot be parsed as a &str — to_str() returns Err.
        headers.insert("origin", HeaderValue::from_bytes(b"\xff\xfe").unwrap());
        assert!(!is_origin_allowed(&headers, false));
    }
}
