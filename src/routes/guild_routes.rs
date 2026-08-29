use axum::routing::get;
use axum::Router;
use crate::state::AppState;
use crate::handlers::guilds;
use tracing::debug;

pub fn routes() -> Router<AppState> {
    debug!("Initializing guild routes");
    Router::new()
        .route("/guilds", get(guilds::get_guilds).post(guilds::create_guild))
        .route("/guilds/stats", get(guilds::get_guild_stats))
        .route("/guilds/{guild_id}", get(guilds::get_guild).post(guilds::update_guild).delete(guilds::delete_guild))
}
