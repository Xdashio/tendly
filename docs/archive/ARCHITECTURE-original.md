# Architecture

## Design goals (from the problem statement)

1. Local-first — nothing leaves the machine unless the user opts in.
2. Extensible — a contributor can add a new OS/editor/browser watcher
   without touching core code.
3. Cross-platform from the start — Linux, Windows, macOS are all
   first-class watchers behind one interface, not three separate apps.
4. Small, swappable AI layer — classification must work with a local
   model, a BYOK cloud model, or (later) nothing at all (raw mode).

## Layer overview

```
 ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐
 │  Watchers   │──▶│  Local API  │──▶│   Storage   │──▶│Classification│
 │ (per-OS/app)│   │ (loopback)  │   │  (SQLite)   │   │  (AI layer)  │
 └─────────────┘   └─────────────┘   └─────────────┘   └──────┬──────┘
                                                                │
                                                                ▼
                                                         ┌─────────────┐
                                                         │      UI     │
                                                         │ (CLI / web) │
                                                         └─────────────┘
```

Each arrow is a boundary a contributor can work on in isolation, as long as
they respect the schema/interface on either side of it.

## 1. Capture layer — Watchers

A **Watcher** is a small, independent program that observes one source of
activity (active window, browser tab, editor session, etc.) and emits
**Raw Activity Events** to the Local API. Watchers know nothing about
storage, classification, or the UI — only about how to observe their one
thing and how to POST an event.

This mirrors ActivityWatch's own "watcher" model deliberately — it's a
proven pattern in this exact space, and keeping it recognizable lowers the
bar for contributors who've used or built for ActivityWatch before.

### Watcher contract

A watcher must:

- Run as its own process (any language — Python, Go, Rust, a shell script,
  whatever fits the platform).
- POST a JSON event to `http://127.0.0.1:<port>/api/events` (loopback only
  — never bound to a non-local interface) whenever the observed state
  changes, or on a short poll interval (recommended: 1–5s poll, but only
  emit on change to avoid flooding storage).
- Never store data itself — storage is the Local API's job.
- Degrade silently if its OS-level permission isn't granted (e.g., macOS
  Accessibility permission denied) rather than crashing the whole app.

### Planned watchers (v1.0 scope, per problem statement)

