//! Classification module: deterministic rules engine, matcher, and default rulesets.

pub mod default_rules;
pub mod engine;
pub mod matcher;

pub use default_rules::{get_default_rules, RuleDefinition};
pub use engine::RuleEngine;
pub use matcher::{matches_app, matches_domain, matches_title};
