mod handlers;
mod models;
mod routes;
mod state;
mod utils;

use crate::state::AppState;
use dotenvy::dotenv;
use std::env;
use std::net::SocketAddr;
use tokio::sync::broadcast;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing::{info, debug};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "backend=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting backend service...");

    debug!("Loading environment variables");
    let db_url = env::var("DATABASE_URL")?;
    let redis_url = env::var("REDIS_CLUSTER_URL")?;
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string()).parse::<u16>()?;
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let db_pool = utils::database::connect(&db_url).await?;
    utils::database::initialize_db(&db_pool).await?;

    debug!("Connecting to Redis Cluster at {}", redis_url);
    let redis_client = utils::redis_client::create_client(&redis_url)?;
    
    debug!("Initializing broadcast channel");
    let (tx, _rx) = broadcast::channel(100);
    let api_key = env::var("API_KEY").unwrap_or_else(|_| "secret_default".to_string());
    
    debug!("Creating AppState");
    let state = AppState::new(db_pool, redis_client, tx, api_key);
    
    debug!("Creating router");
    let app = routes::create_router(state);

    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    info!("Server listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
