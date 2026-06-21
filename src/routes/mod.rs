use axum::Router;
use crate::state::AppState;

pub mod websocket;
pub mod discord;

pub fn app_router() -> Router<AppState> {
    Router::new()
        .merge(websocket::router())
        .merge(discord::router())
}
