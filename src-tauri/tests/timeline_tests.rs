//! Comprehensive integration test suite for Phase 4: Usable Timeline and Activity History.
//!
//! Validates the 11 required timeline and session scenarios:
//! 1. Basic merging: A -> A -> A produces 1 session.
//! 2. Application change: A -> B -> A produces 3 sessions.
//! 3. Activity type change: active -> idle does not merge.
//! 4. Secondary activity: secondary applications are preserved and discoverable.
//! 5. Exact boundaries: [start_ms, end_ms) half-open semantics.
//! 6. Midnight: activity around 23:59:59 and 00:00:00 accurately partitioned.
//! 7. Empty day: valid empty state without errors.
//! 8. Sparse activity: large gaps become unrecorded gap sessions, not giant fake sessions.
//! 9. Long continuous activity: many compatible blocks coalesce while preserving all TimeBlocks.
//! 10. Drill-down: get_session_details returns exact sub-minute composition and app breakdown.
//! 11. Current activity: stale blocks are not treated as current.

use tempfile::NamedTempFile;
use tendly_lib::domain::{ActivityType, RawEvent, RawEventSource, TimeBlock};
use tendly_lib::processing::session::{
    coalesce_blocks_into_sessions, insert_unrecorded_gap_sessions,
};
use tendly_lib::processing::ActivityProcessor;
use tendly_lib::storage::DatabaseManager;

fn make_event(ts: i64, app: &str, title: &str) -> RawEvent {
    RawEvent::new(
        RawEventSource::X11,
        ts,
        app.to_string(),
        title.to_string(),
        None,
        None,
    )
}

fn make_block(start_ms: i64, app: &str, title: &str, act_type: ActivityType) -> TimeBlock {
    TimeBlock {
        id: format!("block:{}", start_ms),
        start_ms,
        end_ms: start_ms + 180_000,
        duration_ms: 180_000,
        activity_type: act_type,
        dominant_app: app.to_string(),
        dominant_title: title.to_string(),
        dominant_url: None,
        classification: None,
        category: None,
        confidence: None,
        classified_by: None,
        user_override: None,
    }
}

fn create_test_db() -> (NamedTempFile, DatabaseManager) {
    let file = NamedTempFile::new().expect("Failed to create temp db file");
    let db = DatabaseManager::open(file.path()).expect("Failed to open test db");
    (file, db)
}

// ---------------------------------------------------------------------------
// 1. BASIC MERGING (A -> A -> A)
// ---------------------------------------------------------------------------
#[test]
fn test_01_basic_merging_a_a_a() {
    let t0 = 1_800_000;
    let b1 = make_block(t0, "code", "main.rs", ActivityType::Active);
    let b2 = make_block(t0 + 180_000, "code", "main.rs", ActivityType::Active);
    let b3 = make_block(t0 + 360_000, "code", "lib.rs", ActivityType::Active);

    let sessions = coalesce_blocks_into_sessions(&[b1, b2, b3]);

    assert_eq!(sessions.len(), 1, "A -> A -> A must merge into 1 session");
    let session = &sessions[0];
    assert_eq!(session.start_ms, t0);
    assert_eq!(session.end_ms, t0 + 540_000);
    assert_eq!(session.duration_ms, 540_000);
    assert_eq!(session.dominant_app, "code");
    assert_eq!(session.dominant_title, "main.rs");
    assert_eq!(session.block_count, 3);
    assert_eq!(session.time_blocks.len(), 3);
}

// ---------------------------------------------------------------------------
// 2. APPLICATION CHANGE (A -> B -> A)
// ---------------------------------------------------------------------------
#[test]
fn test_02_app_change_a_b_a() {
    let t0 = 1_800_000;
    let b1 = make_block(t0, "code", "main.rs", ActivityType::Active);
    let b2 = make_block(t0 + 180_000, "slack", "general", ActivityType::Active);
    let b3 = make_block(t0 + 360_000, "code", "main.rs", ActivityType::Active);

    let sessions = coalesce_blocks_into_sessions(&[b1, b2, b3]);

    assert_eq!(
        sessions.len(),
        3,
        "A -> B -> A must produce 3 distinct sessions"
    );
    assert_eq!(sessions[0].dominant_app, "code");
    assert_eq!(sessions[0].duration_ms, 180_000);
    assert_eq!(sessions[1].dominant_app, "slack");
    assert_eq!(sessions[1].duration_ms, 180_000);
    assert_eq!(sessions[2].dominant_app, "code");
    assert_eq!(sessions[2].duration_ms, 180_000);
}

