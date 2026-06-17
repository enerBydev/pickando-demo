//! WebSocket handler.
//!
//! Each connected client:
//! 1. Receives a `connected` welcome on open.
//! 2. Receives periodic `live_tick` events (every 5 s) with active routes count.
//! 3. Receives every broadcast event from `AppState.ws_broadcaster`
//!    (`route_created`, `route_cancelled`, `ride_request`, etc.).
//! 4. Has its text messages echoed back via an `echo` message.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
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
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
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
