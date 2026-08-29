use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Guild {
    pub id: i64,
    pub guild_id: i64,
    pub guild_name: String,
    pub guild_owner: i64,
    pub guild_members: Vec<i64>,
    pub alert_channel: Option<i64>,
    pub alert_role: Option<i64>,
    pub identifiers_showed: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGuild {
    pub guild_id: String,
    pub guild_name: String,
    pub guild_owner: String,
    pub guild_members: Vec<String>,
    pub alert_channel: Option<String>,
    pub alert_role: Option<String>,
    pub identifiers_showed: Vec<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct GuildStats {
    pub total_guilds: i64,
    pub total_members: i64,
}
