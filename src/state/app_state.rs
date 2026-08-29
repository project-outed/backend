use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;
use redis::Client;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: Arc<Client>,
    pub broadcast: broadcast::Sender<String>,
    pub http_client: reqwest::Client,
    pub api_key: String,
}

impl AppState {
    pub fn new(db: PgPool, redis: Client, broadcast: broadcast::Sender<String>, api_key: String) -> Self {
        Self {
            db,
            redis: Arc::new(redis),
            broadcast,
            http_client: reqwest::Client::new(),
            api_key,
        }
    }
}
