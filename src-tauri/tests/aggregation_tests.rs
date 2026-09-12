//! Automated test suite for Phase 3 Activity Processing and Time-Block Aggregation.
//!
//! Validates:
//! - All 20 required boundary conditions
//! - Core invariant properties (non-overlapping, positive duration, deterministic rebuild)
//! - Performance scaling benchmarks (1-hour and 1-day synthetic datasets)

use std::collections::HashSet;
use tempfile::NamedTempFile;
use tendly_lib::domain::{ActivityType, RawEvent, RawEventSource};
use tendly_lib::processing::{
    aggregate_segments_to_blocks, reconstruct_segments, ActivityProcessor,
};
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

fn make_afk_event(ts: i64, title: &str) -> RawEvent {
    RawEvent::new(
        RawEventSource::Afk,
        ts,
        "system".to_string(),
        title.to_string(),
        None,
        if title == "afk" {
            Some(300_000)
        } else {
            Some(0)
        },
    )
}

fn create_test_db() -> (NamedTempFile, DatabaseManager) {
    let file = NamedTempFile::new().expect("Failed to create temp db file");
    let db = DatabaseManager::open(file.path()).expect("Failed to open test db");
    (file, db)
}

// ---------------------------------------------------------------------------
// 20 REQUIRED BOUNDARY CONDITIONS
// ---------------------------------------------------------------------------

#[test]
fn test_01_single_event() {
    let events = vec![make_event(1_000_000, "code", "main.rs - Tendly")];
    let segments = reconstruct_segments(&events);
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].app, "code");
    assert_eq!(segments[0].activity_type, ActivityType::Active);
    assert_eq!(segments[0].duration_ms(), 60_000); // Heartbeat grace

    let blocks = aggregate_segments_to_blocks(&segments);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].dominant_app, "code");
}

#[test]
fn test_02_two_different_applications() {
    let t0 = 1_800_000; // Aligned to 3-min boundary (10 * 180_000)
    let events = vec![
        make_event(t0, "code", "editor"),
        make_event(t0 + 120_000, "firefox", "browser"),
        make_event(t0 + 180_000, "firefox", "browser"),
    ];
    let segments = reconstruct_segments(&events);
    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].app, "code");
    assert_eq!(segments[0].duration_ms(), 120_000);
    assert_eq!(segments[1].app, "firefox");

    let blocks = aggregate_segments_to_blocks(&segments);
    assert!(!blocks.is_empty());
    // In block [t0, t0 + 180_000), "code" had 120s vs "firefox" 60s -> "code" wins plurality
    assert_eq!(blocks[0].dominant_app, "code");
}

#[test]
fn test_03_same_application_repeated() {
    let t0 = 1_800_000;
    let events = vec![
        make_event(t0, "code", "main.rs"),
        make_event(t0 + 10_000, "code", "main.rs"),
        make_event(t0 + 20_000, "code", "main.rs"),
        make_event(t0 + 30_000, "code", "main.rs"),
    ];
    let segments = reconstruct_segments(&events);
    // Consecutive events with identical app and title are coalesced
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].app, "code");
    assert_eq!(segments[0].event_count, 4);
    assert_eq!(segments[0].end_ms, t0 + 30_000 + 60_000);
}

#[test]
fn test_04_same_application_changed_title() {
    let t0 = 1_800_000;
    let events = vec![
        make_event(t0, "code", "lib.rs"),
        make_event(t0 + 40_000, "code", "main.rs"),
        make_event(t0 + 80_000, "code", "Cargo.toml"),
    ];
    let segments = reconstruct_segments(&events);
    assert_eq!(segments.len(), 3);
    assert_eq!(segments[0].title, "lib.rs");
    assert_eq!(segments[1].title, "main.rs");
    assert_eq!(segments[2].title, "Cargo.toml");
}

#[test]
fn test_05_afk_entry() {
    let t0 = 1_800_000;
    let events = vec![
        make_event(t0, "code", "editor"),
        make_afk_event(t0 + 60_000, "afk"),
    ];
    let segments = reconstruct_segments(&events);
    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].activity_type, ActivityType::Active);
    assert_eq!(segments[1].activity_type, ActivityType::Afk);
    assert_eq!(segments[1].start_ms, t0 + 60_000);
}

