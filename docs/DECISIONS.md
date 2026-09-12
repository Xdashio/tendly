# Architecture Decision Records (ADR)

This document records the major decisions made during Phase 0 of Tendly's discovery and design process.

---

### ADR-001: Primary Target Persona

**Date:** 2026-09-11
**Status:** Accepted

**Context:** Tendly needs a clear target audience to focus initial feature development, messaging, and integrations.

**Problem:** Attempting to build a tool for "everyone" (freelancers, students, general knowledge workers) dilutes the product focus and complicates the technical requirements for window watching and context gathering.

**Options Considered:**
1. General knowledge workers — Broadest market, but fragmented toolset.
2. Students — High need for focus, but lower willingness to pay/invest in setup.
3. Developers — Homogeneous toolset (IDEs, terminals, browsers), high technical literacy, clear value in focus time.

**Decision:** The primary target persona is Software Developers.

**Rationale:** Developers use predictable tools (VS Code, JetBrains, Terminal, specific sites like Stack Overflow/GitHub), making categorization easier. They are also comfortable with local-first, privacy-focused tools, and configuring systems like Ollama.

**Consequences:**
- Focused product scope.
- Less need for complex integrations with enterprise tools early on.
- Some non-developer tools might lack out-of-the-box categorization rules in early versions.

**Reversibility:** Moderate. The core tracking works for anyone, but the UI and default rules are optimized for devs.

---

### ADR-002: Desktop Framework

**Date:** 2026-09-11
**Status:** Accepted

**Context:** Tendly requires a desktop application capable of system-level window monitoring while providing a modern, rich UI.

**Problem:** Selecting the right framework to balance performance, binary size, ecosystem, and cross-platform capabilities.

**Options Considered:**
1. Tauri 2.x — Rust backend, web frontend, small binaries, low memory usage.
2. Electron — Web technologies, large ecosystem, but heavy memory and CPU footprint.
3. Qt / GTK4 — Native performance, but steeper UI learning curve and less flexibility for modern web-like designs.

**Decision:** Use Tauri 2.x.

**Rationale:** Tauri provides the ideal mix of system-level access (via Rust) and UI flexibility (via a web frontend). It keeps the application lightweight, which is critical for a background tracking utility that must not interfere with a developer's primary work.

**Consequences:**
- Very small binary sizes.
- Low background resource consumption.
- Requires writing system watchers in Rust.

**Reversibility:** Difficult. Changing the framework requires rewriting the entire application shell and bridge layer.

---

### ADR-003: Frontend Framework

**Date:** 2026-09-11
**Status:** Accepted

**Context:** The UI needs to be responsive, maintainable, and modern within the Tauri webview.

**Problem:** Choosing a frontend framework that integrates well with Tauri and offers good performance.

**Options Considered:**
1. Svelte 5 — Runes-based reactivity, zero virtual DOM, extremely fast and lightweight.
2. React — Massive ecosystem, but heavier runtime and virtual DOM overhead.
3. Vue 3 — Good middle ground, but Svelte offers slightly better performance characteristics for this use case.

**Decision:** Use Svelte 5.

**Rationale:** Svelte's compile-time approach and the new Runes reactivity model in version 5 provide excellent performance and small bundle sizes, aligning perfectly with Tauri's lightweight philosophy.

**Consequences:**
- Fast, responsive UI.
- Smaller frontend bundle.
- Smaller developer pool compared to React, but sufficient for the project.

**Reversibility:** Moderate. Rewriting the frontend is significant work but doesn't affect the Rust backend.

---

### ADR-004: Backend Language

**Date:** 2026-09-11
**Status:** Accepted

**Context:** The core application logic, database interactions, and system watchers need a robust language.

**Problem:** Selecting a backend language that provides system-level access and high performance.

**Options Considered:**
1. Rust — Native to Tauri, extremely safe, high performance, excellent OS APIs.
2. Python — Easy to write watchers, but requires bundling an interpreter, leading to large binaries and higher memory use (used in the original superseded plan).
3. Go / C++ — Fast, but less integrated with Tauri's ecosystem.

**Decision:** Use Rust.

**Rationale:** Rust is the native backend language for Tauri. It offers unmatched performance, memory safety, and direct access to native OS APIs needed for window watching without the overhead of an interpreter.

**Consequences:**
- High performance and safety.
- Steeper learning curve for contributors not familiar with Rust.
- Seamless integration with Tauri.

**Reversibility:** Difficult. The backend logic is the core of the application.

---

### ADR-005: Database

**Date:** 2026-09-11
**Status:** Accepted

