use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Guild {
    pub id: i64,
    pub owner: i64,
    pub members: Vec<i64>,

    pub status: bool,

    pub alert_channel: i64,
    pub alert_role: i64,
    pub identifiers_showed: Vec<String>,

    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

