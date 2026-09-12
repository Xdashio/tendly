//! Built-in default classification rules for Tendly.
//!
//! Designed for the primary Software Developer persona with a controlled,
//! objective, non-judgmental 9-category taxonomy.

use crate::domain::classification::{ActivityCategory, MatchField, RuleSource};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleDefinition {
    pub id: String,
    pub priority: i32,
    pub match_field: MatchField,
    pub pattern: String,
    pub category: ActivityCategory,
    pub source: RuleSource,
}

/// Generates the curated, default developer-oriented ruleset.
pub fn get_default_rules() -> Vec<RuleDefinition> {
    let mut rules = Vec::new();

    // ── Development ────────────────────────────────────────────────────
    let dev_apps = [
        "code",
        "code-insiders",
        "vscodium",
        "cursor",
        "alacritty",
        "kitty",
        "wezterm",
        "gnome-terminal",
        "konsole",
        "xterm",
    ];
    for app in dev_apps {
        rules.push(RuleDefinition {
            id: format!("default:app:{}", app),
            priority: 0,
            match_field: MatchField::App,
            pattern: app.to_string(),
            category: ActivityCategory::Development,
            source: RuleSource::Default,
        });
    }

    let dev_domains = [
        "github.com",
        "gitlab.com",
        "stackoverflow.com",
        "crates.io",
        "docs.rs",
    ];
    for domain in dev_domains {
        rules.push(RuleDefinition {
            id: format!("default:domain:{}", domain),
            priority: 0,
            match_field: MatchField::Domain,
            pattern: domain.to_string(),
            category: ActivityCategory::Development,
            source: RuleSource::Default,
        });
    }

    // ── Communication ──────────────────────────────────────────────────
    let comm_apps = [
        "slack",
        "discord",
        "telegram-desktop",
        "signal-desktop",
        "element",
        "zoom",
        "thunderbird",
    ];
    for app in comm_apps {
        rules.push(RuleDefinition {
            id: format!("default:app:{}", app),
            priority: 0,
            match_field: MatchField::App,
            pattern: app.to_string(),
            category: ActivityCategory::Communication,
            source: RuleSource::Default,
        });
    }

    let comm_domains = [
        "slack.com",
        "discord.com",
        "web.telegram.org",
        "meet.google.com",
        "zoom.us",
    ];
    for domain in comm_domains {
        rules.push(RuleDefinition {
            id: format!("default:domain:{}", domain),
            priority: 0,
            match_field: MatchField::Domain,
            pattern: domain.to_string(),
            category: ActivityCategory::Communication,
            source: RuleSource::Default,
        });
    }

    // ── Research ───────────────────────────────────────────────────────
    let research_domains = [
        "wikipedia.org",
        "arxiv.org",
        "developer.mozilla.org",
        "devdocs.io",
        "rust-lang.org",
    ];
    for domain in research_domains {
        rules.push(RuleDefinition {
            id: format!("default:domain:{}", domain),
            priority: 0,
            match_field: MatchField::Domain,
            pattern: domain.to_string(),
            category: ActivityCategory::Research,
            source: RuleSource::Default,
        });
    }

    // ── Productivity ───────────────────────────────────────────────────
    let prod_apps = ["obsidian", "notion", "libreoffice", "xournalpp"];
    for app in prod_apps {
        rules.push(RuleDefinition {
            id: format!("default:app:{}", app),
            priority: 0,
            match_field: MatchField::App,
            pattern: app.to_string(),
            category: ActivityCategory::Productivity,
            source: RuleSource::Default,
        });
    }

    let prod_domains = [
        "docs.google.com",
        "notion.so",
        "linear.app",
        "jira.atlassian.com",
        "trello.com",
    ];
    for domain in prod_domains {
        rules.push(RuleDefinition {
            id: format!("default:domain:{}", domain),
            priority: 0,
            match_field: MatchField::Domain,
            pattern: domain.to_string(),
            category: ActivityCategory::Productivity,
            source: RuleSource::Default,
        });
    }

    // ── Design ─────────────────────────────────────────────────────────
    let design_apps = ["figma-linux", "inkscape", "gimp", "blender"];
    for app in design_apps {
        rules.push(RuleDefinition {
            id: format!("default:app:{}", app),
            priority: 0,
            match_field: MatchField::App,
            pattern: app.to_string(),
            category: ActivityCategory::Design,
            source: RuleSource::Default,
        });
    }

    let design_domains = ["figma.com", "dribbble.com"];
    for domain in design_domains {
        rules.push(RuleDefinition {
            id: format!("default:domain:{}", domain),
            priority: 0,
            match_field: MatchField::Domain,
            pattern: domain.to_string(),
            category: ActivityCategory::Design,
            source: RuleSource::Default,
        });
    }

    // ── Entertainment ──────────────────────────────────────────────────
    let ent_apps = ["spotify", "steam", "vlc"];
    for app in ent_apps {
        rules.push(RuleDefinition {
            id: format!("default:app:{}", app),
            priority: 0,
            match_field: MatchField::App,
            pattern: app.to_string(),
            category: ActivityCategory::Entertainment,
            source: RuleSource::Default,
        });
    }

    let ent_domains = ["youtube.com", "netflix.com", "twitch.tv", "reddit.com"];
    for domain in ent_domains {
        rules.push(RuleDefinition {
            id: format!("default:domain:{}", domain),
            priority: 0,
            match_field: MatchField::Domain,
            pattern: domain.to_string(),
            category: ActivityCategory::Entertainment,
            source: RuleSource::Default,
        });
    }

    // ── System ─────────────────────────────────────────────────────────
    let sys_apps = [
        "system",
        "org.gnome.nautilus",
        "nautilus",
        "dolphin",
        "thunar",
        "htop",
        "btop",
    ];
    for app in sys_apps {
        rules.push(RuleDefinition {
            id: format!("default:app:{}", app),
            priority: 0,
            match_field: MatchField::App,
            pattern: app.to_string(),
            category: ActivityCategory::System,
            source: RuleSource::Default,
        });
    }

    rules
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_rules_sanity() {
        let rules = get_default_rules();
        assert!(!rules.is_empty(), "Default rules must not be empty");
        // Ensure no empty patterns or IDs
        for r in &rules {
            assert!(!r.id.is_empty());
            assert!(!r.pattern.is_empty());
            assert_eq!(r.source, RuleSource::Default);
        }
    }
}
