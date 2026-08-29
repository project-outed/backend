use axum::{extract::State, Json, http::StatusCode};
use crate::state::AppState;
use crate::models::Guild;
use tracing::{error, info, debug};

pub async fn get_guilds(
    State(state): State<AppState>,
) -> Result<Json<Vec<Guild>>, StatusCode> {
    info!("Received request to list all guilds");
    debug!("Executing database query for all guilds");

    let guilds = sqlx::query_as::<_, Guild>(
        "SELECT * FROM guilds ORDER BY id ASC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to fetch guilds: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Successfully fetched {} guilds", guilds.len());
    Ok(Json(guilds))
}
