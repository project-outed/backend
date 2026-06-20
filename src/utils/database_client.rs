use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

#[derive(Clone)]
pub struct DatabaseClient {
    pool: PgPool,
}

impl DatabaseClient {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .connect(database_url)
            .await
            .with_context(|| format!("Failed to connect to PostgreSQL database"))?;

        Ok(Self { pool })
    }

    pub fn get_pool(&self) -> PgPool {
        self.pool.clone()
    }

    pub async fn ping(&self) -> Result<()> {
        use sqlx::Connection;
        let mut conn = self.pool.acquire().await
            .context("Failed to acquire connection from pool")?;
        conn.ping().await
            .context("Failed to ping database connection")?;
        Ok(())
    }
}
