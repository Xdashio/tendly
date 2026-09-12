//! Deterministic rules engine and precedence resolver.
//!
//! # Precedence Hierarchy
//! When classifying observed activity:
//! 1. AFK check: If activity_type is Afk -> Unknown (SystemFallback).
//! 2. Active rule evaluation (sorted deterministically):
//!    a. Rule source: User rules take precedence over Default rules.
//!    b. Rule priority (descending integer).
//!    c. Specificity ranking (descending): AppAndDomain (5) > AppAndTitle (4) > Domain (3) > TitleContains (2) > App (1).
//!    d. Pattern length (descending: longer pattern wins).
//!    e. Rule ID (ascending alphanumeric deterministic tie-breaker).
//! 3. Browser Fallback: If active app is a recognized browser -> Browsing (SystemFallback).
//! 4. Generic Fallback: Any other unrecognized app -> Unknown (Unclassified).

use crate::classification::default_rules::{get_default_rules, RuleDefinition};
use crate::classification::matcher;
use crate::domain::activity::ActivityType;
use crate::domain::browser::BrowserContext;
use crate::domain::classification::{
    ActivityCategory, ClassificationResult, ClassificationSource, MatchField, RuleSource,
};

/// Deterministic in-memory rules engine.
#[derive(Debug, Clone)]
pub struct RuleEngine {
    rules: Vec<RuleDefinition>,
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new(get_default_rules())
    }
}

impl RuleEngine {
    /// Creates a new `RuleEngine` from the provided rules, sorted into canonical precedence order.
    pub fn new(mut rules: Vec<RuleDefinition>) -> Self {
        Self::sort_rules(&mut rules);
        Self { rules }
    }

    /// Creates a new `RuleEngine` with custom rules, sorted into canonical precedence order.
    pub fn with_rules(rules: Vec<RuleDefinition>) -> Self {
        Self::new(rules)
    }

    /// Sorts rules deterministically according to Tendly's explicit precedence hierarchy:
    /// 1. Source (User > Default)
    /// 2. Priority (descending)
    /// 3. Specificity (AppAndDomain > AppAndTitle > Domain > TitleContains > App)
    /// 4. Pattern length (descending)
    /// 5. Rule ID (ascending alphanumeric tie-breaker)
    fn sort_rules(rules: &mut [RuleDefinition]) {
        rules.sort_by(|a, b| {
            // 1. User rules precede Default rules
            let source_order_a = match a.source {
                RuleSource::User => 2,
                RuleSource::Default => 1,
            };
            let source_order_b = match b.source {
                RuleSource::User => 2,
                RuleSource::Default => 1,
            };

            source_order_b
                .cmp(&source_order_a)
                // 2. Priority (descending)
                .then_with(|| b.priority.cmp(&a.priority))
                // 3. Match field specificity (descending)
                .then_with(|| {
                    b.match_field
                        .specificity()
                        .cmp(&a.match_field.specificity())
                })
                // 4. Pattern length (descending)
                .then_with(|| b.pattern.len().cmp(&a.pattern.len()))
                // 5. Deterministic tie-breaker: rule id (ascending)
                .then_with(|| a.id.cmp(&b.id))
        });
    }

