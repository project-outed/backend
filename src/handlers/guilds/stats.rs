use axum::{extract::State, Json, http::StatusCode};
use crate::state::AppState;
use crate::models::GuildStats;
use tracing::{error, info, debug};

pub async fn get_guild_stats(
    State(state): State<AppState>,
) -> Result<Json<GuildStats>, StatusCode> {
    info!("Received request for guild statistics");
    debug!("Executing database query for guild stats");

    let stats = sqlx::query_as::<_, GuildStats>(
        "SELECT 
            COUNT(*) as total_guilds,
            COALESCE(ARRAY_LENGTH(ARRAY(SELECT unnest(guild_members) FROM guilds), 1), 0)::BIGINT as total_members
         FROM guilds"
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to fetch guild stats: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Successfully fetched guild stats: total_guilds={}, total_members={}", stats.total_guilds, stats.total_members);
    Ok(Json(stats))
}
