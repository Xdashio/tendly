use crate::storage::queries::DatabaseStats;
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub environment: String,
    pub os: String,
    pub database_status: String,
    pub schema_version: i32,
}

#[tauri::command]
pub fn get_app_info(state: State<'_, AppState>) -> Result<AppInfo, crate::core::IpcError> {
    let schema_version = state
        .db
        .get_schema_version()
        .map_err(crate::core::IpcError::from)?;

    Ok(AppInfo {
        name: "tendly".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        environment: state.config.environment.clone(),
        os: std::env::consts::OS.to_string(),
        database_status: "connected (WAL mode)".to_string(),
        schema_version,
    })
}

#[tauri::command]
pub fn get_database_stats(
    state: State<'_, AppState>,
) -> Result<DatabaseStats, crate::core::IpcError> {
    state.db.get_stats().map_err(crate::core::IpcError::from)
}
