use axum::http::StatusCode;
use crate::state::AppState;
use crate::models::{DiscordEventData, Providers, DiscordProvider};
use tracing::{error, info, debug};

pub async fn handle_authorized(state: &AppState, data: &DiscordEventData) -> Result<(), StatusCode> {
    if let Some(user) = &data.user {
        let user_id_val = user.id.parse::<i64>().map_err(|e| {
            error!("Failed to parse user_id string '{}': {}", user.id, e);
            StatusCode::BAD_REQUEST
        })?;

        let providers = Providers {
            discord: Some(DiscordProvider {
                username: user.username.clone(),
                id: user.id.clone(),
                avatar: user.avatar.clone().unwrap_or_default(),
            }),
            ..Default::default()
        };

        debug!("Ensuring user {} exists in database", user_id_val);
        sqlx::query(
            "INSERT INTO users (user_id, providers, verified, trust_score, created_at, updated_at)
             VALUES ($1, $2, 1, 100, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             ON CONFLICT (user_id) DO NOTHING"
        )
        .bind(user_id_val)
        .bind(serde_json::to_value(providers).unwrap())
        .execute(&state.db)
        .await
        .map_err(|e| {
            error!("Database error ensuring user {} exists: {}", user_id_val, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    let integration_type = data.integration_type.unwrap_or(-1);
    
    match integration_type {
        0 => {
            if let Some(guild) = &data.guild {
                let guild_id = guild.id.parse::<i64>().map_err(|e| {
                    error!("Failed to parse guild_id string '{}': {}", guild.id, e);
                    StatusCode::BAD_REQUEST
                })?;

                let guild_owner = if let Some(owner_id_str) = &guild.owner_id {
                    owner_id_str.parse::<i64>().map_err(|e| {
                        error!("Failed to parse owner_id string '{}': {}", owner_id_str, e);
                        StatusCode::BAD_REQUEST
                    })?
                } else {
                    0
                };

                info!("Processing Guild Authorization: {} ({})", guild.name, guild_id);

                let existing_guild: Option<(i64,)> = sqlx::query_as("SELECT id FROM guilds WHERE guild_id = $1")
                    .bind(guild_id)
                    .fetch_optional(&state.db)
                    .await
                    .map_err(|e| {
                        error!("Database error checking for existing guild {}: {}", guild_id, e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })?;

                if let Some((id,)) = existing_guild {
                    debug!("Guild {} already exists with internal ID {}, updating existing record...", guild_id, id);
                    sqlx::query(
                        "UPDATE guilds SET 
                            guild_name = $1, 
                            guild_owner = $2, 
                            is_active = TRUE, 
                            updated_at = CURRENT_TIMESTAMP 
                         WHERE id = $3"
                    )
                    .bind(&guild.name)
                    .bind(guild_owner)
                    .bind(id)
                    .execute(&state.db)
                    .await
                    .map_err(|e| {
                        error!("Failed to update existing guild {} (ID {}): {}", guild_id, id, e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })?;
                    info!("Guild {} (internal ID {}) updated successfully", guild_id, id);
                } else {
                    info!("Guild {} is a new addition, creating record...", guild_id);
                    sqlx::query(
                        "INSERT INTO guilds (guild_id, guild_name, guild_owner, is_active, updated_at)
                         VALUES ($1, $2, $3, TRUE, CURRENT_TIMESTAMP)"
                    )
                    .bind(guild_id)
                    .bind(&guild.name)
                    .bind(guild_owner)
                    .execute(&state.db)
                    .await
                    .map_err(|e| {
                        error!("Failed to insert new guild {}: {}", guild_id, e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })?;
                    info!("New guild {} added successfully", guild_id);
                }
            } else {
                error!("Received Guild Authorization (Type 0) but no guild data was provided");
            }
        }
        1 => {
            if let Some(user) = &data.user {
                info!("User Authorization (Type 1): {} ({})", user.username, user.id);
            }
        }
        _ => {
            info!("Received authorization with unknown integration type: {}", integration_type);
        }
    }
    Ok(())
}
