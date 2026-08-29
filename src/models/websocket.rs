use serde::Serialize;

#[derive(Serialize)]
pub struct BroadcastAcceptReport {
    pub id: Option<i64>,
    pub target_username: String,
    pub target_user_id: i64,
    pub reporter_username: String,
    pub reporter_user_id: i64,
    pub game: String,
    pub reason: String,
    pub trust_score: i32,
    pub status: String,
}
