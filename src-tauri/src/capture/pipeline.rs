use crate::domain::{RawEvent, RawEventSource};
use crate::storage::DatabaseManager;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;

pub struct CapturePipeline {
    is_running: Arc<AtomicBool>,
}

pub struct DeduplicationFilter {
    last_event: Option<RawEvent>,
    checkpoint_interval_ms: i64,
}

impl Default for DeduplicationFilter {
    fn default() -> Self {
        Self::new(60_000) // 60-second checkpoint
    }
}

impl DeduplicationFilter {
    pub fn new(checkpoint_interval_ms: i64) -> Self {
        Self {
            last_event: None,
            checkpoint_interval_ms,
        }
    }

    /// Determines whether the event should be persisted to SQLite.
    pub fn should_persist(&mut self, event: &RawEvent) -> bool {
        // AFK transitions are always persisted
        if event.source == RawEventSource::Afk {
            self.last_event = Some(event.clone());
            return true;
        }

        match &self.last_event {
            None => {
                self.last_event = Some(event.clone());
                true
            }
            Some(last) => {
                let same_app = last.app == event.app;
                let same_title = last.title == event.title;
                let elapsed = (event.timestamp_ms - last.timestamp_ms).abs();

                if !same_app || !same_title {
                    // State changed
                    self.last_event = Some(event.clone());
                    true
                } else if elapsed >= self.checkpoint_interval_ms {
                    // Checkpoint due for long-duration single window focus
                    self.last_event = Some(event.clone());
                    true
                } else {
                    // Redundant event within checkpoint window
                    false
                }
            }
        }
    }
}

impl Default for CapturePipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl CapturePipeline {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(
        &self,
        mut rx: Receiver<RawEvent>,
        db: DatabaseManager,
    ) -> tauri::async_runtime::JoinHandle<()> {
        self.is_running.store(true, Ordering::SeqCst);
        let is_running = Arc::clone(&self.is_running);

        let fut = async move {
            let mut filter = DeduplicationFilter::default();

            while is_running.load(Ordering::SeqCst) {
                match rx.recv().await {
                    Some(event) => {
                        if filter.should_persist(&event) {
                            if let Err(e) = db.insert_raw_event(&event) {
                                tracing::error!(
                                    error = %e,
                                    "Failed persisting RawEvent to database"
                                );
                            } else {
                                // Automatically aggregate current 6-minute window into TimeBlocks
                                let window_start = (event.timestamp_ms - 360_000).max(0);
                                let _ = crate::processing::ActivityProcessor::process_range(
                                    &db,
                                    window_start,
                                    event.timestamp_ms,
                                );
                            }
                        }
                    }
                    None => {
                        // All sender channels closed
                        break;
                    }
                }
            }
        };

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tauri::async_runtime::JoinHandle::Tokio(handle.spawn(fut))
        } else {
            tauri::async_runtime::spawn(fut)
        }
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication_filter() {
        let mut filter = DeduplicationFilter::new(60_000);

        let evt1 = RawEvent {
            id: "1".to_string(),
            source: RawEventSource::X11,
            timestamp_ms: 1000,
            app: "code".to_string(),
            title: "tendly - main.rs".to_string(),
            url: None,
            idle_ms: None,
            raw_json: None,
        };

        // First event is persisted
        assert!(filter.should_persist(&evt1));

        // Identical event 2 seconds later is suppressed
        let evt2 = RawEvent {
            id: "2".to_string(),
            timestamp_ms: 3000,
            ..evt1.clone()
        };
        assert!(!filter.should_persist(&evt2));

        // Different title is persisted immediately
        let evt3 = RawEvent {
            id: "3".to_string(),
            timestamp_ms: 4000,
            title: "tendly - lib.rs".to_string(),
            ..evt1.clone()
        };
        assert!(filter.should_persist(&evt3));

        // Checkpoint after 60s is persisted
        let evt4 = RawEvent {
            id: "4".to_string(),
            timestamp_ms: 65000,
            ..evt3.clone()
        };
        assert!(filter.should_persist(&evt4));

        // AFK transition is always persisted
        let evt_afk = RawEvent {
            id: "5".to_string(),
            source: RawEventSource::Afk,
            timestamp_ms: 70000,
            app: "system".to_string(),
            title: "afk".to_string(),
            url: None,
            idle_ms: Some(300000),
            raw_json: None,
        };
        assert!(filter.should_persist(&evt_afk));
    }
}
