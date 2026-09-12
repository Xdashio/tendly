use crate::domain::{ActivitySegment, ActivityType, RawEvent, RawEventSource};

pub const EPOCH_BLOCK_DURATION_MS: i64 = 180_000; // 3 minutes
pub const HEARTBEAT_GRACE_MS: i64 = 60_000; // 1 minute
pub const MAX_OBSERVATION_GAP_MS: i64 = 180_000; // 3 minutes without event triggers Unknown gap

/// Reconstructs a continuous sequence of `ActivitySegment`s from a sorted list of `RawEvent`s.
pub fn reconstruct_segments(events: &[RawEvent]) -> Vec<ActivitySegment> {
    if events.is_empty() {
        return Vec::new();
    }

    // 1. Sort events chronologically, breaking ties deterministically by ID
    let mut sorted_events: Vec<RawEvent> = events
        .iter()
        .filter(|e| e.timestamp_ms > 0)
        .cloned()
        .collect();

    sorted_events.sort_by(|a, b| {
        a.timestamp_ms
            .cmp(&b.timestamp_ms)
            .then_with(|| a.id.cmp(&b.id))
    });

    if sorted_events.is_empty() {
        return Vec::new();
    }

    let mut raw_segments: Vec<ActivitySegment> = Vec::new();
    let n = sorted_events.len();

    for i in 0..n {
        let current = &sorted_events[i];
        let next = if i + 1 < n {
            Some(&sorted_events[i + 1])
        } else {
            None
        };

        let start_ms = current.timestamp_ms;
        let is_afk_event =
            current.source == RawEventSource::Afk && current.title.to_lowercase() == "afk";
        let is_active_resume_event =
            current.source == RawEventSource::Afk && current.title.to_lowercase() == "active";

        let mut browser_context = current
            .raw_json
            .as_deref()
            .and_then(|json| serde_json::from_str::<crate::domain::BrowserContext>(json).ok())
            .or_else(|| {
                crate::capture::browser_context::enrich_browser_context(
                    &current.app,
                    &current.title,
                )
            });

        if let (Some(url), Some(ctx)) = (&current.url, &mut browser_context) {
            if ctx.url.is_none() {
                ctx.url = crate::capture::browser_context::normalize_url(url);
            }
            if ctx.domain.is_none() {
                ctx.domain = crate::capture::browser_context::extract_domain(url);
            }
        }

        match next {
            Some(next_event) => {
                let next_ts = next_event.timestamp_ms;
                let delta = (next_ts - start_ms).max(0);

                if is_afk_event {
                    // User entered AFK: period until next event is AFK
                    raw_segments.push(ActivitySegment {
                        start_ms,
                        end_ms: next_ts,
                        app: "system".to_string(),
                        title: "afk".to_string(),
                        activity_type: ActivityType::Afk,
                        source: RawEventSource::Afk,
                        event_count: 1,
                        browser_context: None,
                    });
                } else if is_active_resume_event {
                    // Marker event that user returned: minimal 0-length bridge or up to next event
                    if delta > 0 && delta <= MAX_OBSERVATION_GAP_MS {
                        raw_segments.push(ActivitySegment {
                            start_ms,
                            end_ms: next_ts,
                            app: "system".to_string(),
                            title: "active".to_string(),
                            activity_type: ActivityType::Active,
                            source: RawEventSource::Afk,
                            event_count: 1,
                            browser_context: None,
                        });
                    }
                } else {
                    // Normal application active event
                    if delta <= MAX_OBSERVATION_GAP_MS {
                        // Contiguous active observation
                        raw_segments.push(ActivitySegment {
                            start_ms,
                            end_ms: next_ts,
                            app: current.app.clone(),
                            title: current.title.clone(),
                            activity_type: ActivityType::Active,
                            source: current.source,
                            event_count: 1,
                            browser_context: browser_context.clone(),
                        });
                    } else {
                        // Gap exceeded: cap active observation at HEARTBEAT_GRACE_MS
                        let capped_end = start_ms + HEARTBEAT_GRACE_MS;
                        raw_segments.push(ActivitySegment {
                            start_ms,
                            end_ms: capped_end,
                            app: current.app.clone(),
                            title: current.title.clone(),
                            activity_type: ActivityType::Active,
                            source: current.source,
                            event_count: 1,
                            browser_context: browser_context.clone(),
                        });

                        // Remaining elapsed time is Unknown
                        raw_segments.push(ActivitySegment {
                            start_ms: capped_end,
                            end_ms: next_ts,
                            app: "system".to_string(),
                            title: "unknown".to_string(),
                            activity_type: ActivityType::Unknown,
                            source: current.source,
                            event_count: 0,
                            browser_context: None,
                        });
                    }
                }
            }
            None => {
                // Final event in stream: extend by HEARTBEAT_GRACE_MS
                let end_ms = if is_afk_event {
                    start_ms
                        + current
                            .idle_ms
                            .unwrap_or(HEARTBEAT_GRACE_MS)
                            .max(HEARTBEAT_GRACE_MS)
                } else {
                    start_ms + HEARTBEAT_GRACE_MS
                };

                let activity_type = if is_afk_event {
                    ActivityType::Afk
                } else {
                    ActivityType::Active
                };

                let final_ctx = if is_afk_event {
                    None
                } else {
                    browser_context.clone()
                };

                raw_segments.push(ActivitySegment {
                    start_ms,
                    end_ms,
                    app: current.app.clone(),
                    title: current.title.clone(),
                    activity_type,
                    source: current.source,
                    event_count: 1,
                    browser_context: final_ctx,
                });
            }
        }
    }

    // 2. Coalesce adjacent contiguous segments with identical app, title, and activity_type
    coalesce_segments(raw_segments)
}

