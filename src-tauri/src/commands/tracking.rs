use crate::capture::WatcherStatus;
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStatus {
    pub is_tracking_paused: bool,
    pub active_watchers_count: usize,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureStatus {
    pub is_tracking_paused: bool,
    pub active_watchers_count: usize,
    pub watchers: Vec<WatcherStatus>,
    pub total_raw_events: i64,
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
pub fn get_capture_status(
    state: State<'_, AppState>,
) -> Result<CaptureStatus, crate::core::IpcError> {
    let tracking = state.tracking.lock().map_err(|e| crate::core::IpcError {
        code: "MUTEX_ERROR".to_string(),
        message: e.to_string(),
    })?;

    let watchers = state.watchers.lock().map_err(|e| crate::core::IpcError {
        code: "MUTEX_ERROR".to_string(),
        message: e.to_string(),
    })?;

    let stats = state.db.get_stats().map_err(crate::core::IpcError::from)?;
    let uptime = (chrono::Utc::now().timestamp() - tracking.started_at).max(0) as u64;

    Ok(CaptureStatus {
        is_tracking_paused: tracking.is_paused,
        active_watchers_count: tracking.active_watchers_count,
        watchers: watchers.watcher_statuses(),
        total_raw_events: stats.raw_events_count,
        uptime_seconds: uptime,
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

    // Forward pause/resume state to in-process watchers
    let mut watchers = state.watchers.lock().map_err(|e| crate::core::IpcError {
        code: "MUTEX_ERROR".to_string(),
        message: e.to_string(),
    })?;

    if new_state {
        let _ = watchers.pause_all();
    } else {
        let _ = watchers.resume_all();
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

#[tauri::command]
pub fn get_current_activity(
    state: State<'_, AppState>,
) -> Result<Option<crate::processing::CurrentActivityState>, crate::core::IpcError> {
    crate::processing::ActivityProcessor::get_current_activity(&state.db)
        .map_err(crate::core::IpcError::from)
}

#[tauri::command]
pub fn get_recent_time_blocks(
    limit: usize,
    state: State<'_, AppState>,
) -> Result<Vec<crate::domain::TimeBlock>, crate::core::IpcError> {
    state
        .db
        .get_recent_time_blocks(limit)
        .map_err(crate::core::IpcError::from)
}

#[tauri::command]
pub fn reprocess_time_blocks(state: State<'_, AppState>) -> Result<usize, crate::core::IpcError> {
    crate::processing::ActivityProcessor::rebuild_all_history(&state.db)
        .map_err(crate::core::IpcError::from)
}

#[tauri::command]
pub fn get_block_composition(
    start_ms: i64,
    end_ms: i64,
    state: State<'_, AppState>,
) -> Result<Vec<crate::domain::ActivitySegment>, crate::core::IpcError> {
    crate::processing::ActivityProcessor::get_block_composition(&state.db, start_ms, end_ms)
        .map_err(crate::core::IpcError::from)
}
