use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tendly_lib::capture::afk::AfkStateMachine;
use tendly_lib::capture::pipeline::DeduplicationFilter;
use tendly_lib::capture::watcher::{ActivityWatcher, WatcherError};
use tendly_lib::capture::{CapturePipeline, WatcherManager};
use tendly_lib::domain::{RawEvent, RawEventSource};
use tendly_lib::storage::DatabaseManager;
use tokio::sync::mpsc::Sender;

/// Deterministic synthetic watcher for automated testing without OS dependencies.
struct MockWatcher {
    name: &'static str,
    events_to_emit: Vec<RawEvent>,
    is_running: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
}

impl MockWatcher {
    fn new(name: &'static str, events: Vec<RawEvent>) -> Self {
        Self {
            name,
            events_to_emit: events,
            is_running: Arc::new(AtomicBool::new(false)),
            is_paused: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl ActivityWatcher for MockWatcher {
    fn name(&self) -> &'static str {
        self.name
    }

    fn is_supported(&self) -> bool {
        true
    }

    fn start(&mut self, tx: Sender<RawEvent>) -> Result<(), WatcherError> {
        self.is_running.store(true, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);

        let events = self.events_to_emit.clone();
        let is_running = Arc::clone(&self.is_running);
        let is_paused = Arc::clone(&self.is_paused);

        tokio::spawn(async move {
            for event in events {
                if !is_running.load(Ordering::SeqCst) {
                    break;
                }
                while is_paused.load(Ordering::SeqCst) {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                let _ = tx.send(event).await;
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        });

        Ok(())
    }

    fn stop(&mut self) -> Result<(), WatcherError> {
        self.is_running.store(false, Ordering::SeqCst);
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

#[tokio::test]
async fn test_mock_watcher_to_pipeline_to_database_integration() {
    let db = DatabaseManager::open_in_memory().expect("Must open in-memory db");
    let (tx, rx) = tokio::sync::mpsc::channel(32);

    let pipeline = CapturePipeline::new();
    let _handle = pipeline.start(rx, db.clone());

    let base_time = 1_700_000_000_000i64;

    let fixture_events = vec![
        // Event 1: Initial focus on Code
        RawEvent {
            id: "evt-1".to_string(),
            source: RawEventSource::X11,
            timestamp_ms: base_time,
            app: "code".to_string(),
            title: "tendly - main.rs".to_string(),
            url: None,
            idle_ms: None,
            raw_json: None,
        },
        // Event 2: Duplicate focus 5s later (should be suppressed by deduplication)
        RawEvent {
            id: "evt-2".to_string(),
            source: RawEventSource::X11,
            timestamp_ms: base_time + 5_000,
            app: "code".to_string(),
            title: "tendly - main.rs".to_string(),
            url: None,
            idle_ms: None,
            raw_json: None,
        },
        // Event 3: Switch to browser (should be persisted)
        RawEvent {
            id: "evt-3".to_string(),
            source: RawEventSource::X11,
            timestamp_ms: base_time + 15_000,
            app: "firefox".to_string(),
            title: "Rust Documentation".to_string(),
            url: None,
            idle_ms: None,
            raw_json: None,
        },
        // Event 4: AFK transition (should be persisted)
        RawEvent {
            id: "evt-4".to_string(),
            source: RawEventSource::Afk,
            timestamp_ms: base_time + 315_000,
            app: "system".to_string(),
            title: "afk".to_string(),
            url: None,
            idle_ms: Some(300_000),
            raw_json: None,
        },
        // Event 5: Resume from AFK (should be persisted)
        RawEvent {
            id: "evt-5".to_string(),
            source: RawEventSource::Afk,
            timestamp_ms: base_time + 600_000,
            app: "system".to_string(),
            title: "active".to_string(),
            url: None,
            idle_ms: Some(0),
            raw_json: None,
        },
    ];

    let mut mock_watcher = MockWatcher::new("test-watcher", fixture_events);
    mock_watcher.start(tx).expect("Must start mock watcher");

    // Allow mock events to stream through pipeline
    tokio::time::sleep(Duration::from_millis(200)).await;
    mock_watcher.stop().expect("Must stop mock watcher");

    // Verify persisted events
    let stats = db.get_stats().expect("Must get database stats");
    assert_eq!(
        stats.raw_events_count, 4,
        "Exactly 4 events must be persisted (duplicate evt-2 must be suppressed)"
    );

    let recent = db.get_recent_raw_events(10).expect("Must query events");
    assert_eq!(recent.len(), 4);
    assert_eq!(recent[0].id, "evt-5"); // active
    assert_eq!(recent[1].id, "evt-4"); // afk
    assert_eq!(recent[2].id, "evt-3"); // firefox
    assert_eq!(recent[3].id, "evt-1"); // code (evt-2 suppressed)
}

#[tokio::test]
async fn test_watcher_manager_lifecycle_and_pause_resume() {
    let mut manager = WatcherManager::new();

    let watcher = Arc::new(std::sync::Mutex::new(MockWatcher::new(
        "mock-lifecycle",
        vec![],
    )));
    manager.register(watcher.clone());

    let (tx, _rx) = tokio::sync::mpsc::channel(16);

    // Start all
    manager.start_all(tx).expect("Must start all");
    {
        let w = watcher.lock().unwrap();
        assert!(w.is_running());
        assert!(!w.is_paused());
    }

    let statuses = manager.watcher_statuses();
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0].name, "mock-lifecycle");
    assert!(statuses[0].running);
    assert!(!statuses[0].paused);

    // Pause all
    manager.pause_all().expect("Must pause all");
    {
        let w = watcher.lock().unwrap();
        assert!(w.is_running());
        assert!(w.is_paused());
    }

    // Resume all
    manager.resume_all().expect("Must resume all");
    {
        let w = watcher.lock().unwrap();
        assert!(w.is_running());
        assert!(!w.is_paused());
    }

    // Stop all
    manager.stop_all().expect("Must stop all");
    {
        let w = watcher.lock().unwrap();
        assert!(!w.is_running());
    }
}

#[test]
fn test_afk_state_machine_full_cycle() {
    let mut machine = AfkStateMachine::new(300_000); // 5 min threshold

    // User is actively moving pointer: idle time low
    assert!(machine.update(500).is_none());
    assert!(machine.update(1500).is_none());
    assert!(machine.update(299_999).is_none());

    // Inactivity threshold reached -> Afk transition
    let afk_event = machine.update(300_000).expect("Must trigger Afk");
    assert_eq!(afk_event.source, RawEventSource::Afk);
    assert_eq!(afk_event.title, "afk");

    // Continuous idle state does not duplicate events
    assert!(machine.update(305_000).is_none());
    assert!(machine.update(400_000).is_none());

    // User returns -> Active transition
    let active_event = machine.update(200).expect("Must trigger Active");
    assert_eq!(active_event.source, RawEventSource::Afk);
    assert_eq!(active_event.title, "active");
    assert_eq!(active_event.idle_ms, Some(0));

    // Subsequent normal activity does not generate transition events
    assert!(machine.update(1_000).is_none());
}

#[test]
fn test_deduplication_filter_checkpoint_timing() {
    let mut filter = DeduplicationFilter::new(60_000);

    let event_a = RawEvent {
        id: "a1".to_string(),
        source: RawEventSource::Wayland,
        timestamp_ms: 1000,
        app: "terminal".to_string(),
        title: "bash".to_string(),
        url: None,
        idle_ms: None,
        raw_json: None,
    };

    assert!(filter.should_persist(&event_a));

    // Suppressed: identical within 60s window
    let event_a2 = RawEvent {
        id: "a2".to_string(),
        timestamp_ms: 59_000,
        ..event_a.clone()
    };
    assert!(!filter.should_persist(&event_a2));

    // Allowed: 60s checkpoint reached for continuity
    let event_a3 = RawEvent {
        id: "a3".to_string(),
        timestamp_ms: 61_001,
        ..event_a.clone()
    };
    assert!(filter.should_persist(&event_a3));
}

#[test]
fn test_live_linux_watcher_environment_inspection() {
    use tendly_lib::capture::{AfkWatcher, WaylandWatcher, X11Watcher};

    let x11_watcher = X11Watcher::new();
    let wayland_watcher = WaylandWatcher::new();
    let afk_watcher = AfkWatcher::new(300);

    let x11_supported = x11_watcher.is_supported();
    let wayland_supported = wayland_watcher.is_supported();
    let afk_supported = afk_watcher.is_supported();

    // Verify watcher names
    assert_eq!(x11_watcher.name(), "watcher-x11");
    assert_eq!(wayland_watcher.name(), "watcher-wayland");
    assert_eq!(afk_watcher.name(), "watcher-afk");

    // Check status serialization
    let status = x11_watcher.status();
    assert_eq!(status.name, "watcher-x11");
    assert_eq!(status.supported, x11_supported);
    assert!(!status.running);
    assert!(!status.paused);

    let status_wayland = wayland_watcher.status();
    assert_eq!(status_wayland.name, "watcher-wayland");
    assert_eq!(status_wayland.supported, wayland_supported);
    assert!(!status_wayland.running);

    let status_afk = afk_watcher.status();
    assert_eq!(status_afk.name, "watcher-afk");
    assert_eq!(status_afk.supported, afk_supported);
    assert!(!status_afk.running);

    // Measure idle query latency
    let start = std::time::Instant::now();
    let _ = tendly_lib::capture::afk::query_current_idle_ms();
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_millis(500),
        "Idle detection query took too long: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_capture_pipeline_throughput_and_concurrency() {
    let db = DatabaseManager::open_in_memory().expect("Must open in-memory db");
    let (tx, rx) = tokio::sync::mpsc::channel(256);

    let pipeline = CapturePipeline::new();
    let _handle = pipeline.start(rx, db.clone());

    let start = std::time::Instant::now();

    // Stream 200 distinct application switch events
    for i in 0..200 {
        let event = RawEvent {
            id: format!("perf-{}", i),
            source: RawEventSource::X11,
            timestamp_ms: 1_700_000_000_000 + (i as i64 * 1000),
            app: format!("app-{}", i),
            title: format!("Window Title {}", i),
            url: None,
            idle_ms: None,
            raw_json: None,
        };
        tx.send(event).await.expect("Must send event");
    }

    // Wait briefly for persistence
    tokio::time::sleep(Duration::from_millis(250)).await;

    let elapsed = start.elapsed();
    let stats = db.get_stats().expect("Must get stats");

    assert_eq!(stats.raw_events_count, 200);
    assert!(
        elapsed < Duration::from_secs(2),
        "Processing 200 distinct events took too long: {:?}",
        elapsed
    );
}