**Context:** Tendly needs to store time-series window tracking data and classifications locally.

**Problem:** Choosing a local storage solution that is fast, reliable, and easily queryable.

**Options Considered:**
1. SQLite (via rusqlite) — Ubiquitous, serverless, reliable, supports complex queries.
2. JSON/Flat files — Easy to implement, but scales poorly with time-series data and lacks complex querying.
3. RocksDB / Sled — Key-value stores, very fast, but less flexible for analytical queries (e.g., weekly summaries).

**Decision:** Use SQLite via `rusqlite`.

**Rationale:** SQLite is the industry standard for local, relational data. It handles the anticipated volume of data (window switches every few seconds) efficiently and allows for complex analytical queries required for reporting.

**Consequences:**
- Robust data integrity.
- Easy to inspect data manually.
- Requires defining a schema and managing migrations.

**Reversibility:** Moderate. Migrating data to a different embedded DB is possible but requires writing migration tools.

---

### ADR-006: License

**Date:** 2026-09-11
**Status:** Accepted

**Context:** Tendly is an open-source project that needs a license balancing openness with commercial viability for potential future sync services.

**Problem:** Selecting an open-source license.

**Options Considered:**
1. MIT / Apache 2.0 — Highly permissive.
2. GPLv3 / AGPL — Highly restrictive copyleft.
3. MPL-2.0 (Mozilla Public License 2.0) — Weak copyleft (file-level).

**Decision:** Use MPL-2.0.

**Rationale:** This was an existing decision. MPL-2.0 ensures that modifications to existing Tendly files must be shared under the same license, protecting the core open-source value, while allowing larger proprietary works to link against it if needed. It hits the sweet spot for a modern open-source desktop app.

**Consequences:**
- Encourages contribution back to the core project.
- Slightly more complex compliance than MIT.

**Reversibility:** Difficult. Changing licenses requires consent from all contributors or complete rewrites.

---

### ADR-007: Classification Architecture

**Date:** 2026-09-11
**Status:** Accepted

**Context:** Raw window titles and executable names need to be categorized into Focus, Neutral, or Drift.

**Problem:** Pure rules are too rigid and require constant manual updating. Pure AI is too resource-intensive and slow for every single window switch.

**Options Considered:**
1. Pure Rules — Fast, but brittle.
2. Pure AI — Accurate, but slow and battery-draining.
3. 3-Tier Cascade — Rules -> Local AI -> Optional Cloud AI.

**Decision:** Implement a 3-tier cascade architecture.

**Rationale:** Tier 1 (Rules) handles 80% of common cases instantly with zero overhead (e.g., Code = Focus). Tier 2 (Local AI) handles the nuance (e.g., distinguishing a helpful YouTube tutorial from entertainment). Tier 3 (Cloud AI) is an optional fallback for users who cannot run local models.

**Consequences:**
- Best balance of performance, accuracy, and resource usage.
- More complex implementation than a single approach.

**Reversibility:** Moderate. The architecture is modular, so tiers can be added or removed.

---

### ADR-008: Default Local Model

**Date:** 2026-09-11
**Status:** Accepted

**Context:** The Tier 2 AI classification needs a reliable, small, and fast local LLM.

**Problem:** Selecting the default model that balances size (VRAM usage), speed, and instruction-following capabilities.

**Options Considered:**
1. qwen2.5:3b — Excellent instruction following, fast, handles JSON output well.
2. llama3.2:3b — Very good, but qwen slightly outperformed in structured extraction tests in similar contexts.
3. phi4-mini — Good alternative, but ecosystem support in Ollama often lags slightly behind mainstream Qwen/Llama releases.

**Decision:** Use `qwen2.5:3b` as the default local model.

**Rationale:** The 3B parameter size is the sweet spot for modern laptops, requiring minimal VRAM. Qwen 2.5 has shown exceptional performance for structured JSON outputs and classification tasks relative to its size.

**Consequences:**
- High classification accuracy with low resource usage.
- Users need ~2-3GB of RAM/VRAM to run the model comfortably.

**Reversibility:** Easy. The model string is just configuration; users can change it, and we can update the default in a patch.

---

### ADR-009: Ollama as AI Runtime

**Date:** 2026-09-11
**Status:** Accepted

**Context:** The application needs a way to execute the local LLM.

**Problem:** Deciding whether to embed the AI inference engine or rely on an external service.

**Options Considered:**
1. Ollama (External service) — Requires separate installation, but handles model management, GPU acceleration, and exposes a clean API.
2. Embedded `llama.cpp` — No extra installation, but massively increases binary size and complicates cross-platform GPU support.
3. Bundled ONNX models — Complex to update models, limited ecosystem.

