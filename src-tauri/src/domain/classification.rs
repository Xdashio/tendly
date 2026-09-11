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
