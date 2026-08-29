use axum::{
    extract::{Path, State},
    Json,
    http::StatusCode,
};
use crate::state::AppState;
use crate::models::user::{User, UpdateMailVerified};
use anyhow::Result;

pub async fn update_mail_verified(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateMailVerified>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (user_id, mail, verified, trust_score, providers, updated_at) 
         VALUES ($1, $2, $3, COALESCE($4, 100), '{}'::jsonb, CURRENT_TIMESTAMP)
         ON CONFLICT (user_id) 
         DO UPDATE SET 
            mail = EXCLUDED.mail, 
            verified = EXCLUDED.verified,
            trust_score = COALESCE(EXCLUDED.trust_score, users.trust_score),
            updated_at = CURRENT_TIMESTAMP 
         RETURNING *"
    )
    .bind(id)
    .bind(&payload.mail)
    .bind(payload.verified)
    .bind(payload.trust_score)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Error upserting user status: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(user))
}
