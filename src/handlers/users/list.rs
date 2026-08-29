use axum::{
    extract::State,
    Json,
    http::StatusCode,
};
use crate::state::AppState;
use crate::models::user::User;
use anyhow::Result;

pub async fn get_users(
    State(state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let mut users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching users: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let user_ids: Vec<i64> = users.iter().map(|u| u.user_id).collect();

    if !user_ids.is_empty() {
        let reports = sqlx::query_as::<_, crate::models::Report>(
            "SELECT id, target_username, target_user_id, reporter_username, reporter_user_id, game, reason, status, created_at, updated_at \
             FROM reports WHERE target_user_id = ANY($1)"
        )
        .bind(&user_ids)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching reports for users: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        for user in &mut users {
            user.reports = reports.iter()
                .filter(|r| r.target_user_id == user.user_id)
                .cloned()
                .collect();
        }
    }

    Ok(Json(users))
}
