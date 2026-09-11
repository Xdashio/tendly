use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Watcher error: {0}")]
    Watcher(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcError {
    pub code: String,
    pub message: String,
}

impl From<AppError> for IpcError {
    fn from(err: AppError) -> Self {
        match err {
            AppError::Database(_) => IpcError {
                code: "DATABASE_ERROR".to_string(),
                message: "A database operation failed safely without leaking private context"
                    .to_string(),
            },
            AppError::Migration(msg) => IpcError {
                code: "MIGRATION_ERROR".to_string(),
                message: msg,
            },
            AppError::Configuration(msg) => IpcError {
                code: "CONFIG_ERROR".to_string(),
                message: msg,
            },
            AppError::Watcher(msg) => IpcError {
                code: "WATCHER_ERROR".to_string(),
                message: msg,
            },
            AppError::Io(err) => IpcError {
                code: "IO_ERROR".to_string(),
                message: err.to_string(),
            },
            AppError::Internal(msg) => IpcError {
                code: "INTERNAL_ERROR".to_string(),
                message: msg,
            },
        }
    }
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        IpcError::from(match self {
            AppError::Database(_) => AppError::Internal("Database error".to_string()),
            AppError::Migration(s) => AppError::Migration(s.clone()),
            AppError::Configuration(s) => AppError::Configuration(s.clone()),
            AppError::Watcher(s) => AppError::Watcher(s.clone()),
            AppError::Io(e) => AppError::Internal(e.to_string()),
            AppError::Internal(s) => AppError::Internal(s.clone()),
        })
        .serialize(serializer)
    }
}
