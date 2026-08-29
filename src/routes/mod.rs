use axum::{middleware, Router, extract::DefaultBodyLimit, routing::get};
use crate::state::AppState;
use crate::handlers::{callbacks, health};
use crate::utils::auth;

pub mod report_routes;
pub mod ws_routes;
pub mod user_routes;
pub mod guild_routes;

pub fn create_router(state: AppState) -> Router {
    let auth_routes = Router::new()
        .merge(report_routes::routes())
        .merge(user_routes::routes())
        .merge(guild_routes::routes())
        .layer(middleware::from_fn_with_state(state.clone(), auth::api_key_middleware));

    Router::new()
        .route("/callback/discord", get(callbacks::discord::discord_callback))
        .route("/callback/discord/", get(callbacks::discord::discord_callback))
        .route("/discord/events", axum::routing::any(crate::handlers::discord_events::handle_discord_event))
        .route("/discord/events/", axum::routing::any(crate::handlers::discord_events::handle_discord_event))
        .nest("/api", auth_routes)
        .route("/api/discord/events", axum::routing::any(crate::handlers::discord_events::handle_discord_event))
        .route("/api/discord/events/", axum::routing::any(crate::handlers::discord_events::handle_discord_event))
        .merge(ws_routes::routes())
        .route("/health", axum::routing::get(health::health_check))
        .layer(DefaultBodyLimit::max(250 * 1024 * 1024))
        .with_state(state)
}
