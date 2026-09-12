use crate::capture::watcher::{ActivityWatcher, WatcherError, WatcherStatus};
use crate::domain::RawEvent;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Sender;

/// Manages registration, lifecycle, and health reporting for in-process activity watchers.
#[derive(Default)]
pub struct WatcherManager {
    watchers: Vec<Arc<Mutex<dyn ActivityWatcher>>>,
}

impl WatcherManager {
    pub fn new() -> Self {
        Self {
            watchers: Vec::new(),
        }
    }

    pub fn register(&mut self, watcher: Arc<Mutex<dyn ActivityWatcher>>) {
        self.watchers.push(watcher);
    }

    pub fn active_count(&self) -> usize {
        self.watchers.len()
    }

    pub fn start_all(&mut self, tx: Sender<RawEvent>) -> Result<(), WatcherError> {
        for watcher_arc in &self.watchers {
            if let Ok(mut watcher) = watcher_arc.lock() {
                if watcher.is_supported() && !watcher.is_running() {
                    let _ = watcher.start(tx.clone());
                }
            }
        }
        Ok(())
    }

    pub fn stop_all(&mut self) -> Result<(), WatcherError> {
        for watcher_arc in &self.watchers {
            if let Ok(mut watcher) = watcher_arc.lock() {
                if watcher.is_running() {
                    let _ = watcher.stop();
                }
            }
        }
        Ok(())
    }

    pub fn pause_all(&mut self) -> Result<(), WatcherError> {
        for watcher_arc in &self.watchers {
            if let Ok(mut watcher) = watcher_arc.lock() {
                if watcher.is_running() && !watcher.is_paused() {
                    let _ = watcher.pause();
                }
            }
        }
        Ok(())
    }

    pub fn resume_all(&mut self) -> Result<(), WatcherError> {
        for watcher_arc in &self.watchers {
            if let Ok(mut watcher) = watcher_arc.lock() {
                if watcher.is_running() && watcher.is_paused() {
                    let _ = watcher.resume();
                }
            }
        }
        Ok(())
    }

    pub fn watcher_statuses(&self) -> Vec<WatcherStatus> {
        let mut statuses = Vec::new();
        for watcher_arc in &self.watchers {
            if let Ok(watcher) = watcher_arc.lock() {
                statuses.push(watcher.status());
            }
        }
        statuses
    }

    pub fn is_any_running(&self) -> bool {
        for watcher_arc in &self.watchers {
            if let Ok(watcher) = watcher_arc.lock() {
                if watcher.is_running() {
                    return true;
                }
            }
        }
        false
    }
}
