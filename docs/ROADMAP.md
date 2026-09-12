# Tendly Authoritative Product Roadmap

This document outlines the product roadmap for Tendly. Our approach focuses on delivering the smallest useful thing first, building around validated dependencies and user value. Each phase produces a verifiable outcome before moving to the next.

---

## Phase 1 — Production Foundation (Complete)

**Verifiable outcome:** Tauri 2.x desktop application running on Linux with native Svelte 5 UI, IPC communication, SQLite schema, and test harness.

---

## Phase 2 — Linux Activity Capture (Complete)

**Verifiable outcome:** In-process X11 (`x11rb`) and wlroots Wayland window watchers observing active application and title, coupled with AFK idle detection and deduplicated persistence into SQLite `raw_events`.

---

## Phase 3 — Activity Processing and Time-Block Aggregation (Complete)

**Verifiable outcome:** Raw events deterministically aggregated into non-overlapping 3-minute epoch `TimeBlock`s via Model C hybrid bucketing. Idempotent history reprocessing, classification staleness invalidation, and on-demand segment composition.

---

## Phase 4 — Usable Timeline and Activity History (Current Target)

**Verifiable outcome:** A developer can review a full day of tracked activity on a clean, responsive desktop timeline that coalesces adjacent compatible blocks into meaningful work sessions while allowing drill-down into exact sub-block composition.

**Key Deliverables:**
- Chronological day timeline visualization.
- Session coalescing (e.g., merging continuous 3-minute blocks into `10:00 - 10:45 VS Code (45m)`).
- Sub-block segment composition inspection modal/drawer.
- Date navigation and historical review.
- Initial unclassified / basic application-grouped presentation.

---

## Phase 5 — Browser Context

**Verifiable outcome:** Browser activity accurately captured at the domain level (distinguishing Stack Overflow, GitHub, documentation, and distracting sites) via lightweight extensions communicating with Tendly core.

**Key Deliverables:**
- Privacy-preserving browser extension (domain-only by default).
- Native messaging IPC bridge.
- Domain context association with active window events.

---

## Phase 6 — Rules-Based Classification

**Verifiable outcome:** Fast, deterministic Tier 1 classification engine categorizing unambiguous developer activities as Focus, Neutral, or Drift with zero CPU and memory overhead.

**Key Deliverables:**
- Default rules for developer tools, communications, and media.
- User-defined classification rules editor.
- Immediate deterministic classification of 60-70% of routine blocks.

---

## Phase 7 — Local AI Classification

**Verifiable outcome:** Ambient local AI (Ollama `qwen2.5:3b`) contextually classifies ambiguous blocks based on active title, domain, user goals, and trajectory context.

**Key Deliverables:**
- In-process Ollama HTTP client with structured JSON output.
- Automatic memory unloading on idle (`keep_alive`).
- Optional BYOK cloud fallback (OpenRouter).
- Low-confidence flagging ("Needs Review").

---

## Phase 8 — User Corrections and Evaluation

**Verifiable outcome:** Developer can override any incorrect classification directly in the timeline, with automatic rule generation and accuracy evaluation against historical ground truth.

**Key Deliverables:**
- One-click classification correction in UI.
- Scoped rule generation from corrections.
- Accuracy tracking and validation test fixtures.

---

## Phase 9 — Dashboard and Reflection

**Verifiable outcome:** Developer receives meaningful daily and weekly awareness summaries, Focus/Neutral/Drift ratio analysis, and gentle reflection check-ins.

**Key Deliverables:**
- Daily summary and time distribution metrics.
- Weekly focus trend heatmaps.
- Morning intention and evening review prompts.
- Full data export (JSON/CSV) and nuclear data purge.

---

## Subsequent Platform Phases

- **Phase 10 — Windows Support:** Win32 foreground watcher, UWP resolution, Windows idle detection, NSIS installer.
- **Phase 11 — macOS Support:** NSWorkspace watcher, Accessibility API title tracking, DMG packaging and notarization.
- **Phase 12 — Extended Environments:** KDE Plasma 6 KWin D-Bus watcher, GNOME Shell extension.
