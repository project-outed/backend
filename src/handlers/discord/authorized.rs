use axum::http::StatusCode;
use crate::state::AppState;
use crate::models::{DiscordEventData, Providers, DiscordProvider};
use tracing::{error, info, debug};

fn parse_snowflake(value: &str, field_name: &str) -> Result<i64, StatusCode> {
    value.parse::<i64>().map_err(|e| {
        error!(field = field_name, value = value, error = %e, "Failed to parse snowflake");
        StatusCode::BAD_REQUEST
    })
}

pub async fn handle_authorized(state: &AppState, data: &DiscordEventData) -> Result<(), StatusCode> {
    if let Some(user) = &data.user {
        let user_id = parse_snowflake(&user.id, "user_id")?;

        let providers = Providers {
            discord: Some(DiscordProvider {
                id: user_id,
                username: user.username.clone(),
                avatar: user.avatar.clone().unwrap_or_default(),
                added_at: None,
                updated_at: None,
            }),
            ..Default::default()
        };

        let providers_json = serde_json::to_value(&providers).map_err(|e| {
            error!(user_id = user_id, error = %e, "Failed to serialize providers");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        debug!(user_id = user_id, "Ensuring user exists in database");

        sqlx::query(
            "INSERT INTO users (user_id, providers, verified, trust_score, created_at, updated_at)
             VALUES ($1, $2, 1, 100, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             ON CONFLICT (user_id) DO NOTHING"
        )
        .bind(user_id)
        .bind(providers_json)
        .execute(&state.database)
        .await
        .map_err(|e| {
            error!(user_id = user_id, error = %e, "Database error ensuring user exists");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    match data.integration_type {
        Some(0) => {
            let guild = data.guild.as_ref().ok_or_else(|| {
                error!("Received Guild Authorization (Type 0) but no guild data was provided");
                StatusCode::BAD_REQUEST
            })?;

            let guild_id = parse_snowflake(&guild.id, "guild_id")?;
            let guild_owner = guild.owner_id
                .as_deref()
                .map(|id| parse_snowflake(id, "owner_id"))
                .transpose()?
                .unwrap_or(0);

            info!(guild_id = guild_id, guild_name = %guild.name, "Processing guild authorization");

            sqlx::query(
                "INSERT INTO guilds (guild_id, guild_name, guild_owner, is_active, updated_at)
                 VALUES ($1, $2, $3, TRUE, CURRENT_TIMESTAMP)
                 ON CONFLICT (guild_id) DO UPDATE SET
                    guild_name = EXCLUDED.guild_name,
                    guild_owner = EXCLUDED.guild_owner,
                    is_active = TRUE,
                    updated_at = CURRENT_TIMESTAMP"
            )
            .bind(guild_id)
            .bind(&guild.name)
            .bind(guild_owner)
            .execute(&state.database)
            .await
            .map_err(|e| {
                error!(guild_id = guild_id, error = %e, "Failed to upsert guild");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            info!(guild_id = guild_id, "Guild upserted successfully");
        }
        Some(1) => {
            if let Some(user) = &data.user {
                info!(username = %user.username, user_id = %user.id, "User-only authorization");
            }
        }
        Some(other) => {
            info!(integration_type = other, "Unknown integration type");
        }
        None => {
            info!("Authorization event with no integration type");
        }
    }

    Ok(())
}
