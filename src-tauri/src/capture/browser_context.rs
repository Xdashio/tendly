//! Browser context enrichment: detection, title parsing, URL normalization, and domain extraction.
//!
//! This module provides the enrichment layer that sits between raw window capture
//! and the activity pipeline. It identifies browser applications, extracts structured
//! page titles from window titles, and provides URL normalization utilities for
//! future browser extension integration.
//!
//! # Privacy
//!
//! - No URLs are sent to remote services.
//! - Credentials embedded in URLs are always stripped.
//! - Tracking parameters (utm_*, fbclid, gclid) are removed during normalization.
//! - Full URLs are never logged in normal operation.

use crate::domain::browser::{BrowserContext, BrowserType};

/// Known WM_CLASS / process name patterns mapped to browser types.
/// Matching is case-insensitive.
const BROWSER_PATTERNS: &[(&str, BrowserType)] = &[
    // Firefox
    ("firefox", BrowserType::Firefox),
    ("firefox-esr", BrowserType::Firefox),
    ("navigator", BrowserType::Firefox),
    ("firefox-default", BrowserType::Firefox),
    // Chrome
    ("google-chrome", BrowserType::Chrome),
    ("google-chrome-stable", BrowserType::Chrome),
    // Chromium
    ("chromium", BrowserType::Chromium),
    ("chromium-browser", BrowserType::Chromium),
    // Brave
    ("brave-browser", BrowserType::Brave),
    ("brave", BrowserType::Brave),
    // Vivaldi
    ("vivaldi", BrowserType::Vivaldi),
    ("vivaldi-stable", BrowserType::Vivaldi),
    // Edge
    ("microsoft-edge", BrowserType::Edge),
    ("microsoft-edge-stable", BrowserType::Edge),
    ("msedge", BrowserType::Edge),
];

/// Tracking query parameters that are removed during URL normalization.
const TRACKING_PARAMS: &[&str] = &[
    "utm_source",
    "utm_medium",
    "utm_campaign",
    "utm_term",
    "utm_content",
    "utm_id",
    "fbclid",
    "gclid",
    "ref",
];

/// Identifies whether the given application name corresponds to a known browser.
///
/// Returns the `BrowserType` if recognized, `None` otherwise.
/// Matching is case-insensitive.
pub fn detect_browser(app_name: &str) -> Option<BrowserType> {
    let lower = app_name.to_lowercase();
    for &(pattern, browser_type) in BROWSER_PATTERNS {
        if lower == pattern {
            return Some(browser_type);
        }
    }
    None
}

/// Extracts the page title from a browser window title by stripping the browser suffix.
///
/// Browser window titles typically follow the pattern:
/// - Firefox: `"Page Title \u{2014} Mozilla Firefox"`
/// - Chrome: `"Page Title - Google Chrome"`
///
/// Returns `None` if the title is empty, consists only of the browser name,
/// or no known suffix is found (in which case the caller can use the full title).
pub fn extract_page_title(window_title: &str, browser: &BrowserType) -> Option<String> {
    let trimmed = window_title.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Try each known suffix for this browser
    for &suffix in browser.title_suffixes() {
        let clean_suffix = suffix.trim_start();
        if trimmed == clean_suffix {
            return None;
        }
        if let Some(stripped) = trimmed
            .strip_suffix(suffix)
            .or_else(|| trimmed.strip_suffix(clean_suffix))
        {
            let page_title = stripped.trim();
            if page_title.is_empty() {
                return None;
            }
            return Some(page_title.to_string());
        }
    }

    // No known suffix found. If the title IS the browser name itself, return None.
    let browser_only_names = match browser {
        BrowserType::Firefox => &["Mozilla Firefox", "Firefox"][..],
        BrowserType::Chrome => &["Google Chrome", "Chrome"][..],
        BrowserType::Chromium => &["Chromium"][..],
        BrowserType::Brave => &["Brave"][..],
        BrowserType::Vivaldi => &["Vivaldi"][..],
        BrowserType::Edge => &["Microsoft Edge"][..],
    };

    for &name in browser_only_names {
        if trimmed.eq_ignore_ascii_case(name) {
            return None;
        }
    }

    // The title has content but no recognizable suffix.
    // Return the full title as page_title since the app is already identified as a browser.
    Some(trimmed.to_string())
}

