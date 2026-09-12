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

pub fn insert_raw_event(conn: &Connection, event: &crate::domain::RawEvent) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO raw_events (id, source, timestamp_ms, app, title, url, idle_ms, raw_json)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
        rusqlite::params![
            event.id,
            event.source.as_str(),
            event.timestamp_ms,
            event.app,
            event.title,
            event.url,
            event.idle_ms,
            event.raw_json,
        ],
    )?;
    Ok(())
}

pub fn insert_raw_events_batch(
    conn: &mut Connection,
    events: &[crate::domain::RawEvent],
) -> Result<()> {
    if events.is_empty() {
        return Ok(());
    }

    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            r#"
            INSERT INTO raw_events (id, source, timestamp_ms, app, title, url, idle_ms, raw_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
        )?;

        for event in events {
            stmt.execute(rusqlite::params![
                event.id,
                event.source.as_str(),
                event.timestamp_ms,
                event.app,
                event.title,
                event.url,
                event.idle_ms,
                event.raw_json,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn get_recent_raw_events(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<crate::domain::RawEvent>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, source, timestamp_ms, app, title, url, idle_ms, raw_json
        FROM raw_events
        ORDER BY timestamp_ms DESC
        LIMIT ?1
        "#,
    )?;

    let rows = stmt.query_map(rusqlite::params![limit as i64], |row| {
        let source_str: String = row.get(1)?;
        let source = source_str
            .parse::<crate::domain::RawEventSource>()
            .unwrap_or(crate::domain::RawEventSource::X11);

        Ok(crate::domain::RawEvent {
            id: row.get(0)?,
            source,
            timestamp_ms: row.get(2)?,
            app: row.get(3)?,
            title: row.get(4)?,
            url: row.get(5)?,
            idle_ms: row.get(6)?,
            raw_json: row.get(7)?,
        })
    })?;

    let mut events = Vec::new();
    for row in rows {
        events.push(row?);
    }
    Ok(events)
}
