use crate::core::error::{AppError, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub data_dir: PathBuf,
    pub db_path: PathBuf,
    pub environment: String,
    pub block_duration_seconds: u64,
    pub idle_threshold_seconds: u64,
}

impl AppConfig {
    pub fn for_production() -> Result<Self> {
        let base_dir = dirs::data_local_dir()
            .or_else(dirs::data_dir)
            .ok_or_else(|| {
                AppError::Configuration("Could not determine user data directory".to_string())
            })?;

        let data_dir = base_dir.join("tendly");
        let db_path = data_dir.join("tendly.db");

        Ok(Self {
            data_dir,
            db_path,
            environment: "production".to_string(),
            block_duration_seconds: 180, // 3 minutes per Activity Model
            idle_threshold_seconds: 300, // 5 minutes per Activity Model
        })
    }

    pub fn for_test(temp_dir: &Path) -> Self {
        let data_dir = temp_dir.to_path_buf();
        let db_path = data_dir.join("test_tendly.db");

        Self {
            data_dir,
            db_path,
            environment: "test".to_string(),
            block_duration_seconds: 180,
            idle_threshold_seconds: 300,
        }
    }
}
