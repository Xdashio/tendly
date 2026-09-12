use crate::capture::watcher::{ActivityWatcher, WatcherError};
use crate::domain::{RawEvent, RawEventSource};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::Sender;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ConnectionExt as _, EventMask, GetPropertyReply, Window,
};
use x11rb::rust_connection::RustConnection;

pub struct X11Watcher {
    is_running: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl Default for X11Watcher {
    fn default() -> Self {
        Self::new()
    }
}

impl X11Watcher {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            is_paused: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        }
    }
}

impl ActivityWatcher for X11Watcher {
    fn name(&self) -> &'static str {
        "watcher-x11"
    }

    fn is_supported(&self) -> bool {
        if std::env::var("DISPLAY").is_err() {
            return false;
        }
        x11rb::connect(None).is_ok()
    }

    fn start(&mut self, tx: Sender<RawEvent>) -> Result<(), WatcherError> {
        if self.is_running.load(Ordering::SeqCst) {
            return Ok(());
        }

        if !self.is_supported() {
            return Err(WatcherError::Unsupported(
                "X11 display is unavailable or cannot connect to DISPLAY".to_string(),
            ));
        }

        self.is_running.store(true, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);

        let is_running = Arc::clone(&self.is_running);
        let is_paused = Arc::clone(&self.is_paused);

        let handle = thread::Builder::new()
            .name("tendly-x11-watcher".to_string())
            .spawn(move || {
                run_x11_watcher_loop(is_running, is_paused, tx);
            })
            .map_err(|e| {
                WatcherError::Runtime(format!("Failed to spawn X11 watcher thread: {}", e))
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

struct X11Atoms {
    net_active_window: Atom,
    net_wm_name: Atom,
    net_wm_pid: Atom,
    utf8_string: Atom,
}

impl X11Atoms {
    fn intern(conn: &RustConnection) -> Result<Self, WatcherError> {
        let net_active_window = conn
            .intern_atom(false, b"_NET_ACTIVE_WINDOW")
            .map_err(|e| WatcherError::Runtime(e.to_string()))?
            .reply()
            .map_err(|e| WatcherError::Runtime(e.to_string()))?
            .atom;

        let net_wm_name = conn
            .intern_atom(false, b"_NET_WM_NAME")
            .map_err(|e| WatcherError::Runtime(e.to_string()))?
            .reply()
            .map_err(|e| WatcherError::Runtime(e.to_string()))?
            .atom;

        let net_wm_pid = conn
            .intern_atom(false, b"_NET_WM_PID")
            .map_err(|e| WatcherError::Runtime(e.to_string()))?
            .reply()
            .map_err(|e| WatcherError::Runtime(e.to_string()))?
            .atom;

        let utf8_string = conn
            .intern_atom(false, b"UTF8_STRING")
            .map_err(|e| WatcherError::Runtime(e.to_string()))?
            .reply()
            .map_err(|e| WatcherError::Runtime(e.to_string()))?
            .atom;

        Ok(Self {
            net_active_window,
            net_wm_name,
            net_wm_pid,
            utf8_string,
        })
    }
}

fn run_x11_watcher_loop(
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

        let conn_res = x11rb::connect(None);
        let (conn, screen_num) = match conn_res {
            Ok(c) => {
                backoff = Duration::from_secs(1);
                c
            }
            Err(e) => {
                tracing::warn!(error = %e, "Cannot connect to X11 display. Retrying with backoff");
                thread::sleep(backoff);
                backoff = (backoff * 2).min(Duration::from_secs(30));
                continue;
            }
        };

        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;

        let atoms = match X11Atoms::intern(&conn) {
            Ok(a) => a,
            Err(e) => {
                tracing::warn!(error = %e, "Failed interning X11 atoms");
                thread::sleep(Duration::from_secs(2));
                continue;
            }
        };

        // Select PropertyChange on root to observe _NET_ACTIVE_WINDOW
        let _ = conn.change_window_attributes(
            root,
            &x11rb::protocol::xproto::ChangeWindowAttributesAux::new()
                .event_mask(EventMask::PROPERTY_CHANGE),
        );
        let _ = conn.flush();

        let mut last_window: Window = 0;
        let mut last_app = String::new();
        let mut last_title = String::new();
        let mut last_emit = Instant::now() - Duration::from_secs(100);

        while is_running.load(Ordering::SeqCst) {
            if is_paused.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(500));
                continue;
            }

            // Query current active window
            match get_active_window(&conn, root, atoms.net_active_window) {
                Ok(win_id) => {
                    let mut app = String::new();
                    let mut title = String::new();

                    if win_id != 0 {
                        // Subscribe to active window property changes if new
                        if win_id != last_window {
                            let _ = conn.change_window_attributes(
                                win_id,
                                &x11rb::protocol::xproto::ChangeWindowAttributesAux::new()
                                    .event_mask(EventMask::PROPERTY_CHANGE),
                            );
                            let _ = conn.flush();
                        }

                        app = get_window_class(&conn, win_id).unwrap_or_default();
                        if app.is_empty() {
                            app = get_process_name_by_pid(&conn, win_id, atoms.net_wm_pid)
                                .unwrap_or_else(|| "unknown".to_string());
                        }

                        title = get_window_title(&conn, win_id, &atoms)
                            .unwrap_or_else(|_| "Untitled".to_string());
                    }

                    let state_changed =
                        win_id != last_window || app != last_app || title != last_title;
                    let checkpoint_due = last_emit.elapsed() >= Duration::from_secs(60);

                    if (state_changed || checkpoint_due) && (!app.is_empty() || !title.is_empty()) {
                        let now_ms = chrono::Utc::now().timestamp_millis();

                        // Enrich with browser context if this is a known browser
                        let raw_json =
                            crate::capture::browser_context::enrich_browser_context(&app, &title)
                                .and_then(|ctx| serde_json::to_string(&ctx).ok());

                        let event = RawEvent {
                            id: uuid::Uuid::new_v4().to_string(),
                            source: RawEventSource::X11,
                            timestamp_ms: now_ms,
                            app: app.clone(),
                            title: title.clone(),
                            url: None,
                            idle_ms: None,
                            raw_json,
                        };

                        if let Err(e) = tx.blocking_send(event) {
                            tracing::warn!(error = %e, "Failed sending X11 RawEvent through channel");
                        }

                        last_window = win_id;
                        last_app = app;
                        last_title = title;
                        last_emit = Instant::now();
                    }
                }
                Err(_) => {
                    // X11 error, might be temporary
                }
            }

            // Sleep 1 second before next poll / heartbeat
            thread::sleep(Duration::from_secs(1));
        }
    }
}