// ---------------------------------------------------------------------------
// 3. ACTIVITY TYPE CHANGE (ACTIVE -> IDLE)
// ---------------------------------------------------------------------------
#[test]
fn test_03_activity_type_change_active_idle() {
    let t0 = 1_800_000;
    let b1 = make_block(t0, "code", "main.rs", ActivityType::Active);
    let b2 = make_block(t0 + 180_000, "code", "main.rs", ActivityType::Afk);

    let sessions = coalesce_blocks_into_sessions(&[b1, b2]);

    assert_eq!(
        sessions.len(),
        2,
        "Active -> Afk transition must split sessions"
    );
    assert_eq!(sessions[0].activity_type, ActivityType::Active);
    assert_eq!(sessions[1].activity_type, ActivityType::Afk);
}

// ---------------------------------------------------------------------------
// 4. SECONDARY ACTIVITY PRESERVATION AND DISCOVERY
// ---------------------------------------------------------------------------
#[test]
fn test_04_secondary_activity_preservation() {
    let (_file, db) = create_test_db();
    let t0 = 360_000; // aligned epoch

    // Raw events: 140s VS Code, 25s Firefox, 15s Slack in a 180s block
    db.insert_raw_event(&make_event(t0, "code", "main.rs"))
        .unwrap();
    db.insert_raw_event(&make_event(t0 + 140_000, "firefox", "docs"))
        .unwrap();
    db.insert_raw_event(&make_event(t0 + 165_000, "slack", "chat"))
        .unwrap();
    db.insert_raw_event(&make_event(t0 + 180_000, "code", "main.rs"))
        .unwrap();

    ActivityProcessor::process_range(&db, t0, t0 + 180_000).unwrap();

    let timeline = ActivityProcessor::get_daily_timeline(&db, t0, t0 + 180_000).unwrap();
    assert_eq!(timeline.sessions.len(), 1);
    let session = &timeline.sessions[0];
    assert_eq!(session.dominant_app, "code");
    assert!(
        session.has_secondary_activity,
        "Session should have secondary activity detected"
    );
    assert!(
        session
            .secondary_apps
            .iter()
            .any(|a| a.app == "firefox" && a.duration_ms > 0),
        "Firefox must be present in secondary apps"
    );
    assert!(
        session
            .secondary_apps
            .iter()
            .any(|a| a.app == "slack" && a.duration_ms > 0),
        "Slack must be present in secondary apps"
    );
}

// ---------------------------------------------------------------------------
// 5. EXACT HALF-OPEN BOUNDARY SEMANTICS [start_ms, end_ms)
// ---------------------------------------------------------------------------
#[test]
fn test_05_exact_half_open_boundaries() {
    let (_file, db) = create_test_db();
    let epoch_1_start = 180_000;
    let epoch_1_end = 360_000;
    let epoch_2_end = 540_000;

    // Event exactly at epoch_1_end (360_000)
    db.insert_raw_event(&make_event(epoch_1_start, "terminal", "bash"))
        .unwrap();
    db.insert_raw_event(&make_event(epoch_1_end, "code", "main.rs"))
        .unwrap();
    db.insert_raw_event(&make_event(epoch_2_end, "slack", "chat"))
        .unwrap();

    ActivityProcessor::process_range(&db, epoch_1_start, epoch_2_end).unwrap();

    // Query strictly for epoch 1: [180_000, 360_000)
    let timeline_1 =
        ActivityProcessor::get_daily_timeline(&db, epoch_1_start, epoch_1_end).unwrap();
    assert_eq!(timeline_1.sessions.len(), 1);
    assert_eq!(timeline_1.sessions[0].dominant_app, "terminal");

    // Query strictly for epoch 2: [360_000, 540_000)
    let timeline_2 = ActivityProcessor::get_daily_timeline(&db, epoch_1_end, epoch_2_end).unwrap();
    assert_eq!(timeline_2.sessions.len(), 1);
    assert_eq!(timeline_2.sessions[0].dominant_app, "code");
}

