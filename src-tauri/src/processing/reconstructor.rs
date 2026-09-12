use crate::core::error::Result;
use crate::domain::{ActivityType, TimeBlock};
use crate::processing::aggregator::aggregate_segments_to_blocks;
use crate::processing::segment::reconstruct_segments;
use crate::storage::DatabaseManager;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentActivityState {
    pub active_app: String,
    pub active_title: String,
    pub activity_type: ActivityType,
    pub current_block: Option<TimeBlock>,
    pub elapsed_in_state_seconds: u64,
}

pub struct ActivityProcessor;

impl ActivityProcessor {
    /// Reconstructs and persists TimeBlocks for a given time range from raw events.
    pub fn process_range(
        db: &DatabaseManager,
        start_ms: i64,
        end_ms: i64,
    ) -> Result<Vec<TimeBlock>> {
        let events = db.get_raw_events_range(start_ms, end_ms)?;
        if events.is_empty() {
            return Ok(Vec::new());
        }

        let segments = reconstruct_segments(&events);
        let blocks = aggregate_segments_to_blocks(&segments);

        if !blocks.is_empty() {
            db.insert_time_blocks_batch(&blocks)?;
        }

        Ok(blocks)
    }

    /// Rebuilds the entire TimeBlock history from raw events.
    pub fn rebuild_all_history(db: &DatabaseManager) -> Result<usize> {
        let earliest = db.get_earliest_raw_event_timestamp()?;
        let now_ms = chrono::Utc::now().timestamp_millis();

        match earliest {
            Some(start_ms) => {
                let blocks = Self::process_range(db, start_ms, now_ms)?;
                Ok(blocks.len())
            }
            None => Ok(0),
        }
    }

    /// Evaluates the current user presence and active window state.
    pub fn get_current_activity(db: &DatabaseManager) -> Result<Option<CurrentActivityState>> {
        let recent_events = db.get_recent_raw_events(2)?;
        if recent_events.is_empty() {
            return Ok(None);
        }

        let latest = &recent_events[0];
        let now_ms = chrono::Utc::now().timestamp_millis();
        let elapsed = (now_ms - latest.timestamp_ms).max(0) as u64 / 1000;

        let activity_type = if latest.source == crate::domain::RawEventSource::Afk {
            if latest.title.to_lowercase() == "afk" {
                ActivityType::Afk
            } else {
                ActivityType::Active
            }
        } else if elapsed > 180 {
            ActivityType::Unknown
        } else {
            ActivityType::Active
        };

        // Fetch recent timeblock if it encloses the current instant
        let recent_blocks = db.get_recent_time_blocks(1)?;
        let current_block = recent_blocks
            .into_iter()
            .next()
            .filter(|b| now_ms >= b.start_ms && now_ms < b.end_ms);

        Ok(Some(CurrentActivityState {
            active_app: latest.app.clone(),
            active_title: latest.title.clone(),
            activity_type,
            current_block,
            elapsed_in_state_seconds: elapsed,
        }))
    }

    /// Derives the exact contiguous activity segment composition of an interval on demand from raw events.
    pub fn get_block_composition(
        db: &DatabaseManager,
        start_ms: i64,
        end_ms: i64,
    ) -> Result<Vec<crate::domain::ActivitySegment>> {
        let events = db.get_raw_events_range(start_ms, end_ms)?;
        if events.is_empty() {
            return Ok(Vec::new());
        }

        let segments = reconstruct_segments(&events);
        let mut composition = Vec::new();

        for seg in segments {
            if seg.end_ms <= start_ms || seg.start_ms >= end_ms {
                continue;
            }
            let overlap_start = seg.start_ms.max(start_ms);
            let overlap_end = seg.end_ms.min(end_ms);
            if overlap_end > overlap_start {
                composition.push(crate::domain::ActivitySegment {
                    start_ms: overlap_start,
                    end_ms: overlap_end,
                    app: seg.app,
                    title: seg.title,
                    activity_type: seg.activity_type,
                    source: seg.source,
                    event_count: seg.event_count,
                });
            }
        }

        Ok(composition)
    }
}
