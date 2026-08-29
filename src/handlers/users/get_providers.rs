use axum::{
    extract::{Path, State},
    Json,
    http::StatusCode,
};
use crate::state::AppState;
use crate::models::user::Providers;
use anyhow::Result;
use sqlx::FromRow;

#[derive(FromRow)]
struct ProvidersRow {
    #[sqlx(json)]
    pub providers: Providers,
}

pub async fn get_user_providers(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let row = sqlx::query_as::<_, ProvidersRow>("SELECT providers FROM users WHERE user_id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(row.providers))
}
