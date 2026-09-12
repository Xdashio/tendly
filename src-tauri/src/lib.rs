pub mod capture;
pub mod commands;
pub mod core;
pub mod domain;
pub mod storage;

use crate::capture::{AfkWatcher, CapturePipeline, WatcherManager, WaylandWatcher, X11Watcher};
use crate::core::config::AppConfig;
use crate::core::logging::init_logging;
use crate::domain::TrackingState;
use crate::storage::DatabaseManager;
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub struct AppState {
    pub db: DatabaseManager,
    pub config: AppConfig,
    pub tracking: Arc<Mutex<TrackingState>>,
    pub watchers: Arc<Mutex<WatcherManager>>,
    pub pipeline: Arc<CapturePipeline>,
}

pub fn run() {
    init_logging();
    tracing::info!("Starting Tendly application core");

    let config =
        AppConfig::for_production().expect("Failed to initialize application configuration");
    let db = DatabaseManager::open(&config.db_path).expect("Failed to initialize database");

    // Restore persisted tracking pause state if present
    let is_paused = db
        .get_app_state("is_tracking_paused")
        .ok()
        .flatten()
        .map(|v| v == "true")
        .unwrap_or(false);

    // Initialize watchers
    let mut watcher_manager = WatcherManager::new();
    watcher_manager.register(Arc::new(Mutex::new(X11Watcher::new())));
    watcher_manager.register(Arc::new(Mutex::new(WaylandWatcher::new())));
    watcher_manager.register(Arc::new(Mutex::new(AfkWatcher::new(
        config.idle_threshold_seconds,
    ))));

    // Activity capture pipeline channel (bounded capacity 256)
    let (event_tx, event_rx) = tokio::sync::mpsc::channel(256);
    let pipeline = Arc::new(CapturePipeline::new());
    let _pipeline_handle = pipeline.start(event_rx, db.clone());

    // Start supported watchers
    let _ = watcher_manager.start_all(event_tx);
    if is_paused {
        let _ = watcher_manager.pause_all();
    }

    let active_count = watcher_manager
        .watcher_statuses()
        .iter()
        .filter(|w| w.running)
        .count();

    let initial_tracking = TrackingState {
        is_paused,
        active_watchers_count: active_count,
        ..Default::default()
    };

    let app_state = AppState {
        db,
        config,
        tracking: Arc::new(Mutex::new(initial_tracking)),
        watchers: Arc::new(Mutex::new(watcher_manager)),
        pipeline,
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::app_info::get_app_info,
            commands::app_info::get_database_stats,
            commands::tracking::get_app_status,
            commands::tracking::get_capture_status,
            commands::tracking::toggle_tracking_pause,
        ])
        .setup(|app| {
            tracing::info!("Tendly application setup completed");
            #[cfg(desktop)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error while running Tendly application");
}
