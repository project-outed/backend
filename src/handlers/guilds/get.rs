use axum::{extract::{Path, State}, Json, http::StatusCode};
use crate::state::AppState;
use crate::models::Guild;
use tracing::{error, info, debug};

pub async fn get_guild(
    State(state): State<AppState>,
    Path(guild_id): Path<i64>,
) -> Result<Json<Guild>, StatusCode> {
    info!("Received request to get guild: {}", guild_id);
    debug!("Executing database lookup for guild {}", guild_id);

    let guild = sqlx::query_as::<_, Guild>(
        "SELECT * FROM guilds WHERE guild_id = $1"
    )
    .bind(guild_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to get guild {}: {}", guild_id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match guild {
        Some(g) => {
            info!("Successfully found guild: {}", guild_id);
            Ok(Json(g))
        },
        None => {
            info!("Guild {} not found", guild_id);
            Err(StatusCode::NOT_FOUND)
        },
    }
}
