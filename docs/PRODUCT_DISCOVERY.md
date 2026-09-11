# Product Discovery: Tendly

## 1. What is Tendly?

**Tendly** is a free, open-source, privacy-first, local-first AI-assisted time-awareness desktop application built specifically for software developers. Powered by Tauri 2.x (Rust + Svelte 5), it silently observes active window titles and applications, classifying time into meaningful categories (Focus, Neutral, Drift) using a lightweight 3-tier cascade approach (Rules → Local Ollama → Optional BYOK Cloud).

## 2. What Problem Does It Solve?

Software developers need to know where their time goes to improve focus, avoid burnout, and bill accurately. However, the current market forces them to choose between two extremes:
*   **"Free but Dumb" (e.g., ActivityWatch):** Highly private and open-source, but relies on brittle, manual regex rules that fail to distinguish context (e.g., watching a tutorial vs. watching a let's play on YouTube). 
*   **"Smart but Locked" (e.g., Rize, RescueTime):** Excellent UI and AI categorization, but requires monthly subscriptions, mandates cloud data storage, and introduces severe privacy and NDA risks for developers.

Tendly bridges this gap by offering "Smart but Free/Local" time tracking. 

## 3. Why Does Automatic Tracking Matter?

Manual time trackers (like Toggl) suffer from a critical flaw: they require active user intervention. 
*   **Flow State Interruption:** Starting and stopping timers breaks the developer's concentration.
*   **Timer Fatigue:** Forgetting to stop a timer ruins the data integrity for the entire day.
*   **High Abandonment Rate:** Because of the constant cognitive overhead, developers typically abandon manual time trackers within 3 to 7 days.

Automatic tracking happens invisibly in the background, ensuring data is collected without disrupting the user's primary workflow.

## 4. Why Does Local-First Matter?

For a software developer, window titles and active applications leak highly sensitive information:
*   **Proprietary Codebases & NDAs:** File paths often contain client names or unreleased project codenames.
*   **Security Risks:** API tokens, passwords, or temporary credentials can occasionally appear in window titles or terminal outputs.
*   **Telemetry Backlash:** The developer demographic is uniquely hostile to cloud telemetry, data harvesting, and invasive analytics. A local-first application ensures zero data leaves the machine without explicit consent, eliminating compliance and privacy risks.

## 5. Why Does AI Matter?

Context-aware classification is Tendly's core value proposition. The same application can represent vastly different states of productivity:
*   **YouTube:** Watching a Rust concurrency tutorial (Focus) vs. watching gaming highlights (Drift).
*   **Terminal:** Running test suites (Focus) vs. playing a terminal-based game (Drift).
*   **Firefox:** Reading documentation on MDN (Focus) vs. browsing Reddit (Drift).

Regex rules (like those in ActivityWatch) cannot handle this nuance. Tendly's AI layer analyzes the semantics of the window title and context to intelligently classify the activity.

## 6. What Does Tendly Do Differently?

Tendly provides a unique, unmatched combination in the market:
*   **Free & Open Source:** Accessible to everyone, auditable by anyone.
*   **Local-First Privacy:** Data stays on the user's machine.
*   **AI-Classified Context:** Nuanced categorization using local LLMs (Ollama) without sacrificing privacy.
*   **Cross-Platform (Tauri):** Low resource footprint (Rust) combined with a modern UI (Svelte 5).

## 7. What Does Tendly Deliberately NOT Do?

To maintain focus and avoid bloat, Tendly explicitly rejects the following:
*   **No Teams or Manager Dashboards:** This is a tool for the individual, not for surveillance.
*   **No Billing or Subscriptions:** The core product remains free.
*   **No Gamification:** No arbitrary scores or leaderboards that incentivize "gaming the system."
*   **No Cloud Accounts:** No mandatory logins or centralized databases.
*   **No Aggressive Blocking:** Tendly observes and reports; it does not block apps or enforce Pomodoro locks.

## 8. What is the User's Core Journey?

1.  **Install:** Download and run the lightweight Tauri binary.
2.  **Onboard:** Learn how privacy works, select the AI classification tier (Rules vs. Ollama vs. Cloud), and grant OS accessibility permissions.
3.  **Track:** Work normally while Tendly silently collects window data in the background.
4.  **Review:** Open the dashboard to view the day's timeline and categorization.
5.  **Understand:** Gain insights into how much time was actually spent in Focus vs. Drift.
6.  **Correct:** Fix any misclassifications (which trains the rules engine).
7.  **Improve:** Use the insights to structure better work days tomorrow.

## 9. What Makes Tendly Valuable After One Day?

Even if the user only utilizes rules-only classification, day one provides immediate value: **"I can see where my time went."** The timeline visualization reveals interruptions, context-switching penalties, and the reality of time spent versus time perceived.

## 10. What Makes Tendly Valuable After One Week?

After a week, patterns emerge. The AI classification becomes highly accurate as the user corrects edge cases (feeding the rules-override layer). The user transitions from passive observation to active intention, understanding their peak focus hours and biggest distractions.

## 11. What Would Make a User Uninstall It?

The product will fail if it exhibits any of the following:
*   **High Resource Usage:** If the Rust tracker or local LLM spins up the fans or drains the battery excessively.
*   **Inaccurate Classification:** If the AI consistently mislabels coding as Drift.
*   **Privacy Concerns:** If the app makes unexpected network requests or if the local promise feels hollow.
*   **Complexity:** If setup requires deep terminal knowledge or complicated configuration.
*   **Broken Capture:** If the OS tracking silently fails and drops data.

---

## Competitive Landscape

| Product | Strengths | Weaknesses | Threat Level to Tendly |
| :--- | :--- | :--- | :--- |
| **ActivityWatch** | Free, open-source, local-first, strong Linux community. | Utilitarian UI, high RAM (Python), raw logs, brittle manual regex, no AI context. | Low/Medium. Tendly is the modern, AI-powered evolution of AW. |
| **Rize.io** | Excellent UX, strong AI categorization. | Expensive subscription ($15-20/mo), mandatory cloud, privacy risks for developers. | Medium. Tendly targets users who churn from Rize due to price/privacy. |
| **RescueTime** | Established, detailed reporting. | Centralized cloud, subscription, outdated UI, relies on rigid categories. | Low. |
| **Drifty** | Local AI option, BYOK, closest comparable. | Mac-first, requires high RAM (5-7GB) for local, 3-minute block resolution. | High. The closest conceptual competitor, but Tendly targets cross-platform/Linux and lower resource usage. |
| **WakaTime** | Excellent code-time tracking via IDE plugins. | Misses browser research, terminal work, and general desktop activity. | Low. Can be complementary. |
| **Timing (Mac)** | Good UX, automatic tracking. | Paid, Mac-only, limited AI capability. | Low. |
| **ManicTime** | Native app, local-first, detailed timeline. | Windows-focused, no native AI categorization, visually dated. | Low. |
