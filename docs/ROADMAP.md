# Tendly Roadmap

This document outlines the product roadmap for Tendly. Our approach focuses on delivering the smallest useful thing first, building around validated dependencies and user value. Each phase produces a verifiable outcome before moving to the next.

## v0.1 — Linux Desktop MVP ("Where did my time go?")

**Verifiable outcome:** A developer on Linux (X11 or wlroots Wayland) can install Tendly, track their work for a day, and see a meaningful timeline of Focus/Neutral/Drift blocks.

**Features:**
- Tauri desktop application with system tray.
- X11 active window watcher (in-process, Rust).
- wlroots Wayland watcher (wlr-foreign-toplevel protocol).
- AFK/idle detection.
- SQLite local storage.
- Rules-based classification (Tier 1 only — no AI required).
- Day timeline view (color-coded Focus/Neutral/Drift blocks).
- Daily summary (total hours, F/N/D breakdown).
- User can override classifications.
- Pause/resume tracking.
- Delete all data.
- First-run onboarding.
- Data inspection panel.
- AppImage packaging.

**Dependencies:** None (foundation phase).
**Platform scope:** Linux (X11 & wlroots Wayland).

## v0.2 — Local AI Classification

**Verifiable outcome:** Classification accuracy jumps from ~60% (rules) to ~85%+ with Ollama.

**Features:**
- Ollama integration (qwen2.5:3b).
- Structured output via `format` field.
- User profile (role + current goals).
- Low-confidence flagging ("Needs Review").
- BYOK (Bring Your Own Key) cloud option (OpenRouter/Groq).
- Classification settings UI.

**Dependencies:** v0.1 (Base tracking and UI).
**Platform scope:** Linux (X11 & wlroots Wayland).

## v0.3 — Browser Context

**Verifiable outcome:** Browser activity correctly distinguished (e.g., Stack Overflow = Focus, BuzzFeed = Drift).

**Features:**
- Chrome extension (Manifest V3).
- Firefox extension (WebExtensions).
- Native Messaging to loopback API.
- Domain-level tracking (not full URLs by default).
- Extension privacy controls.

**Dependencies:** v0.1 (Data storage) and v0.2 (AI classification handles new browser context).
**Platform scope:** Linux (Chrome/Firefox).

## v0.4 — Visualization & Review

**Verifiable outcome:** User can understand weekly patterns and trends.

**Features:**
- Weekly heatmap.
- Category breakdown charts.
- Trend analysis (Focus ratio over time).
- Data export (JSON/CSV).

**Dependencies:** v0.1 (Data structure) and v0.2 (Accurate classification data).
**Platform scope:** Linux.

## v0.5 — Reflection & Intentions

**Verifiable outcome:** User can set daily intentions and compare against actual activity.

**Features:**
- Morning intention check-in.
- Drift-streak reflection prompts.
- End-of-day review.

**Dependencies:** v0.4 (Visualization UI).
**Platform scope:** Linux.

## v0.6 — Windows Support

**Verifiable outcome:** Windows developer can use Tendly with full functionality.

**Features:**
- Win32 watcher (GetForegroundWindow + SetWinEventHook).
- UWP app resolution (ApplicationFrameHost).
- Windows idle detection + session lock.
- MSI/NSIS installer.

**Dependencies:** v0.5 (Mature core).
**Platform scope:** Windows 10/11.

## v0.7 — macOS Support

**Verifiable outcome:** macOS developer can use Tendly.

**Features:**
- macOS watcher (NSWorkspace + Accessibility API).
- macOS permission flow.
- DMG packaging + notarization.

**Dependencies:** v0.6 (Cross-platform architecture proven).
**Platform scope:** macOS.

## v0.8+ — Future

**Features/Scope:**
- KDE Plasma 6 Wayland support (KWin D-Bus).
- GNOME Wayland support (Shell Extension).
- Plugin/watcher API.
- Graduated friction (opt-in interventions).
- Encrypted multi-device sync (AGPL-3.0 component).
- Auto-update functionality.

**Dependencies:** v0.7 (Full initial cross-platform support).
**Platform scope:** All platforms.