#[test]
fn test_06_afk_exit() {
    let t0 = 1_800_000;
    let events = vec![
        make_event(t0, "code", "editor"),
        make_afk_event(t0 + 60_000, "afk"),
        make_afk_event(t0 + 360_000, "active"),
        make_event(t0 + 360_000, "code", "editor"),
    ];
    let segments = reconstruct_segments(&events);
    assert!(segments
        .iter()
        .any(|s| s.activity_type == ActivityType::Afk));
    assert_eq!(segments.last().unwrap().activity_type, ActivityType::Active);
}

#[test]
fn test_07_multiple_afk_periods() {
    let t0 = 1_800_000;
    let events = vec![
        make_event(t0, "code", "editor"),
        make_afk_event(t0 + 60_000, "afk"),
        make_afk_event(t0 + 120_000, "active"),
        make_event(t0 + 120_000, "code", "editor"),
        make_afk_event(t0 + 200_000, "afk"),
        make_afk_event(t0 + 400_000, "active"),
        make_event(t0 + 400_000, "terminal", "bash"),
    ];
    let segments = reconstruct_segments(&events);
    let afk_count = segments
        .iter()
        .filter(|s| s.activity_type == ActivityType::Afk)
        .count();
    assert_eq!(afk_count, 2);
}

#[test]
fn test_08_event_immediately_before_afk() {
    let t0 = 1_800_000;
    let events = vec![
        make_event(t0, "code", "editor"),
        make_event(t0 + 59_999, "terminal", "bash"),
        make_afk_event(t0 + 60_000, "afk"),
    ];
    let segments = reconstruct_segments(&events);
    assert_eq!(segments.len(), 3);
    assert_eq!(segments[1].app, "terminal");
    assert_eq!(segments[1].end_ms, t0 + 60_000);
    assert_eq!(segments[2].activity_type, ActivityType::Afk);
}

#[test]
fn test_09_event_immediately_after_afk() {
    let t0 = 1_800_000;
    let events = vec![
        make_afk_event(t0, "afk"),
        make_afk_event(t0 + 180_000, "active"),
        make_event(t0 + 180_001, "code", "editor"),
    ];
    let segments = reconstruct_segments(&events);
    assert_eq!(segments[0].activity_type, ActivityType::Afk);
    assert_eq!(segments.last().unwrap().activity_type, ActivityType::Active);
    assert_eq!(segments.last().unwrap().app, "code");
}

#[test]
fn test_10_missing_events_large_gaps() {
    let t0 = 1_800_000;
    let events = vec![
        make_event(t0, "code", "editor"),
        // 2 hours unobserved gap without AFK or active events
        make_event(t0 + 7_200_000, "code", "editor"),
    ];
    let segments = reconstruct_segments(&events);
    // Must contain an explicit Unknown segment between active runs
    let unknown_segments: Vec<_> = segments
        .iter()
        .filter(|s| s.activity_type == ActivityType::Unknown)
        .collect();
    assert_eq!(unknown_segments.len(), 1);
    assert_eq!(unknown_segments[0].start_ms, t0 + 60_000);
    assert_eq!(unknown_segments[0].end_ms, t0 + 7_200_000);
}

#[test]
fn test_11_out_of_order_events() {
    let t0 = 1_800_000;
    let events = vec![
        make_event(t0 + 60_000, "firefox", "docs"),
        make_event(t0, "code", "editor"),
        make_event(t0 + 120_000, "terminal", "cargo check"),
    ];
    let segments = reconstruct_segments(&events);
    assert_eq!(segments[0].app, "code");
    assert_eq!(segments[1].app, "firefox");
    assert_eq!(segments[2].app, "terminal");
}

#[test]
fn test_12_duplicate_timestamps() {
    let t0 = 1_800_000;
    let mut e1 = make_event(t0, "code", "editor");
    e1.id = "id-001".to_string();
    let mut e2 = make_event(t0, "firefox", "browser");
    e2.id = "id-002".to_string();

    let events = vec![e2, e1];
    let segments = reconstruct_segments(&events);
    // Does not crash or panic; handles ties deterministically
    assert!(!segments.is_empty());
}

#[test]
fn test_13_duplicate_events() {
    let t0 = 1_800_000;
    let e = make_event(t0, "code", "editor");
    let events = vec![e.clone(), e];
    let segments = reconstruct_segments(&events);
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].app, "code");
}

