# Privacy Model

Tendly is a privacy-first, local-first time-awareness application. Privacy is not a feature—it is the foundational product promise. Because Tendly is designed primarily for developers and knowledge workers who frequently handle proprietary codebases, NDAs, API tokens, and sensitive information, protecting user data is paramount. 

If privacy is compromised, the product has no reason to exist.

This document outlines our data collection practices, data handling principles, and our strict privacy guarantees.

## 1. Data Collection Inventory

For every type of data Tendly collects (or explicitly does not collect), the table below details our handling policies.

| Data Type | Collected? | Storage | Sensitivity | User Control |
|-----------|-----------|---------|-------------|-------------|
| Application name (WM_CLASS) | Yes | SQLite local | Low | Can exclude apps |
| Window title | Yes | SQLite local | **HIGH** (may contain filenames, URLs, passwords, secrets) | Can exclude apps |
| Active tab URL (browser extension) | Post-MVP, domain only by default | SQLite local | **HIGH** | Can disable, exclude domains |
| Full URL path | Configurable, OFF by default | SQLite local | **CRITICAL** (query params may contain PII, tokens) | Opt-in only |
| Query parameters | Never | Not stored | **CRITICAL** | Stripped always |
| Tab content/body | Never | Not stored | **CRITICAL** | N/A |
| Idle time | Yes | SQLite local | Low | N/A |
| Process ID/path | Transient only | Not persisted | Low | N/A |
| Keyboard/mouse input | Never | Not stored | **CRITICAL** | N/A |
| Screenshots/screen capture | Never | Not stored | **CRITICAL** | N/A |
| AI prompts sent to Ollama | Yes (local) | In-memory only | Medium | Local only |
| AI prompts sent to BYOK cloud | Yes (if opted in) | Transient (provider's policy) | **HIGH** | Explicit opt-in, user's own key |
| Classification results | Yes | SQLite local | Low | Can delete |
| User corrections | Yes | SQLite local | Low | Can delete |
| API keys | Yes | OS keychain or encrypted config | **CRITICAL** | User manages |

## 2. Sensitive Data in Window Titles

Window titles represent the most important privacy concern. They are incredibly useful for determining activity context, but routinely contain highly sensitive information:
- File paths with project/client names
- Terminal commands potentially containing secrets/tokens
- Email subjects
- Private direct messages
- Bank/financial information
- Medical information
- URLs with query parameters (sometimes exposed in titles)

**How Tendly mitigates this risk:**
- **Local-First:** All data stays local on the user's machine by default.
- **Exclusion Rules:** Users can effortlessly exclude specific applications or window titles from tracking via blocklists.
- **Full Deletion Control:** Users can permanently delete any or all data at any time.
- **No Telemetry:** No analytics, telemetry, or phone-home mechanisms exist.
- **Cloud Warnings:** When a user explicitly opts in to a Bring-Your-Own-Key (BYOK) cloud AI provider, Tendly clearly warns exactly what data will leave the device within the prompt.

## 3. Retention Policy

Tendly’s retention policy ensures that data is stored only as long as the user desires:
- **Default:** Data is kept indefinitely, as long-term patterns are valuable for personal analytics.
- **Deletion:** The user has full control to delete all data, delete data by specific date ranges, or selectively delete specific activity blocks.
- **Future Feature:** Optional auto-delete after N days.

## 4. Private and Sensitive Application Handling

Certain applications are notoriously sensitive and require special handling:
- **Password Managers:** Should be auto-excluded by default, or their window titles aggressively redacted.
- **Banking/Financial Apps:** Users should be warned or these apps categorized appropriately to avoid over-collection.
- **Incognito Browser Tabs:** Not tracked by default (the browser extension is disabled in incognito mode by default).
- **Terminal Emulators:** Activity is tracked, but users are advised that titles may occasionally expose secrets depending on the shell configuration.

## 5. AI Privacy

Tendly uses AI to categorize and summarize activities. The privacy approach depends on the tier:
- **Local AI (Ollama):** All processing happens locally. Prompts never leave the machine.
- **BYOK Cloud:** Users must explicitly opt-in and use their own API key. Data is sent to the chosen provider.
- **No Middleman:** There is no Tendly-operated cloud service, no Tendly account, and no Tendly API key required.
- **Prompt Minimization:** AI prompts contain only the app name, window title, URL domain (if enabled), and user profile text. They explicitly **do not** contain full URLs, query parameters, file contents, or keystrokes.

## 6. Export and Deletion

Users have total sovereignty over their data:
- **Export:** Users can export all activity data in standardized formats (JSON/CSV).
- **Nuclear Deletion:** A single action allows the user to instantly delete all data.
- **Physical Deletion:** Deletion triggers a physical `VACUUM` of the SQLite database to ensure data is permanently removed, not just marked as deleted.
- **No Silent Backups:** Tendly does not create hidden backups of user data.

## 7. No Telemetry Guarantee

Tendly guarantees the absence of implicit tracking:
- No analytics.
- No usage tracking.
- No phone-home mechanisms.
- No crash reporting (unless explicitly requested/opted-in as a future feature).
- No auto-update checks that transmit unique machine information.
