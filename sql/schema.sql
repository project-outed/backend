-- Consolidated schema for the backend, reflecting the tables and indexes
-- created/migrated by src/utils/database.rs::initialize_db.
-- Safe to run against a fresh database.

CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT UNIQUE NOT NULL,
    providers JSONB NOT NULL,
    mail VARCHAR(255),
    verified INT DEFAULT 0,
    trust_score INT DEFAULT 100,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS messages (
    id BIGSERIAL PRIMARY KEY,
    content TEXT NOT NULL,
    sender VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS reports (
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
);

CREATE INDEX IF NOT EXISTS idx_reports_target_user_id ON reports(target_user_id);

CREATE TABLE IF NOT EXISTS report_evidence (
    id BIGSERIAL PRIMARY KEY,
    report_id BIGINT NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    evidence_type VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS tickets (
    channel_id BIGINT PRIMARY KEY,
    guild_id BIGINT NOT NULL,
    owner_id BIGINT NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    ticket_type VARCHAR(50) NOT NULL,
    claimed_by BIGINT,
    status TEXT DEFAULT 'open' CHECK (status IN ('open', 'closed')),
    added_users JSONB,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_tickets_owner_id ON tickets(owner_id);

CREATE TABLE IF NOT EXISTS guilds (
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
);
