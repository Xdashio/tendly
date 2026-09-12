pub mod afk;
pub mod browser_context;
pub mod manager;
pub mod pipeline;
pub mod watcher;
pub mod wayland;
pub mod x11;

pub use afk::AfkWatcher;
pub use manager::WatcherManager;
pub use pipeline::CapturePipeline;
pub use watcher::{ActivityWatcher, WatcherError, WatcherStatus};
pub use wayland::WaylandWatcher;
pub use x11::X11Watcher;
