use crate::capture::watcher::{ActivityWatcher, WatcherError};
use crate::domain::{RawEvent, RawEventSource};
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::Sender;

pub struct WaylandWatcher {
    is_running: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl Default for WaylandWatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl WaylandWatcher {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            is_paused: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        }
    }

    /// Determines the active Wayland backend available on this machine.
    pub fn detect_backend() -> Option<WaylandBackend> {
        if std::env::var("WAYLAND_DISPLAY").is_err() {
            return None;
        }

        // 1. Hyprland IPC socket
        if let Ok(sig) = std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
            let socket_path = get_hyprland_socket_path(&sig);
            if socket_path.exists() {
                return Some(WaylandBackend::Hyprland(socket_path));
            }
        }

        // 2. Sway IPC socket
        if let Ok(sock) = std::env::var("SWAYSOCK") {
            let p = PathBuf::from(sock);
            if p.exists() {
                return Some(WaylandBackend::Sway(p));
            }
        }

        // If on GNOME or KDE or unknown without supported IPC, foreign toplevel is unavailable
        let desktop = std::env::var("XDG_CURRENT_DESKTOP")
            .unwrap_or_default()
            .to_lowercase();
        if desktop.contains("gnome") || desktop.contains("kde") {
            return None;
        }

        None
    }
}

#[derive(Debug, Clone)]
pub enum WaylandBackend {
    Hyprland(PathBuf),
    Sway(PathBuf),
}

fn get_hyprland_socket_path(signature: &str) -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let p = PathBuf::from(runtime_dir)
            .join("hypr")
            .join(signature)
            .join(".socket2.sock");
        if p.exists() {
            return p;
        }
    }
    PathBuf::from("/tmp/hypr")
        .join(signature)
        .join(".socket2.sock")
}

impl ActivityWatcher for WaylandWatcher {
    fn name(&self) -> &'static str {
        "watcher-wayland"
    }

    fn is_supported(&self) -> bool {
        Self::detect_backend().is_some()
    }

    fn start(&mut self, tx: Sender<RawEvent>) -> Result<(), WatcherError> {
        if self.is_running.load(Ordering::SeqCst) {
            return Ok(());
        }

        let backend = Self::detect_backend().ok_or_else(|| {
            WatcherError::Unsupported(
                "No supported wlroots / Wayland IPC backend (Hyprland or Sway) detected in this session"
                    .to_string(),
            )
        })?;

        self.is_running.store(true, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);

        let is_running = Arc::clone(&self.is_running);
        let is_paused = Arc::clone(&self.is_paused);

        let handle = thread::Builder::new()
            .name("tendly-wayland-watcher".to_string())
            .spawn(move || {
                run_wayland_loop(backend, is_running, is_paused, tx);
            })
            .map_err(|e| {
                WatcherError::Runtime(format!("Failed to spawn Wayland watcher thread: {}", e))
            })?;

        self.thread_handle = Some(handle);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), WatcherError> {
        self.is_running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
        Ok(())
    }

    fn pause(&mut self) -> Result<(), WatcherError> {
        self.is_paused.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn resume(&mut self) -> Result<(), WatcherError> {
        self.is_paused.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::SeqCst)
    }
}

fn run_wayland_loop(
    backend: WaylandBackend,
    is_running: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    tx: Sender<RawEvent>,
) {
    match backend {
        WaylandBackend::Hyprland(socket_path) => {
            run_hyprland_socket_loop(socket_path, is_running, is_paused, tx);
        }
        WaylandBackend::Sway(socket_path) => {
            run_sway_socket_loop(socket_path, is_running, is_paused, tx);
        }
    }
}

fn run_hyprland_socket_loop(
    socket_path: PathBuf,
    is_running: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    tx: Sender<RawEvent>,
) {
    let mut backoff = Duration::from_secs(1);

    while is_running.load(Ordering::SeqCst) {
        if is_paused.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(500));
            continue;
        }

        let stream = match UnixStream::connect(&socket_path) {
            Ok(s) => {
                backoff = Duration::from_secs(1);
                let _ = s.set_read_timeout(Some(Duration::from_secs(2)));
                s
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed connecting to Hyprland socket2. Retrying");
                thread::sleep(backoff);
                backoff = (backoff * 2).min(Duration::from_secs(30));
                continue;
            }
        };

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        let mut last_app = String::new();
        let mut last_title = String::new();
        let mut last_emit = Instant::now() - Duration::from_secs(100);

        while is_running.load(Ordering::SeqCst) {
            if is_paused.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(500));
                continue;
            }

            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF, reconnect
                Ok(_) => {
                    let trimmed = line.trim();
                    // Event format: activewindow>>windowclass,windowtitle
                    if let Some(rest) = trimmed.strip_prefix("activewindow>>") {
                        let (app, title) = parse_hyprland_event(rest);

                        let state_changed = app != last_app || title != last_title;
                        let checkpoint_due = last_emit.elapsed() >= Duration::from_secs(60);

                        if (state_changed || checkpoint_due)
                            && (!app.is_empty() || !title.is_empty())
                        {
                            let now_ms = chrono::Utc::now().timestamp_millis();
                            let event = RawEvent {
                                id: uuid::Uuid::new_v4().to_string(),
                                source: RawEventSource::Wayland,
                                timestamp_ms: now_ms,
                                app: app.clone(),
                                title: title.clone(),
                                url: None,
                                idle_ms: None,
                                raw_json: None,
                            };

                            let _ = tx.blocking_send(event);
                            last_app = app;
                            last_title = title;
                            last_emit = Instant::now();
                        }
                    }
                }
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::TimedOut
                        || e.kind() == std::io::ErrorKind::WouldBlock =>
                {
                    // Checkpoint check on read timeout
                    if !last_app.is_empty() && last_emit.elapsed() >= Duration::from_secs(60) {
                        let now_ms = chrono::Utc::now().timestamp_millis();
                        let event = RawEvent {
                            id: uuid::Uuid::new_v4().to_string(),
                            source: RawEventSource::Wayland,
                            timestamp_ms: now_ms,
                            app: last_app.clone(),
                            title: last_title.clone(),
                            url: None,
                            idle_ms: None,
                            raw_json: None,
                        };
                        let _ = tx.blocking_send(event);
                        last_emit = Instant::now();
                    }
                }
                Err(_) => break, // Socket error, reconnect
            }
        }
    }
}

pub fn parse_hyprland_event(rest: &str) -> (String, String) {
    if let Some((app, title)) = rest.split_once(',') {
        (app.trim().to_string(), title.trim().to_string())
    } else {
        (rest.trim().to_string(), String::new())
    }
}

fn run_sway_socket_loop(
    _socket_path: PathBuf,
    is_running: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    _tx: Sender<RawEvent>,
) {
    // Sway IPC loop stub
    while is_running.load(Ordering::SeqCst) {
        if is_paused.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(500));
            continue;
        }
        thread::sleep(Duration::from_secs(2));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hyprland_event() {
        let (app, title) = parse_hyprland_event("firefox,Mozilla Firefox");
        assert_eq!(app, "firefox");
        assert_eq!(title, "Mozilla Firefox");

        let (app2, title2) = parse_hyprland_event("Alacritty,");
        assert_eq!(app2, "Alacritty");
        assert_eq!(title2, "");

        let (app3, title3) = parse_hyprland_event("kitty");
        assert_eq!(app3, "kitty");
        assert_eq!(title3, "");
    }
}
