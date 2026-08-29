use axum::{extract::{Path, State}, Json, http::StatusCode};
use crate::state::AppState;
use crate::models::UpdateGuild;
use tracing::{error, info, debug};

pub async fn update_guild(
    State(state): State<AppState>,
    Path(guild_id_param): Path<i64>,
    Json(payload): Json<UpdateGuild>,
) -> Result<StatusCode, StatusCode> {
    info!("Received request to update guild: {}", guild_id_param);
    debug!("Payload: {:?}", payload);

    let _guild_id = payload.guild_id.parse::<i64>().map_err(|e| {
        error!("Failed to parse guild_id in payload: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    let guild_owner = payload.guild_owner.parse::<i64>().map_err(|e| {
        error!("Failed to parse guild_owner: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    
    debug!("Parsing guild members for update...");
    let guild_members: Vec<i64> = payload.guild_members
        .iter()
        .map(|s| s.parse::<i64>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            error!("Failed to parse member ID during update: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    let alert_channel = payload.alert_channel.as_ref()
        .map(|s| s.parse::<i64>())
        .transpose()
        .map_err(|e| {
            error!("Failed to parse alert_channel during update: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    let alert_role = payload.alert_role.as_ref()
        .map(|s| s.parse::<i64>())
        .transpose()
        .map_err(|e| {
            error!("Failed to parse alert_role during update: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    debug!("Executing database update for guild {}", guild_id_param);
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
         WHERE guild_id = $8"
    )
    .bind(&payload.guild_name)
    .bind(guild_owner)
    .bind(&guild_members)
    .bind(alert_channel)
    .bind(alert_role)
    .bind(&payload.identifiers_showed)
    .bind(payload.is_active)
    .bind(guild_id_param)
    .execute(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to update guild {}: {}", guild_id_param, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Guild {} updated successfully", guild_id_param);
    Ok(StatusCode::OK)
}
