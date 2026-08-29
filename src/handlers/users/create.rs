use axum::{
    extract::State,
    Json,
    http::StatusCode,
};
use crate::state::AppState;
use crate::models::user::{User, CreateUser};
use anyhow::Result;

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUser>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let user_id_extracted = if let Some(id) = payload.user_id {
        id
    } else if let Some(discord) = &payload.providers.discord {
        discord.id.parse::<i64>().map_err(|_| StatusCode::BAD_REQUEST)?
    } else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let mut providers = payload.providers.clone();

    if let Some(discord) = &mut providers.discord {
        if let Ok(Some(info)) = crate::utils::discord::get_discord_info(&state.http_client, &discord.id).await {
            discord.avatar = info["avatar"].as_str().unwrap_or("").to_string();
            discord.username = info["username"].as_str().unwrap_or(&discord.username).to_string();
        }
    }

    let providers_json = serde_json::to_value(&providers).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let trust_score = payload.trust_score.unwrap_or(100);

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (user_id, providers, mail, verified, trust_score) VALUES ($1, $2, $3, $4, $5) RETURNING *"
    )
    .bind(user_id_extracted)
    .bind(providers_json)
    .bind(&payload.mail)
    .bind(payload.verified)
    .bind(trust_score)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Error creating user: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(user)))
}
