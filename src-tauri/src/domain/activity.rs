use crate::domain::classification::Classification;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RawEventSource {
    X11,
    Wayland,
    Windows,
    Macos,
    Browser,
    Afk,
}

impl RawEventSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            RawEventSource::X11 => "x11",
            RawEventSource::Wayland => "wayland",
            RawEventSource::Windows => "windows",
            RawEventSource::Macos => "macos",
            RawEventSource::Browser => "browser",
            RawEventSource::Afk => "afk",
        }
    }
}

impl std::str::FromStr for RawEventSource {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x11" => Ok(RawEventSource::X11),
            "wayland" => Ok(RawEventSource::Wayland),
            "windows" => Ok(RawEventSource::Windows),
            "macos" => Ok(RawEventSource::Macos),
            "browser" => Ok(RawEventSource::Browser),
            "afk" => Ok(RawEventSource::Afk),
            _ => Err(()),
        }
    }
}

/// An immutable, atomic observation from an activity watcher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawEvent {
    pub id: String,
    pub source: RawEventSource,
    pub timestamp_ms: i64,
    pub app: String,
    pub title: String,
    pub url: Option<String>,
    pub idle_ms: Option<i64>,
    pub raw_json: Option<String>,
}

impl RawEvent {
    pub fn new(
        source: RawEventSource,
        timestamp_ms: i64,
        app: String,
        title: String,
        url: Option<String>,
        idle_ms: Option<i64>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source,
            timestamp_ms,
            app,
            title,
            url,
            idle_ms,
            raw_json: None,
        }
    }
}

/// The fundamental category of user presence during a time interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActivityType {
    Active,
    Afk,
    Unknown,
}

impl ActivityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActivityType::Active => "active",
            ActivityType::Afk => "afk",
            ActivityType::Unknown => "unknown",
        }
    }
}

impl std::str::FromStr for ActivityType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(ActivityType::Active),
            "afk" => Ok(ActivityType::Afk),
            "unknown" => Ok(ActivityType::Unknown),
            _ => Err(()),
        }
    }
}

/// A contiguous reconstructed run of homogeneous activity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivitySegment {
    pub start_ms: i64,
    pub end_ms: i64,
    pub app: String,
    pub title: String,
    pub activity_type: ActivityType,
    pub source: RawEventSource,
    pub event_count: usize,
    #[serde(default)]
    pub browser_context: Option<crate::domain::BrowserContext>,
    #[serde(default)]
    pub category: crate::domain::ActivityCategory,
    #[serde(default)]
    pub classification: Option<crate::domain::ClassificationResult>,
}

impl ActivitySegment {
    pub fn duration_ms(&self) -> i64 {
        (self.end_ms - self.start_ms).max(0)
    }
}

/// A standardized 3-minute aggregated bucket of user activity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeBlock {
    pub id: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub duration_ms: i64,
    pub activity_type: ActivityType,
    pub dominant_app: String,
    pub dominant_title: String,
    pub dominant_url: Option<String>,
    pub classification: Option<Classification>,
    pub category: Option<String>,
    pub confidence: Option<f64>,
    pub classified_by: Option<String>,
    pub user_override: Option<Classification>,
}

/// A summary of duration spent in an application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppDurationSummary {
    pub app: String,
    pub duration_ms: i64,
}

/// A summary of duration spent in an activity category.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryDurationSummary {
    pub category: crate::domain::ActivityCategory,
    pub duration_ms: i64,
}

/// A user-facing contiguous activity session coalesced from adjacent compatible TimeBlocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivitySession {
    pub id: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub duration_ms: i64,
    pub dominant_app: String,
    pub dominant_title: String,
    pub activity_type: ActivityType,
    pub block_count: usize,
    pub time_blocks: Vec<TimeBlock>,
    pub has_secondary_activity: bool,
    pub secondary_apps: Vec<AppDurationSummary>,
    #[serde(default)]
    pub dominant_category: Option<crate::domain::ActivityCategory>,
    #[serde(default)]
    pub category_breakdown: Vec<CategoryDurationSummary>,
}

/// Complete detailed drill-down data for a single user session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionDetails {
    pub session: ActivitySession,
    pub segments: Vec<ActivitySegment>,
    pub app_breakdown: Vec<AppDurationSummary>,
    #[serde(default)]
    pub category_breakdown: Vec<CategoryDurationSummary>,
}

/// Chronological user-facing activity timeline for a discrete date window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyTimeline {
    pub day_start_ms: i64,
    pub day_end_ms: i64,
    pub sessions: Vec<ActivitySession>,
    pub total_active_ms: i64,
    pub total_afk_ms: i64,
    pub total_unknown_ms: i64,
    pub block_count: usize,
}
