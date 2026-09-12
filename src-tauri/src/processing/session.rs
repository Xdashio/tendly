use crate::domain::{ActivitySession, ActivityType, TimeBlock};
use std::collections::HashMap;

/// Minimum gap between recorded sessions to be considered an unrecorded period (3 minutes).
pub const MIN_UNRECORDED_GAP_MS: i64 = 180_000;

/// Coalesces a chronologically sorted slice of `TimeBlock`s into user-facing `ActivitySession`s.
///
/// Two blocks are eligible for the same session if and only if:
/// 1. They are temporally contiguous (current.end_ms == next.start_ms).
/// 2. They share the same dominant application.
/// 3. They share the same activity type (e.g. Active, Afk, Unknown).
pub fn coalesce_blocks_into_sessions(blocks: &[TimeBlock]) -> Vec<ActivitySession> {
    if blocks.is_empty() {
        return Vec::new();
    }

    let mut sorted_blocks = blocks.to_vec();
    sorted_blocks.sort_by_key(|b| b.start_ms);

    let mut sessions: Vec<ActivitySession> = Vec::new();
    let mut current_group: Vec<TimeBlock> = Vec::new();

    for block in sorted_blocks {
        if current_group.is_empty() {
            current_group.push(block);
            continue;
        }

        let last = current_group.last().unwrap();
        let is_contiguous = block.start_ms == last.end_ms;
        let is_same_app = block.dominant_app == last.dominant_app;
        let is_same_type = block.activity_type == last.activity_type;

        if is_contiguous && is_same_app && is_same_type {
            current_group.push(block);
        } else {
            sessions.push(build_session_from_blocks(&current_group));
            current_group.clear();
            current_group.push(block);
        }
    }

    if !current_group.is_empty() {
        sessions.push(build_session_from_blocks(&current_group));
    }

    sessions
}

/// Builds an `ActivitySession` from a contiguous slice of compatible `TimeBlock`s.
fn build_session_from_blocks(blocks: &[TimeBlock]) -> ActivitySession {
    let first = blocks.first().expect("Group must have at least one block");
    let last = blocks.last().expect("Group must have at least one block");

    let start_ms = first.start_ms;
    let end_ms = last.end_ms;
    let duration_ms = (end_ms - start_ms).max(0);
    let dominant_app = first.dominant_app.clone();
    let activity_type = first.activity_type;

    // Pick dominant title by frequency across blocks
    let mut title_counts: HashMap<&str, usize> = HashMap::new();
    for b in blocks {
        *title_counts.entry(&b.dominant_title).or_insert(0) += 1;
    }
    let dominant_title = title_counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(title, _)| title.to_string())
        .unwrap_or_else(|| first.dominant_title.clone());

    let id = format!("session:{}:{}", start_ms, end_ms);

    ActivitySession {
        id,
        start_ms,
        end_ms,
        duration_ms,
        dominant_app,
        dominant_title,
        activity_type,
        block_count: blocks.len(),
        time_blocks: blocks.to_vec(),
        has_secondary_activity: false,
        secondary_apps: Vec::new(),
    }
}

/// Detects gaps between recorded sessions and inserts explicit unrecorded gap sessions.
pub fn insert_unrecorded_gap_sessions(
    sessions: Vec<ActivitySession>,
    _day_start_ms: i64,
    _day_end_ms: i64,
) -> Vec<ActivitySession> {
    if sessions.is_empty() {
        return sessions;
    }

    let mut result = Vec::new();
    let n = sessions.len();

    for i in 0..n {
        let current = &sessions[i];

        // Check if there is a gap before the next session
        if i + 1 < n {
            let next = &sessions[i + 1];
            result.push(current.clone());

            let gap_duration = next.start_ms - current.end_ms;
            if gap_duration >= MIN_UNRECORDED_GAP_MS {
                result.push(ActivitySession {
                    id: format!("gap:{}:{}", current.end_ms, next.start_ms),
                    start_ms: current.end_ms,
                    end_ms: next.start_ms,
                    duration_ms: gap_duration,
                    dominant_app: "unrecorded".to_string(),
                    dominant_title: "No recorded activity".to_string(),
                    activity_type: ActivityType::Unknown,
                    block_count: 0,
                    time_blocks: Vec::new(),
                    has_secondary_activity: false,
                    secondary_apps: Vec::new(),
                });
            }
        } else {
            result.push(current.clone());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::TimeBlock;

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

    #[test]
    fn test_coalesce_identical_adjacent_blocks() {
        let t0 = 1_800_000;
        let b1 = make_block(t0, "code", "main.rs", ActivityType::Active);
        let b2 = make_block(t0 + 180_000, "code", "lib.rs", ActivityType::Active);
        let b3 = make_block(t0 + 360_000, "code", "lib.rs", ActivityType::Active);

        let sessions = coalesce_blocks_into_sessions(&[b1, b2, b3]);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_ms, t0);
        assert_eq!(sessions[0].end_ms, t0 + 540_000);
        assert_eq!(sessions[0].duration_ms, 540_000);
        assert_eq!(sessions[0].block_count, 3);
        assert_eq!(sessions[0].dominant_app, "code");
        assert_eq!(sessions[0].dominant_title, "lib.rs"); // 2 blocks vs 1
    }

    #[test]
    fn test_no_coalesce_across_different_applications() {
        let t0 = 1_800_000;
        let b1 = make_block(t0, "code", "main.rs", ActivityType::Active);
        let b2 = make_block(t0 + 180_000, "slack", "general", ActivityType::Active);
        let b3 = make_block(t0 + 360_000, "code", "main.rs", ActivityType::Active);

        let sessions = coalesce_blocks_into_sessions(&[b1, b2, b3]);
        assert_eq!(sessions.len(), 3);
        assert_eq!(sessions[0].dominant_app, "code");
        assert_eq!(sessions[1].dominant_app, "slack");
        assert_eq!(sessions[2].dominant_app, "code");
    }

    #[test]
    fn test_no_coalesce_across_different_activity_types() {
        let t0 = 1_800_000;
        let b1 = make_block(t0, "code", "editor", ActivityType::Active);
        let b2 = make_block(t0 + 180_000, "code", "editor", ActivityType::Afk);

        let sessions = coalesce_blocks_into_sessions(&[b1, b2]);
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].activity_type, ActivityType::Active);
        assert_eq!(sessions[1].activity_type, ActivityType::Afk);
    }

    #[test]
    fn test_no_coalesce_across_time_gaps() {
        let t0 = 1_800_000;
        let b1 = make_block(t0, "code", "editor", ActivityType::Active);
        // 1 hour gap before next block
        let b2 = make_block(t0 + 3_600_000, "code", "editor", ActivityType::Active);

        let sessions = coalesce_blocks_into_sessions(&[b1, b2]);
        assert_eq!(sessions.len(), 2);
    }
}
