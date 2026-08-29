use axum::{
    extract::{ws::{Message as WsMessage, WebSocket, WebSocketUpgrade}, State},
    response::IntoResponse,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use crate::state::AppState;
use crate::models::{Message, CreateMessage};
use redis::AsyncCommands;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    if let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await {
        let history: Vec<String> = conn.lrange("recent_messages", 0, -1).await.unwrap_or_default();
        for msg in history {
            let _ = sender.send(WsMessage::Text(msg.into())).await;
        }
    }

    let mut rx = state.broadcast.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(WsMessage::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            match message {
                WsMessage::Text(text) => {
                    if let Ok(payload) = serde_json::from_str::<CreateMessage>(&text) {
                        let result: Result<(i64,), sqlx::Error> = sqlx::query_as(
                            "INSERT INTO messages (content, sender) VALUES ($1, $2) RETURNING id"
                        )
                        .bind(&payload.content)
                        .bind(&payload.sender)
                        .fetch_one(&state.db)
                        .await;

                        if let Ok(row) = result {
                            let id = row.0;
                            let new_message = Message {
                                id: Some(id),
                                content: payload.content.clone(),
                                sender: payload.sender.clone(),
                                created_at: Some(chrono::Utc::now()),
                            };

                            if let Ok(serialized) = serde_json::to_string(&new_message) {
                                if let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await {
                                    let _: () = conn.rpush("recent_messages", &serialized).await.unwrap_or_default();
                                    let _: () = conn.ltrim("recent_messages", -10, -1).await.unwrap_or_default();
                                }

                                let _ = state.broadcast.send(serialized);
                            }
                        }
                    } else {
                        let _ = state.broadcast.send(text.to_string());
                    }
                }
                WsMessage::Close(_) => {
                    break;
                }
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}