**Decision:** Rely on Ollama as the AI runtime.

**Rationale:** Ollama abstracts away the immense complexity of cross-platform hardware acceleration (CUDA, Metal, ROCm). Developers are already likely to have Ollama installed. It keeps Tendly's core light and avoids reinventing the wheel.

**Consequences:**
- User must install and run Ollama separately.
- Simplifies Tendly's codebase significantly.

**Reversibility:** Moderate. Switching to embedded `llama.cpp` would require major architectural changes to the backend.

---

### ADR-010: MVP Platform Scope

**Date:** 2026-09-11
**Status:** Accepted

**Context:** Initial development needs focus to reach a viable product quickly.

**Problem:** Cross-platform window watching is notoriously difficult due to varying OS APIs (X11, Wayland, Win32, macOS Accessibility).

**Options Considered:**
1. Cross-platform from Day 1 — High risk, slow time to market.
2. Windows-first — Largest market, but developers are heavily represented on Linux/macOS.
3. Linux-first — Solves the hardest problem (fragmentation) early and appeals strongly to privacy-conscious developers.

**Decision:** MVP (v0.1) will be Linux-first (X11 & wlroots Wayland).

**Rationale:** Targeting a specific platform allows the team to validate the core UI, database, and classification logic without getting bogged down in platform-specific quirks. Linux users are highly tolerant of MVP software and are the perfect target for a privacy-first, local AI tool.

**Consequences:**
- Faster time to initial release.
- Temporarily excludes Windows/macOS users.

**Reversibility:** Easy. Subsequent phases (v0.6, v0.7) explicitly add Windows and macOS support.

---

### ADR-011: Desktop-First over CLI-First

**Date:** 2026-09-11
**Status:** Accepted

**Context:** The previous superseded roadmap proposed a CLI-first approach.

**Problem:** Determining the primary interface for the application.

**Options Considered:**
1. CLI-first — Easier to build, but hard to visualize timelines and less appealing for constant background monitoring.
2. Desktop (GUI) application — Better visualization, easier onboarding, system tray support.

**Decision:** Build a Desktop GUI application from v0.1.

**Rationale:** Time tracking inherently requires visual feedback (timelines, charts, alerts). A CLI is insufficient for the rich interactive experience needed for reviewing and categorizing daily activity.

**Consequences:**
- Requires building a UI framework immediately.
- Discards previous CLI-centric plans.

**Reversibility:** Difficult. The entire architecture is built around the Tauri desktop model.

---

### ADR-012: In-Process Watchers for MVP

**Date:** 2026-09-11
**Status:** Accepted

**Context:** The application needs to monitor active windows.

**Problem:** Deciding how the watcher code interacts with the main application.

**Options Considered:**
1. In-process (Rust threads) — Lower overhead, easier to package, direct access to database.
2. Out-of-process (Polyglot subprocesses) — More flexible (e.g., python scripts), but complex IPC, deployment, and higher overhead.

**Decision:** Use Rust in-process watchers for the MVP.

**Rationale:** Keep the architecture simple and performant for the MVP. Managing separate processes introduces unnecessary complexity and potential points of failure.

**Consequences:**
- Watchers must be written in Rust.
- Simpler deployment and lower resource usage.

**Reversibility:** Moderate. A plugin architecture (v0.8+) may eventually introduce out-of-process watchers.

---

### ADR-013: Internal Communication Architecture and Local HTTP API

**Date:** 2026-09-11
**Status:** Accepted

**Context:** The original repository architecture proposed a loopback HTTP API (`axum` on `127.0.0.1`) for communication between watchers, storage, and the UI. With the approved shift to a native Tauri 2.x desktop application and in-process Rust watchers, the need for an internal HTTP server was re-evaluated.

**Problem:** Determining the communication mechanism between the Svelte frontend, the Rust backend, and future integrations.

**Options Considered:**
1. Architecture A: Svelte GUI -> Local HTTP API (axum) -> Application server -> SQLite.
2. Architecture B: Svelte GUI -> Tauri IPC -> Rust application core -> SQLite (in-process).
3. Hybrid: Tauri IPC for internal GUI, loopback HTTP server always running for hypothetical external watchers.

**Decision:** Adopt **Architecture B**. All internal communication between the Svelte GUI and Rust core uses direct Tauri IPC. In-process Rust watchers communicate with the aggregation pipeline via internal channels (`tokio::sync::mpsc`). The core application does not start or depend on an internal HTTP server for the MVP.

