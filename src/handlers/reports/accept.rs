use axum::{extract::{Path, State}, Json};
use crate::state::AppState;
use crate::models::{Report, Evidence, User, BroadcastAcceptReport};
use crate::utils::AppError;
use redis::AsyncCommands;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub async fn accept_report(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<Report>, AppError> {
    let mut tx = state.db.begin().await?;

    sqlx::query(
        "UPDATE reports SET status = 'accepted', updated_at = NOW() WHERE id = $1"
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;

    let mut report = sqlx::query_as::<_, Report>(
        "SELECT id, target_username, target_user_id, reporter_username, reporter_user_id, game, reason, status, created_at, updated_at \
         FROM reports WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    report.evidence = sqlx::query_as::<_, Evidence>(
        "SELECT id, report_id, url, evidence_type, created_at FROM report_evidence WHERE report_id = $1"
    )
    .bind(id)
    .fetch_all(&mut *tx)
    .await?;

    let count: (i64,) = sqlx::query_as(
        "SELECT count(*) FROM reports WHERE target_user_id = $1 AND status = 'accepted'"
    )
    .bind(report.target_user_id)
    .fetch_one(&mut *tx)
    .await?;

    let new_trust_score = User::calculate_trust_score(count.0);

    sqlx::query(
        "UPDATE users SET trust_score = $1, updated_at = NOW() WHERE user_id = $2"
    )
    .bind(new_trust_score)
    .bind(report.target_user_id)
    .execute(&mut *tx)
    .await?;

    let providers_row: Option<(serde_json::Value,)> = sqlx::query_as(
        "SELECT providers FROM users WHERE user_id = $1"
    )
    .bind(report.target_user_id)
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;

    {
        let timestamp = chrono::Utc::now();

        let mut ids = serde_json::Map::new();
        if let Some((ref providers,)) = providers_row {
            if let Some(discord_id) = providers.get("discord").and_then(|d| d.get("id")).and_then(|v| v.as_str()) {
                ids.insert("discord".to_string(), serde_json::Value::String(discord_id.to_string()));
            }
        }
        let ids_json = serde_json::Value::Object(ids).to_string();

        let files: Vec<String> = report.evidence.iter().map(|e| e.url.clone()).collect();
        let files_json = serde_json::json!(files).to_string();

        let journal_entry = format!(
            "\n--- REPORT [{timestamp}] ---\nReporter: {reporter}\nGame: {game}\nCheat: {cheat}\nIDs: {ids}\nFiles: {files}\n---------------------\n",
            timestamp = timestamp,
            reporter = report.reporter_username,
            game = report.game,
            cheat = report.reason,
            ids = ids_json,
            files = files_json,
        );

        let user_dir = format!("data/exposes/{}", report.target_user_id);
        let _ = fs::create_dir_all(&user_dir).await;

        let journal_path = format!("{}/journal.txt", user_dir);
        if let Ok(mut file) = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&journal_path)
            .await
        {
            let _ = file.write_all(journal_entry.as_bytes()).await;
        }
    }

    let mut conn = state.redis.get_multiplexed_async_connection().await?;
    let _: () = conn.set_ex(format!("cheater_status:{}", report.target_user_id), "cheater", 3600).await?;

    let broadcast_payload = BroadcastAcceptReport {
        id: report.id,
        target_username: report.target_username.clone(),
        target_user_id: report.target_user_id.clone(),
        reporter_username: report.reporter_username.clone(),
        reporter_user_id: report.reporter_user_id.clone(),
        game: report.game.clone(),
        reason: report.reason.clone(),
        status: report.status.clone(),
        trust_score: new_trust_score,
    };

    let _ = state.broadcast.send(serde_json::json!({
        "event": "exposed_cheater",
        "data": broadcast_payload
    }).to_string());

    Ok(Json(report))
}
