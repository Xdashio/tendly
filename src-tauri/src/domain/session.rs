use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingState {
    pub is_paused: bool,
    pub paused_at: Option<i64>,
    pub active_watchers_count: usize,
    pub started_at: i64,
}

impl Default for TrackingState {
    fn default() -> Self {
        Self {
            is_paused: false,
            paused_at: None,
            active_watchers_count: 0,
            started_at: chrono::Utc::now().timestamp(),
        }
    }
}