/// Top-level enrichment function: detects browser and extracts structured context.
///
/// Returns `None` for non-browser applications.
/// Returns a `BrowserContext` with optional fields for browser applications.
/// Failure to extract any enrichment field does not invalidate the result.
pub fn enrich_browser_context(app: &str, title: &str) -> Option<BrowserContext> {
    let browser = detect_browser(app)?;
    let page_title = extract_page_title(title, &browser);

    Some(BrowserContext {
        browser,
        page_title,
        url: None,
        domain: None,
    })
}

/// Normalizes a URL according to Tendly's deterministic normalization policy.
///
/// # Normalization steps
/// 1. Parse with WHATWG URL parser
/// 2. Lowercase scheme and host
/// 3. Remove default ports (80 for http, 443 for https)
/// 4. Strip credentials (username:password@)
/// 5. Remove tracking query parameters (utm_*, fbclid, gclid, ref)
/// 6. Normalize trailing slash on root path
/// 7. Preserve non-tracking query parameters and fragment
///
/// Returns `None` if the URL cannot be parsed.
pub fn normalize_url(raw: &str) -> Option<String> {
    let mut parsed = url::Url::parse(raw).ok()?;

    // Only normalize http/https URLs
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Some(parsed.to_string());
    }

    // Strip credentials
    let _ = parsed.set_username("");
    let _ = parsed.set_password(None);

    // Remove default ports
    if let Some(port) = parsed.port() {
        if (parsed.scheme() == "http" && port == 80) || (parsed.scheme() == "https" && port == 443)
        {
            let _ = parsed.set_port(None);
        }
    }

    // Remove tracking query parameters
    let original_query: Vec<(String, String)> = parsed
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    if !original_query.is_empty() {
        let filtered: Vec<(String, String)> = original_query
            .into_iter()
            .filter(|(key, _)| {
                let lower_key = key.to_lowercase();
                !TRACKING_PARAMS.contains(&lower_key.as_str())
            })
            .collect();

        if filtered.is_empty() {
            parsed.set_query(None);
        } else {
            let query_string: String = filtered
                .iter()
                .map(|(k, v)| {
                    if v.is_empty() {
                        k.clone()
                    } else {
                        format!("{}={}", k, v)
                    }
                })
                .collect::<Vec<_>>()
                .join("&");
            parsed.set_query(Some(&query_string));
        }
    }

    Some(parsed.to_string())
}

/// Extracts a normalized domain from a URL string.
///
/// Uses proper URL parsing rather than manual string splitting.
/// Returns `None` if the URL cannot be parsed or has no host.
pub fn extract_domain(raw_url: &str) -> Option<String> {
    let parsed = url::Url::parse(raw_url).ok()?;
    parsed.host_str().map(|h| h.to_lowercase())
}