#[test]
fn test_14_midnight_boundary() {
    // 2026-09-12 23:58:00 UTC (1789257480000) through midnight
    let t_pre_midnight = 1_789_257_480_000i64;
    let events = vec![
        make_event(t_pre_midnight, "code", "night work"),
        make_event(t_pre_midnight + 300_000, "code", "morning work"),
    ];
    let segments = reconstruct_segments(&events);
    let blocks = aggregate_segments_to_blocks(&segments);
    assert!(!blocks.is_empty());
    for block in &blocks {
        assert_eq!(block.duration_ms, 180_000);
        assert_eq!(block.end_ms - block.start_ms, 180_000);
    }
}

#[test]
fn test_15_large_time_gap_sleep_suspend() {
    let t0 = 1_800_000;
    let events = vec![
        make_event(t0, "code", "before sleep"),
        // Laptop closed for 8 hours (28,800,000 ms)
        make_event(t0 + 28_800_000, "code", "after wake"),
    ];
    let segments = reconstruct_segments(&events);
    let unknown_seg = segments
        .iter()
        .find(|s| s.activity_type == ActivityType::Unknown)
        .expect("Must have Unknown segment for suspend gap");
    assert!(unknown_seg.duration_ms() >= 28_700_000);

    let blocks = aggregate_segments_to_blocks(&segments);
    let unknown_blocks = blocks
        .iter()
        .filter(|b| b.activity_type == ActivityType::Unknown)
        .count();
    assert!(unknown_blocks > 100);
}

#[test]
fn test_16_watcher_restart() {
    let t0 = 1_800_000;
    let events = vec![
        make_event(t0, "code", "session 1"),
        make_event(t0 + 30_000, "code", "session 1"),
        // Watcher restarted 5 seconds later
        make_event(t0 + 35_000, "code", "session 1"),
        make_event(t0 + 70_000, "code", "session 1"),
    ];
    let segments = reconstruct_segments(&events);
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].event_count, 4);
}

#[test]
fn test_17_historical_reprocessing() {
    let (_tmp, db) = create_test_db();
    let t0 = 1_800_000;

    let e1 = make_event(t0, "code", "task 1");
    let e2 = make_event(t0 + 100_000, "firefox", "task 2");
    let e3 = make_event(t0 + 300_000, "terminal", "task 3");

    db.insert_raw_event(&e1).unwrap();
    db.insert_raw_event(&e2).unwrap();
    db.insert_raw_event(&e3).unwrap();

    let processed_count = ActivityProcessor::rebuild_all_history(&db).unwrap();
    assert!(processed_count > 0);

    let blocks = db.get_recent_time_blocks(10).unwrap();
    assert_eq!(blocks.len(), processed_count);
}

#[test]
fn test_18_reprocessing_same_data_twice_idempotency() {
    let (_tmp, db) = create_test_db();
    let t0 = 1_800_000;

    for i in 0..10 {
        let ev = make_event(t0 + i * 30_000, "code", "dev");
        db.insert_raw_event(&ev).unwrap();
    }

    let first_run = ActivityProcessor::rebuild_all_history(&db).unwrap();
    let blocks_after_first = db.get_recent_time_blocks(50).unwrap();

    // Second run with exact same data
    let second_run = ActivityProcessor::rebuild_all_history(&db).unwrap();
    let blocks_after_second = db.get_recent_time_blocks(50).unwrap();

    assert_eq!(first_run, second_run);
    assert_eq!(blocks_after_first.len(), blocks_after_second.len());
    for (b1, b2) in blocks_after_first.iter().zip(blocks_after_second.iter()) {
        assert_eq!(b1.id, b2.id);
        assert_eq!(b1.start_ms, b2.start_ms);
        assert_eq!(b1.end_ms, b2.end_ms);
        assert_eq!(b1.dominant_app, b2.dominant_app);
    }
}

#[test]
fn test_19_empty_database() {
    let (_tmp, db) = create_test_db();
    let count = ActivityProcessor::rebuild_all_history(&db).unwrap();
    assert_eq!(count, 0);

    let blocks = db.get_recent_time_blocks(10).unwrap();
    assert!(blocks.is_empty());

    let current = ActivityProcessor::get_current_activity(&db).unwrap();
    assert!(current.is_none());
}

