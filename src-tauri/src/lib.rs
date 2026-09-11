pub mod capture;
pub mod commands;
pub mod core;
pub mod domain;
pub mod storage;

use crate::capture::WatcherManager;
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
}

pub fn run() {
    init_logging();
    tracing::info!("Starting Tendly application foundation");

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

    let initial_tracking = TrackingState {
        is_paused,
        ..Default::default()
    };

    let app_state = AppState {
        db,
        config,
        tracking: Arc::new(Mutex::new(initial_tracking)),
        watchers: Arc::new(Mutex::new(WatcherManager::new())),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::app_info::get_app_info,
            commands::app_info::get_database_stats,
            commands::tracking::get_app_status,
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
