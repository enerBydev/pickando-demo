use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};
use pickando_shared::models::WsMessage;

/// GET /ws — WebSocket endpoint.
///
/// Establishes a bidirectional WebSocket connection. Currently implements
/// an echo server to demonstrate real-time connectivity.
///
/// TODO in M2: GPS coordinate streaming for live tracking,
/// ride status updates, driver-passenger chat relay.
pub async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    // Send welcome message on connection
    let welcome = WsMessage {
        msg_type: "connected".into(),
        message: "Pickando WebSocket live — TODO: GPS streaming in M2".into(),
        data: None,
    };

    if let Ok(json) = serde_json::to_string(&welcome) {
        let _ = socket.send(Message::Text(json.into())).await;
    }

    tracing::info!("WebSocket client connected");

    // Echo loop — demonstrates bidirectional communication
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(msg) => {
                let text = msg.to_text().unwrap_or("").to_string();
                tracing::debug!("WS received: {}", text);

                let echo = WsMessage {
                    msg_type: "echo".into(),
                    message: "WebSocket bidireccional funcional — TODO: live tracking in M2".into(),
                    data: Some(serde_json::json!({ "received": text })),
                };

                if let Ok(json) = serde_json::to_string(&echo) {
                    if socket.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
            }
            Err(e) => {
                tracing::warn!("WebSocket error: {e}");
                break;
            }
        }
    }

    tracing::info!("WebSocket client disconnected");
}
