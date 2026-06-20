use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State, Query,
    },
    response::IntoResponse,
    http::StatusCode,
};
use futures_util::{StreamExt, SinkExt};
use serde::Deserialize;
use crate::state::AppState;
use std::time::{Instant, Duration};

#[derive(Deserialize)]
pub struct WsParams {
    token: Option<String>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(params): Query<WsParams>,
) -> impl IntoResponse {
    let authenticated = match params.token {
        Some(ref t) => t == &state.encryption_apikey,
        None => false,
    };

    if !authenticated {
        tracing::warn!("Unauthorized WebSocket connection attempt.");
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let mut broadcast_rx = state.websocket.subscribe();

    // Heartbeat configuration: ping every 15 seconds, timeout if no response in 30 seconds
    let ping_interval = Duration::from_secs(15);
    let timeout_duration = Duration::from_secs(30);
    let mut last_pong = Instant::now();
    let mut interval = tokio::time::interval(ping_interval);

    // Skip the first tick of the interval immediately
    interval.tick().await;

    tracing::info!("New WebSocket connection established.");

    loop {
        tokio::select! {
            // 1. Listen for system-wide broadcast messages
            Ok(msg) = broadcast_rx.recv() => {
                if ws_sender.send(Message::Text(msg)).await.is_err() {
                    tracing::error!("Failed to send broadcast message to client. Closing socket.");
                    break;
                }
            }

            // 2. Heartbeat Ping validation
            _ = interval.tick() => {
                if last_pong.elapsed() > timeout_duration {
                    tracing::warn!("WebSocket client heartbeat timeout. Disconnecting.");
                    break;
                }

                if ws_sender.send(Message::Ping(vec![])).await.is_err() {
                    tracing::error!("Failed to send Ping frame. Closing socket.");
                    break;
                }
            }

            // 3. Receive client messages
            msg_opt = ws_receiver.next() => {
                match msg_opt {
                    Some(Ok(msg)) => {
                        match msg {
                            Message::Text(text) => {
                                tracing::info!("Received message from WebSocket client: {}", text);
                                
                                // Fast echo test
                                if text == "PING" {
                                    if ws_sender.send(Message::Text("PONG".to_string())).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            Message::Binary(bin) => {
                                tracing::info!("Received binary message of length: {} bytes", bin.len());
                            }
                            Message::Pong(_) => {
                                // Heartbeat pong response received; update last_pong timestamp
                                last_pong = Instant::now();
                            }
                            Message::Close(_) => {
                                tracing::info!("Client requested WebSocket closure.");
                                break;
                            }
                            Message::Ping(_) => {
                                // Axum handles responding to standard Pings automatically,
                                // but we can catch it if needed.
                            }
                        }
                    }
                    Some(Err(e)) => {
                        tracing::error!("Error reading message from WebSocket: {:?}", e);
                        break;
                    }
                    None => {
                        tracing::info!("WebSocket stream closed by client.");
                        break;
                    }
                }
            }
        }
    }

    tracing::info!("WebSocket connection closed.");
}
