use axum::{routing::get, Router};
use crate::state::AppState;
use crate::handlers::ws_handlers;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/ws", get(ws_handlers::ws_handler))
}
