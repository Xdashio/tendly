use crate::domain::{ActivitySegment, ActivityType, TimeBlock};
use crate::processing::segment::EPOCH_BLOCK_DURATION_MS;
use std::collections::HashMap;

/// Aggregates a sequence of continuous `ActivitySegment`s into canonical 3-minute `TimeBlock`s.
pub fn aggregate_segments_to_blocks(segments: &[ActivitySegment]) -> Vec<TimeBlock> {
    if segments.is_empty() {
        return Vec::new();
    }

    let min_start = match segments.iter().map(|s| s.start_ms).min() {
        Some(v) => v,
        None => return Vec::new(),
    };

    let max_end = match segments.iter().map(|s| s.end_ms).max() {
        Some(v) => v,
        None => return Vec::new(),
    };

    let first_epoch = min_start - (min_start % EPOCH_BLOCK_DURATION_MS);
    let mut last_epoch = max_end;
    let rem = max_end % EPOCH_BLOCK_DURATION_MS;
    if rem != 0 {
        last_epoch += EPOCH_BLOCK_DURATION_MS - rem;
    }

    let mut blocks: Vec<TimeBlock> = Vec::new();
    let mut epoch_start = first_epoch;

    while epoch_start < last_epoch {
        let epoch_end = epoch_start + EPOCH_BLOCK_DURATION_MS;

        // Collect all overlapping segments within this epoch
        let mut app_title_durations: HashMap<(String, String), i64> = HashMap::new();
        let mut total_active_ms: i64 = 0;
        let mut total_afk_ms: i64 = 0;
        let mut total_unknown_ms: i64 = 0;

        for seg in segments {
            if seg.end_ms <= epoch_start || seg.start_ms >= epoch_end {
                continue; // No overlap
            }

            let overlap_start = seg.start_ms.max(epoch_start);
            let overlap_end = seg.end_ms.min(epoch_end);
            let duration = (overlap_end - overlap_start).max(0);

            if duration == 0 {
                continue;
            }

            match seg.activity_type {
                ActivityType::Active => {
                    total_active_ms += duration;
                    let key = (seg.app.clone(), seg.title.clone());
                    *app_title_durations.entry(key).or_insert(0) += duration;
                }
                ActivityType::Afk => {
                    total_afk_ms += duration;
                }
                ActivityType::Unknown => {
                    total_unknown_ms += duration;
                }
            }
        }

        let total_represented_ms = total_active_ms + total_afk_ms + total_unknown_ms;
        if total_represented_ms > 0 {
            // Determine dominant activity type
            let (activity_type, dominant_app, dominant_title) = if total_active_ms >= total_afk_ms
                && total_active_ms >= total_unknown_ms
                && total_active_ms > 0
            {
                // Active plurality winner
                let (app, title) = app_title_durations
                    .into_iter()
                    .max_by_key(|(_, d)| *d)
                    .map(|(k, _)| k)
                    .unwrap_or_else(|| ("unknown".to_string(), "unknown".to_string()));

                (ActivityType::Active, app, title)
            } else if total_afk_ms >= total_unknown_ms && total_afk_ms > 0 {
                (ActivityType::Afk, "system".to_string(), "afk".to_string())
            } else {
                (
                    ActivityType::Unknown,
                    "system".to_string(),
                    "unknown".to_string(),
                )
            };

            // Deterministic UUID v5 based on start timestamp
            let block_id = uuid::Uuid::new_v5(
                &uuid::Uuid::NAMESPACE_OID,
                format!("timeblock:{}", epoch_start).as_bytes(),
            )
            .to_string();

            blocks.push(TimeBlock {
                id: block_id,
                start_ms: epoch_start,
                end_ms: epoch_end,
                duration_ms: EPOCH_BLOCK_DURATION_MS,
                activity_type,
                dominant_app,
                dominant_title,
                dominant_url: None,
                classification: None,
                category: None,
                confidence: None,
                classified_by: None,
                user_override: None,
            });
        }

        epoch_start = epoch_end;
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::RawEventSource;

    #[test]
    fn test_single_active_segment_aggregation() {
        let segment = ActivitySegment {
            start_ms: 0,
            end_ms: 180_000,
            app: "code".to_string(),
            title: "main.rs".to_string(),
            activity_type: ActivityType::Active,
            source: RawEventSource::X11,
            event_count: 3,
        };

        let blocks = aggregate_segments_to_blocks(&[segment]);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].start_ms, 0);
        assert_eq!(blocks[0].end_ms, 180_000);
        assert_eq!(blocks[0].dominant_app, "code");
        assert_eq!(blocks[0].dominant_title, "main.rs");
        assert_eq!(blocks[0].activity_type, ActivityType::Active);
    }

    #[test]
    fn test_plurality_winner_in_mixed_block() {
        // In window [0, 180_000]:
        // 50s in terminal, 130s in VS Code -> VS Code should be plurality winner
        let seg1 = ActivitySegment {
            start_ms: 0,
            end_ms: 50_000,
            app: "alacritty".to_string(),
            title: "zsh".to_string(),
            activity_type: ActivityType::Active,
            source: RawEventSource::X11,
            event_count: 1,
        };
        let seg2 = ActivitySegment {
            start_ms: 50_000,
            end_ms: 180_000,
            app: "code".to_string(),
            title: "main.rs".to_string(),
            activity_type: ActivityType::Active,
            source: RawEventSource::X11,
            event_count: 2,
        };

        let blocks = aggregate_segments_to_blocks(&[seg1, seg2]);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].dominant_app, "code");
        assert_eq!(blocks[0].dominant_title, "main.rs");
    }

    #[test]
    fn test_afk_dominant_block() {
        // 140s AFK, 40s Code -> AFK should dominate
        let seg_afk = ActivitySegment {
            start_ms: 0,
            end_ms: 140_000,
            app: "system".to_string(),
            title: "afk".to_string(),
            activity_type: ActivityType::Afk,
            source: RawEventSource::Afk,
            event_count: 1,
        };
        let seg_code = ActivitySegment {
            start_ms: 140_000,
            end_ms: 180_000,
            app: "code".to_string(),
            title: "main.rs".to_string(),
            activity_type: ActivityType::Active,
            source: RawEventSource::X11,
            event_count: 1,
        };

        let blocks = aggregate_segments_to_blocks(&[seg_afk, seg_code]);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].activity_type, ActivityType::Afk);
        assert_eq!(blocks[0].dominant_app, "system");
        assert_eq!(blocks[0].dominant_title, "afk");
    }
}