| Watcher | Platform | Mechanism |
|---|---|---|
| `watcher-x11` | Linux (X11) | `xprop`/`wmctrl` or direct Xlib bindings for active window + title |
| `watcher-wayland` | Linux (Wayland) | `wlr-foreign-toplevel-management` protocol where the compositor supports it (GNOME's restrictions are a known gap — see spike notes) |
| `watcher-windows` | Windows | `GetForegroundWindow` + `GetWindowText` via `pywin32` or raw `ctypes` |
| `watcher-macos` | macOS | `NSWorkspace.frontmostApplication` + Accessibility API for window title, via `pyobjc` or a small Swift helper |
| `watcher-browser` | Cross-platform | Browser extension (Chrome/Firefox) posting active tab title + URL to the Local API |
| `watcher-afk` | Cross-platform | Idle/away detection via OS idle-time APIs |

Each lives in its own directory under `src/watchers/`, so a contributor
adding, say, a Vim/Neovim watcher never touches the X11 or storage code.

## 2. Local API — the decoupling point

A lightweight HTTP server bound to `127.0.0.1` only, with a minimal
surface:

- `POST /api/events` — a watcher submits one Raw Activity Event.
- `GET /api/events?since=<ts>` — UI/classification layer reads events.
- `GET /api/blocks?since=<ts>` — read classified blocks (see below).
- `GET /api/health` — liveness check.

**Why an API instead of watchers writing directly to SQLite:** it lets
watchers be written in any language without a SQLite driver dependency,
keeps SQLite writes single-threaded and safe, and matches the extensibility
goal — a third-party watcher never needs to know the storage schema, only
this HTTP contract.

## 3. Storage layer

SQLite, single file, lives in the user's local app-data directory. Two core
tables:

### `raw_events`

The unmodified record of what a watcher observed.

```sql
CREATE TABLE raw_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  watcher_id TEXT NOT NULL,       -- e.g. "watcher-x11"
  timestamp INTEGER NOT NULL,     -- unix epoch, ms
  app TEXT,                       -- e.g. "firefox", "code"
  title TEXT,                     -- window/tab title
  url TEXT,                       -- browser watcher only, nullable
  platform TEXT NOT NULL,         -- "linux" | "windows" | "macos"
  raw_json TEXT                   -- full original payload, for forward-compat
);
```

### `blocks`

Raw events are bucketed into fixed-length blocks (default 3 minutes,
matching the interval Drifty and similar tools settled on) before
classification, so the AI layer isn't invoked per-event.

```sql
CREATE TABLE blocks (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  start_ts INTEGER NOT NULL,
  end_ts INTEGER NOT NULL,
  dominant_app TEXT,
  dominant_title TEXT,
  classification TEXT,            -- "focus" | "neutral" | "drift" | NULL (unclassified)
  category TEXT,                  -- e.g. "coding", "communication" — see classification layer
  confidence REAL,
  classified_by TEXT              -- "local-model" | "byok:<model>" | "rules" | NULL
);
```

Raw events are kept (not discarded after bucketing) so re-classification —
e.g. after the user edits their profile, or a better local model ships —
never requires re-capturing data.

### Event schema note

`raw_json` exists specifically so a new watcher can include extra
platform-specific fields (e.g., a future editor watcher including file
path and language) without a migration — the core schema stays stable,
extra data rides along and can be mined later.

## 4. Classification layer

Takes unclassified blocks and produces a Focus/Neutral/Drift label plus a
category. Three interchangeable backends, selected by the user:

- **Local model** (default, free, private) — a small model run via Ollama.
  Input: block's app/title/URL + a rolling window of recent blocks +
  the user's editable `profile.md` (role, current goals, and
  patterns learned from the user's own corrections over time).
  Output: `{classification, category, confidence}`.
- **BYOK cloud** (optional, opt-in) — same prompt/output contract, routed
  to OpenRouter or a directly configured provider using the user's own key.
  Exists for users who want faster or higher-quality classification and
  don't mind sending activity data off-device.
- **Rules-only / off** (fallback, always available) — simple pattern
  matching on app name (e.g. a user-maintained allow/deny list) when no AI
  backend is configured. Ensures the tool is still useful with zero setup
  and zero dependencies, matching the "free forever, no cloud requirement"
  constraint even in the degraded case.

The classifier is a pluggable interface
(`classify(block, context) -> ClassificationResult`) specifically so
swapping in a better local model later, or letting a contributor add a new
backend, doesn't touch storage or UI code.

## 5. Reflection & coaching layer (sits above classification)

Per the behavioral research reviewed before building this (see the
[license/problem-statement discussion history] — reflection and
intention-setting outperform passive dashboards and alarms):

- **Intention check-in** — optional, dismissible prompt at the start of a
  session: "what are you working on?" Stored alongside blocks so it can
  later be compared to what was actually observed.
- **Drift-streak reflection** — after N consecutive Drift blocks, a
  reflection prompt, not an alarm: "you've drifted for the last 12 minutes
  — still on track?"
- **Graduated, opt-in friction** — only if the user explicitly enables it:
  a short delay before opening a flagged app/site. Hard blocking or
  auto-closing tabs is a clearly-labeled advanced opt-in, never a default.

This layer is intentionally separate from classification — classification
just labels what happened; this layer decides what (if anything) to do
about a pattern of Drift, and is the most "opinionated" part of the system,
so keeping it isolated makes it the easiest layer to make configurable or
disable entirely.

## 6. UI layer

v0.1 is a CLI (`tendly status`, `tendly today`) reading from the Local API.
Later versions add a local web UI (timeline, weekly heatmap, category
breakdown) served from the same loopback API — no separate backend needed,
since the Local API already exposes everything the UI needs.

## Cross-cutting: why the Local API matters for macOS/Windows/Linux parity

Because every watcher talks to the same Local API with the same event
schema, the storage/classification/UI layers are written **once** and are
completely platform-agnostic. Adding macOS support later means writing
`watcher-macos` — it does not mean forking or conditionally branching any
of the other layers. This is the concrete mechanism behind the "macOS is a
first-class target, staged by practicality not by design" position in the
problem statement.
