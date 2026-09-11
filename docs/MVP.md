# MVP Specification: Tendly

This document defines the Minimum Viable Product (MVP) for Tendly, mapping the 15 core product requirements to tangible features. 

## Platform Scope & Constraints
*   **Target Platform:** Linux-first (X11 and wlroots-based Wayland compositors).
*   **Technology:** Tauri 2.x, Rust (Backend/Tracker), Svelte 5 (Frontend).
*   **AI Scope:** Rules engine and optional local Ollama (qwen2.5:3b) integration.

## What Makes This Genuinely Usable?
A tech demo tracks a window and prints to a terminal. The MVP is genuinely usable because it operates entirely in the background, survives system reboots, provides a non-technical UI to view data, and allows the user to immediately derive insights from their day without writing regex rules.

---

## Feature Mapping

### 1. Install & Launch (Req: 1, 2)
*   **User Problem:** Users need a simple way to get the application onto their system and understand its purpose before committing to tracking.
*   **Value:** Lowers the barrier to entry; builds immediate trust regarding privacy.
*   **Complexity:** Low.
*   **Dependencies:** Build pipeline, Tauri packaging.
*   **Acceptance Criteria:** 
    *   User can download a single executable or standard Linux package (AppImage/deb).
    *   On first launch, a welcome screen explicitly states: "Data stays on your device."

### 2. Permissions & Initialization (Req: 3)
*   **User Problem:** Desktop trackers require specific OS permissions (e.g., accessibility/window reading APIs) which can be confusing to grant.
*   **Value:** Ensures the app actually works without frustrating the user with silent failures.
*   **Complexity:** Medium (X11 vs Wayland handling).
*   **Dependencies:** OS integration layer.
*   **Acceptance Criteria:**
    *   App checks if it has permission to read active window titles.
    *   If permission is denied, UI guides the user to the correct OS settings.

### 3. Background Tracking (Req: 4, 5, 6)
*   **User Problem:** Manual timers are disruptive. Tracking must happen invisibly while the user works, and must pause when the user is away from the keyboard.
*   **Value:** Zero cognitive overhead time tracking without false activity accumulation during AFK periods.
*   **Complexity:** Medium.
*   **Dependencies:** Permissions (Feature 2).
*   **Acceptance Criteria:**
    *   App can be minimized to the system tray.
    *   Tracker captures active window title and executable name event-driven or on a short poll interval.
    *   AFK/idle detection automatically pauses block generation when user input ceases for longer than threshold (default 5 minutes).
    *   Tracking survives system sleep and network disconnects.

### 4. Local Storage Engine (Req: 7)
*   **User Problem:** Data must be saved locally and efficiently so the app doesn't consume massive amounts of disk space or memory.
*   **Value:** Guarantees privacy and fast historical retrieval.
*   **Complexity:** Low.
*   **Dependencies:** Background Tracking (Feature 3).
*   **Acceptance Criteria:**
    *   Data is written to a local SQLite database (or similar lightweight store) in the user's data directory.
    *   Schema efficiently stores: timestamp, duration, app name, window title.

### 5. Timeline & History UI (Req: 8)
*   **User Problem:** Raw database rows are useless. Users need to visually comprehend their day.
*   **Value:** Immediate realization of "where did my time go?"
*   **Complexity:** High (Data visualization in Svelte).
*   **Dependencies:** Local Storage (Feature 4).
*   **Acceptance Criteria:**
    *   Dashboard displays a daily timeline view (e.g., a stacked bar chart or calendar view).
    *   User can see a list of top applications used that day by duration.

### 6. Classification & AI Integration (Req: 9, 10)
*   **User Problem:** Knowing "Firefox was open for 4 hours" doesn't indicate productivity. Users need to know if it was Focus or Drift.
*   **Value:** Context-aware categorization that saves the user from writing complex rules.
*   **Complexity:** High.
*   **Dependencies:** Background Tracking (Feature 3).
*   **Acceptance Criteria:**
    *   Tier 1 (Rules): Basic mapping (e.g., `code` -> Focus, `steam` -> Drift).
    *   Tier 2 (Ollama): App can send unknown window titles to a local Ollama instance (if configured) to return Focus/Neutral/Drift.
    *   UI displays total time spent in Focus vs. Drift.

### 7. Correction & Review (Req: 11, 12)
*   **User Problem:** AI and rules are occasionally wrong. Users need agency to fix misclassifications.
*   **Value:** Increases accuracy over time and builds user trust.
*   **Complexity:** Medium.
*   **Dependencies:** Classification (Feature 6), Timeline (Feature 5).
*   **Acceptance Criteria:**
    *   User can click any tracked activity in the UI and override its category (e.g., change "YouTube" from Drift to Focus).
    *   Corrections are saved locally and override future identical events (Rule generation).

### 8. Data Transparency & Management (Req: 13, 14, 15)
*   **User Problem:** Users need absolute control over their tracking state and historical data to feel safe.
*   **Value:** Fulfills the "privacy-first" promise.
*   **Complexity:** Low.
*   **Dependencies:** Local Storage (Feature 4).
*   **Acceptance Criteria:**
    *   System tray icon allows one-click "Pause Tracking" for a specified duration (e.g., 15 mins, 1 hour).
    *   Settings panel clearly shows the file path of the database.
    *   Settings panel includes a "Delete All Data" button that securely wipes the local database.

---

## What is NOT in the MVP
 
The following items are explicitly excluded from the MVP scope to ensure a focused, rapid release:

*   **Browser Extensions:** Reading raw URL paths provides granular context, but introduces extension state management, cross-browser WebExtensions maintenance, and additional attack surface. Window titles are sufficient for initial classification (deferred to v0.3).
*   **Windows & macOS Support:** Building reliable OS-level window trackers for all three platforms simultaneously stalls delivery. Linux X11/Wayland is the initial target to prove the concept within the primary developer demographic (deferred to v0.6 and v0.7).
*   **Cloud BYOK (Bring Your Own Key):** The third tier of the AI cascade introduces API key management, network error handling, and privacy toggles. The MVP focuses on deterministic Rules and optional local Ollama (cloud BYOK deferred to v0.2).
*   **Goals & Notifications:** Active interventions such as "You've been in Drift for 30 minutes" are post-MVP enhancements (deferred to v0.5).
*   **Multi-Device Sync:** Synchronizing data across devices requires end-to-end encryption (deferred to v0.8+).
