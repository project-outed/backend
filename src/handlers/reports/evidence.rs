use axum::{extract::{Path, State}, Json, response::IntoResponse};
use axum::http::{header, StatusCode};
use crate::state::AppState;
use crate::models::Evidence;
use crate::utils::AppError;
use tokio::fs;

pub async fn get_evidence(
    Path(report_id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Evidence>>, AppError> {
    let evidence = sqlx::query_as::<_, Evidence>(
        "SELECT id, report_id, url, evidence_type, created_at FROM report_evidence WHERE report_id = $1"
    )
    .bind(report_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(evidence))
}

pub async fn get_evidence_file(
    Path((report_id, evidence_id)): Path<(i64, i64)>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let evidence = sqlx::query_as::<_, Evidence>(
        "SELECT id, report_id, url, evidence_type, created_at FROM report_evidence WHERE report_id = $1 AND id = $2"
    )
    .bind(report_id)
    .bind(evidence_id)
    .fetch_optional(&state.db)
    .await?;

    let evidence = evidence.ok_or_else(|| AppError::NotFound("Evidence not found".to_string()))?;

    let file_path = evidence.url.trim_start_matches('/');
    let data = fs::read(&file_path).await.map_err(|_| AppError::NotFound("Evidence file not found".to_string()))?;

    let content_type = match file_path.rsplit('.').next().map(|e| e.to_lowercase()) {
        Some(ext) => match ext.as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "mp4" => "video/mp4",
            "webm" => "video/webm",
            "pdf" => "application/pdf",
            _ => "application/octet-stream",
        },
        None => "application/octet-stream",
    };

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, content_type)],
        data,
    ))
}
