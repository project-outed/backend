use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DiscordEventWrapper {
    pub version: Option<i32>,
    pub application_id: Option<String>,
    #[serde(rename = "type")]
    pub event_type: i32,
    pub event: Option<DiscordEvent>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DiscordEvent {
    #[serde(rename = "type")]
    pub type_name: String,
    pub timestamp: String,
    pub data: DiscordEventData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DiscordEventData {
    pub integration_type: Option<i32>,
    pub scopes: Option<Vec<String>>,
    pub user: Option<DiscordUser>,
    pub guild: Option<DiscordGuild>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
    pub discriminator: Option<String>,
    pub public_flags: Option<i64>,
    pub flags: Option<i64>,
    pub banner: Option<String>,
    pub banner_color: Option<String>,
    pub accent_color: Option<i64>,
    pub locale: Option<String>,
    pub mfa_enabled: Option<bool>,
    pub premium_type: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DiscordGuild {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub owner_id: Option<String>,
}
