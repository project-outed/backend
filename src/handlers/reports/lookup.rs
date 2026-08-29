use axum::{extract::{State, Query}, Json};
use crate::state::AppState;
use crate::models::Report;
use crate::utils::AppError;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LookupParams {
    pub target_id: String,
}

pub async fn lookup_reports(
    State(state): State<AppState>,
    Query(params): Query<LookupParams>,
) -> Result<Json<Vec<Report>>, AppError> {
    let target_user_id_val: i64 = params.target_id.parse().unwrap_or(0);

    let reports_raw = sqlx::query_as::<_, Report>(
        "SELECT id, target_username, target_user_id, reporter_username, reporter_user_id, game, reason, status, created_at, updated_at \
         FROM reports \
         WHERE target_user_id = $1 AND status = 'accepted'"
    )
    .bind(target_user_id_val)
    .fetch_all(&state.db)
    .await?;

    let mut reports = Vec::new();
    for mut report in reports_raw {
        report.reporter_username = "ANONYMOUS".to_string();
        report.reporter_user_id = 0;
        report.evidence = Vec::new(); 
        reports.push(report);
    }

    Ok(Json(reports))
}
