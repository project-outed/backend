use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebsocketReportAccept {
    pub id: i64,
    pub target_user_id: i64,
    pub target_username: String,
    pub reporter_user_id: i64,
    pub reporter_username: String,
    pub game: String,
    pub reason: String,
    pub trust_score: i32,
    pub status: String,
}