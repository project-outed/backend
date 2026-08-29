use axum::{
    extract::{Path, State},
    Json,
    http::StatusCode,
};
use crate::state::AppState;
use crate::models::user::{User, UpdateProviders};
use anyhow::Result;

pub async fn update_providers(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateProviders>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let mut update_data = payload.clone();

    if let Some(discord) = &mut update_data.discord {
        if let Ok(Some(info)) = crate::utils::discord::get_discord_info(&state.http_client, &discord.id).await {
            discord.avatar = info["avatar"].as_str().unwrap_or("").to_string();
            discord.username = info["username"].as_str().unwrap_or(&discord.username).to_string();
        }
    }


    let providers_json = serde_json::to_value(&update_data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (user_id, providers, verified, trust_score, updated_at) 
         VALUES ($1, $2, 0, 100, CURRENT_TIMESTAMP)
         ON CONFLICT (user_id) 
         DO UPDATE SET 
            providers = users.providers || EXCLUDED.providers, 
            updated_at = CURRENT_TIMESTAMP 
         RETURNING *"
    )
    .bind(id)
    .bind(providers_json)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Error upserting providers: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(user))
}
