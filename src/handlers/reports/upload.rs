use axum::{extract::{State, Multipart}, Json};
use crate::state::AppState;
use crate::utils::AppError;
use std::path::PathBuf;
use tokio::fs;

pub async fn upload_evidence(
    State(_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let upload_dir = "data/tmp";
    fs::create_dir_all(upload_dir).await.map_err(|e| AppError::Internal(e.to_string()))?;

    let mut urls = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        let name = field.name().unwrap_or("file").to_string();
        let file_name = field.file_name().unwrap_or("upload.dat").to_string();
        let data = field.bytes().await.map_err(|e| AppError::Internal(e.to_string()))?;

        let unique_name = format!("{}_{}", chrono::Utc::now().timestamp(), file_name);
        let path = PathBuf::from(upload_dir).join(&unique_name);
        fs::write(&path, data).await.map_err(|e| AppError::Internal(e.to_string()))?;

        urls.push(serde_json::json!({
            "field": name,
            "url": format!("/data/exposes/{}", unique_name),
            "original_name": file_name
        }));
    }

    Ok(Json(serde_json::json!({ "files": urls })))
}
