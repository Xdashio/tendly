use crate::domain::RawEvent;
use thiserror::Error;
use tokio::sync::mpsc::Sender;

#[derive(Error, Debug)]
pub enum WatcherError {
    #[error("Platform not supported: {0}")]
    Unsupported(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Watcher runtime error: {0}")]
    Runtime(String),
}

/// Abstract interface for activity capture watchers.
/// Platform-specific watchers (X11, Wayland, AFK, Windows, macOS) will implement this trait in Phase 2.
pub trait ActivityWatcher: Send + Sync {
    /// Human-readable identifier for the watcher (e.g. "watcher-x11").
    fn name(&self) -> &'static str;

    /// Checks if the current OS and desktop environment supports this watcher.
    fn is_supported(&self) -> bool;

    /// Starts observing activity and emitting events on the provided channel.
    fn start(&mut self, tx: Sender<RawEvent>) -> Result<(), WatcherError>;

    /// Stops observing activity gracefully.
    fn stop(&mut self) -> Result<(), WatcherError>;
}
