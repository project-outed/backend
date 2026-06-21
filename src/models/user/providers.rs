use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Providers {
    pub discord: Option<DiscordProvider>,
    pub fivem: Option<FivemProvider>,
    pub steam: Option<SteamProvider>,
    pub microsoft: Option<MicrosoftProvider>,
    pub network: Option<NetworkProvider>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscordProvider {
    pub id: i64,
    pub username: String,
    pub avatar: String,
    pub added_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FivemProvider {
    pub fivem: Option<String>,
    pub license: Option<String>,
    pub license2: Option<String>,    
    pub added_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SteamProvider {
    pub added_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MicrosoftProvider {
    pub xbl: Option<String>,
    pub live: Option<String>,
    pub added_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkProvider {
    pub ipv4: Option<String>,
    pub country: Option<String>,
    pub asn: Option<i64>,
    pub added_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