fn get_active_window(
    conn: &RustConnection,
    root: Window,
    atom: Atom,
) -> Result<Window, WatcherError> {
    let prop = conn
        .get_property(false, root, atom, AtomEnum::WINDOW, 0, 1)
        .map_err(|e| WatcherError::Runtime(e.to_string()))?
        .reply()
        .map_err(|e| WatcherError::Runtime(e.to_string()))?;

    if prop.value.len() >= 4 {
        let win = u32::from_ne_bytes([prop.value[0], prop.value[1], prop.value[2], prop.value[3]]);
        Ok(win)
    } else {
        Ok(0)
    }
}

fn get_window_class(conn: &RustConnection, window: Window) -> Result<String, WatcherError> {
    let prop = conn
        .get_property(false, window, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 512)
        .map_err(|e| WatcherError::Runtime(e.to_string()))?
        .reply()
        .map_err(|e| WatcherError::Runtime(e.to_string()))?;

    Ok(parse_wm_class(&prop.value))
}

pub fn parse_wm_class(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }

    let parts: Vec<&[u8]> = data.split(|&b| b == 0).filter(|p| !p.is_empty()).collect();
    if let Some(class_name) = parts.last() {
        String::from_utf8_lossy(class_name).trim().to_string()
    } else if let Some(first) = parts.first() {
        String::from_utf8_lossy(first).trim().to_string()
    } else {
        String::new()
    }
}

fn get_window_title(
    conn: &RustConnection,
    window: Window,
    atoms: &X11Atoms,
) -> Result<String, WatcherError> {
    // Try _NET_WM_NAME (UTF-8) first
    let prop = conn
        .get_property(false, window, atoms.net_wm_name, atoms.utf8_string, 0, 1024)
        .map_err(|e| WatcherError::Runtime(e.to_string()))?
        .reply()
        .map_err(|e| WatcherError::Runtime(e.to_string()))?;

    if !prop.value.is_empty() {
        return Ok(clean_title(&prop));
    }

    // Fall back to WM_NAME
    let prop = conn
        .get_property(false, window, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 1024)
        .map_err(|e| WatcherError::Runtime(e.to_string()))?
        .reply()
        .map_err(|e| WatcherError::Runtime(e.to_string()))?;

    Ok(clean_title(&prop))
}

fn clean_title(prop: &GetPropertyReply) -> String {
    let title = String::from_utf8_lossy(&prop.value).trim().to_string();
    if title.len() > 256 {
        title[..256].to_string()
    } else {
        title
    }
}

fn get_process_name_by_pid(
    conn: &RustConnection,
    window: Window,
    net_wm_pid: Atom,
) -> Option<String> {
    let prop = conn
        .get_property(false, window, net_wm_pid, AtomEnum::CARDINAL, 0, 1)
        .ok()?
        .reply()
        .ok()?;

    if prop.value.len() >= 4 {
        let pid = u32::from_ne_bytes([prop.value[0], prop.value[1], prop.value[2], prop.value[3]]);
        let comm_path = format!("/proc/{}/comm", pid);
        std::fs::read_to_string(comm_path)
            .ok()
            .map(|s| s.trim().to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_wm_class() {
        let raw = b"alacritty\0Alacritty\0";
        let parsed = parse_wm_class(raw);
        assert_eq!(parsed, "Alacritty");

        let single = b"code\0";
        assert_eq!(parse_wm_class(single), "code");

        let empty = b"";
        assert_eq!(parse_wm_class(empty), "");
    }
}
