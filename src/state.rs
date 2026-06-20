use sqlx::PgPool;
use reqwest::Client as HttpClient;
use crate::utils::redis_client::RedisClient;
use tokio::sync::broadcast::Sender as WebsocketSender;

use std::time::Duration as Time;

#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub websocket: WebsocketSender<String>,
    pub database: PgPool,
    pub redis: RedisClient,
    pub http: HttpClient,
    pub encryption_secret: String,
    pub encryption_apikey: String,
}

impl AppState {
    pub fn new(
        websocket: WebsocketSender<String>,
        database: PgPool,
        redis: RedisClient,
        encryption_secret: String,
        encryption_apikey: String,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Time::from_secs(10))
            .pool_max_idle_per_host(10)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            websocket,
            database,
            redis,
            http: http_client,
            encryption_secret,
            encryption_apikey,
        }
    }
}