#[test]
fn test_20_corrupt_invalid_timestamps() {
    let events = vec![
        make_event(-500, "corrupt", "negative ts"),
        make_event(0, "corrupt", "zero ts"),
        make_event(1_800_000, "valid", "good ts"),
    ];
    let segments = reconstruct_segments(&events);
    // Invalid timestamps (< 0 or 0) are discarded
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].app, "valid");
}

// ---------------------------------------------------------------------------
// CORE INVARIANTS PROPERTY TESTS
// ---------------------------------------------------------------------------

#[test]
fn test_invariants_no_block_overlap() {
    let t0 = 1_800_000;
    let mut events = Vec::new();
    for i in 0..50 {
        events.push(make_event(
            t0 + i * 25_000,
            if i % 2 == 0 { "code" } else { "browser" },
            "work",
        ));
    }

    let segments = reconstruct_segments(&events);
    let blocks = aggregate_segments_to_blocks(&segments);

    assert!(blocks.len() > 1);
    let mut seen_intervals: Vec<(i64, i64)> = Vec::new();

    for block in &blocks {
        assert!(block.duration_ms >= 0, "Duration must be non-negative");
        assert!(block.end_ms >= block.start_ms, "End must be >= start");
        assert_eq!(
            block.end_ms - block.start_ms,
            block.duration_ms,
            "End - start must equal duration"
        );

        for &(prev_start, prev_end) in &seen_intervals {
            let overlaps = block.start_ms < prev_end && block.end_ms > prev_start;
            assert!(
                !overlaps,
                "Discovered overlapping blocks: [{}, {}) vs [{}, {})",
                block.start_ms, block.end_ms, prev_start, prev_end
            );
        }
        seen_intervals.push((block.start_ms, block.end_ms));
    }
}

#[test]
fn test_invariants_unique_block_ids() {
    let t0 = 1_800_000;
    let mut events = Vec::new();
    for i in 0..30 {
        events.push(make_event(t0 + i * 60_000, "code", "work"));
    }

    let segments = reconstruct_segments(&events);
    let blocks = aggregate_segments_to_blocks(&segments);

    let mut ids: HashSet<String> = HashSet::new();
    for block in &blocks {
        assert!(
            ids.insert(block.id.clone()),
            "Duplicate block ID found: {}",
            block.id
        );
    }
}

// ---------------------------------------------------------------------------
// PERFORMANCE SCALE BENCHMARKS
// ---------------------------------------------------------------------------

#[test]
fn test_benchmark_synthetic_1_hour_dataset() {
    let t0 = 1_800_000;
    // 1 hour at 2s sampling = 1,800 events
    let mut events = Vec::with_capacity(1_800);
    for i in 0..1_800 {
        events.push(make_event(
            t0 + i * 2_000,
            if i % 300 < 200 { "code" } else { "firefox" },
            "coding",
        ));
    }

    let start = std::time::Instant::now();
    let segments = reconstruct_segments(&events);
    let blocks = aggregate_segments_to_blocks(&segments);
    let elapsed = start.elapsed();

    assert!(!blocks.is_empty());
    println!("Processed 1,800 events (1 hour) in {:?}", elapsed);
    assert!(
        elapsed.as_millis() < 500,
        "1-hour processing took too long: {:?}",
        elapsed
    );
}

#[test]
fn test_benchmark_synthetic_1_day_dataset() {
    let t0 = 1_800_000;
    // 1 day at 2s sampling = 43,200 events
    let mut events = Vec::with_capacity(43_200);
    for i in 0..43_200 {
        events.push(make_event(
            t0 + i * 2_000,
            if i % 900 < 600 { "code" } else { "slack" },
            "workday",
        ));
    }

    let start = std::time::Instant::now();
    let segments = reconstruct_segments(&events);
    let blocks = aggregate_segments_to_blocks(&segments);
    let elapsed = start.elapsed();

    let rate = 43_200.0 / elapsed.as_secs_f64();
    println!(
        "Processed 43,200 events (1 day) in {:?} ({:.0} events/sec)",
        elapsed, rate
    );

    assert!(!blocks.is_empty());
    assert!(
        rate > 10_000.0,
        "Throughput below target: {:.0} events/sec",
        rate
    );
}
