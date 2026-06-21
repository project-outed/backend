use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Evidence {
    pub id: Option<i64>,
    pub report_id: i64,
    pub url: String,
    pub evidence_type: String, 
    pub created_at: Option<DateTime<Utc>>,
}
