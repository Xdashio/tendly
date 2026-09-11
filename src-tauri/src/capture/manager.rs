use crate::capture::watcher::ActivityWatcher;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Manages registration and lifecycle of in-process activity watchers.
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
}
