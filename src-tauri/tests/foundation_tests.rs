use tendly_lib::core::config::AppConfig;
use tendly_lib::core::error::{AppError, IpcError};
use tendly_lib::domain::{
    Classification, ClassificationRule, MatchField, RawEvent, RawEventSource, RuleSource,
};
use tendly_lib::storage::DatabaseManager;

#[test]
fn test_app_config_creation() {
    let temp_dir = tempfile::tempdir().expect("Must create temp dir");
    let config = AppConfig::for_test(temp_dir.path());

    assert_eq!(config.environment, "test");
    assert_eq!(config.block_duration_seconds, 180);
    assert_eq!(config.idle_threshold_seconds, 300);
    assert!(config.db_path.ends_with("test_tendly.db"));
}

#[test]
fn test_database_initialization_and_migrations() {
    let db = DatabaseManager::open_in_memory().expect("Must open in-memory database");
    let version = db.get_schema_version().expect("Must fetch schema version");
    assert_eq!(
        version, 1,
        "Schema version must be 1 after foundation migration"
    );

    let stats = db.get_stats().expect("Must fetch database stats");
    assert_eq!(stats.raw_events_count, 0);
    assert_eq!(stats.blocks_count, 0);
}

#[test]
fn test_database_state_persistence() {
    let db = DatabaseManager::open_in_memory().expect("Must open in-memory database");

    // Initially unset
    let unset = db
        .get_app_state("is_tracking_paused")
        .expect("Must query state");
    assert_eq!(unset, None);

    // Set state
    db.set_app_state("is_tracking_paused", "true")
        .expect("Must set state");
    let set_val = db
        .get_app_state("is_tracking_paused")
        .expect("Must query state");
    assert_eq!(set_val, Some("true".to_string()));

    // Update state
    db.set_app_state("is_tracking_paused", "false")
        .expect("Must update state");
    let updated_val = db
        .get_app_state("is_tracking_paused")
        .expect("Must query state");
    assert_eq!(updated_val, Some("false".to_string()));
}

#[test]
fn test_database_nuclear_wipe() {
    let db = DatabaseManager::open_in_memory().expect("Must open in-memory database");

    db.set_app_state("session_token", "abc123xyz")
        .expect("Must set state");
    assert!(db
        .get_app_state("session_token")
        .expect("Must read")
        .is_some());

    db.wipe_all_data().expect("Nuclear wipe must succeed");
    assert_eq!(db.get_app_state("session_token").expect("Must read"), None);

    let stats = db.get_stats().expect("Must get stats");
    assert_eq!(stats.raw_events_count, 0);
    assert_eq!(stats.blocks_count, 0);
}

#[test]
fn test_domain_model_serialization() {
    let event = RawEvent::new(
        RawEventSource::X11,
        1700000000000,
        "alacritty".to_string(),
        "nvim src/main.rs".to_string(),
        None,
        None,
    );

    let json = serde_json::to_string(&event).expect("Must serialize RawEvent");
    let deserialized: RawEvent = serde_json::from_str(&json).expect("Must deserialize RawEvent");

    assert_eq!(deserialized.app, "alacritty");
    assert_eq!(deserialized.source, RawEventSource::X11);
    assert_eq!(deserialized.title, "nvim src/main.rs");

    let rule = ClassificationRule {
        id: "rule-1".to_string(),
        priority: 100,
        match_field: MatchField::App,
        pattern: "alacritty".to_string(),
        classification: Classification::Focus,
        category: Some("Terminal".to_string()),
        source: RuleSource::Default,
        created_at: 1700000000000,
    };

    let rule_json = serde_json::to_string(&rule).expect("Must serialize rule");
    assert!(rule_json.contains("\"classification\":\"focus\""));
}

#[test]
fn test_ipc_error_sanitization() {
    let db_err = AppError::Database(rusqlite::Error::ExecuteReturnedResults);
    let ipc_err = IpcError::from(db_err);

    assert_eq!(ipc_err.code, "DATABASE_ERROR");
    assert!(
        !ipc_err.message.contains("sqlite"),
        "Must not leak internal SQLite details"
    );
}
