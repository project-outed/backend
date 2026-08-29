use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use crate::state::AppState;
use anyhow::Result;

pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let result = sqlx::query("DELETE FROM users WHERE user_id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}
