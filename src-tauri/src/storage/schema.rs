//! DDL statements for database initialization and migrations.

pub const MIGRATION_TABLE_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at_ms INTEGER NOT NULL,
    description TEXT NOT NULL
);
"#;

pub const MIGRATION_001_FOUNDATION: &str = r#"
-- Raw immutable activity observations
CREATE TABLE IF NOT EXISTS raw_events (
    id TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    timestamp_ms INTEGER NOT NULL,
    app TEXT NOT NULL,
    title TEXT NOT NULL,
    url TEXT,
    idle_ms INTEGER,
    raw_json TEXT
);
CREATE INDEX IF NOT EXISTS idx_raw_events_timestamp ON raw_events(timestamp_ms);

-- 3-minute aggregated activity blocks
CREATE TABLE IF NOT EXISTS blocks (
    id TEXT PRIMARY KEY,
    start_ms INTEGER NOT NULL,
    end_ms INTEGER NOT NULL,
    dominant_app TEXT NOT NULL,
    dominant_title TEXT NOT NULL,
    dominant_url TEXT,
    classification TEXT,
    category TEXT,
    confidence REAL,
    classified_by TEXT,
    user_override TEXT
);
CREATE INDEX IF NOT EXISTS idx_blocks_start ON blocks(start_ms);

-- Tier 1 deterministic classification rules
CREATE TABLE IF NOT EXISTS classification_rules (
    id TEXT PRIMARY KEY,
    priority INTEGER NOT NULL DEFAULT 0,
    match_field TEXT NOT NULL,
    pattern TEXT NOT NULL,
    classification TEXT NOT NULL,
    category TEXT,
    source TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_rules_priority ON classification_rules(priority DESC);

-- Key-value persistent application state
CREATE TABLE IF NOT EXISTS app_state (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at_ms INTEGER NOT NULL
);
"#;

pub const MIGRATION_002_TIME_BLOCK_METADATA: &str = r#"
ALTER TABLE blocks ADD COLUMN activity_type TEXT NOT NULL DEFAULT 'active';
ALTER TABLE blocks ADD COLUMN duration_ms INTEGER NOT NULL DEFAULT 180000;
CREATE UNIQUE INDEX IF NOT EXISTS idx_blocks_unique_start ON blocks(start_ms);
"#;
