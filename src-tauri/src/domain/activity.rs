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

/// A fixed 3-minute aggregated bucket of user activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBlock {
    pub id: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub dominant_app: String,
    pub dominant_title: String,
    pub dominant_url: Option<String>,
    pub classification: Option<Classification>,
    pub category: Option<String>,
    pub confidence: Option<f64>,
    pub classified_by: Option<String>,
    pub user_override: Option<Classification>,
}
