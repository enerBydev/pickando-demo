use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use pickando_shared::models::WsMessage;
use std::time::{Duration, Instant};

/// GET /ws — WebSocket endpoint.
///
/// Establishes a bidirectional WebSocket connection. Sends a welcome
/// message on connect, echoes incoming messages, and emits periodic
/// "live" telemetry events so the frontend can demonstrate real-time
/// updates without needing the client to send anything first.
pub async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let connected_at = Instant::now();

    // Send welcome message on connection
    let welcome = WsMessage {
        msg_type: "connected".into(),
        message: "Pickando WebSocket en vivo — conexión establecida".into(),
        data: Some(serde_json::json!({
            "server_time": unix_now(),
            "protocol": "pickando-ws-v1",
        })),
    };

    if let Ok(json) = serde_json::to_string(&welcome) {
        let _ = socket.send(Message::Text(json.into())).await;
    }

    tracing::info!("WebSocket client connected");

    // Split into sender + receiver so we can:
    //   - periodically push live telemetry events via the sender
    //   - concurrently read incoming messages via the receiver
    let (mut sender, mut receiver) = socket.split();

    // Background ticker: push a live telemetry event every 5 seconds
    let mut tick_interval = tokio::time::interval(Duration::from_secs(5));
    tick_interval.tick().await; // skip the first immediate tick

    loop {
        tokio::select! {
            // Periodic live telemetry push
            _ = tick_interval.tick() => {
                let uptime = connected_at.elapsed().as_secs();
                let live = WsMessage {
                    msg_type: "live_tick".into(),
                    message: format!("Tick #{uptime}s — servidor activo"),
                    data: Some(serde_json::json!({
                        "uptime_seconds": uptime,
                        "server_time": unix_now(),
                        "active_routes": 6,
                    })),
                };
                if let Ok(json) = serde_json::to_string(&live) {
                    if sender.send(Message::Text(json.into())).await.is_err() {
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

                        let echo = WsMessage {
                            msg_type: "echo".into(),
                            message: "WebSocket bidireccional funcional".into(),
                            data: Some(serde_json::json!({ "received": text })),
                        };
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

    tracing::info!(
        "WebSocket client disconnected (uptime: {:?})",
        connected_at.elapsed()
    );
}

/// Cheap unix-seconds timestamp without depending on chrono.
fn unix_now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