/// Strips credential components from a URL string.
///
/// Credentials embedded in URLs (e.g., `https://user:pass@example.com`)
/// are a security concern and must never be stored.
pub fn strip_credentials(raw_url: &str) -> String {
    match url::Url::parse(raw_url) {
        Ok(mut parsed) => {
            let _ = parsed.set_username("");
            let _ = parsed.set_password(None);
            parsed.to_string()
        }
        Err(_) => raw_url.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Browser detection ──────────────────────────────────────────────

    #[test]
    fn test_detect_firefox() {
        assert_eq!(detect_browser("firefox"), Some(BrowserType::Firefox));
        assert_eq!(detect_browser("Firefox"), Some(BrowserType::Firefox));
        assert_eq!(detect_browser("FIREFOX"), Some(BrowserType::Firefox));
        assert_eq!(detect_browser("firefox-esr"), Some(BrowserType::Firefox));
        assert_eq!(detect_browser("Navigator"), Some(BrowserType::Firefox));
    }

    #[test]
    fn test_detect_chrome() {
        assert_eq!(detect_browser("google-chrome"), Some(BrowserType::Chrome));
        assert_eq!(detect_browser("Google-chrome"), Some(BrowserType::Chrome));
        assert_eq!(
            detect_browser("google-chrome-stable"),
            Some(BrowserType::Chrome)
        );
    }

    #[test]
    fn test_detect_chromium() {
        assert_eq!(detect_browser("chromium"), Some(BrowserType::Chromium));
        assert_eq!(
            detect_browser("Chromium-browser"),
            Some(BrowserType::Chromium)
        );
    }

    #[test]
    fn test_detect_brave() {
        assert_eq!(detect_browser("brave-browser"), Some(BrowserType::Brave));
        assert_eq!(detect_browser("brave"), Some(BrowserType::Brave));
    }

    #[test]
    fn test_detect_vivaldi() {
        assert_eq!(detect_browser("vivaldi"), Some(BrowserType::Vivaldi));
        assert_eq!(detect_browser("vivaldi-stable"), Some(BrowserType::Vivaldi));
    }

    #[test]
    fn test_detect_edge() {
        assert_eq!(detect_browser("microsoft-edge"), Some(BrowserType::Edge));
        assert_eq!(detect_browser("msedge"), Some(BrowserType::Edge));
    }

    #[test]
    fn test_detect_non_browser() {
        assert_eq!(detect_browser("code"), None);
        assert_eq!(detect_browser("alacritty"), None);
        assert_eq!(detect_browser("slack"), None);
        assert_eq!(detect_browser(""), None);
        assert_eq!(detect_browser("unknown"), None);
    }

    // ── Page title extraction ──────────────────────────────────────────

    #[test]
    fn test_extract_page_title_firefox() {
        assert_eq!(
            extract_page_title(
                "GitHub - xdashio/tendly \u{2014} Mozilla Firefox",
                &BrowserType::Firefox
            ),
            Some("GitHub - xdashio/tendly".to_string())
        );
    }

    #[test]
    fn test_extract_page_title_firefox_private() {
        assert_eq!(
            extract_page_title(
                "Private Page \u{2014} Mozilla Firefox Private Browsing",
                &BrowserType::Firefox
            ),
            Some("Private Page".to_string())
        );
    }

    #[test]
    fn test_extract_page_title_chrome() {
        assert_eq!(
            extract_page_title("YouTube - Google Chrome", &BrowserType::Chrome),
            Some("YouTube".to_string())
        );
    }

    #[test]
    fn test_extract_page_title_chrome_incognito() {
        assert_eq!(
            extract_page_title("Page - Google Chrome (Incognito)", &BrowserType::Chrome),
            Some("Page".to_string())
        );
    }

    #[test]
    fn test_extract_page_title_chromium() {
        assert_eq!(
            extract_page_title("Docs - Chromium", &BrowserType::Chromium),
            Some("Docs".to_string())
        );
    }

    #[test]
    fn test_extract_page_title_brave() {
        assert_eq!(
            extract_page_title("Reddit - Brave", &BrowserType::Brave),
            Some("Reddit".to_string())
        );
    }

    #[test]
    fn test_extract_page_title_empty() {
        assert_eq!(extract_page_title("", &BrowserType::Firefox), None);
    }

    #[test]
    fn test_extract_page_title_browser_name_only() {
        assert_eq!(
            extract_page_title("Mozilla Firefox", &BrowserType::Firefox),
            None
        );
        assert_eq!(
            extract_page_title("Google Chrome", &BrowserType::Chrome),
            None
        );
    }

    #[test]
    fn test_extract_page_title_no_suffix_returns_full_title() {
        // When no known suffix is found, return the full title as the page title
        assert_eq!(
            extract_page_title("Some Custom Title", &BrowserType::Firefox),
            Some("Some Custom Title".to_string())
        );
    }

    #[test]
    fn test_extract_page_title_only_suffix() {
        assert_eq!(
            extract_page_title(" \u{2014} Mozilla Firefox", &BrowserType::Firefox),
            None
        );
    }

    // ── Full enrichment ────────────────────────────────────────────────

    #[test]
    fn test_enrich_browser_context_firefox() {
        let ctx = enrich_browser_context(
            "firefox",
            "GitHub - xdashio/tendly \u{2014} Mozilla Firefox",
        );
        assert!(ctx.is_some());
        let ctx = ctx.unwrap();
        assert_eq!(ctx.browser, BrowserType::Firefox);
        assert_eq!(ctx.page_title, Some("GitHub - xdashio/tendly".to_string()));
        assert!(ctx.url.is_none());
        assert!(ctx.domain.is_none());
    }

    #[test]
    fn test_enrich_browser_context_non_browser() {
        assert!(enrich_browser_context("code", "main.rs - VS Code").is_none());
    }

    #[test]
    fn test_enrich_browser_context_missing_title() {
        let ctx = enrich_browser_context("firefox", "Mozilla Firefox");
        assert!(ctx.is_some());
        let ctx = ctx.unwrap();
        assert_eq!(ctx.browser, BrowserType::Firefox);
        assert!(ctx.page_title.is_none());
    }

    // ── URL normalization ──────────────────────────────────────────────

    #[test]
    fn test_normalize_url_valid() {
        let result = normalize_url("https://github.com/user/repo");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "https://github.com/user/repo");
    }

    #[test]
    fn test_normalize_url_invalid() {
        assert!(normalize_url("not a url").is_none());
        assert!(normalize_url("").is_none());
    }

    #[test]
    fn test_normalize_url_host_casing() {
        let result = normalize_url("https://GitHub.COM/user/repo");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "https://github.com/user/repo");
    }

    #[test]
    fn test_normalize_url_default_port_removed() {
        let result = normalize_url("https://example.com:443/path");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "https://example.com/path");

        let result = normalize_url("http://example.com:80/path");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "http://example.com/path");
    }

    #[test]
    fn test_normalize_url_non_default_port_preserved() {
        let result = normalize_url("https://example.com:8443/path");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "https://example.com:8443/path");
    }

    #[test]
    fn test_normalize_url_strips_credentials() {
        let result = normalize_url("https://user:password@example.com/path");
        assert!(result.is_some());
        let normalized = result.unwrap();
        assert!(!normalized.contains("user:password"));
        assert!(!normalized.contains("user@"));
        assert!(normalized.contains("example.com"));
    }

    #[test]
    fn test_normalize_url_removes_tracking_params() {
        let result =
            normalize_url("https://example.com/page?utm_source=test&utm_medium=email&id=42");
        assert!(result.is_some());
        let normalized = result.unwrap();
        assert!(!normalized.contains("utm_source"));
        assert!(!normalized.contains("utm_medium"));
        assert!(normalized.contains("id=42"));
    }

    #[test]
    fn test_normalize_url_removes_fbclid_gclid() {
        let result = normalize_url("https://example.com/page?q=search&fbclid=abc123&gclid=xyz789");
        assert!(result.is_some());
        let normalized = result.unwrap();
        assert!(!normalized.contains("fbclid"));
        assert!(!normalized.contains("gclid"));
        assert!(normalized.contains("q=search"));
    }

    #[test]
    fn test_normalize_url_preserves_meaningful_params() {
        let result = normalize_url("https://example.com/search?q=rust+lang&page=2");
        assert!(result.is_some());
        let normalized = result.unwrap();
        assert!(normalized.contains("q=rust"));
        assert!(normalized.contains("page=2"));
    }

    #[test]
    fn test_normalize_url_all_tracking_params_removed() {
        let result = normalize_url("https://example.com/page?utm_source=x&utm_campaign=y");
        assert!(result.is_some());
        let normalized = result.unwrap();
        assert!(!normalized.contains('?'));
    }

    #[test]
    fn test_normalize_url_missing_scheme() {
        // url crate cannot parse URLs without scheme
        assert!(normalize_url("example.com/path").is_none());
    }

    #[test]
    fn test_normalize_url_with_fragment() {
        let result = normalize_url("https://example.com/page#section");
        assert!(result.is_some());
        assert!(result.unwrap().contains("#section"));
    }

    // ── Domain extraction ──────────────────────────────────────────────

    #[test]
    fn test_extract_domain_github() {
        assert_eq!(
            extract_domain("https://github.com/user/repo"),
            Some("github.com".to_string())
        );
    }

    #[test]
    fn test_extract_domain_google_docs() {
        assert_eq!(
            extract_domain("https://docs.google.com/document/d/123"),
            Some("docs.google.com".to_string())
        );
    }

    #[test]
    fn test_extract_domain_with_port() {
        assert_eq!(
            extract_domain("http://localhost:3000/app"),
            Some("localhost".to_string())
        );
    }

    #[test]
    fn test_extract_domain_invalid_url() {
        assert!(extract_domain("not a url").is_none());
        assert!(extract_domain("").is_none());
    }

    #[test]
    fn test_extract_domain_case_normalization() {
        assert_eq!(
            extract_domain("https://GitHub.COM/user/repo"),
            Some("github.com".to_string())
        );
    }

    // ── Credential stripping ───────────────────────────────────────────

    #[test]
    fn test_strip_credentials() {
        let result = strip_credentials("https://user:pass@example.com/path");
        assert!(!result.contains("user:pass"));
        assert!(!result.contains("user@"));
        assert!(result.contains("example.com"));
    }

    #[test]
    fn test_strip_credentials_no_credentials() {
        let url = "https://example.com/path";
        assert_eq!(strip_credentials(url), url);
    }

    #[test]
    fn test_strip_credentials_invalid_url() {
        let raw = "not a url";
        assert_eq!(strip_credentials(raw), raw);
    }

    // ── Context change scenarios ───────────────────────────────────────

    #[test]
    fn test_context_changes_produce_different_enrichments() {
        let ctx_a = enrich_browser_context("firefox", "GitHub \u{2014} Mozilla Firefox");
        let ctx_b = enrich_browser_context("firefox", "YouTube \u{2014} Mozilla Firefox");
        let ctx_c = enrich_browser_context("firefox", "GitHub \u{2014} Mozilla Firefox");

        assert!(ctx_a.is_some());
        assert!(ctx_b.is_some());
        assert!(ctx_c.is_some());

        let a = ctx_a.unwrap();
        let b = ctx_b.unwrap();
        let c = ctx_c.unwrap();

        assert_ne!(a.page_title, b.page_title);
        assert_eq!(a.page_title, c.page_title);
        assert_eq!(a.browser, b.browser); // Same browser
    }

    // ── Failure isolation ──────────────────────────────────────────────

    #[test]
    fn test_enrichment_never_panics_on_unusual_input() {
        // None of these should panic
        let _ = enrich_browser_context("", "");
        let _ = enrich_browser_context("firefox", "");
        let _ = enrich_browser_context("", "Some title");
        let _ = enrich_browser_context("firefox", "\0\0\0");
        let _ = enrich_browser_context("firefox", "a".repeat(10000).as_str());
        let _ = normalize_url("\0\0\0");
        let _ = normalize_url("a".repeat(10000).as_str());
        let _ = extract_domain("\0\0\0");
        let _ = strip_credentials("\0\0\0");
    }
}
