# Tendly Rules-Based Classification Engine

## 1. Overview

Tendly provides deterministic, explainable, and local-only activity classification. The classification engine runs entirely offline within the Rust core, assigning observations to a controlled 9-category taxonomy without cloud telemetry, heuristics, embeddings, or machine learning models.

The system is designed with three core principles:
1. **Explainability**: Every classification decision produces a traceable reason (`rule_id`, `matched_field`, `pattern`, and a human-readable `explanation`).
2. **Neutrality**: Categories are descriptive, not judgmental. Tendly does not calculate productivity or distraction scores.
3. **Lossless Pipeline**: Raw events and analytical time blocks remain canonical. Classifications are computed deterministically during segment reconstruction and aggregated into time blocks and user-facing activity sessions.

---

## 2. Taxonomy Specification

Tendly defines a controlled taxonomy of nine mutually exclusive categories:

| Category | Description | Primary Targets |
| :--- | :--- | :--- |
| `development` | Software engineering, code editing, terminal commands, debugging, code review | IDEs (VS Code, JetBrains, Neovim), terminals (Alacritty, Kitty, WezTerm), Git interfaces, GitHub, GitLab, Stack Overflow, localhost |
| `communication` | Team messaging, email, meetings, voice/video calls | Slack, Discord, Microsoft Teams, Zoom, Thunderbird, Gmail, Telegram, Signal |
| `research` | Technical documentation, reference reading, API guides, whitepapers, search engines | MDN Web Docs, DevDocs, Rust docs, Wikipedia, DuckDuckGo, Google Search, arXiv |
| `productivity` | Task management, issue tracking, note taking, writing, spreadsheets | Obsidian, Notion, Jira, Linear, Trello, Google Docs/Sheets, LibreOffice |
| `design` | UI/UX design, diagramming, 3D modeling, vector/raster graphics | Figma, Inkscape, GIMP, Blender, Draw.io, Excalidraw, Penpot |
| `entertainment` | Video streaming, music players, social media, gaming | YouTube, Spotify, Netflix, Twitch, Steam, Reddit, Twitter/X |
| `system` | Operating system utilities, file managers, system settings, desktop environment | System monitors (htop, btop), file managers (Nautilus, Thunar, Dolphin), desktop settings, lock screens |
| `browsing` | General web navigation where domain or title does not match a more specific category | Web browsers (Firefox, Chrome, Chromium, Brave) with generic or unmatched content |
| `unknown` | Idle/AFK periods, unrecorded time gaps, or unrecognized processes | System AFK state, lock periods, blank or invalid observation titles |

---

## 3. Engine Architecture & Precedence

The classification engine (`RuleEngine`) evaluates incoming segment attributes against active rules:

```text
ActivitySegment (app, title, browser_context, activity_type)
    │
    ▼
Rule Engine Evaluation
    │
    ├─► 1. Is activity_type == Afk? ────► Category: Unknown ("User idle / Away from keyboard")
    │
    ├─► 2. Active Rules Precedence:
    │       Rule Source (User > Default)
    │       └─► Priority (DESC)
    │           └─► Specificity (AppAndDomain > AppAndTitle > Domain > TitleContains > App)
    │               └─► Pattern Length (DESC)
    │                   └─► Rule ID (ASC deterministic tie-breaker)
    │
    ├─► 3. Browser Fallback:
    │       └─► Recognized browser without domain rule: Browsing
    │
    └─► 4. Generic Fallback:
            └─► Unmatched process: Unknown
```

### Precedence Algorithm

Rules are sorted once upon engine creation according to Tendly's explicit deterministic ordering:

