use anyhow::{Context, Result};
use redis::Client as RawRedisClient;
use redis::aio::MultiplexedConnection;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

#[derive(Clone)]
pub struct RedisClient {
    client: RawRedisClient,
    connection: Arc<RwLock<Option<MultiplexedConnection>>>,
}

impl RedisClient {
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = RawRedisClient::open(redis_url)
            .with_context(|| format!("Failed to create Redis client with URL: {}", redis_url))?;
        
        let connection: Arc<RwLock<Option<MultiplexedConnection>>> = Arc::new(RwLock::new(None));
        let client_clone = client.clone();
        let connection_clone = connection.clone();
        let url_clone = redis_url.to_string();

        tokio::spawn(async move {
            loop {
                let mut is_healthy = true;

                {
                    let read_guard = connection_clone.read().await;
                    if let Some(ref conn) = *read_guard {
                        let mut conn_clone = conn.clone();
                        let ping_res: Result<String, redis::RedisError> = redis::cmd("PING")
                            .query_async(&mut conn_clone)
                            .await;

                        if let Ok(res) = ping_res {
                            if res == "PONG" {
                                is_healthy = true;
                            } else {
                                is_healthy = false;
                            }
                        } else {
                            is_healthy = false;
                        }
                    }
                }

                if is_healthy {
                    sleep(Duration::from_secs(10)).await;
                } else {
                    tracing::warn!("Redis connection lost or not established. Retrying connection in 30 seconds...");
                    sleep(Duration::from_secs(30)).await;

                    match client_clone.get_multiplexed_async_connection().await {
                        Ok(new_conn) => {
                            let mut write_guard = connection_clone.write().await;
                            *write_guard = Some(new_conn);
                            tracing::info!("Successfully established/reconnected Redis connection.");
                        }
                        Err(e) => {
                            tracing::error!("Failed to reconnect to Redis at {}: {:?}", url_clone, e);
                        }
                    }
                }
            }
        });
        
        Ok(Self { client, connection })
    }

    pub async fn get_connection(&self) -> Result<MultiplexedConnection> {
        {
            let read_guard = self.connection.read().await;
            if let Some(ref conn) = *read_guard {
                return Ok(conn.clone());
            }
        }

        let mut write_guard = self.connection.write().await;
        if let Some(ref conn) = *write_guard {
            return Ok(conn.clone());
        }

        let conn = self.client
            .get_multiplexed_async_connection()
            .await
            .context("Failed to establish active async connection to Redis server")?;
        
        *write_guard = Some(conn.clone());
        Ok(conn)
    }
}