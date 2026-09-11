use crate::core::error::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub database_path: String,
    pub database_size_bytes: u64,
    pub raw_events_count: i64,
    pub blocks_count: i64,
}

pub fn get_stats(conn: &Connection, db_path: &str) -> Result<DatabaseStats> {
    let raw_events_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM raw_events", [], |r| r.get(0))?;
    let blocks_count: i64 = conn.query_row("SELECT COUNT(*) FROM blocks", [], |r| r.get(0))?;

    let size_bytes = if db_path == ":memory:" {
        0
    } else {
        std::fs::metadata(db_path).map(|m| m.len()).unwrap_or(0)
    };

    Ok(DatabaseStats {
        database_path: db_path.to_string(),
        database_size_bytes: size_bytes,
        raw_events_count,
        blocks_count,
    })
}

pub fn get_state(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM app_state WHERE key = ?1")?;
    let mut rows = stmt.query(rusqlite::params![key])?;

    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub fn set_state(conn: &Connection, key: &str, value: &str) -> Result<()> {
    let now_ms = chrono::Utc::now().timestamp_millis();
    conn.execute(
        r#"
        INSERT INTO app_state (key, value, updated_at_ms)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(key) DO UPDATE SET
            value = excluded.value,
            updated_at_ms = excluded.updated_at_ms
        "#,
        rusqlite::params![key, value, now_ms],
    )?;
    Ok(())
}

pub fn wipe_all_data(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        DELETE FROM raw_events;
        DELETE FROM blocks;
        DELETE FROM classification_rules WHERE source = 'user';
        DELETE FROM app_state;
        VACUUM;
        "#,
    )?;
    Ok(())
}
