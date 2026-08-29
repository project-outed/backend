use sqlx::PgPool;
use anyhow::{Context, Result};
use tracing::{info, debug};

pub async fn connect(url: &str) -> Result<PgPool> {
    info!("Connecting to database...");
    let pool = PgPool::connect(url)
        .await
        .context("Failed to connect to PostgreSQL database")?;
    info!("Database connection established");
    Ok(pool)
}

pub async fn initialize_db(pool: &PgPool) -> Result<()> {
    info!("Initializing database tables...");

    debug!("Creating users table if not exists");
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id BIGSERIAL PRIMARY KEY,
            user_id BIGINT UNIQUE NOT NULL,
            providers JSONB NOT NULL,
            mail VARCHAR(255),
            verified INT DEFAULT 0,
            trust_score INT DEFAULT 100,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(pool)
    .await?;

    let _ = sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS user_id BIGINT UNIQUE").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE users ALTER COLUMN mail DROP NOT NULL").execute(pool).await;

    debug!("Creating messages table if not exists");
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS messages (
            id BIGSERIAL PRIMARY KEY,
            content TEXT NOT NULL,
            sender VARCHAR(255) NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(pool)
    .await?;

    debug!("Creating reports table if not exists");
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS reports (
            id BIGSERIAL PRIMARY KEY,
            target_username VARCHAR(255) NOT NULL,
            target_user_id BIGINT NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
            reporter_username VARCHAR(255) NOT NULL,
            reporter_user_id BIGINT NOT NULL,
            game VARCHAR(255) NOT NULL,
            reason TEXT NOT NULL,
            status VARCHAR(20) DEFAULT 'pending',
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(pool)
    .await?;

    let _ = sqlx::query("ALTER TABLE reports RENAME COLUMN target_user_name TO target_username").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE reports RENAME COLUMN target_userid TO target_user_id").execute(pool).await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_reports_target_user_id ON reports(target_user_id)").execute(pool).await;

    debug!("Creating report_evidence table if not exists");
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS report_evidence (
            id BIGSERIAL PRIMARY KEY,
            report_id BIGINT NOT NULL,
            url TEXT NOT NULL,
            evidence_type VARCHAR(50) NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (report_id) REFERENCES reports(id) ON DELETE CASCADE
        )"
    )
    .execute(pool)
    .await?;

    debug!("Creating tickets table if not exists");
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS tickets (
            channel_id BIGINT PRIMARY KEY,
            guild_id BIGINT NOT NULL,
            owner_id BIGINT NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
            ticket_type VARCHAR(50) NOT NULL,
            claimed_by BIGINT,
            status TEXT DEFAULT 'open' CHECK (status IN ('open', 'closed')),
            added_users JSONB,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(pool)
    .await?;

    debug!("Creating guilds table if not exists");
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS guilds (
            id BIGSERIAL PRIMARY KEY,
            guild_id BIGINT UNIQUE NOT NULL,
            guild_name TEXT NOT NULL,
            guild_owner BIGINT NOT NULL,
            guild_members BIGINT[] DEFAULT '{}',
            alert_channel BIGINT,
            alert_role BIGINT,
            identifiers_showed TEXT[] DEFAULT '{}',
            is_active BOOLEAN DEFAULT TRUE,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(pool)
    .await?;

    debug!("Running migrations for guilds table");
    let _ = sqlx::query("ALTER TABLE guilds ADD COLUMN IF NOT EXISTS id BIGSERIAL").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE guilds ADD COLUMN IF NOT EXISTS guild_name TEXT").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE guilds ADD COLUMN IF NOT EXISTS guild_owner BIGINT").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE guilds ADD COLUMN IF NOT EXISTS guild_members BIGINT[] DEFAULT '{}'").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE guilds ADD COLUMN IF NOT EXISTS alert_channel BIGINT").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE guilds ADD COLUMN IF NOT EXISTS alert_role BIGINT").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE guilds ADD COLUMN IF NOT EXISTS identifiers_showed TEXT[] DEFAULT '{}'").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE guilds ADD COLUMN IF NOT EXISTS is_active BOOLEAN DEFAULT TRUE").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE guilds RENAME COLUMN name TO guild_name").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE guilds RENAME COLUMN owner_id TO guild_owner").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE guilds DROP COLUMN IF EXISTS member_count").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE guilds DROP COLUMN IF EXISTS icon_url").execute(pool).await;

    let _ = sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS trust_score INT DEFAULT 100").execute(pool).await;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_tickets_owner_id ON tickets(owner_id)").execute(pool).await?;
    
    info!("Database initialization complete");
    Ok(())
}
