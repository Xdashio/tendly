# Tendly Rules-Based Classification Engine

## 1. Overview

Tendly provides deterministic, explainable, and local-only activity classification. The classification engine runs entirely offline within the Rust core, assigning observations to a controlled 9-category taxonomy without cloud telemetry, heuristics, embeddings, or machine learning models.

The system is designed with three core principles:
1. **Explainability**: Every classification decision produces a traceable reason (`rule_id`, `matched_by`, and a human-readable `explanation`).
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
    ├─► 2. User Rules (Priority 1000..1999)
    │       └─► Domain Match > Title Match > App Match
    │
    ├─► 3. Default Rules (Priority 100..999)
    │       └─► Domain Match > Title Match > App Match
    │
    └─► 4. Generic Fallback (Priority 0..99)
            └─► Browser generic fallback: Browsing
            └─► Unmatched process: Unknown
```

### Precedence Algorithm

1. **Precedence Tiers**:
   - **User Rules**: Priority 1000 to 1999. User-configured rules always take precedence over system defaults.
   - **Default Rules**: Priority 100 to 999. Developer-focused built-in defaults.
   - **Generic Fallback**: Priority 0 to 99. Fallback for recognized browser applications (`browsing`) or unrecognized processes (`unknown`).

2. **Condition Specificity**:
   Within any priority tier, rules are evaluated by matcher specificity:
   - **Domain Match**: Most specific. When browser context is available (e.g. `github.com`), domain matching supersedes application matching.
   - **Title Match**: Case-insensitive substring matching against window titles or page titles.
   - **App Match**: Exact case-insensitive matching against normalized application identifiers (`WM_CLASS` / Hyprland class).

3. **Deterministic Tie-Breaking**:
   Rules are sorted once upon engine creation by:
   - `priority DESC`
   - `rule_id ASC` (deterministic alphabetical tie-breaker)
   The first matching rule terminates evaluation.

---

## 4. Default Ruleset Reference

### Development
- **Applications**: `code`, `vscodium`, `cursor`, `idea`, `clion`, `pycharm`, `webstorm`, `rustrover`, `sublime_text`, `alacritty`, `kitty`, `wezterm`, `gnome-terminal`, `konsole`, `xterm`, `foot`, `ghostty`, `zed`, `emacs`, `neovim`
- **Domains**: `github.com`, `gitlab.com`, `stackoverflow.com`, `crates.io`, `docs.rs`, `npm.im`, `localhost`, `127.0.0.1`
- **Title Patterns**: `pull request`, `merge request`, `commit`, `vim`, `nvim`, `cargo`, `git`

### Communication
- **Applications**: `slack`, `discord`, `teams`, `zoom`, `thunderbird`, `telegram-desktop`, `signal-desktop`, `element`, `hexchat`
- **Domains**: `slack.com`, `discord.com`, `meet.google.com`, `zoom.us`, `teams.microsoft.com`, `web.telegram.org`
- **Title Patterns**: `meet:`, `zoom meeting`, `call with`

### Research
- **Domains**: `developer.mozilla.org`, `devdocs.io`, `wikipedia.org`, `duckduckgo.com`, `google.com/search`, `arxiv.org`
- **Title Patterns**: `documentation`, `api reference`, `handbook`, `specifications`, `manual`

### Productivity
- **Applications**: `obsidian`, `notion`, `linear`, `jira`, `libreoffice`, `evince`, `okular`
- **Domains**: `notion.so`, `linear.app`, `atlassian.net`, `trello.com`, `docs.google.com`, `sheets.google.com`

### Design
- **Applications**: `figma`, `gimp`, `inkscape`, `blender`, `penpot`
- **Domains**: `figma.com`, `excalidraw.com`, `draw.io`

### Entertainment
- **Applications**: `spotify`, `steam`, `vlc`, `mpv`
- **Domains**: `youtube.com`, `netflix.com`, `twitch.tv`, `reddit.com`, `x.com`, `twitter.com`

### System
- **Applications**: `nautilus`, `thunar`, `dolphin`, `gnome-control-center`, `pavucontrol`, `htop`, `btop`, `systemsettings`

### Browsing (Generic Fallback)
- **Applications**: `firefox`, `chrome`, `chromium`, `brave-browser`, `google-chrome`, `microsoft-edge`, `opera`, `vivaldi`

---

## 5. Aggregation and Presentation Semantics

### TimeBlock Classification
Tendly aggregates raw activity into fixed 3-minute analytical units `[start_ms, end_ms)`:
- During aggregation, the engine sums the active duration contributed by each observed category within the 180-second window.
- The **plurality winner** (category with the largest cumulative active duration) becomes `TimeBlock.category`.
- The rule identifier that determined the plurality winner is preserved in `TimeBlock.classified_by`.

### ActivitySession Dominant Category & Breakdown
When `TimeBlock`s are coalesced into continuous `ActivitySession`s:
- The session computes a **category breakdown** (`category_breakdown`), summarizing the exact total duration spent in each category.
- The **dominant category** (`dominant_category`) is determined by plurality duration across all constituent time blocks and underlying segments.

### Explainability Output
Every classified segment contains structured metadata:
```json
{
  "category": "development",
  "confidence": 0.95,
  "source": "default_rule",
  "rule_id": "default-domain-github",
  "matched_by": "domain:github.com",
  "explanation": "Matched default domain rule for github.com"
}
```
In the desktop UI, users can inspect any session to view the exact category breakdown, segment-by-segment classifications, and mouse-over rule explanations.

---

## 6. Non-Goals and Boundary Constraints

1. **No Productivity Judgment**: Tendly categorizes what an activity is, never whether it is good, bad, or productive. A YouTube video watching a compiler design lecture or entertainment music stream are both processed objectively without judgment scores.
2. **No Cloud Telemetry or APIs**: Classification runs entirely in-process in Rust without external network requests.
3. **No Machine Learning or AI Models in Phase 6**: All classification in Phase 6 is deterministic and rules-based. Local AI and LLM summarization are reserved for Phase 7.
4. **No Database Schema Mutations**: Time block classification utilizes existing columns (`category`, `classified_by`, `confidence`). Session category breakdowns are computed dynamically in-memory.
