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

pub fn get_raw_events_range(
    conn: &Connection,
    start_ms: i64,
    end_ms: i64,
) -> Result<Vec<crate::domain::RawEvent>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, source, timestamp_ms, app, title, url, idle_ms, raw_json
        FROM raw_events
        WHERE timestamp_ms >= ?1 AND timestamp_ms <= ?2
        ORDER BY timestamp_ms ASC, id ASC
        "#,
    )?;

    let rows = stmt.query_map(rusqlite::params![start_ms, end_ms], |row| {
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

pub fn get_earliest_raw_event_timestamp(conn: &Connection) -> Result<Option<i64>> {
    let ts: Option<i64> = conn
        .query_row("SELECT MIN(timestamp_ms) FROM raw_events", [], |row| {
            row.get(0)
        })
        .unwrap_or(None);
    Ok(ts)
}

pub fn get_latest_time_block_timestamp(conn: &Connection) -> Result<Option<i64>> {
    let ts: Option<i64> = conn
        .query_row("SELECT MAX(end_ms) FROM blocks", [], |row| row.get(0))
        .unwrap_or(None);
    Ok(ts)
}

pub fn insert_time_block(conn: &Connection, block: &crate::domain::TimeBlock) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO blocks (
            id, start_ms, end_ms, duration_ms, activity_type, dominant_app, dominant_title,
            dominant_url, classification, category, confidence, classified_by, user_override
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        ON CONFLICT(start_ms) DO UPDATE SET
            end_ms = excluded.end_ms,
            duration_ms = excluded.duration_ms,
            activity_type = excluded.activity_type,
            dominant_app = excluded.dominant_app,
            dominant_title = excluded.dominant_title,
            dominant_url = excluded.dominant_url,
            classification = COALESCE(blocks.classification, excluded.classification),
            category = COALESCE(blocks.category, excluded.category),
            confidence = COALESCE(blocks.confidence, excluded.confidence),
            classified_by = COALESCE(blocks.classified_by, excluded.classified_by),
            user_override = COALESCE(blocks.user_override, excluded.user_override)
        "#,
        rusqlite::params![
            block.id,
            block.start_ms,
            block.end_ms,
            block.duration_ms,
            block.activity_type.as_str(),
            block.dominant_app,
            block.dominant_title,
            block.dominant_url,
            block.classification.map(|c| c.as_str()),
            block.category,
            block.confidence,
            block.classified_by,
            block.user_override.map(|c| c.as_str()),
        ],
    )?;
    Ok(())
}

pub fn insert_time_blocks_batch(
    conn: &mut Connection,
    blocks: &[crate::domain::TimeBlock],
) -> Result<()> {
    if blocks.is_empty() {
        return Ok(());
    }

    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            r#"
            INSERT INTO blocks (
                id, start_ms, end_ms, duration_ms, activity_type, dominant_app, dominant_title,
                dominant_url, classification, category, confidence, classified_by, user_override
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(start_ms) DO UPDATE SET
                end_ms = excluded.end_ms,
                duration_ms = excluded.duration_ms,
                activity_type = excluded.activity_type,
                dominant_app = excluded.dominant_app,
                dominant_title = excluded.dominant_title,
                dominant_url = excluded.dominant_url,
                classification = COALESCE(blocks.classification, excluded.classification),
                category = COALESCE(blocks.category, excluded.category),
                confidence = COALESCE(blocks.confidence, excluded.confidence),
                classified_by = COALESCE(blocks.classified_by, excluded.classified_by),
                user_override = COALESCE(blocks.user_override, excluded.user_override)
            "#,
        )?;

        for block in blocks {
            stmt.execute(rusqlite::params![
                block.id,
                block.start_ms,
                block.end_ms,
                block.duration_ms,
                block.activity_type.as_str(),
                block.dominant_app,
                block.dominant_title,
                block.dominant_url,
                block.classification.map(|c| c.as_str()),
                block.category,
                block.confidence,
                block.classified_by,
                block.user_override.map(|c| c.as_str()),
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn get_time_blocks_range(
    conn: &Connection,
    start_ms: i64,
    end_ms: i64,
) -> Result<Vec<crate::domain::TimeBlock>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, start_ms, end_ms, duration_ms, activity_type, dominant_app, dominant_title,
               dominant_url, classification, category, confidence, classified_by, user_override
        FROM blocks
        WHERE start_ms >= ?1 AND start_ms <= ?2
        ORDER BY start_ms ASC
        "#,
    )?;

    let rows = stmt.query_map(rusqlite::params![start_ms, end_ms], map_time_block_row)?;

    let mut blocks = Vec::new();
    for row in rows {
        blocks.push(row?);
    }
    Ok(blocks)
}

pub fn get_recent_time_blocks(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<crate::domain::TimeBlock>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, start_ms, end_ms, duration_ms, activity_type, dominant_app, dominant_title,
               dominant_url, classification, category, confidence, classified_by, user_override
        FROM blocks
        ORDER BY start_ms DESC
        LIMIT ?1
        "#,
    )?;

    let rows = stmt.query_map(rusqlite::params![limit as i64], map_time_block_row)?;

    let mut blocks = Vec::new();
    for row in rows {
        blocks.push(row?);
    }
    Ok(blocks)
}

fn map_time_block_row(row: &rusqlite::Row) -> rusqlite::Result<crate::domain::TimeBlock> {
    let type_str: String = row.get(4)?;
    let activity_type = type_str
        .parse::<crate::domain::ActivityType>()
        .unwrap_or(crate::domain::ActivityType::Active);

    let class_str: Option<String> = row.get(8)?;
    let classification = class_str.and_then(|s| s.parse::<crate::domain::Classification>().ok());

    let override_str: Option<String> = row.get(12)?;
    let user_override = override_str.and_then(|s| s.parse::<crate::domain::Classification>().ok());

    Ok(crate::domain::TimeBlock {
        id: row.get(0)?,
        start_ms: row.get(1)?,
        end_ms: row.get(2)?,
        duration_ms: row.get(3)?,
        activity_type,
        dominant_app: row.get(5)?,
        dominant_title: row.get(6)?,
        dominant_url: row.get(7)?,
        classification,
        category: row.get(9)?,
        confidence: row.get(10)?,
        classified_by: row.get(11)?,
        user_override,
    })
}
