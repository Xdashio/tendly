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
        assert_eq!(version, 1);
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
}
