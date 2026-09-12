use crate::capture::watcher::{ActivityWatcher, WatcherError};
use crate::domain::{RawEvent, RawEventSource};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use x11rb::connection::Connection;

pub struct AfkWatcher {
    threshold_seconds: u64,
    is_running: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl AfkWatcher {
    pub fn new(threshold_seconds: u64) -> Self {
        Self {
            threshold_seconds,
            is_running: Arc::new(AtomicBool::new(false)),
            is_paused: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        }
    }

    pub fn is_any_idle_source_available() -> bool {
        // 1. Check X11 ScreenSaver
        if let Ok((conn, screen_num)) = x11rb::connect(None) {
            let root = conn.setup().roots[screen_num].root;
            if x11rb::protocol::screensaver::query_info(&conn, root).is_ok() {
                return true;
            }
        }

        // 2. Check GNOME Mutter IdleMonitor D-Bus
        if check_mutter_idle_available() {
            return true;
        }

        false
    }
}

impl Default for AfkWatcher {
    fn default() -> Self {
        Self::new(300) // Default 5 minutes
    }
}

impl ActivityWatcher for AfkWatcher {
    fn name(&self) -> &'static str {
        "watcher-afk"
    }

    fn is_supported(&self) -> bool {
        Self::is_any_idle_source_available()
    }

    fn start(&mut self, tx: Sender<RawEvent>) -> Result<(), WatcherError> {
        if self.is_running.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.is_running.store(true, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);

        let is_running = Arc::clone(&self.is_running);
        let is_paused = Arc::clone(&self.is_paused);
        let threshold_ms = self.threshold_seconds * 1000;

        let handle = thread::Builder::new()
            .name("tendly-afk-watcher".to_string())
            .spawn(move || {
                run_afk_loop(threshold_ms, is_running, is_paused, tx);
            })
            .map_err(|e| WatcherError::Runtime(format!("Failed spawning AFK thread: {}", e)))?;

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

#[derive(Debug, PartialEq, Eq)]
pub enum UserActivityState {
    Active,
    Afk,
}

pub struct AfkStateMachine {
    threshold_ms: u64,
    current_state: UserActivityState,
}

impl AfkStateMachine {
    pub fn new(threshold_ms: u64) -> Self {
        Self {
            threshold_ms,
            current_state: UserActivityState::Active,
        }
    }

    /// Evaluates current idle duration and returns an optional transition event.
    pub fn update(&mut self, idle_ms: u64) -> Option<RawEvent> {
        let is_idle = idle_ms >= self.threshold_ms;

        match self.current_state {
            UserActivityState::Active => {
                if is_idle {
                    self.current_state = UserActivityState::Afk;
                    let now_ms = chrono::Utc::now().timestamp_millis();
                    Some(RawEvent {
                        id: uuid::Uuid::new_v4().to_string(),
                        source: RawEventSource::Afk,
                        timestamp_ms: now_ms,
                        app: "system".to_string(),
                        title: "afk".to_string(),
                        url: None,
                        idle_ms: Some(idle_ms as i64),
                        raw_json: None,
                    })
                } else {
                    None
                }
            }
            UserActivityState::Afk => {
                if !is_idle {
                    self.current_state = UserActivityState::Active;
                    let now_ms = chrono::Utc::now().timestamp_millis();
                    Some(RawEvent {
                        id: uuid::Uuid::new_v4().to_string(),
                        source: RawEventSource::Afk,
                        timestamp_ms: now_ms,
                        app: "system".to_string(),
                        title: "active".to_string(),
                        url: None,
                        idle_ms: Some(0),
                        raw_json: None,
                    })
                } else {
                    None
                }
            }
        }
    }

    pub fn state(&self) -> &UserActivityState {
        &self.current_state
    }
}

fn run_afk_loop(
    threshold_ms: u64,
    is_running: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    tx: Sender<RawEvent>,
) {
    let mut state_machine = AfkStateMachine::new(threshold_ms);

    while is_running.load(Ordering::SeqCst) {
        if is_paused.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(500));
            continue;
        }

        if let Some(idle_ms) = query_current_idle_ms() {
            if let Some(event) = state_machine.update(idle_ms) {
                let _ = tx.blocking_send(event);
            }
        }

        // Poll every 3 seconds
        thread::sleep(Duration::from_secs(3));
    }
}

