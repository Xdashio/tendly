//! Safe, deterministic matching algorithms for domains, applications, and titles.
//!
//! # Safety Invariants
//! - No dynamic evaluation, regex compilation, or command execution.
//! - All matching operates on normalized strings.
//! - Domain matching strictly respects dot boundaries to prevent prefix/suffix spoofing
//!   (e.g., `notgithub.com` must never match `github.com`).

/// Evaluates whether `input_domain` matches the rule's `pattern`.
///
/// Matching semantics:
/// 1. Both domain and pattern are lowercased and stripped of leading/trailing dots and whitespace.
/// 2. If `input == pattern`, it is an exact domain match.
/// 3. If `input.ends_with(&format!(".{}", pattern))`, it is a valid subdomain match.
///
/// Returns `false` for invalid, empty, or deceptive inputs.
pub fn matches_domain(input_domain: &str, pattern: &str) -> bool {
    let input = input_domain.trim().trim_matches('.').to_lowercase();
    let pat = pattern.trim().trim_matches('.').to_lowercase();

    if input.is_empty() || pat.is_empty() {
        return false;
    }

    // Exact domain match
    if input == pat {
        return true;
    }

    // Subdomain match: must end with ".<pat>"
    if input.ends_with(&format!(".{}", pat)) {
        return true;
    }

    false
}

/// Evaluates whether `input_app` matches `pattern` via case-insensitive exact matching.
pub fn matches_app(input_app: &str, pattern: &str) -> bool {
    let input = input_app.trim().to_lowercase();
    let pat = pattern.trim().to_lowercase();

    if input.is_empty() || pat.is_empty() {
        return false;
    }

    input == pat
}

/// Evaluates whether `input_title` contains `pattern` via case-insensitive substring matching.
pub fn matches_title(input_title: &str, pattern: &str) -> bool {
    let input = input_title.trim().to_lowercase();
    let pat = pattern.trim().to_lowercase();

    if input.is_empty() || pat.is_empty() {
        return false;
    }

    input.contains(&pat)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Domain matching tests ──────────────────────────────────────────

    #[test]
    fn test_domain_exact_match() {
        assert!(matches_domain("github.com", "github.com"));
        assert!(matches_domain("GITHUB.COM", "github.com"));
        assert!(matches_domain("github.com", "GITHUB.COM"));
        assert!(matches_domain("  github.com. ", "github.com"));
    }

    #[test]
    fn test_domain_subdomain_match() {
        assert!(matches_domain("www.github.com", "github.com"));
        assert!(matches_domain("gist.github.com", "github.com"));
        assert!(matches_domain("api.github.com", "github.com"));
        assert!(matches_domain(
            "raw.githubusercontent.com",
            "githubusercontent.com"
        ));
        assert!(matches_domain("sub.sub.github.com", "github.com"));
    }

    #[test]
    fn test_domain_boundary_spoof_prevention() {
        // Must NOT match prefixes or suffixes that are different domains
        assert!(!matches_domain("notgithub.com", "github.com"));
        assert!(!matches_domain("evil-github.com", "github.com"));
        assert!(!matches_domain("fakegithub.com", "github.com"));
        assert!(!matches_domain("my-github.com", "github.com"));
        assert!(!matches_domain("github.com.attacker.test", "github.com"));
        assert!(!matches_domain("github.com.evil.test", "github.com"));
        assert!(!matches_domain("example.com.evil.test", "example.com"));
    }

    #[test]
    fn test_domain_empty_and_malformed() {
        assert!(!matches_domain("", "github.com"));
        assert!(!matches_domain("github.com", ""));
        assert!(!matches_domain("", ""));
        assert!(!matches_domain("...", "..."));
        assert!(!matches_domain("\0\0", "github.com"));
    }

    // ── App matching tests ─────────────────────────────────────────────

    #[test]
    fn test_app_matching() {
        assert!(matches_app("code", "code"));
        assert!(matches_app("Code", "code"));
        assert!(matches_app("CODE", "code"));
        assert!(matches_app("Alacritty", "alacritty"));
        assert!(!matches_app("code-insiders", "code"));
        assert!(!matches_app("", "code"));
        assert!(!matches_app("code", ""));
    }

    // ── Title matching tests ───────────────────────────────────────────

    #[test]
    fn test_title_matching() {
        assert!(matches_title("Pull Request #123 - tendly", "pull request"));
        assert!(matches_title("Google Meet: Weekly Sync", "meet"));
        assert!(matches_title("Zoom Meeting", "meeting"));
        assert!(!matches_title("main.rs - Visual Studio Code", "meeting"));
        assert!(!matches_title("", "meeting"));
        assert!(!matches_title("anything", ""));
    }
}
