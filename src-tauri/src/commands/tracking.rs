use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStatus {
    pub is_tracking_paused: bool,
    pub active_watchers_count: usize,
    pub uptime_seconds: u64,
}

#[tauri::command]
pub fn get_app_status(state: State<'_, AppState>) -> Result<AppStatus, crate::core::IpcError> {
    let tracking = state.tracking.lock().map_err(|e| crate::core::IpcError {
        code: "MUTEX_ERROR".to_string(),
        message: e.to_string(),
    })?;

    let uptime = chrono::Utc::now().timestamp() - tracking.started_at;

    Ok(AppStatus {
        is_tracking_paused: tracking.is_paused,
        active_watchers_count: tracking.active_watchers_count,
        uptime_seconds: uptime.max(0) as u64,
    })
}

#[tauri::command]
pub fn toggle_tracking_pause(state: State<'_, AppState>) -> Result<bool, crate::core::IpcError> {
    let mut tracking = state.tracking.lock().map_err(|e| crate::core::IpcError {
        code: "MUTEX_ERROR".to_string(),
        message: e.to_string(),
    })?;

    tracking.is_paused = !tracking.is_paused;
    let new_state = tracking.is_paused;

    if new_state {
        tracking.paused_at = Some(chrono::Utc::now().timestamp_millis());
    } else {
        tracking.paused_at = None;
    }

    // Persist paused state into database app_state table
    state
        .db
        .set_app_state(
            "is_tracking_paused",
            if new_state { "true" } else { "false" },
        )
        .map_err(crate::core::IpcError::from)?;

    tracing::info!(paused = new_state, "Toggled tracking pause state");

    Ok(new_state)
}