pub fn query_current_idle_ms() -> Option<u64> {
    // 1. Try X11 ScreenSaver
    if let Ok((conn, screen_num)) = x11rb::connect(None) {
        let root = conn.setup().roots[screen_num].root;
        if let Ok(cookie) = x11rb::protocol::screensaver::query_info(&conn, root) {
            if let Ok(reply) = cookie.reply() {
                // If extension is missing on XWayland, reply.ms_since_last_user_input might not be populated or error
                if reply.ms_since_user_input > 0 {
                    return Some(reply.ms_since_user_input as u64);
                }
            }
        }
    }

    // 2. Try GNOME Mutter D-Bus
    query_mutter_idletime()
}

fn check_mutter_idle_available() -> bool {
    query_mutter_idletime().is_some()
}

fn query_mutter_idletime() -> Option<u64> {
    // Use gdbus command to query org.gnome.Mutter.IdleMonitor /org/gnome/Mutter/IdleMonitor/Core GetIdletime
    let output = std::process::Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.gnome.Mutter.IdleMonitor",
            "--object-path",
            "/org/gnome/Mutter/IdleMonitor/Core",
            "--method",
            "org.gnome.Mutter.IdleMonitor.GetIdletime",
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let out_str = String::from_utf8_lossy(&output.stdout);
        parse_gdbus_uint64(&out_str)
    } else {
        None
    }
}

pub fn parse_gdbus_uint64(output: &str) -> Option<u64> {
    // Expected output format: (uint64 12345,)
    let trimmed = output.trim();
    let inner = trimmed.strip_prefix('(')?.strip_suffix(')')?;
    let inner = inner.trim().strip_suffix(',')?;
    let val_str = inner.strip_prefix("uint64")?.trim();
    val_str.parse::<u64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_afk_state_machine_transitions() {
        let mut machine = AfkStateMachine::new(300_000); // 5 minutes
        assert_eq!(*machine.state(), UserActivityState::Active);

        // Under threshold: stays Active, no event
        assert!(machine.update(10_000).is_none());
        assert_eq!(*machine.state(), UserActivityState::Active);

        // Exceeds threshold: transitions to Afk, emits Afk event
        let evt_afk = machine.update(300_001).expect("Must emit AFK transition");
        assert_eq!(evt_afk.source, RawEventSource::Afk);
        assert_eq!(evt_afk.app, "system");
        assert_eq!(evt_afk.title, "afk");
        assert_eq!(evt_afk.idle_ms, Some(300_001));
        assert_eq!(*machine.state(), UserActivityState::Afk);

        // Remains idle: no duplicate event
        assert!(machine.update(350_000).is_none());
        assert_eq!(*machine.state(), UserActivityState::Afk);

        // User returns: idle drops below threshold, transitions to Active
        let evt_active = machine.update(100).expect("Must emit Active transition");
        assert_eq!(evt_active.source, RawEventSource::Afk);
        assert_eq!(evt_active.app, "system");
        assert_eq!(evt_active.title, "active");
        assert_eq!(evt_active.idle_ms, Some(0));
        assert_eq!(*machine.state(), UserActivityState::Active);
    }

    #[test]
    fn test_parse_gdbus_uint64() {
        assert_eq!(parse_gdbus_uint64("(uint64 178,)"), Some(178));
        assert_eq!(parse_gdbus_uint64("(uint64 305412,)\n"), Some(305412));
        assert_eq!(parse_gdbus_uint64("invalid"), None);
    }
}
