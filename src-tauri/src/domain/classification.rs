use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Classification {
    Focus,
    Neutral,
    Drift,
}

impl Classification {
    pub fn as_str(&self) -> &'static str {
        match self {
            Classification::Focus => "focus",
            Classification::Neutral => "neutral",
            Classification::Drift => "drift",
        }
    }
}

impl std::str::FromStr for Classification {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "focus" => Ok(Classification::Focus),
            "neutral" => Ok(Classification::Neutral),
            "drift" => Ok(Classification::Drift),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchField {
    App,
    Domain,
    TitleContains,
}

impl MatchField {
    pub fn as_str(&self) -> &'static str {
        match self {
            MatchField::App => "app",
            MatchField::Domain => "domain",
            MatchField::TitleContains => "title_contains",
        }
    }
}

impl std::str::FromStr for MatchField {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "app" => Ok(MatchField::App),
            "domain" => Ok(MatchField::Domain),
            "title_contains" => Ok(MatchField::TitleContains),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleSource {
    User,
    Default,
}

impl RuleSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            RuleSource::User => "user",
            RuleSource::Default => "default",
        }
    }
}

impl std::str::FromStr for RuleSource {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(RuleSource::User),
            "default" => Ok(RuleSource::Default),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationRule {
    pub id: String,
    pub priority: i32,
    pub match_field: MatchField,
    pub pattern: String,
    pub classification: Classification,
    pub category: Option<String>,
    pub source: RuleSource,
    pub created_at: i64,
}

/// Standardized controlled taxonomy for activity classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ActivityCategory {
    Development,
    Communication,
    Research,
    Productivity,
    Design,
    Entertainment,
    System,
    Browsing,
    #[default]
    Unknown,
}

impl ActivityCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActivityCategory::Development => "development",
            ActivityCategory::Communication => "communication",
            ActivityCategory::Research => "research",
            ActivityCategory::Productivity => "productivity",
            ActivityCategory::Design => "design",
            ActivityCategory::Entertainment => "entertainment",
            ActivityCategory::System => "system",
            ActivityCategory::Browsing => "browsing",
            ActivityCategory::Unknown => "unknown",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ActivityCategory::Development => "Development",
            ActivityCategory::Communication => "Communication",
            ActivityCategory::Research => "Research",
            ActivityCategory::Productivity => "Productivity",
            ActivityCategory::Design => "Design",
            ActivityCategory::Entertainment => "Entertainment",
            ActivityCategory::System => "System",
            ActivityCategory::Browsing => "Browsing",
            ActivityCategory::Unknown => "Unknown",
        }
    }
}

impl std::fmt::Display for ActivityCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display_name())
    }
}

impl std::str::FromStr for ActivityCategory {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "development" => Ok(ActivityCategory::Development),
            "communication" => Ok(ActivityCategory::Communication),
            "research" => Ok(ActivityCategory::Research),
            "productivity" => Ok(ActivityCategory::Productivity),
            "design" => Ok(ActivityCategory::Design),
            "entertainment" => Ok(ActivityCategory::Entertainment),
            "system" => Ok(ActivityCategory::System),
            "browsing" => Ok(ActivityCategory::Browsing),
            "unknown" => Ok(ActivityCategory::Unknown),
            _ => Err(()),
        }
    }
}

/// Source or justification mechanism that assigned a classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassificationSource {
    Rule,
    SystemFallback,
    Unclassified,
}

impl ClassificationSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClassificationSource::Rule => "rule",
            ClassificationSource::SystemFallback => "system_fallback",
            ClassificationSource::Unclassified => "unclassified",
        }
    }
}

/// Full explainability record for a classified activity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub category: ActivityCategory,
    pub source: ClassificationSource,
    pub rule_id: Option<String>,
    pub matched_field: Option<MatchField>,
    pub pattern: Option<String>,
    pub explanation: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_category_roundtrip() {
        let categories = [
            ActivityCategory::Development,
            ActivityCategory::Communication,
            ActivityCategory::Research,
            ActivityCategory::Productivity,
            ActivityCategory::Design,
            ActivityCategory::Entertainment,
            ActivityCategory::System,
            ActivityCategory::Browsing,
            ActivityCategory::Unknown,
        ];

        for cat in categories {
            let s = cat.as_str();
            let parsed: ActivityCategory = s.parse().expect("Must parse");
            assert_eq!(cat, parsed);
            assert_eq!(format!("{}", cat), cat.display_name());
        }
    }

    #[test]
    fn test_activity_category_serde() {
        let cat = ActivityCategory::Development;
        let json = serde_json::to_string(&cat).unwrap();
        assert_eq!(json, "\"development\"");
        let deserialized: ActivityCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(cat, deserialized);
    }

    #[test]
    fn test_classification_result_explainability() {
        let res = ClassificationResult {
            category: ActivityCategory::Development,
            source: ClassificationSource::Rule,
            rule_id: Some("rule:domain:github.com".to_string()),
            matched_field: Some(MatchField::Domain),
            pattern: Some("github.com".to_string()),
            explanation: "Browser domain 'github.com' matched development rule".to_string(),
        };

        let json = serde_json::to_string(&res).unwrap();
        let deserialized: ClassificationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(res, deserialized);
        assert_eq!(res.category, ActivityCategory::Development);
        assert_eq!(res.source, ClassificationSource::Rule);
        assert!(res.rule_id.is_some());
    }
}
