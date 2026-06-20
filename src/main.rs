use anyhow::{Context, Result};
use tokio::sync::broadcast;

mod state;
mod utils;
mod routes;
mod handlers;

use utils::redis_client::RedisClient;
use utils::database_client::DatabaseClient;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();

    // 1. Initialize Redis Client
    let redis_url = std::env::var("REDIS_URL")
        .context("REDIS_URL environment variable is not set in .env")?;

    let redis_client = RedisClient::new(&redis_url)
        .context("Failed to initialize Redis client")?;

    if let Ok(mut connection) = redis_client.get_connection().await {
        let ping_response: String = redis::cmd("PING")
            .query_async(&mut connection)
            .await
            .context("Failed to ping Redis server")?;

        if ping_response == "PONG" {
            tracing::info!("Successfully connected to Redis.");
        }
    }

    // 2. Initialize PostgreSQL Client
    let postgres_url = std::env::var("POSTGRES_URL")
        .context("POSTGRES_URL environment variable is not set in .env")?;

    let db_client = DatabaseClient::new(&postgres_url)
        .await
        .context("Failed to initialize database client")?;

    db_client.ping().await
        .context("Failed to ping database")?;

    tracing::info!("Successfully connected to PostgreSQL database.");

    // 3. Retrieve secrets
    let encryption_secret = std::env::var("ENCRYPTION_SECRET")
        .context("ENCRYPTION_SECRET environment variable is not set in .env")?;
    let encryption_apikey = std::env::var("ENCRYPTION_APIKEY")
        .context("ENCRYPTION_APIKEY environment variable is not set in .env")?;

    // 4. Initialize Broadcast channel for WebSockets
    let (ws_sender, _) = broadcast::channel::<String>(100);

    // 5. Build AppState
    let state = state::AppState::new(
        ws_sender,
        db_client.get_pool(),
        redis_client,
        encryption_secret,
        encryption_apikey,
    );

    // 6. Build Router and Bind server listener
    let app = routes::app_router().with_state(state);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await
        .context("Failed to bind TCP listener to port 8080")?;
    
    tracing::info!("Web server running on http://0.0.0.0:8080");

    // 7. Start Axum Server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Error running Axum server")?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C handler");
    tracing::info!("Shutdown signal received, exiting backend server.");
}
