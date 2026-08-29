use axum::{
    extract::{Path, State},
    Json,
    http::StatusCode,
};
use crate::state::AppState;
use crate::models::user::User;
use anyhow::Result;

use crate::models::Report;

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let mut user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE user_id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            eprintln!("Error fetching user: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let reports = sqlx::query_as::<_, Report>(
        "SELECT id, target_username, target_user_id, reporter_username, reporter_user_id, game, reason, status, created_at, updated_at \
         FROM reports WHERE target_user_id = $1 AND status = 'accepted'"
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Error fetching reports in get_user: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    user.reports = reports;

    Ok(Json(user))
}