// ---------------------------------------------------------------------------
// 6. MIDNIGHT BOUNDARY AND DAY-CROSSING BEHAVIOR
// ---------------------------------------------------------------------------
#[test]
fn test_06_midnight_boundary_query() {
    let (_file, db) = create_test_db();

    // Standard epoch alignment: 86_400_000 ms per day, divisible by 180_000 ms
    let day_1_start: i64 = 0;
    let day_2_start: i64 = 86_400_000;
    let day_3_start: i64 = 172_800_000;

    // Day 1 activity at 23:57:00 (86_220_000)
    db.insert_raw_event(&make_event(86_220_000, "code", "midnight_prep.rs"))
        .unwrap();
    // Day 2 activity at 00:00:00 (86_400_000)
    db.insert_raw_event(&make_event(day_2_start, "code", "midnight_run.rs"))
        .unwrap();
    db.insert_raw_event(&make_event(
        day_2_start + 180_000,
        "code",
        "midnight_run.rs",
    ))
    .unwrap();

    ActivityProcessor::process_range(&db, 0, day_3_start).unwrap();

    // Query Day 1: [0, 86_400_000)
    let timeline_day_1 =
        ActivityProcessor::get_daily_timeline(&db, day_1_start, day_2_start).unwrap();
    for session in &timeline_day_1.sessions {
        assert!(
            session.end_ms <= day_2_start,
            "Day 1 session end_ms {} must be <= day_2_start {}",
            session.end_ms,
            day_2_start
        );
    }

    // Query Day 2: [86_400_000, 172_800_000)
    let timeline_day_2 =
        ActivityProcessor::get_daily_timeline(&db, day_2_start, day_3_start).unwrap();
    for session in &timeline_day_2.sessions {
        assert!(
            session.start_ms >= day_2_start,
            "Day 2 session start_ms {} must be >= day_2_start {}",
            session.start_ms,
            day_2_start
        );
    }
}

// ---------------------------------------------------------------------------
// 7. EMPTY DAY QUERYING AND CLEAN EMPTY STATE
// ---------------------------------------------------------------------------
#[test]
fn test_07_empty_day() {
    let (_file, db) = create_test_db();
    let day_start = 86_400_000;
    let day_end = 172_800_000;

    let timeline = ActivityProcessor::get_daily_timeline(&db, day_start, day_end).unwrap();

    assert_eq!(timeline.sessions.len(), 0);
    assert_eq!(timeline.total_active_ms, 0);
    assert_eq!(timeline.total_afk_ms, 0);
    assert_eq!(timeline.total_unknown_ms, 0);
    assert_eq!(timeline.block_count, 0);
    assert_eq!(timeline.day_start_ms, day_start);
    assert_eq!(timeline.day_end_ms, day_end);
}

// ---------------------------------------------------------------------------
// 8. SPARSE ACTIVITY AND GAP SESSION INSERTION
// ---------------------------------------------------------------------------
#[test]
fn test_08_sparse_activity_gaps() {
    let t0 = 1_800_000; // 00:30:00
    let b1 = make_block(t0, "code", "morning.rs", ActivityType::Active);
    // Gap of 4 hours (14_400_000 ms)
    let t1 = t0 + 180_000 + 14_400_000;
    let b2 = make_block(t1, "code", "afternoon.rs", ActivityType::Active);

    let raw_sessions = coalesce_blocks_into_sessions(&[b1, b2]);
    assert_eq!(
        raw_sessions.len(),
        2,
        "Blocks across 4-hour gap must not merge"
    );

    let sessions_with_gaps = insert_unrecorded_gap_sessions(raw_sessions, t0, t1 + 180_000);
    assert_eq!(
        sessions_with_gaps.len(),
        3,
        "Expected session 1, unrecorded gap, and session 2"
    );

    let gap_session = &sessions_with_gaps[1];
    assert_eq!(gap_session.dominant_app, "unrecorded");
    assert_eq!(gap_session.dominant_title, "No recorded activity");
    assert_eq!(gap_session.activity_type, ActivityType::Unknown);
    assert_eq!(gap_session.duration_ms, 14_400_000);
    assert_eq!(gap_session.start_ms, t0 + 180_000);
    assert_eq!(gap_session.end_ms, t1);
}

