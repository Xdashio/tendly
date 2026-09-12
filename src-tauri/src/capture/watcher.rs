use crate::domain::RawEvent;
use thiserror::Error;
use tokio::sync::mpsc::Sender;

use serde::{Deserialize, Serialize};

#[derive(Error, Debug)]
pub enum WatcherError {
    #[error("Platform not supported: {0}")]
    Unsupported(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Watcher runtime error: {0}")]
    Runtime(String),
}

/// Runtime status descriptor for an activity watcher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherStatus {
    pub name: String,
    pub supported: bool,
    pub running: bool,
    pub paused: bool,
    pub last_error: Option<String>,
}

/// Abstract interface for activity capture watchers.
/// Platform-specific watchers (X11, Wayland, AFK, Windows, macOS) implement this trait.
pub trait ActivityWatcher: Send + Sync {
    /// Human-readable identifier for the watcher (e.g. "watcher-x11").
    fn name(&self) -> &'static str;

    /// Checks if the current OS and desktop environment supports this watcher.
    fn is_supported(&self) -> bool;

    /// Starts observing activity and emitting events on the provided channel.
    fn start(&mut self, tx: Sender<RawEvent>) -> Result<(), WatcherError>;

    /// Stops observing activity gracefully.
    fn stop(&mut self) -> Result<(), WatcherError>;

    /// Pauses activity observation without terminating the worker thread.
    fn pause(&mut self) -> Result<(), WatcherError>;

    /// Resumes activity observation.
    fn resume(&mut self) -> Result<(), WatcherError>;

    /// Returns whether the watcher is currently running.
    fn is_running(&self) -> bool;

    /// Returns whether the watcher is currently paused.
    fn is_paused(&self) -> bool;

    /// Returns the current runtime status.
    fn status(&self) -> WatcherStatus {
        WatcherStatus {
            name: self.name().to_string(),
            supported: self.is_supported(),
            running: self.is_running(),
            paused: self.is_paused(),
            last_error: None,
        }
    }
}
