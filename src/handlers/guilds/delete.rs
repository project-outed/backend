use axum::{extract::{Path, State}, http::StatusCode};
use crate::state::AppState;
use tracing::{error, info, debug};

pub async fn delete_guild(
    State(state): State<AppState>,
    Path(guild_id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    info!("Received request to delete guild: {}", guild_id);
    debug!("Executing database delete for guild {}", guild_id);

    sqlx::query("DELETE FROM guilds WHERE guild_id = $1")
    .bind(guild_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to delete guild {}: {}", guild_id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Guild {} deleted successfully", guild_id);
    Ok(StatusCode::OK)
}