    /// Classifies an observed activity with full explainability.
    pub fn classify(
        &self,
        app: &str,
        title: &str,
        activity_type: ActivityType,
        browser_ctx: Option<&BrowserContext>,
    ) -> ClassificationResult {
        // Step 1: AFK check - idle periods are always Unknown
        if activity_type == ActivityType::Afk {
            return ClassificationResult {
                category: ActivityCategory::Unknown,
                source: ClassificationSource::SystemFallback,
                rule_id: None,
                matched_field: None,
                pattern: None,
                explanation: "User idle / Away from keyboard".to_string(),
            };
        }

        // Step 2: Evaluate sorted rules in order
        for rule in &self.rules {
            match rule.match_field {
                MatchField::AppAndDomain => {
                    if let Some((rule_app, rule_domain)) = rule.pattern.split_once('|') {
                        if matcher::matches_app(app, rule_app) {
                            if let Some(domain) = browser_ctx.and_then(|c| c.domain.as_deref()) {
                                if matcher::matches_domain(domain, rule_domain) {
                                    return ClassificationResult {
                                        category: rule.category,
                                        source: ClassificationSource::Rule,
                                        rule_id: Some(rule.id.clone()),
                                        matched_field: Some(MatchField::AppAndDomain),
                                        pattern: Some(rule.pattern.clone()),
                                        explanation: format!(
                                            "Application '{}' and domain '{}' matched rule '{}'",
                                            app, domain, rule.id
                                        ),
                                    };
                                }
                            }
                        }
                    }
                }
                MatchField::AppAndTitle => {
                    if let Some((rule_app, rule_title)) = rule.pattern.split_once('|') {
                        if matcher::matches_app(app, rule_app) {
                            let candidate = browser_ctx
                                .and_then(|c| c.page_title.as_deref())
                                .unwrap_or(title);
                            if matcher::matches_title(candidate, rule_title) {
                                return ClassificationResult {
                                    category: rule.category,
                                    source: ClassificationSource::Rule,
                                    rule_id: Some(rule.id.clone()),
                                    matched_field: Some(MatchField::AppAndTitle),
                                    pattern: Some(rule.pattern.clone()),
                                    explanation: format!(
                                        "Application '{}' and title '{}' matched rule '{}'",
                                        app, candidate, rule.id
                                    ),
                                };
                            }
                        }
                    }
                }
                MatchField::Domain => {
                    if let Some(domain) = browser_ctx.and_then(|c| c.domain.as_deref()) {
                        if matcher::matches_domain(domain, &rule.pattern) {
                            return ClassificationResult {
                                category: rule.category,
                                source: ClassificationSource::Rule,
                                rule_id: Some(rule.id.clone()),
                                matched_field: Some(MatchField::Domain),
                                pattern: Some(rule.pattern.clone()),
                                explanation: format!(
                                    "Browser domain '{}' matched rule '{}'",
                                    domain, rule.id
                                ),
                            };
                        }
                    }
                }
                MatchField::App => {
                    if matcher::matches_app(app, &rule.pattern) {
                        return ClassificationResult {
                            category: rule.category,
                            source: ClassificationSource::Rule,
                            rule_id: Some(rule.id.clone()),
                            matched_field: Some(MatchField::App),
                            pattern: Some(rule.pattern.clone()),
                            explanation: format!(
                                "Application '{}' matched rule '{}'",
                                app, rule.id
                            ),
                        };
                    }
                }
                MatchField::TitleContains => {
                    let candidate = browser_ctx
                        .and_then(|c| c.page_title.as_deref())
                        .unwrap_or(title);
                    if matcher::matches_title(candidate, &rule.pattern) {
                        return ClassificationResult {
                            category: rule.category,
                            source: ClassificationSource::Rule,
                            rule_id: Some(rule.id.clone()),
                            matched_field: Some(MatchField::TitleContains),
                            pattern: Some(rule.pattern.clone()),
                            explanation: format!(
                                "Title '{}' matched rule '{}'",
                                candidate, rule.id
                            ),
                        };
                    }
                }
            }
        }

        // Step 3: Browser fallback
        if crate::capture::browser_context::detect_browser(app).is_some() {
            return ClassificationResult {
                category: ActivityCategory::Browsing,
                source: ClassificationSource::SystemFallback,
                rule_id: None,
                matched_field: None,
                pattern: None,
                explanation: format!("Web browser '{}' without specific domain rule", app),
            };
        }

        // Step 4: Generic fallback
        ClassificationResult {
            category: ActivityCategory::Unknown,
            source: ClassificationSource::Unclassified,
            rule_id: None,
            matched_field: None,
            pattern: None,
            explanation: "No classification rule matched".to_string(),
        }
    }