fn coalesce_segments(segments: Vec<ActivitySegment>) -> Vec<ActivitySegment> {
    if segments.is_empty() {
        return Vec::new();
    }

    let mut merged: Vec<ActivitySegment> = Vec::new();

    for seg in segments {
        if seg.duration_ms() == 0 {
            continue;
        }

        if let Some(last) = merged.last_mut() {
            let contiguous = last.end_ms == seg.start_ms;
            let same_app = last.app == seg.app;
            let same_title = last.title == seg.title;
            let same_type = last.activity_type == seg.activity_type;
            let same_ctx = last.browser_context == seg.browser_context;

            if contiguous && same_app && same_title && same_type && same_ctx {
                last.end_ms = seg.end_ms;
                last.event_count += seg.event_count;
                continue;
            }
        }

        merged.push(seg);
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_event_reconstruction() {
        let events = vec![RawEvent {
            id: "1".to_string(),
            source: RawEventSource::X11,
            timestamp_ms: 1_000_000,
            app: "code".to_string(),
            title: "main.rs".to_string(),
            url: None,
            idle_ms: None,
            raw_json: None,
        }];

        let segments = reconstruct_segments(&events);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].start_ms, 1_000_000);
        assert_eq!(segments[0].end_ms, 1_000_000 + HEARTBEAT_GRACE_MS);
        assert_eq!(segments[0].app, "code");
        assert_eq!(segments[0].activity_type, ActivityType::Active);
    }

    #[test]
    fn test_two_different_applications() {
        let events = vec![
            RawEvent {
                id: "1".to_string(),
                source: RawEventSource::X11,
                timestamp_ms: 10_000,
                app: "code".to_string(),
                title: "main.rs".to_string(),
                url: None,
                idle_ms: None,
                raw_json: None,
            },
            RawEvent {
                id: "2".to_string(),
                source: RawEventSource::X11,
                timestamp_ms: 25_000,
                app: "firefox".to_string(),
                title: "Rust Docs".to_string(),
                url: None,
                idle_ms: None,
                raw_json: None,
            },
        ];

        let segments = reconstruct_segments(&events);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].app, "code");
        assert_eq!(segments[0].start_ms, 10_000);
        assert_eq!(segments[0].end_ms, 25_000);

        assert_eq!(segments[1].app, "firefox");
        assert_eq!(segments[1].start_ms, 25_000);
        assert_eq!(segments[1].end_ms, 25_000 + HEARTBEAT_GRACE_MS);
    }

    #[test]
    fn test_large_gap_creates_unknown_segment() {
        let events = vec![
            RawEvent {
                id: "1".to_string(),
                source: RawEventSource::X11,
                timestamp_ms: 10_000,
                app: "code".to_string(),
                title: "main.rs".to_string(),
                url: None,
                idle_ms: None,
                raw_json: None,
            },
            RawEvent {
                id: "2".to_string(),
                source: RawEventSource::X11,
                timestamp_ms: 600_000, // 10 minutes later (e.g. system suspend)
                app: "code".to_string(),
                title: "main.rs".to_string(),
                url: None,
                idle_ms: None,
                raw_json: None,
            },
        ];

        let segments = reconstruct_segments(&events);
        assert_eq!(segments.len(), 3);
        // 1. Capped active segment
        assert_eq!(segments[0].activity_type, ActivityType::Active);
        assert_eq!(segments[0].start_ms, 10_000);
        assert_eq!(segments[0].end_ms, 10_000 + HEARTBEAT_GRACE_MS);

        // 2. Unknown segment
        assert_eq!(segments[1].activity_type, ActivityType::Unknown);
        assert_eq!(segments[1].start_ms, 10_000 + HEARTBEAT_GRACE_MS);
        assert_eq!(segments[1].end_ms, 600_000);

        // 3. Resumed active segment
        assert_eq!(segments[2].activity_type, ActivityType::Active);
        assert_eq!(segments[2].start_ms, 600_000);
    }
}