1. **Rule Source**: User rules (`RuleSource::User`, order 2) always take precedence over Default rules (`RuleSource::Default`, order 1).
2. **Priority**: Descending integer (`priority DESC`).
3. **Condition Specificity Ranking**: At equal priority, rules are ordered by condition specificity (`specificity DESC`):
   - `AppAndDomain` (rank 5): Matches both application identity and browser domain.
   - `AppAndTitle` (rank 4): Matches both application identity and window/page title.
   - `Domain` (rank 3): Matches browser domain or subdomain.
   - `TitleContains` (rank 2): Substring match against window or page title.
   - `App` (rank 1): Exact match against application process name / class.
4. **Pattern Length**: Descending length (`pattern.len() DESC`), ensuring more specific patterns match before shorter prefixes.
5. **Deterministic Tie-Breaker**: Ascending rule identifier (`rule_id ASC`).

The first matching rule terminates evaluation.

---

## 4. Domain Matching Semantics

Domain matching in `matches_domain(input_domain, pattern)` evaluates host and arbitrary subdomains on strict dot boundaries:

- **Definition**: A domain rule matches the exact host or any subdomain on a dot boundary (`*.<pattern>`).
- **Valid Matches**:
  - Exact host: `github.com` matches `github.com` (case-insensitive, whitespace and trailing dots trimmed).
  - Subdomains: `github.com` matches `www.github.com`, `gist.github.com`, `api.github.com`, and `sub.sub.github.com`.
- **Spoof Prevention**:
  - `github.com` does **not** match prefix variants: `notgithub.com`, `evil-github.com`, `fakegithub.com`.
  - `github.com` does **not** match suffix/domain-spoof variants: `github.com.attacker.test`, `github.com.evil.test`.

---

## 5. Aggregation and Presentation Semantics

### TimeBlock Classification
Tendly aggregates raw activity into fixed 3-minute analytical units `[start_ms, end_ms)`:
- During aggregation, the engine sums the active duration contributed by each observed category within the 180-second window.
- The **plurality winner** (category with the largest cumulative active duration) becomes `TimeBlock.category`.
- The rule identifier that determined the plurality winner is preserved in `TimeBlock.classified_by`.
- Ties are broken deterministically by alphabetical order of category name.

### ActivitySession Dominant Category & Breakdown
When `TimeBlock`s are coalesced into continuous `ActivitySession`s:
- The session computes a **category breakdown** (`category_breakdown`), summarizing the exact total duration spent in each category from the underlying segments.
- The **dominant category** (`dominant_category`) is determined by plurality duration across all constituent time blocks and underlying segments.

### Explainability Output
Every classified segment contains structured metadata:
```json
{
  "category": "development",
  "source": "rule",
  "rule_id": "default:domain:github.com",
  "matched_field": "domain",
  "pattern": "github.com",
  "explanation": "Browser domain 'github.com' matched rule 'default:domain:github.com'"
}
```
In the desktop UI, users can inspect any session to view the exact category breakdown, segment-by-segment classifications, and mouse-over rule explanations.

---

## 6. Performance Characteristics

- **Zero Network / Database Overhead**: Classification uses pre-sorted in-memory rules and simple string matching. It performs no network requests, regular-expression evaluation, or database scans during ordinary classification.
- **Single Traversal**: Rules are iterated in pre-sorted order; the first matching rule terminates evaluation.
- **Idempotent Reprocessing**: Historical time blocks can be re-evaluated from raw events deterministically at any time without data loss or classification drift.

---

## 7. Non-Goals and Boundary Constraints

1. **No Productivity Judgment**: Tendly categorizes what an activity is, never whether it is good, bad, or productive. A YouTube video watching a compiler design lecture or entertainment music stream are both processed objectively without judgment scores.
2. **No Cloud Telemetry or APIs**: Classification runs entirely in-process in Rust without external network requests.
3. **No Machine Learning or AI Models in Phase 6**: All classification in Phase 6 is deterministic and rules-based. Local AI and LLM summarization are reserved for Phase 7.
4. **No Database Schema Mutations**: Time block classification utilizes existing columns (`category`, `classified_by`). Session category breakdowns are computed dynamically in-memory.