**Rationale:** Running an HTTP server in the core desktop app introduces an unnecessary local attack surface (open port on `127.0.0.1`), requires authentication tokens, adds serialization overhead, and introduces extra crate dependencies. For the MVP, all watchers and UI components are in-process. When external integrations (like browser extensions in v0.3) are built, standard OS mechanisms like Browser Native Messaging (`stdio` JSON streams) or an opt-in local adapter can be introduced.

**Consequences:**
- Zero open network ports on the local machine for the core desktop application.
- Lower memory and CPU footprint.
- Type-safe, asynchronous communication via Tauri IPC.
- Deferral of HTTP networking code until external integrations actually require it.

**Reversibility:** Moderate. If a third-party plugin ecosystem demands a loopback HTTP API in future phases (e.g., v0.8), an adapter crate can be added without altering the internal Tauri IPC or storage core.

---

### ADR-014: Block Duration

**Date:** 2026-09-11
**Status:** Accepted

**Context:** Window switches happen constantly. Recording and analyzing every 5-second switch is noisy and unhelpful.

**Problem:** Determining the minimum time resolution for a "block" of activity.

**Options Considered:**
1. 1-minute blocks — High granularity, but still somewhat noisy.
2. 3-minute blocks — Smooths out rapid switching (e.g., checking docs for 10 seconds while coding) without losing the overall context.
3. 5-minute blocks — Too coarse, might miss meaningful context switches.

**Decision:** Use a 3-minute default block duration for categorization and UI visualization.

**Rationale:** Three minutes is short enough to capture genuine context switches but long enough to absorb micro-interruptions. If a user spends 2.5 minutes in VS Code and 30 seconds on a browser, the block remains "Focus".

**Consequences:**
- Smoother timeline visualization.
- Reduces database size and AI API calls (we only classify the dominant activity of the block).

**Reversibility:** Easy. Can be made a user-configurable setting.

---

### ADR-015: Privacy-First Architecture

**Date:** 2026-09-11
**Status:** Accepted

**Context:** The application monitors everything a user does on their computer.

**Problem:** Establishing trust regarding highly sensitive personal data.

**Options Considered:**
1. Cloud-sync by default — Easy cross-device, but massive privacy concerns and liability.
2. Local-only default, opt-in cloud AI — Ensures user data never leaves the machine unless explicitly authorized.

**Decision:** Tendly will be local-only by default. Any use of cloud services (like BYOK API keys for AI) must be strictly opt-in.

**Rationale:** Developers are highly sensitive to telemetry and data harvesting. A core value proposition of Tendly is that it does not spy on you. Local SQLite and local Ollama ensure this promise is kept.

**Consequences:**
- Builds immense user trust.
- Prevents us from using centralized analytics to improve the product.
- Syncing across devices becomes a harder technical problem (requires end-to-end encryption, planned for future).

**Reversibility:** Very Difficult. Reversing this would destroy user trust and violate the core ethos of the project.

---

### ADR-016: Application State and Mutex Concurrency Strategy in Tauri

**Date:** 2026-09-11
**Status:** Accepted

**Context:** Phase 1 establishes the production Rust application core that must share database connections, runtime configuration, and tracking states between Tauri IPC commands and background watcher tasks.

**Problem:** Selecting an in-process concurrency and state management architecture in Rust that guarantees thread safety, avoids race conditions, and maintains predictable performance.

**Options Considered:**
1. Global static `lazy_static` / `once_cell` singletons - Unidiomatic in modern Rust, difficult to isolate in unit tests.
2. Tauri managed state (`app.manage(AppState)`) with `Arc<Mutex<T>>` - Type-safe, idiomatic Tauri pattern, clean dependency injection.
3. Message-passing actor model for all state - Overengineered for initial phase, introduces latency for simple status reads.

**Decision:** Use Tauri's native state management (`app.manage(AppState)`) combined with an `Arc<Mutex<Connection>>` for SQLite (configured with WAL mode, `NORMAL` synchronous mode, and a 5000ms busy timeout), and `Arc<Mutex<TrackingState>>` for session status.

