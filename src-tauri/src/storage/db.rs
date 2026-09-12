use crate::core::error::{AppError, Result};
use crate::storage::migrations::run_migrations;
use crate::storage::queries::{self, DatabaseStats};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct DatabaseManager {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl DatabaseManager {
    pub fn open(db_path: &Path) -> Result<Self> {
        // Ensure parent directory exists if not an in-memory database
        if db_path != Path::new(":memory:") {
            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let mut conn = Connection::open(db_path)?;

        // Configure connection for performance, durability, and concurrency
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
            PRAGMA busy_timeout = 5000;
            "#,
        )?;

        // Restrict filesystem permissions to 0600 on Unix systems
        #[cfg(unix)]
        if db_path != Path::new(":memory:") && db_path.exists() {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            let _ = std::fs::set_permissions(db_path, perms);
        }

        // Execute pending migrations
        let current_version = run_migrations(&mut conn)?;
        tracing::info!(version = current_version, path = ?db_path, "Database opened and migrations initialized");

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: db_path.to_path_buf(),
        })
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::open(Path::new(":memory:"))
    }

    pub fn get_schema_version(&self) -> Result<i32> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let version: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(version)
    }

    pub fn get_stats(&self) -> Result<DatabaseStats> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        queries::get_stats(&conn, &self.db_path.to_string_lossy())
    }

    pub fn get_app_state(&self, key: &str) -> Result<Option<String>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        queries::get_state(&conn, key)
    }

    pub fn set_app_state(&self, key: &str, value: &str) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        queries::set_state(&conn, key, value)
    }

    pub fn wipe_all_data(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        queries::wipe_all_data(&conn)
    }

    pub fn insert_raw_event(&self, event: &crate::domain::RawEvent) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        queries::insert_raw_event(&conn, event)
    }

    pub fn insert_raw_events_batch(&self, events: &[crate::domain::RawEvent]) -> Result<()> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        queries::insert_raw_events_batch(&mut conn, events)
    }

    pub fn get_recent_raw_events(&self, limit: usize) -> Result<Vec<crate::domain::RawEvent>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        queries::get_recent_raw_events(&conn, limit)
    }

    pub fn get_raw_events_range(
        &self,
        start_ms: i64,
        end_ms: i64,
    ) -> Result<Vec<crate::domain::RawEvent>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        queries::get_raw_events_range(&conn, start_ms, end_ms)
    }

    pub fn get_earliest_raw_event_timestamp(&self) -> Result<Option<i64>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        queries::get_earliest_raw_event_timestamp(&conn)
    }

    pub fn get_latest_time_block_timestamp(&self) -> Result<Option<i64>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        queries::get_latest_time_block_timestamp(&conn)
    }

    pub fn insert_time_block(&self, block: &crate::domain::TimeBlock) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        queries::insert_time_block(&conn, block)
    }

    pub fn insert_time_blocks_batch(&self, blocks: &[crate::domain::TimeBlock]) -> Result<()> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        queries::insert_time_blocks_batch(&mut conn, blocks)
    }

    pub fn get_time_blocks_range(
        &self,
        start_ms: i64,
        end_ms: i64,
    ) -> Result<Vec<crate::domain::TimeBlock>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        queries::get_time_blocks_range(&conn, start_ms, end_ms)
    }

    pub fn get_recent_time_blocks(&self, limit: usize) -> Result<Vec<crate::domain::TimeBlock>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        queries::get_recent_time_blocks(&conn, limit)
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory_and_migrate() {
        let db = DatabaseManager::open_in_memory().expect("Must open in-memory db");
        let version = db.get_schema_version().expect("Must read schema version");
        assert_eq!(version, 2);
    }

    #[test]
    fn test_state_get_set() {
        let db = DatabaseManager::open_in_memory().expect("Must open in-memory db");
        db.set_app_state("test_key", "test_value")
            .expect("Must set state");

        let val = db.get_app_state("test_key").expect("Must get state");
        assert_eq!(val, Some("test_value".to_string()));

        let none_val = db.get_app_state("non_existent").expect("Must get none");
        assert_eq!(none_val, None);
    }

    #[test]
    fn test_stats_and_wipe() {
        let db = DatabaseManager::open_in_memory().expect("Must open in-memory db");
        let stats = db.get_stats().expect("Must get stats");
        assert_eq!(stats.raw_events_count, 0);
        assert_eq!(stats.blocks_count, 0);

        db.wipe_all_data().expect("Wipe must succeed");
    }

    #[test]
    fn test_raw_events_insert_and_query() {
        use crate::domain::{RawEvent, RawEventSource};

        let db = DatabaseManager::open_in_memory().expect("Must open in-memory db");
        let event1 = RawEvent {
            id: "evt-1".to_string(),
            source: RawEventSource::X11,
            timestamp_ms: 1000,
            app: "code".to_string(),
            title: "tendly - Visual Studio Code".to_string(),
            url: None,
            idle_ms: None,
            raw_json: None,
        };
        let event2 = RawEvent {
            id: "evt-2".to_string(),
            source: RawEventSource::Afk,
            timestamp_ms: 2000,
            app: "system".to_string(),
            title: "afk".to_string(),
            url: None,
            idle_ms: Some(300000),
            raw_json: None,
        };

        db.insert_raw_event(&event1)
            .expect("Must insert single event");
        db.insert_raw_events_batch(&[event2])
            .expect("Must insert batch");

        let stats = db.get_stats().expect("Must get stats");
        assert_eq!(stats.raw_events_count, 2);

        let recent = db
            .get_recent_raw_events(10)
            .expect("Must get recent events");
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].id, "evt-2");
        assert_eq!(recent[1].id, "evt-1");
    }
}
