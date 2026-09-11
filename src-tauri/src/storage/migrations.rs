use crate::core::error::{AppError, Result};
use crate::storage::schema::{MIGRATION_001_FOUNDATION, MIGRATION_TABLE_DDL};
use rusqlite::{Connection, Transaction};

struct Migration {
    version: i32,
    description: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    description: "001_initial_foundation",
    sql: MIGRATION_001_FOUNDATION,
}];

pub fn run_migrations(conn: &mut Connection) -> Result<i32> {
    // 1. Ensure migrations table exists
    conn.execute_batch(MIGRATION_TABLE_DDL)?;

    // 2. Fetch current applied version
    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let mut applied_version = current_version;

    // 3. Apply pending migrations sequentially in transactions
    for m in MIGRATIONS {
        if m.version > current_version {
            let tx = conn.transaction()?;
            apply_migration(&tx, m)?;
            tx.commit()?;
            applied_version = m.version;
            tracing::info!(
                version = m.version,
                description = m.description,
                "Applied schema migration"
            );
        }
    }

    Ok(applied_version)
}

fn apply_migration(tx: &Transaction, migration: &Migration) -> Result<()> {
    tx.execute_batch(migration.sql).map_err(|e| {
        AppError::Migration(format!("Failed executing {}: {}", migration.description, e))
    })?;

    let now_ms = chrono::Utc::now().timestamp_millis();
    tx.execute(
        "INSERT INTO schema_migrations (version, applied_at_ms, description) VALUES (?1, ?2, ?3)",
        rusqlite::params![migration.version, now_ms, migration.description],
    )
    .map_err(|e| {
        AppError::Migration(format!("Failed recording {}: {}", migration.description, e))
    })?;

    Ok(())
}
