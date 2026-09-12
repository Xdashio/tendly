# Browser Context Specification

This document details Tendly's browser context capture architecture, supported browsers, title extraction mechanics, URL normalization, privacy principles, and failure isolation.

---

## 1. Objectives and Scope

The goal of Browser Context in Tendly is to make web browser activity informative by capturing browser-specific context when available:
- **Browser identification**: Identifying the active browser application (Firefox, Chrome, Chromium, Brave, Vivaldi, Edge).
- **Page title extraction**: Isolating the human-readable page title by stripping browser-specific window decoration suffixes.
- **URL & domain normalization**: Providing deterministic normalization rules, credential stripping, and tracking parameter removal.

### Explicit Scope Boundaries
- **Context capture only**: Phase 5 captures and surfaces context. It does **not** classify, score, or categorize browser activity.
- **Single source of truth**: Browser context enriches existing atomic `RawEvent`s and `ActivitySegment`s. No secondary browser database or parallel history table is created.
- **Zero remote transmission**: All processing occurs strictly on the local machine. URLs and browser titles are never sent to external servers.

---

## 2. Acquisition Mechanism: Window Title Parsing

### Architecture Decision

Tendly adopts **window title parsing** as its core browser context acquisition mechanism:
1. **Zero invasive hooks**: Does not require browser extensions, native messaging binaries, or remote debugging ports (CDP).
2. **Deterministic & reliable**: Modern desktop browsers on Linux (X11 and Wayland) reliably encode the active tab's page title in the top-level window title (`_NET_WM_NAME` on X11, Hyprland/Wayland socket IPC).
3. **Failure isolated**: If window title parsing fails or encounters an unrecognized window pattern, Tendly gracefully falls back to the raw window title with zero disruption to tracking.

### Platform Availability

| Platform | Window Title Availability | Browser Identification | Page Title Extraction |
|---|---|---|---|
| **Linux (X11)** | `_NET_WM_NAME` via `x11rb` | `WM_CLASS` / `/proc/{pid}/comm` | Full |
| **Linux (Hyprland Wayland)** | `activewindow` via UNIX socket | IPC window class | Full |
| **Linux (GNOME/KDE Wayland)** | Limited (security sandboxing) | Limited | Fallback to app name |

---

## 3. Supported Browsers and Title Suffixes

Tendly identifies browsers via case-insensitive matching against standard window classes and process names:

| Browser | Window Class Patterns | Recognized Title Suffixes |
|---|---|---|
| **Firefox** | `firefox`, `firefox-esr`, `navigator` | ` — Mozilla Firefox`, ` — Mozilla Firefox Private Browsing`, ` - Mozilla Firefox` |
| **Google Chrome** | `google-chrome`, `google-chrome-stable` | ` - Google Chrome`, ` - Google Chrome (Incognito)` |
| **Chromium** | `chromium`, `chromium-browser` | ` - Chromium`, ` - Chromium (Incognito)` |
| **Brave** | `brave`, `brave-browser` | ` - Brave`, ` - Brave (Private)` |
| **Vivaldi** | `vivaldi`, `vivaldi-stable` | ` - Vivaldi` |
| **Microsoft Edge** | `microsoft-edge`, `microsoft-edge-stable`, `msedge` | ` - Microsoft Edge`, ` - Microsoft Edge (InPrivate)` |

### Title Parsing Algorithm

1. Trim leading and trailing whitespace from the window title.
2. If the trimmed title is empty, return `None`.
3. Check against known suffixes for the detected browser:
   - If the title matches the suffix alone (e.g. `" — Mozilla Firefox"`), return `None`.
   - If the title ends with the suffix, strip the suffix and trim the remaining string. If non-empty, return it as `page_title`.
4. If the title is simply the browser name (e.g. `"Mozilla Firefox"`), return `None`.
5. If no suffix matches but the application is a known browser, return the full trimmed title as `page_title`.

---

## 4. URL Normalization and Privacy

When URLs are processed (such as from future extension feeds or synthetic events), Tendly applies a strict privacy-preserving normalization policy:

### Deterministic Pipeline
1. **WHATWG Compliance**: Parsed using the Rust `url` crate.
2. **Scheme & Host Normalization**: Lowercase scheme and host.
3. **Default Port Removal**: Strip `:80` for HTTP and `:443` for HTTPS.
4. **Credential Stripping**: Remove all `username:password@` components.
5. **Tracking Parameter Elimination**: Strip marketing, tracking, and referral parameters:
   - `utm_source`, `utm_medium`, `utm_campaign`, `utm_term`, `utm_content`, `utm_id`
   - `fbclid`, `gclid`, `ref`
6. **Query Preservation**: Retain meaningful query parameters (e.g., search queries `?q=...`, documentation anchors `#section`).

---

## 5. Pipeline Integration

```
[Window Watcher (X11 / Wayland)]
             │
             ▼
   enrich_browser_context()
             │
             ├──> raw_json: Serialized BrowserContext
             ▼
      [CapturePipeline]
             │
             ▼
     reconstruct_segments()
             │
             ├──> ActivitySegment.browser_context
             ▼
  aggregate_segments_to_blocks()
             │
             ├──> TimeBlock.dominant_url (extracted domain)
             ▼
      [Timeline / UI]
             │
             └──> Surfaces clean page title & domain badge
```

- **Context Changes**: Page title changes within the same browser session naturally generate distinct `ActivitySegment`s because segment reconstruction keys on `(app, title)`.
- **Deduplication**: Deduplication filters permit event emission whenever the window title changes, guaranteeing rapid context-switch detection.
