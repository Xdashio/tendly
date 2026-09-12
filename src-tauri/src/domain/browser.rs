use serde::{Deserialize, Serialize};
use std::fmt;

/// Identifies a known browser application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BrowserType {
    Firefox,
    Chrome,
    Chromium,
    Brave,
    Vivaldi,
    Edge,
}

impl BrowserType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BrowserType::Firefox => "firefox",
            BrowserType::Chrome => "chrome",
            BrowserType::Chromium => "chromium",
            BrowserType::Brave => "brave",
            BrowserType::Vivaldi => "vivaldi",
            BrowserType::Edge => "edge",
        }
    }

    /// Returns the suffix pattern(s) that this browser appends to window titles.
    pub fn title_suffixes(&self) -> &[&'static str] {
        match self {
            BrowserType::Firefox => &[
                " \u{2014} Mozilla Firefox",
                " \u{2014} Mozilla Firefox Private Browsing",
                " - Mozilla Firefox",
            ],
            BrowserType::Chrome => &[" - Google Chrome", " - Google Chrome (Incognito)"],
            BrowserType::Chromium => &[" - Chromium", " - Chromium (Incognito)"],
            BrowserType::Brave => &[" - Brave", " - Brave (Private)"],
            BrowserType::Vivaldi => &[" - Vivaldi"],
            BrowserType::Edge => &[" - Microsoft Edge", " - Microsoft Edge (InPrivate)"],
        }
    }
}

impl fmt::Display for BrowserType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for BrowserType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "firefox" => Ok(BrowserType::Firefox),
            "chrome" => Ok(BrowserType::Chrome),
            "chromium" => Ok(BrowserType::Chromium),
            "brave" => Ok(BrowserType::Brave),
            "vivaldi" => Ok(BrowserType::Vivaldi),
            "edge" => Ok(BrowserType::Edge),
            _ => Err(()),
        }
    }
}

/// Structured browser context enrichment for activity observations.
///
/// All fields except `browser` are optional. A valid `BrowserContext` may contain
/// only the browser type with all other fields as `None`. Failure to capture
/// any enrichment field must never invalidate the underlying activity observation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserContext {
    /// Which browser is active.
    pub browser: BrowserType,
    /// Extracted page title (without browser suffix), if available.
    pub page_title: Option<String>,
    /// Full normalized URL, if available (None for title-based capture).
    pub url: Option<String>,
    /// Extracted or inferred domain, if available.
    pub domain: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_type_roundtrip() {
        let types = [
            BrowserType::Firefox,
            BrowserType::Chrome,
            BrowserType::Chromium,
            BrowserType::Brave,
            BrowserType::Vivaldi,
            BrowserType::Edge,
        ];
        for bt in types {
            let s = bt.as_str();
            let parsed: BrowserType = s.parse().expect("Must parse");
            assert_eq!(bt, parsed);
        }
    }

    #[test]
    fn test_browser_type_serde() {
        let ctx = BrowserContext {
            browser: BrowserType::Firefox,
            page_title: Some("GitHub".to_string()),
            url: None,
            domain: None,
        };
        let json = serde_json::to_string(&ctx).expect("Must serialize");
        let deserialized: BrowserContext = serde_json::from_str(&json).expect("Must deserialize");
        assert_eq!(ctx, deserialized);
    }

    #[test]
    fn test_browser_context_all_none() {
        let ctx = BrowserContext {
            browser: BrowserType::Firefox,
            page_title: None,
            url: None,
            domain: None,
        };
        assert_eq!(ctx.browser, BrowserType::Firefox);
        assert!(ctx.page_title.is_none());
        assert!(ctx.url.is_none());
        assert!(ctx.domain.is_none());
    }

    #[test]
    fn test_browser_type_display() {
        assert_eq!(format!("{}", BrowserType::Firefox), "firefox");
        assert_eq!(format!("{}", BrowserType::Chrome), "chrome");
    }

    #[test]
    fn test_title_suffixes_not_empty() {
        let types = [
            BrowserType::Firefox,
            BrowserType::Chrome,
            BrowserType::Chromium,
            BrowserType::Brave,
            BrowserType::Vivaldi,
            BrowserType::Edge,
        ];
        for bt in types {
            assert!(
                !bt.title_suffixes().is_empty(),
                "{} must have at least one title suffix",
                bt
            );
        }
    }
}