// ---------------------------------------------------------------------------
// 9. LONG CONTINUOUS ACTIVITY COALESCING WITH PRESERVED BLOCKS
// ---------------------------------------------------------------------------
#[test]
fn test_09_long_continuous_activity() {
    let t0 = 1_800_000;
    let mut blocks = Vec::new();
    let num_blocks = 20; // 60 minutes (20 * 3m)

    for i in 0..num_blocks {
        blocks.push(make_block(
            t0 + (i as i64) * 180_000,
            "code",
            "feature.rs",
            ActivityType::Active,
        ));
    }

    let sessions = coalesce_blocks_into_sessions(&blocks);

    assert_eq!(
        sessions.len(),
        1,
        "All 20 contiguous blocks must coalesce into 1 session"
    );
    let session = &sessions[0];
    assert_eq!(session.duration_ms, 60 * 60 * 1000); // 1 hour
    assert_eq!(session.block_count, 20);
    assert_eq!(
        session.time_blocks.len(),
        20,
        "Underlying TimeBlocks must be preserved in the session"
    );
}

// ---------------------------------------------------------------------------
// 10. DRILL-DOWN COMPOSITION WITH SUB-MINUTE SEGMENTS
// ---------------------------------------------------------------------------
#[test]
fn test_10_drill_down_composition() {
    let (_file, db) = create_test_db();
    let t0 = 360_000;

    // Sub-minute raw activity events:
    // 0s-90s: code
    // 90s-150s: firefox
    // 150s-180s: code
    db.insert_raw_event(&make_event(t0, "code", "main.rs"))
        .unwrap();
    db.insert_raw_event(&make_event(t0 + 90_000, "firefox", "search"))
        .unwrap();
    db.insert_raw_event(&make_event(t0 + 150_000, "code", "main.rs"))
        .unwrap();
    db.insert_raw_event(&make_event(t0 + 180_000, "code", "main.rs"))
        .unwrap();

    ActivityProcessor::process_range(&db, t0, t0 + 180_000).unwrap();

    let details =
        ActivityProcessor::get_session_details(&db, "test_session", t0, t0 + 180_000).unwrap();

    assert_eq!(details.session.dominant_app, "code");
    assert!(
        !details.segments.is_empty(),
        "Detailed segments must be returned"
    );

    // Verify app breakdown contains both code and firefox with exact durations
    let code_summary = details
        .app_breakdown
        .iter()
        .find(|a| a.app == "code")
        .expect("code must be in breakdown");
    let firefox_summary = details
        .app_breakdown
        .iter()
        .find(|a| a.app == "firefox")
        .expect("firefox must be in breakdown");

    assert_eq!(code_summary.duration_ms, 120_000); // 90s + 30s
    assert_eq!(firefox_summary.duration_ms, 60_000); // 60s
}

// ---------------------------------------------------------------------------
// 11. CURRENT ACTIVITY FRESHNESS AND STALENESS FILTERING
// ---------------------------------------------------------------------------
#[test]
fn test_11_current_activity_freshness() {
    let (_file, db) = create_test_db();
    let now_ms = chrono::Utc::now().timestamp_millis();

    // Event from 10 minutes ago (stale)
    let stale_ts = now_ms - 600_000;
    db.insert_raw_event(&make_event(stale_ts, "stale_app", "stale_title"))
        .unwrap();

    let state = ActivityProcessor::get_current_activity(&db).unwrap();
    assert!(state.is_some());
    let current = state.unwrap();
    // Because elapsed > 180 seconds, activity_type must be Unknown
    assert_eq!(
        current.activity_type,
        ActivityType::Unknown,
        "Stale activity (>180s) must transition to Unknown"
    );
    assert!(current.current_block.is_none());

    // Now insert a fresh event (5 seconds ago)
    let fresh_ts = now_ms - 5_000;
    db.insert_raw_event(&make_event(fresh_ts, "code", "active_editor"))
        .unwrap();

    let fresh_state = ActivityProcessor::get_current_activity(&db)
        .unwrap()
        .unwrap();
    assert_eq!(fresh_state.active_app, "code");
    assert_eq!(fresh_state.activity_type, ActivityType::Active);
}
