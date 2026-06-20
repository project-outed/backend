use axum::Router;
use crate::state::AppState;

pub mod websocket;

pub fn app_router() -> Router<AppState> {
    Router::new()
        .merge(websocket::router())
}
