use axum::http::StatusCode;
use crate::state::AppState;
use crate::models::DiscordEventData;
use tracing::{error, info};

pub async fn handle_deauthorized(state: &AppState, data: &DiscordEventData) -> Result<(), StatusCode> {
    if let Some(guild) = &data.guild {
        let guild_id = guild.id.parse::<i64>().map_err(|e| {
            error!("Failed to parse guild_id {}: {}", guild.id, e);
            StatusCode::BAD_REQUEST
        })?;

        info!("Processing deauthorization for guild: {}", guild_id);

        sqlx::query(
            "UPDATE guilds SET is_active = FALSE, updated_at = CURRENT_TIMESTAMP WHERE guild_id = $1"
        )
        .bind(guild_id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to deactivate guild {}: {}", guild_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        info!("Guild {} deauthorized and deactivated", guild_id);
    } else if let Some(user) = &data.user {
        info!("User {} deauthorized the application", user.id);
    }

    Ok(())
}
