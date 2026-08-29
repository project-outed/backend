use axum::{extract::{Path, State}, Json};
use crate::state::AppState;
use crate::models::{Report, Evidence};
use crate::utils::AppError;

pub async fn save_report(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(payload): Json<Report>,
) -> Result<Json<Report>, AppError> {
    sqlx::query(
        "UPDATE reports SET reason = $1, updated_at = NOW() WHERE id = $2"
    )
    .bind(&payload.reason)
    .bind(id)
    .execute(&state.db)
    .await?;

    let mut report = sqlx::query_as::<_, Report>(
        "SELECT id, target_username, target_user_id, reporter_username, reporter_user_id, game, reason, status, created_at, updated_at \
         FROM reports WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    report.evidence = sqlx::query_as::<_, Evidence>(
        "SELECT id, report_id, url, evidence_type, created_at FROM report_evidence WHERE report_id = $1"
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(report))
}
