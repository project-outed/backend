use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

use super::providers::Providers;
use crate::models::report::Report;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct User {
    pub id: Option<i64>,
    pub user_id: i64,

    #[sqlx(json)]
    pub providers: Providers,

    pub mail: Option<String>,
    pub verified: Option<i32>,
    pub trust_score: Option<i32>,

    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,

    #[sqlx(skip)]
    pub reports: Vec<Report>,
}

impl User {
    pub fn calculate_trust_score(report_count: i64) -> i32 {
        match report_count {
            0 => 100,
            1 => 85,
            2 => 70,
            3 => 55,
            4 => 35,
            5 => 22,
            6 => 14,
            _ => 6,
        }
    }
}