**Rationale:** Tauri's `State<'_, AppState>` injection allows commands to cleanly access shared state. SQLite's WAL mode permits concurrent readers with a serialized writer, protected by the Mutex and connection busy handler.

**Consequences:**
- Deterministic, panic-free state access across asynchronous IPC invocations.
- Testable: `DatabaseManager::open_in_memory()` can be instantiated in test fixtures without global state pollution.
- Thread contention is negligible given 3-minute block aggregation intervals.

**Reversibility:** Moderate. Can be upgraded to connection pooling (`r2d2`) if concurrent write volume increases in later phases.

---

### ADR-017: Privacy-Preserving Structured Logging Policy

**Date:** 2026-09-11
**Status:** Accepted

**Context:** Desktop applications require diagnostic logging for error analysis and troubleshooting. However, Tendly monitors user window titles and system activity.

**Problem:** How to provide useful operational logs without accidentally recording sensitive personal activity, proprietary code names, URLs, or secrets.

**Options Considered:**
1. Unfiltered logging with disclaimer - Dangerous; users routinely paste logs into GitHub issues.
2. No logging at all - Makes debugging application crashes or platform permission failures nearly impossible.
3. Strict structural logging policy - Only log lifecycle events, component status, and sanitized error categories; explicitly prohibit logging window titles, URLs, file paths, and prompt contents.

**Decision:** Adopt a strict structural logging policy using `tracing` and `tracing-subscriber`.

**Rationale:** The logging policy is enforced at the architecture level. Default logs are guaranteed safe to share publicly in bug reports. Raw activity strings (`RawEvent.title`, `RawEvent.url`, `RawEvent.app`) are never logged.

**Consequences:**
- User activity remains private even if logs are shared or inspected.
- Diagnostic logs focus on component state transitions and error codes rather than user content.

**Reversibility:** Low. Privacy guarantees are foundational to Tendly's trust model.

---

### ADR-018: Linux Activity Capture Architecture: Hybrid Event-Driven X11 and Protocol-Based Wayland

**Date:** 2026-09-12
**Status:** Accepted

**Context:** Phase 2 implements active window and title tracking on Linux desktop environments. Linux desktop display servers are split between native X11 and fragmented Wayland compositors.

**Problem:** How to observe active desktop application and window title changes with minimal latency, low CPU/battery consumption, and resilience against server disconnects or compositor differences.

**Options Considered:**
1. High-frequency polling (e.g. 500ms sleep loop querying `xprop` via subprocess) - High CPU overhead, spawns hundreds of subprocesses per minute, causes battery drain.
2. Pure event-driven without heartbeat - Risk of missed window unmap/destruction events causing permanent tracking stall.
3. Hybrid architecture: Event-driven property change subscriptions (`PropertyChangeMask`) combined with a 2-second heartbeat fallback and exponential backoff recovery.

**Decision:** Implement the hybrid architecture for X11 using `x11rb` protocol crate, and dedicated socket/protocol streaming for supported Wayland compositors (Hyprland and wlroots).

**Rationale:** The hybrid model provides sub-millisecond response to active window transitions while guaranteeing that dropped events or destroyed window races are self-healed within 2 seconds. Connection errors recover automatically via backoff without terminating the host application.

**Consequences:**
- Sub-millisecond latency on window transitions.
- Extremely low CPU utilization (< 0.1%).
- Graceful degradation when running under unsupported Wayland compositors (e.g. GNOME Mutter without shell extension).

**Reversibility:** Moderate. The watcher interface is isolated behind the `ActivityWatcher` trait.

---

### ADR-019: In-Memory Event Deduplication and Inactivity Transition Semantics

**Date:** 2026-09-12
**Status:** Accepted

**Context:** Watchers observe state continuously. If every observation were written directly to SQLite, the database would rapidly accumulate hundreds of thousands of redundant rows, causing excessive disk I/O and storage bloat.

**Problem:** Defining precise `RawEvent` generation semantics and deduplication rules to minimize disk writes while preserving time continuity for 3-minute time-block aggregation.

**Options Considered:**
1. Write every observed tick to SQLite - Massive storage bloat (> 50 MB/day), high disk wear.
2. Only write on state change (app or title change) - If a developer writes code in one window for 3 hours, no intermediate records exist, complicating crash recovery or mid-day time-block classification.
3. State-change driven persistence with a 60-second checkpoint interval and distinct AFK transition events.

**Decision:** Adopt state-change driven persistence with a 60-second checkpoint interval and distinct entry/exit AFK transition events.

**Rationale:**
- An event is persisted immediately when `app` or `title` changes.
- Consecutive events with identical `app` and `title` within 60 seconds are suppressed in memory.
- If the user works uninterrupted in the same window, a checkpoint record is written every 60 seconds.
- When the user is inactive beyond the idle threshold (default 300s), exactly one AFK event is written (`title = "afk"`). When the user returns, exactly one Active event is written (`title = "active"`). Continuous idle does not generate duplicate records.

**Consequences:**
- SQLite database writes drop from 1,200 writes/hour to approximately 20 to 60 writes/hour.
- Total database growth remains < 500 KB per week of typical development work.
- Time continuity is preserved for downstream time-block aggregation.

**Reversibility:** High. Deduplication parameters are configurable in `DeduplicationFilter`.

---

### ADR-020: Model C Hybrid Segment-First Epoch Bucketing for Time-Block Aggregation

**Date:** 2026-09-12
**Status:** Accepted

**Context:** Phase 3 requires converting raw OS observations (`RawEvent`) into meaningful, non-overlapping `TimeBlock` records for user review and downstream AI classification.

**Problem:** How to group heterogeneous, non-uniform window focus durations and AFK periods into time blocks without producing fragmented micro-blocks or artificial, oversized blocks.

**Options Considered:**
1. Model A: Pure fixed-time epoch buckets (rigid 3-minute windows into which raw events are sliced). Simple, but fragments continuous flow when an activity spans across bucket boundaries.
2. Model B: Variable-length activity segments (every window switch starts a new block). Preserves exact switch boundaries, but produces noisy micro-blocks (e.g. 4 seconds) and massive multi-hour blocks that defy uniform AI classification.
3. Model C: Hybrid segment-first epoch bucketing. Reconstructs contiguous homogeneous `ActivitySegment`s first, then maps segments into standardized 3-minute epoch intervals (`[T, T + 180s)`) with plurality attribution.

**Decision:** Adopt Model C (Hybrid Segment-First Epoch Bucketing).

**Rationale:**
- Preserves accurate contiguous session context across switches.
- Produces clean, uniform 3-minute blocks (`TimeBlock`) essential for consistent downstream AI classification and regular UI timelines.
- Attributes block metadata using plurality of active duration within the epoch interval.

**Consequences:**
- Timeline visualization is regular and deterministic.
- Small sub-minute switches during a long coding session do not pollute the primary classification of the block.
- Downstream classification in Phase 4 operates on predictable 3-minute units.

**Reversibility:** Moderate. Bucketing duration (180s) and aggregation algorithms are isolated in the `processing` module.

---

### ADR-021: Canonical Raw Event Authority, Unknown Gap Handling, and Idempotent Rebuilding

**Date:** 2026-09-12
**Status:** Accepted

**Context:** Time-block derivation algorithms may evolve over time (e.g., improved heuristics or refined threshold values). The system must allow recalculation of historical blocks without data loss or duplication. In addition, system sleep, suspend, or crashes can create unobserved time gaps.

**Problem:** Establishing data ownership, preventing misclassification of unknown gaps as active work or AFK, and ensuring idempotent reprocessing of historical records.

**Options Considered:**
1. Store only derived `TimeBlock`s and discard `RawEvent`s - Destroys the ability to re-aggregate or improve classification retrospectively.
2. Assume unobserved time gaps are active work or AFK - False assumption; an unobserved gap during laptop sleep is neither active keyboard work nor normal desk idle.
3. Establish `RawEvent` as the canonical source of truth, classify unobserved gaps > 180s as `ActivityType::Unknown`, and generate deterministic UUID v5 IDs for idempotent replacement.

**Decision:**
1. `RawEvent` is the immutable, canonical source of truth. `TimeBlock`s are purely derived.
2. Unobserved time gaps greater than 180s without watcher heartbeats are capped after a 60-second grace period and explicitly typed as `ActivityType::Unknown`. Unknown time is never converted to active work.
3. Each `TimeBlock` ID is deterministically generated via UUID v5 derived from its start timestamp (`timeblock:{start_ms}`). SQLite enforces `UNIQUE(start_ms)` with `ON CONFLICT(start_ms) DO UPDATE`.

**Rationale:**
- Complete auditability and future-proofing: Users can reprocess their entire history whenever aggregation rules or AI models are updated.
- Truthfulness: Tendly never fabricates user activity during system sleep or suspend.
- Idempotency: Reprocessing identical raw events 100 times produces identical blocks with zero duplicates.

**Consequences:**
- Historical reprocessing is instantaneous, safe, and repeatable.
- Sleep/suspend periods are accurately recognized as unobserved time.
- Storage requirement for `raw_events` remains lightweight (< 500 KB per week).

**Reversibility:** Very Difficult. Reversing raw event immutability would compromise data integrity.

---

### ADR-022: Classification Staleness Invalidation, User Override Scoping, and On-Demand Block Composition

**Date:** 2026-09-12
**Status:** Accepted

**Context:** During the Phase 3 review gate, two architectural concerns were audited: (1) what happens when historical reprocessing modifies a block's dominant application or title after it has been classified, and (2) whether plurality attribution discards sub-block detail that downstream classification or timeline views require.

**Problem:**
1. Blindly preserving existing classifications via `COALESCE` when `dominant_app` changes causes stale, false classifications (e.g., Slack retaining a "Focus" label previously assigned to VS Code).
2. Slicing complex multi-tasking into a single plurality winner might hide secondary activities from future classifiers or timeline views.

**Options Considered:**
1. Blindly retain previous classification on conflict (`COALESCE`) - Rejected; causes stale classifications when raw history changes.
2. Denormalize segment breakdown as JSON within each `TimeBlock` - Adds redundant storage (~28 KB/day) and duplicates data already indexed in `raw_events`.
3. Invalidate automatic classification on content change, scope user overrides by app, and derive sub-block composition on demand from `raw_events`.

**Decision:**
1. On `TimeBlock` conflict resolution, automatic classifications (`classification`, `category`, `confidence`, `classified_by`) are preserved if and only if `dominant_app`, `dominant_title`, and `activity_type` remain identical. If any of these change, automatic classifications are reset to `NULL`.
2. A `user_override` is preserved only if `dominant_app` remains identical. If the dominant application changed, the user override is invalidated (`NULL`) to avoid applying manual decisions to different software.
3. Sub-block composition is not denormalized in `TimeBlock`. Instead, `ActivityProcessor::get_block_composition(start_ms, end_ms)` derives the exact contiguous `ActivitySegment`s on demand directly from `raw_events` with sub-millisecond query latency (~0.02 ms).

**Rationale:**
- Prevents corrupt or stale classification state during reprocessing.
- Respects user manual overrides while preventing cross-application contamination.
- Follows the single-source-of-truth principle without data duplication or premature schema bloat.

**Consequences:**
- Historical reprocessing is provably safe and self-healing.
- Downstream classification (Phase 6/7) and timeline rendering (Phase 4) can inspect full sub-minute fidelity on demand.

**Reversibility:** Moderate. Can add cached composition if profiling indicates query overhead in future phases.

---

### ADR-023: User-Facing Activity Sessions, Deterministic Coalescing, and Timeline Presentation

**Date:** 2026-09-12
**Status:** Accepted

**Context:** In Phase 4, Tendly needs to present activity history to the user as meaningful, continuous activity sessions (e.g. "VS Code 45m") rather than a fragmented stream of sixteen identical 3-minute blocks. At the same time, the 3-minute TimeBlock architecture must remain the authoritative analytical unit for future AI/rules classification, and users must be able to inspect sub-minute secondary activity.

**Problem:**
1. Directly presenting 3-minute TimeBlocks in the UI produces visual fragmentation, high cognitive load, and a debugging-tool aesthetic.
2. Persisting merged sessions into a new SQLite database table introduces data duplication, mutable state synchronisation bugs, and cache-invalidation hazards during historical reprocessing.
3. Gaps in activity (computer suspend, tracking pause, unrecorded intervals) must not be falsely merged into giant sessions or confused with active desk idle (AFK).

**Options Considered:**
1. Mutate the analytical model to use variable-length blocks - Rejected; breaks deterministic 3-minute epoch boundaries, complicates LLM prompt token budgeting, and violates Phase 3 architecture.
2. Create a persistent `sessions` table in SQLite - Rejected; creates duplicate mutable storage, violates the single-source-of-truth hierarchy, and requires dual-write/invalidation logic.
3. Maintain `TimeBlock` as the immutable analytical storage unit in SQLite, derive `ActivitySession` purely in-memory at the presentation layer via deterministic coalescing, and reconstruct sub-minute composition on demand via `get_block_composition`.

**Decision:**
1. **Separation of Representations:**
   - Analytical unit: `TimeBlock` (fixed 3-minute epoch `[start_ms, end_ms)`, persisted in SQLite).
   - User-facing unit: `ActivitySession` (variable-duration continuous activity, derived in-memory on demand).
2. **Deterministic Coalescing Rules:** Adjacent `TimeBlock`s are merged into an `ActivitySession` if and only if:
   - They are temporally contiguous (`block[i].end_ms == block[i+1].start_ms`).
   - They share the exact same `dominant_app`.
   - They share the exact same `activity_type` (`Active`, `Afk`, or `Unknown`).
3. **Application and Type Transitions Preserved:** Any change in application (e.g., VS Code -> Slack -> VS Code) or activity type (e.g., Active -> Afk) splits sessions deterministically.
4. **Unrecorded Gap Insertion:** Gaps between recorded sessions of 3 minutes or greater (180,000 ms) are surfaced as explicit unrecorded gap sessions (`dominant_app = "unrecorded"`, `activity_type = Unknown`), cleanly separating absence of observations from active desk idle (`Afk`).
5. **On-Demand Secondary Activity Drill-Down:** The timeline surfaces secondary applications (`secondary_apps`) by calculating duration breakdown from underlying raw segments on the fly. Detailed drill-down (`get_session_details`) queries `get_block_composition` without redundant storage.
6. **Strict Half-Open Daily Querying:** Daily timelines query `[day_start_ms, day_end_ms)` where `day_end_ms` is the start of the subsequent local day. Boundaries at midnight are strictly disjoint and prevent duplicate or lost sessions.

**Rationale:**
- Preserves the purity and immutability of the analytical 3-minute aggregation pipeline.
- Zero database migrations or extra table overhead.
- Provides a clean, calm, and readable user experience tailored for developers.
- Guarantees losslessness: sub-minute precision remains available on demand.

**Consequences:**
- User-facing timeline displays continuous, readable sessions with low visual noise.
- Under-the-hood analytical fidelity is preserved for future AI classification (Phases 6-7).
- Query latency remains bounded and lightweight on indexed range lookups, without loading the entire raw event history.

**Reversibility:** High. Coalescing logic is entirely in-memory in the presentation layer and does not alter the underlying SQLite schema.

---

### ADR-024: Browser Context Acquisition, Title Parsing, and Enrichment Pipeline

**Date:** 2026-09-13
**Status:** Accepted

**Context:** In Phase 5, Tendly needs to enrich browser activity observations with browser-specific context (browser application identity, page title, URL/domain) without violating Tendly's privacy-first, local-first principles and without destabilizing the analytical 3-minute TimeBlock pipeline.

**Problem:**
1. Determining the most appropriate, reliable, and non-invasive mechanism for capturing browser context on Linux desktop environments.
2. Avoiding invasive browser extensions or native messaging hosts in the initial context capture release.
3. Preserving the single-source-of-truth invariant (no separate browser database or parallel history store).
4. Ensuring complete failure isolation: failure to extract browser context must never disrupt primary activity capture.

**Options Considered:**
1. Browser Extension + Native Messaging: Captures full URLs and tab IDs via WebExtension APIs, but requires per-browser user installation, permissions, native messaging binary manifests, and high maintenance overhead.
2. Accessibility / AT-SPI: Reads address bar text via AT-SPI D-Bus interfaces, but is fragile, sluggish, and often disabled by default in modern desktop distributions.
3. Chrome DevTools Protocol (CDP) / Remote Debugging: Requires launching browsers with specialized debugging flags; unacceptable security risk.
4. Window Title Parsing: Extracts the page title and browser identity directly from the OS window manager titles (`_NET_WM_NAME` on X11, Hyprland/Wayland socket IPC) that Tendly's watchers already capture.

**Decision:**
1. **Acquisition Mechanism:** Adopt Window Title Parsing as the primary browser context mechanism. Tendly identifies the active browser via window class (`WM_CLASS` / Hyprland class) and strips standard browser decoration suffixes (e.g., ` — Mozilla Firefox`, ` - Google Chrome`) to extract clean page titles.
2. **Deterministic Enrichment Layer:** `enrich_browser_context(app, title)` executes synchronously in the capture watcher loop and enriches `RawEvent` with structured metadata stored in `raw_json`.
3. **Single Source of Truth:** No secondary database tables or history caches are introduced. Existing columns `raw_events.raw_json` and `blocks.dominant_url` accommodate the enrichment without schema migrations.
4. **Context Switch Fidelity:** Because segment reconstruction keys on `(app, title)`, navigating between tabs or pages within the same browser naturally generates distinct `ActivitySegment`s.
5. **URL Normalization Rules:** Implemented deterministic URL normalization (WHATWG parser, port stripping, credential stripping, and removal of `utm_*`, `fbclid`, `gclid` tracking parameters) in preparation for future extension feeds.
6. **Graceful Degradation:** If title parsing does not recognize a pattern or encounters an atypical window title, the full window title is preserved and tracking continues unaffected.

**Rationale:**
- Zero user setup or extension installation required.
- Zero external dependencies or invasive permissions.
- Rock-solid stability: cannot crash or freeze the user's browser.
- Respects privacy: no local browsing history harvesting.

**Consequences:**
- Users immediately see clean page titles and domain context in their activity timeline.
- The pipeline architecture is forward-compatible with future browser extension feeds (via `RawEventSource::Browser` and `BrowserContext`).
- Full URLs with query parameters remain unavailable for window-title-captured events (a documented Phase 5 trade-off).

**Reversibility:** High. Enrichment logic is isolated to the capture and reconstruction layer and does not alter database schema invariants.

