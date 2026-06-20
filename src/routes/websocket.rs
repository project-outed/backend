use axum::{routing::get, Router};
use crate::state::AppState;
use crate::handlers::websocket::ws_handler;

pub fn router() -> Router<AppState> {
    Router::new().route("/ws", get(ws_handler))
}
