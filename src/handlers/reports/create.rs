use axum::{extract::{State, Multipart}, Json};
use crate::state::AppState;
use crate::models::{Report, Evidence};
use crate::utils::AppError;
use crate::utils::discord::get_discord_info;
use tokio::fs;

struct CreateEvidenceRequest {
    pub url: String,
    pub evidence_type: String,
}

pub async fn create_report(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Report>, AppError> {

    let mut target_username_input = String::new();
    let mut target_id = String::new();
    let mut reporter_username_input = String::new();
    let mut reporter_id = String::new();
    let mut game = String::new();
    let mut reason = String::new();
    let mut temp_files = Vec::new();

    while let Some(mut field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "target_user_id" | "target_id" | "target_userid" => target_id = field.text().await.unwrap_or_default(),
            "target_username" | "target_user_name" => target_username_input = field.text().await.unwrap_or_default(),
            "reporter_user_id" | "reporter_id" => reporter_id = field.text().await.unwrap_or_default(),
            "reporter_username" => reporter_username_input = field.text().await.unwrap_or_default(),
            "game" => game = field.text().await.unwrap_or_default(),
            "reason" | "cheat" => reason = field.text().await.unwrap_or_default(),
            "evidence[]" | "evidence" => {
                let file_name = field.file_name().unwrap_or("evidence.dat").to_string();
                let target_dir = format!("data/tmp");
                fs::create_dir_all(&target_dir).await.map_err(|e| AppError::Internal(e.to_string()))?;
                let temp_path = std::path::PathBuf::from(&target_dir).join(format!("{}_{}", chrono::Utc::now().timestamp(), file_name));
                
                let mut file = fs::File::create(&temp_path).await.map_err(|e| AppError::Internal(e.to_string()))?;
                while let Some(chunk) = field.chunk().await.map_err(|e| AppError::Internal(e.to_string()))? {
                    tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await.map_err(|e| AppError::Internal(e.to_string()))?;
                }
                temp_files.push((file_name, temp_path));
            }
            _ => {}
        }
    }

    if target_id.is_empty() || reporter_id.is_empty() {
        return Err(AppError::BadRequest("target_id and reporter_id are required".to_string()));
    }

    let mut target_username = if !target_username_input.is_empty() { target_username_input } else { "Unknown".to_string() };
    let mut target_discord_avatar = String::new();
    let target_user_id_val: i64 = target_id.parse().unwrap_or(0);

    if !target_id.is_empty() {
        if let Ok(Some(info)) = get_discord_info(&state.http_client, &target_id).await {
            if let Some(uname) = info.get("username").and_then(|v| v.as_str()) {
                target_username = uname.to_string();
            }
            if let Some(avatar) = info.get("avatar").and_then(|v| v.as_str()) {
                target_discord_avatar = avatar.to_string();
            }
        }
    }

    let mut reporter_username = if !reporter_username_input.is_empty() { reporter_username_input } else { "Unknown".to_string() };
    let reporter_user_id_val: i64 = reporter_id.parse().unwrap_or(0);

    if let Ok(Some(info)) = get_discord_info(&state.http_client, &reporter_id).await {
        if let Some(uname) = info.get("username").and_then(|v| v.as_str()) {
            reporter_username = uname.to_string();
        }
    }

    let mut files_metadata = Vec::new();
    if !temp_files.is_empty() {
        let target_dir = format!("data/exposes/{}", target_user_id_val);
        let evidence_dir = format!("{}/evidence", target_dir);
        fs::create_dir_all(&evidence_dir).await.map_err(|e| AppError::Internal(e.to_string()))?;

        for (file_name, temp_path) in temp_files {
            let unique_name = format!("{}_{}", chrono::Utc::now().timestamp(), file_name);
            let final_path = std::path::PathBuf::from(&evidence_dir).join(&unique_name);
            fs::rename(&temp_path, &final_path).await.map_err(|e| AppError::Internal(e.to_string()))?;

            files_metadata.push(CreateEvidenceRequest {
                url: format!("/data/exposes/{}/evidence/{}", target_user_id_val, unique_name),
                evidence_type: "image".to_string(),
            });
        }
    }

    let mut tx = state.db.begin().await?;

    let mut user_providers = serde_json::Map::new();
    let mut discord_provider = serde_json::Map::new();
    discord_provider.insert("id".to_string(), serde_json::Value::String(target_id.clone()));
    discord_provider.insert("username".to_string(), serde_json::Value::String(target_username.clone()));
    if !target_discord_avatar.is_empty() {
        discord_provider.insert("avatar".to_string(), serde_json::Value::String(target_discord_avatar.clone()));
    }
    user_providers.insert("discord".to_string(), serde_json::Value::Object(discord_provider));
    
    let providers_value = serde_json::Value::Object(user_providers);

    sqlx::query(
        "INSERT INTO users (user_id, providers, trust_score) VALUES ($1, $2, 100) ON CONFLICT (user_id) DO UPDATE SET providers = users.providers || EXCLUDED.providers"
    )
    .bind(target_user_id_val)
    .bind(providers_value)
    .execute(&mut *tx)
    .await?;

    let row: (i64,) = sqlx::query_as(
        "INSERT INTO reports (target_username, target_user_id, reporter_username, reporter_user_id, game, reason, status) VALUES ($1, $2, $3, $4, $5, $6, 'pending') RETURNING id"
    )
    .bind(&target_username)
    .bind(target_user_id_val)
    .bind(&reporter_username)
    .bind(reporter_user_id_val)
    .bind(&game)
    .bind(&reason)
    .fetch_one(&mut *tx)
    .await?;

    let report_id = row.0;

    let mut evidence_list = Vec::new();
    if !files_metadata.is_empty() {
        for ev in files_metadata {
            let ev_row: (i64,) = sqlx::query_as(
                "INSERT INTO report_evidence (report_id, url, evidence_type) VALUES ($1, $2, $3) RETURNING id"
            )
            .bind(report_id)
            .bind(&ev.url)
            .bind(&ev.evidence_type)
            .fetch_one(&mut *tx)
            .await?;

            evidence_list.push(Evidence {
                id: Some(ev_row.0),
                report_id,
                url: ev.url,
                evidence_type: ev.evidence_type,
                created_at: Some(chrono::Utc::now()),
            });
        }
    }

    tx.commit().await?;

    let new_report = Report {
        id: Some(report_id),
        target_username,
        target_user_id: target_user_id_val,
        reporter_username,
        reporter_user_id: reporter_user_id_val,
        game,
        reason,
        status: "pending".to_string(),
        created_at: Some(chrono::Utc::now()),
        updated_at: Some(chrono::Utc::now()),
        evidence: evidence_list,
    };

    Ok(Json(new_report))
}