    /// Read-only slice of loaded rules in precedence order.
    pub fn rules(&self) -> &[RuleDefinition] {
        &self.rules
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::browser::BrowserType;

    #[test]
    fn test_afk_classification() {
        let engine = RuleEngine::default();
        let res = engine.classify("code", "main.rs", ActivityType::Afk, None);
        assert_eq!(res.category, ActivityCategory::Unknown);
        assert_eq!(res.source, ClassificationSource::SystemFallback);
    }

    #[test]
    fn test_app_classification_code() {
        let engine = RuleEngine::default();
        let res = engine.classify("code", "src/main.rs", ActivityType::Active, None);
        assert_eq!(res.category, ActivityCategory::Development);
        assert_eq!(res.source, ClassificationSource::Rule);
        assert_eq!(res.matched_field, Some(MatchField::App));
    }

    #[test]
    fn test_browser_domain_precedence_over_browser_app() {
        let engine = RuleEngine::default();
        let ctx = BrowserContext {
            browser: BrowserType::Firefox,
            page_title: Some("Pull Request #4 - tendly".to_string()),
            url: Some("https://github.com/xdashio/tendly".to_string()),
            domain: Some("github.com".to_string()),
        };

        // Firefox + github.com -> Development
        let res = engine.classify("firefox", "GitHub", ActivityType::Active, Some(&ctx));
        assert_eq!(res.category, ActivityCategory::Development);
        assert_eq!(res.matched_field, Some(MatchField::Domain));
    }

    #[test]
    fn test_browser_fallback_when_no_domain_matches() {
        let engine = RuleEngine::default();
        let ctx = BrowserContext {
            browser: BrowserType::Firefox,
            page_title: Some("Some Random Blog".to_string()),
            url: Some("https://example.com".to_string()),
            domain: Some("example.com".to_string()),
        };

        let res = engine.classify("firefox", "Blog", ActivityType::Active, Some(&ctx));
        assert_eq!(res.category, ActivityCategory::Browsing);
        assert_eq!(res.source, ClassificationSource::SystemFallback);
    }

    #[test]
    fn test_generic_fallback_unknown() {
        let engine = RuleEngine::default();
        let res = engine.classify("unrecognized_app_xyz", "title", ActivityType::Active, None);
        assert_eq!(res.category, ActivityCategory::Unknown);
        assert_eq!(res.source, ClassificationSource::Unclassified);
    }

    #[test]
    fn test_user_rule_precedence_over_default() {
        let mut rules = get_default_rules();
        // Add a user rule that overrides youtube.com from Entertainment to Research
        rules.push(RuleDefinition {
            id: "user:domain:youtube".to_string(),
            priority: 10,
            match_field: MatchField::Domain,
            pattern: "youtube.com".to_string(),
            category: ActivityCategory::Research,
            source: RuleSource::User,
        });

        let engine = RuleEngine::new(rules);
        let ctx = BrowserContext {
            browser: BrowserType::Firefox,
            page_title: Some("Lecture on Quantum Mechanics - YouTube".to_string()),
            url: Some("https://youtube.com/watch?v=123".to_string()),
            domain: Some("youtube.com".to_string()),
        };

        let res = engine.classify("firefox", "YouTube", ActivityType::Active, Some(&ctx));
        assert_eq!(res.category, ActivityCategory::Research);
        assert_eq!(res.rule_id, Some("user:domain:youtube".to_string()));
    }

    #[test]
    fn test_conflicting_rules_deterministic_tie_breaker() {
        // Two rules with identical priority, match field, and pattern length
        let rule_a = RuleDefinition {
            id: "rule_a".to_string(),
            priority: 0,
            match_field: MatchField::Domain,
            pattern: "test.com".to_string(),
            category: ActivityCategory::Development,
            source: RuleSource::Default,
        };
        let rule_b = RuleDefinition {
            id: "rule_b".to_string(),
            priority: 0,
            match_field: MatchField::Domain,
            pattern: "test.com".to_string(),
            category: ActivityCategory::Entertainment,
            source: RuleSource::Default,
        };

        // In both insertion orders, rule_a must win deterministically by rule ID (rule_a < rule_b)
        let engine1 = RuleEngine::new(vec![rule_a.clone(), rule_b.clone()]);
        let engine2 = RuleEngine::new(vec![rule_b.clone(), rule_a.clone()]);

        let ctx = BrowserContext {
            browser: BrowserType::Firefox,
            page_title: None,
            url: None,
            domain: Some("test.com".to_string()),
        };

        let res1 = engine1.classify("firefox", "", ActivityType::Active, Some(&ctx));
        let res2 = engine2.classify("firefox", "", ActivityType::Active, Some(&ctx));

        assert_eq!(res1.category, res2.category);
        assert_eq!(res1.category, ActivityCategory::Development);
        assert_eq!(res1.rule_id, Some("rule_a".to_string()));
    }

    // ── Equal-priority specificity conflict tests ──────────────────────

    #[test]
    fn test_equal_priority_domain_vs_app() {
        // At equal priority (0), Domain (specificity 3) MUST beat App (specificity 1)
        let app_rule = RuleDefinition {
            id: "rule_z_app".to_string(), // Even with later ID or earlier ID
            priority: 0,
            match_field: MatchField::App,
            pattern: "firefox".to_string(),
            category: ActivityCategory::Browsing,
            source: RuleSource::Default,
        };
        let domain_rule = RuleDefinition {
            id: "rule_a_domain".to_string(),
            priority: 0,
            match_field: MatchField::Domain,
            pattern: "github.com".to_string(),
            category: ActivityCategory::Development,
            source: RuleSource::Default,
        };

        let engine = RuleEngine::new(vec![app_rule, domain_rule]);
        let ctx = BrowserContext {
            browser: BrowserType::Firefox,
            page_title: Some("tendly".to_string()),
            url: Some("https://github.com".to_string()),
            domain: Some("github.com".to_string()),
        };

        let res = engine.classify("firefox", "tendly", ActivityType::Active, Some(&ctx));
        assert_eq!(res.category, ActivityCategory::Development);
        assert_eq!(res.matched_field, Some(MatchField::Domain));
    }

    #[test]
    fn test_equal_priority_title_vs_app() {
        // At equal priority (0), TitleContains (specificity 2) MUST beat App (specificity 1)
        let app_rule = RuleDefinition {
            id: "rule_app_code".to_string(),
            priority: 0,
            match_field: MatchField::App,
            pattern: "code".to_string(),
            category: ActivityCategory::Development,
            source: RuleSource::Default,
        };
        let title_rule = RuleDefinition {
            id: "rule_title_meet".to_string(),
            priority: 0,
            match_field: MatchField::TitleContains,
            pattern: "zoom meeting".to_string(),
            category: ActivityCategory::Communication,
            source: RuleSource::Default,
        };

        let engine = RuleEngine::new(vec![app_rule, title_rule]);
        let res = engine.classify("code", "Zoom Meeting with Team", ActivityType::Active, None);
        assert_eq!(res.category, ActivityCategory::Communication);
        assert_eq!(res.matched_field, Some(MatchField::TitleContains));
    }

    #[test]
    fn test_equal_priority_app_and_domain_vs_domain() {
        // At equal priority (0), AppAndDomain (specificity 5) MUST beat Domain (specificity 3)
        let domain_rule = RuleDefinition {
            id: "rule_domain".to_string(),
            priority: 0,
            match_field: MatchField::Domain,
            pattern: "github.com".to_string(),
            category: ActivityCategory::Development,
            source: RuleSource::Default,
        };
        let app_domain_rule = RuleDefinition {
            id: "rule_app_domain".to_string(),
            priority: 0,
            match_field: MatchField::AppAndDomain,
            pattern: "firefox|github.com".to_string(),
            category: ActivityCategory::Productivity,
            source: RuleSource::Default,
        };

        let engine = RuleEngine::new(vec![domain_rule, app_domain_rule]);
        let ctx = BrowserContext {
            browser: BrowserType::Firefox,
            page_title: None,
            url: None,
            domain: Some("github.com".to_string()),
        };

        let res = engine.classify("firefox", "", ActivityType::Active, Some(&ctx));
        assert_eq!(res.category, ActivityCategory::Productivity);
        assert_eq!(res.matched_field, Some(MatchField::AppAndDomain));
    }

    #[test]
    fn test_equal_priority_app_and_title_vs_title() {
        // At equal priority (0), AppAndTitle (specificity 4) MUST beat Title (specificity 2)
        let title_rule = RuleDefinition {
            id: "rule_title".to_string(),
            priority: 0,
            match_field: MatchField::TitleContains,
            pattern: "standup".to_string(),
            category: ActivityCategory::Communication,
            source: RuleSource::Default,
        };
        let app_title_rule = RuleDefinition {
            id: "rule_app_title".to_string(),
            priority: 0,
            match_field: MatchField::AppAndTitle,
            pattern: "slack|standup".to_string(),
            category: ActivityCategory::Productivity,
            source: RuleSource::Default,
        };

        let engine = RuleEngine::new(vec![title_rule, app_title_rule]);
        let res = engine.classify("slack", "Daily Standup", ActivityType::Active, None);
        assert_eq!(res.category, ActivityCategory::Productivity);
        assert_eq!(res.matched_field, Some(MatchField::AppAndTitle));
    }

    #[test]
    fn test_classification_granularity_browsers() {
        let engine = RuleEngine::default();

        // 1. Firefox + github.com -> Development
        let ctx_gh = BrowserContext {
            browser: BrowserType::Firefox,
            page_title: Some("tendly PR".to_string()),
            url: Some("https://github.com/xdashio/tendly".to_string()),
            domain: Some("github.com".to_string()),
        };
        let res_gh = engine.classify("firefox", "tendly", ActivityType::Active, Some(&ctx_gh));
        assert_eq!(res_gh.category, ActivityCategory::Development);

        // 2. Firefox + youtube.com -> Entertainment
        let ctx_yt = BrowserContext {
            browser: BrowserType::Firefox,
            page_title: Some("Music Video - YouTube".to_string()),
            url: Some("https://youtube.com/watch?v=abc".to_string()),
            domain: Some("youtube.com".to_string()),
        };
        let res_yt = engine.classify("firefox", "YouTube", ActivityType::Active, Some(&ctx_yt));
        assert_eq!(res_yt.category, ActivityCategory::Entertainment);

        // 3. Firefox + unmatched domain -> Browsing
        let ctx_other = BrowserContext {
            browser: BrowserType::Firefox,
            page_title: Some("My Personal Blog".to_string()),
            url: Some("https://blog.example.org".to_string()),
            domain: Some("blog.example.org".to_string()),
        };
        let res_other = engine.classify("firefox", "Blog", ActivityType::Active, Some(&ctx_other));
        assert_eq!(res_other.category, ActivityCategory::Browsing);

        // 4. Chrome + github.com -> Development
        let ctx_chrome_gh = BrowserContext {
            browser: BrowserType::Chrome,
            page_title: Some("tendly".to_string()),
            url: Some("https://github.com".to_string()),
            domain: Some("github.com".to_string()),
        };
        let res_chrome_gh = engine.classify(
            "google-chrome",
            "tendly",
            ActivityType::Active,
            Some(&ctx_chrome_gh),
        );
        assert_eq!(res_chrome_gh.category, ActivityCategory::Development);
    }
}
