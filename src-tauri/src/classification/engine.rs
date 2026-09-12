//! Deterministic rules engine and precedence resolver.
//!
//! # Precedence Hierarchy
//! When classifying observed activity:
//! 1. AFK / System check: If activity_type is Afk -> System (SystemFallback).
//! 2. Active rule evaluation (sorted deterministically):
//!    a. Rule source: User rules take precedence over Default rules.
//!    b. Rule priority (descending integer).
//!    c. Match field specificity: Domain (3) > TitleContains (2) > App (1).
//!    d. Pattern length (descending: longer pattern wins).
//!    e. Rule ID (ascending alphanumeric tie-breaker).
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

    /// Sorts rules deterministically according to Tendly's explicit precedence hierarchy.
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
                // 3. Match field specificity: Domain > TitleContains > App
                .then_with(|| {
                    let spec = |f: MatchField| match f {
                        MatchField::Domain => 3,
                        MatchField::TitleContains => 2,
                        MatchField::App => 1,
                    };
                    spec(b.match_field).cmp(&spec(a.match_field))
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
        // Step 1: AFK check
        if activity_type == ActivityType::Afk {
            return ClassificationResult {
                category: ActivityCategory::System,
                source: ClassificationSource::SystemFallback,
                rule_id: None,
                matched_field: None,
                pattern: None,
                explanation: "System activity (afk or idle)".to_string(),
            };
        }

        // Step 2: Evaluate sorted rules in order
        for rule in &self.rules {
            match rule.match_field {
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
        assert_eq!(res.category, ActivityCategory::System);
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
        // Two rules with identical priority and match field
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

        // In both insertion orders, rule_a must win deterministically by rule ID
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
}
