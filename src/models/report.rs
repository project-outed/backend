use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Report {
    pub id: Option<i64>,
    pub target_username: String,
    pub target_user_id: i64,
    pub reporter_username: String,
    pub reporter_user_id: i64,
    pub game: String,
    pub reason: String,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    #[sqlx(skip)]
    pub evidence: Vec<Evidence>,
}


#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Evidence {
    pub id: Option<i64>,
    pub report_id: i64,
    pub url: String,
    pub evidence_type: String, 
    pub created_at: Option<DateTime<Utc>>,
}
