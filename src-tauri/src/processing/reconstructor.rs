use crate::core::error::Result;
use crate::domain::{
    ActivitySegment, ActivitySession, ActivityType, AppDurationSummary, DailyTimeline,
    SessionDetails, TimeBlock,
};
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
    ) -> Result<Vec<ActivitySegment>> {
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
                composition.push(ActivitySegment {
                    start_ms: overlap_start,
                    end_ms: overlap_end,
                    app: seg.app,
                    title: seg.title,
                    activity_type: seg.activity_type,
                    source: seg.source,
                    event_count: seg.event_count,
                    browser_context: seg.browser_context,
                });
            }
        }

        Ok(composition)
    }

    /// Derives the daily timeline for a date range [day_start_ms, day_end_ms).
    pub fn get_daily_timeline(
        db: &DatabaseManager,
        day_start_ms: i64,
        day_end_ms: i64,
    ) -> Result<DailyTimeline> {
        let blocks = db.get_time_blocks_range(day_start_ms, day_end_ms)?;
        let block_count = blocks.len();

        let mut total_active_ms = 0;
        let mut total_afk_ms = 0;
        let mut total_unknown_ms = 0;

        for b in &blocks {
            match b.activity_type {
                ActivityType::Active => total_active_ms += b.duration_ms,
                ActivityType::Afk => total_afk_ms += b.duration_ms,
                ActivityType::Unknown => total_unknown_ms += b.duration_ms,
            }
        }

        if blocks.is_empty() {
            return Ok(DailyTimeline {
                day_start_ms,
                day_end_ms,
                sessions: Vec::new(),
                total_active_ms: 0,
                total_afk_ms: 0,
                total_unknown_ms: 0,
                block_count: 0,
            });
        }

        let mut coalesced = crate::processing::session::coalesce_blocks_into_sessions(&blocks);

        // Calculate secondary activity across the day using reconstructed raw segments
        let day_events = db.get_raw_events_range(day_start_ms, day_end_ms)?;
        let day_segments = reconstruct_segments(&day_events);

        for session in &mut coalesced {
            let mut app_totals: std::collections::HashMap<String, i64> =
                std::collections::HashMap::new();
            for seg in &day_segments {
                if seg.end_ms <= session.start_ms || seg.start_ms >= session.end_ms {
                    continue;
                }
                let overlap_start = seg.start_ms.max(session.start_ms);
                let overlap_end = seg.end_ms.min(session.end_ms);
                let dur = (overlap_end - overlap_start).max(0);
                if dur > 0 {
                    *app_totals.entry(seg.app.clone()).or_insert(0) += dur;
                }
            }

            let mut secondary: Vec<AppDurationSummary> = app_totals
                .into_iter()
                .filter(|(app, _)| app != &session.dominant_app)
                .map(|(app, duration_ms)| AppDurationSummary { app, duration_ms })
                .collect();
            secondary.sort_by(|a, b| {
                b.duration_ms
                    .cmp(&a.duration_ms)
                    .then_with(|| a.app.cmp(&b.app))
            });

            session.has_secondary_activity = !secondary.is_empty();
            session.secondary_apps = secondary;
        }

        let sessions_with_gaps = crate::processing::session::insert_unrecorded_gap_sessions(
            coalesced,
            day_start_ms,
            day_end_ms,
        );

        Ok(DailyTimeline {
            day_start_ms,
            day_end_ms,
            sessions: sessions_with_gaps,
            total_active_ms,
            total_afk_ms,
            total_unknown_ms,
            block_count,
        })
    }

    /// Returns detailed session breakdown including exact underlying activity segments and app duration totals.
    pub fn get_session_details(
        db: &DatabaseManager,
        session_id: &str,
        start_ms: i64,
        end_ms: i64,
    ) -> Result<SessionDetails> {
        let blocks = db.get_time_blocks_range(start_ms, end_ms)?;
        let segments = Self::get_block_composition(db, start_ms, end_ms)?;

        let mut app_totals: std::collections::HashMap<String, i64> =
            std::collections::HashMap::new();
        for seg in &segments {
            *app_totals.entry(seg.app.clone()).or_insert(0) += seg.duration_ms();
        }

        let mut app_breakdown: Vec<AppDurationSummary> = app_totals
            .into_iter()
            .map(|(app, duration_ms)| AppDurationSummary { app, duration_ms })
            .collect();
        app_breakdown.sort_by(|a, b| {
            b.duration_ms
                .cmp(&a.duration_ms)
                .then_with(|| a.app.cmp(&b.app))
        });

        let coalesced = crate::processing::session::coalesce_blocks_into_sessions(&blocks);
        let session = if let Some(mut s) = coalesced
            .iter()
            .find(|s| s.id == session_id)
            .cloned()
            .or_else(|| coalesced.into_iter().next())
        {
            s.secondary_apps = app_breakdown
                .iter()
                .filter(|item| item.app != s.dominant_app)
                .cloned()
                .collect();
            s.has_secondary_activity = !s.secondary_apps.is_empty();
            s
        } else {
            ActivitySession {
                id: session_id.to_string(),
                start_ms,
                end_ms,
                duration_ms: (end_ms - start_ms).max(0),
                dominant_app: "unrecorded".to_string(),
                dominant_title: "No recorded activity".to_string(),
                activity_type: ActivityType::Unknown,
                block_count: 0,
                time_blocks: Vec::new(),
                has_secondary_activity: false,
                secondary_apps: Vec::new(),
            }
        };

        Ok(SessionDetails {
            session,
            segments,
            app_breakdown,
        })
    }
}
