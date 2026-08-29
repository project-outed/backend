use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

use crate::models::Report;

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
            1 => 80,
            2 => 70,
            3 => 50,
            4 => 40,
            5 => 25,
            6 => 10,
            _ => 5,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Providers {
    #[serde(default)]
    pub discord: Option<DiscordProvider>,
    #[serde(default)]
    pub fivem: Option<FiveMProvider>,
    #[serde(default)]
    pub hardware_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DiscordProvider {
    #[serde(default)]
    pub username: String,
    pub id: String,
    #[serde(default)]
    pub avatar: String,
}


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FiveMProvider {
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub identifiers: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    #[serde(default)]
    pub user_id: Option<i64>,
    #[serde(default)]
    pub providers: Providers,
    #[serde(default)]
    pub mail: Option<String>,
    #[serde(default)]
    pub verified: i32,
    pub trust_score: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProviders {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discord: Option<DiscordProvider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fivem: Option<FiveMProvider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardware_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMailVerified {
    #[serde(default)]
    pub mail: Option<String>,
    #[serde(default)]
    pub verified: i32,
    pub trust_score: Option<i32>,
}
