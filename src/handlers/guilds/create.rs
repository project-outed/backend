use axum::{extract::State, Json, http::StatusCode};
use crate::state::AppState;
use crate::models::UpdateGuild;
use tracing::{error, info, debug};

pub async fn create_guild(
    State(state): State<AppState>,
    Json(payload): Json<UpdateGuild>,
) -> Result<StatusCode, StatusCode> {
    info!("Received request to create guild: {}", payload.guild_id);
    debug!("Payload: {:?}", payload);

    let guild_id = payload.guild_id.parse::<i64>().map_err(|e| {
        error!("Failed to parse guild_id {}: {}", payload.guild_id, e);
        StatusCode::BAD_REQUEST
    })?;

    let guild_owner = payload.guild_owner.parse::<i64>().map_err(|e| {
        error!("Failed to parse guild_owner {}: {}", payload.guild_owner, e);
        StatusCode::BAD_REQUEST
    })?;
    
    debug!("Parsing guild members...");
    let guild_members: Vec<i64> = payload.guild_members
        .iter()
        .map(|s| s.parse::<i64>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            error!("Failed to parse guild member ID: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    let alert_channel = payload.alert_channel.as_ref()
        .map(|s| s.parse::<i64>())
        .transpose()
        .map_err(|e| {
            error!("Failed to parse alert_channel: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    let alert_role = payload.alert_role.as_ref()
        .map(|s| s.parse::<i64>())
        .transpose()
        .map_err(|e| {
            error!("Failed to parse alert_role: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    debug!("Checking if guild {} already exists...", guild_id);
    let existing_id: Option<(i64,)> = sqlx::query_as("SELECT id FROM guilds WHERE guild_id = $1")
        .bind(guild_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            error!("Database error checking for guild {}: {}", guild_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if let Some((id,)) = existing_id {
        info!("Guild {} already exists (ID {}), updating data...", guild_id, id);
        sqlx::query(
            "UPDATE guilds SET
                guild_name = $1,
                guild_owner = $2,
                guild_members = $3,
                alert_channel = $4,
                alert_role = $5,
                identifiers_showed = $6,
                is_active = COALESCE($7, is_active),
                updated_at = CURRENT_TIMESTAMP
             WHERE id = $8"
        )
        .bind(&payload.guild_name)
        .bind(guild_owner)
        .bind(&guild_members)
        .bind(alert_channel)
        .bind(alert_role)
        .bind(&payload.identifiers_showed)
        .bind(payload.is_active)
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to update existing guild {} (ID {}): {}", guild_id, id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        info!("Guild {} (ID {}) updated successfully", guild_id, id);
    } else {
        info!("Guild {} not found, performing new insertion...", guild_id);
        sqlx::query(
            "INSERT INTO guilds (guild_id, guild_name, guild_owner, guild_members, alert_channel, alert_role, identifiers_showed, is_active, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, CURRENT_TIMESTAMP)"
        )
        .bind(guild_id)
        .bind(&payload.guild_name)
        .bind(guild_owner)
        .bind(&guild_members)
        .bind(alert_channel)
        .bind(alert_role)
        .bind(&payload.identifiers_showed)
        .bind(payload.is_active.unwrap_or(true))
        .execute(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to insert new guild {}: {}", guild_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        info!("New guild {} created successfully", guild_id);
    }

    info!("Guild {} created successfully", guild_id);
    Ok(StatusCode::CREATED)
}
