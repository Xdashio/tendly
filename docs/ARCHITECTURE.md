# System Architecture

## 1. Overview and Design Principles

Tendly is an open-source, privacy-first, local-first AI-assisted time-awareness desktop application designed primarily for software developers.

The architecture is governed by five non-negotiable principles:

1. **Local-First & Private**: All activity data is stored in a local SQLite database on the user's filesystem. No data leaves the machine unless the user explicitly configures an external cloud AI provider.
2. **Desktop-First**: Tendly is built from the ground up as a native desktop application with system tray presence, desktop notifications, and window-state observation.
3. **Resource Efficiency**: As a continuously running background utility, the application must maintain minimal idle CPU (<1%) and memory footprint (<30MB idle RAM).
4. **Resilient & Usable without AI**: The core time-tracking, activity bucketing, and rule-based classification function completely offline with zero AI dependencies. AI enhances classification but is not a prerequisite for utility.
5. **Architectural Simplicity**: Communication pathways use native in-process mechanisms (Tauri IPC and Rust channels) rather than unnecessary network overhead.

---

## 2. Core Architecture: In-Process Desktop Model (Architecture B)

Tendly implements **Architecture B**, a direct in-process desktop model using Tauri 2.x:

```
+-------------------------------------------------------------------------+
|                        TENDLY DESKTOP PROCESS                           |
|                                                                         |
|  +-------------------------------------------------------------------+  |
|  |             Frontend Layer (Svelte 5 + Tailwind CSS)              |  |
|  |                                                                   |  |
|  |  +------------------+  +------------------+  +-----------------+  |  |
|  |  |  Timeline View   |  |   Day Summary    |  | Settings / Data |  |  |
|  |  +------------------+  +------------------+  +-----------------+  |  |
|  +-----------------------------------|-------------------------------+  |
|                                      |                                  |
|                         Tauri IPC (Commands & Events)                   |
|                                      |                                  |
|  +-----------------------------------|-------------------------------+  |
|  |                 Application Core & Services (Rust)                |  |
|  |                                                                   |  |
|  |  +-------------------------------------------------------------+  |  |
|  |  |                     App Lifecycle & Tray                    |  |  |
|  |  |         (System Tray, Window Visibility, Autostart)         |  |  |
|  |  +-------------------------------------------------------------+  |  |
|  |                                   |                               |  |
|  |  +--------------------------------+----------------------------+  |  |
|  |  |                     Classification Engine                   |  |  |
|  |  |  Tier 1: Rules & Cache (In-Process)                         |  |  |
|  |  |  Tier 2: Local AI (Ollama via localhost HTTP, Optional)     |  |  |
|  |  |  Tier 3: BYOK Cloud AI (HTTPS, Optional)                    |  |  |
|  |  +--------------------------------+----------------------------+  |  |
|  |                                   |                               |  |
|  |  +--------------------------------+----------------------------+  |  |
|  |  |                      Activity Pipeline                      |  |  |
|  |  |         (Event Ingestion -> 3-Minute Block Aggregator)      |  |  |
|  |  +--------------------------------+----------------------------+  |  |
|  |                  ^                |                               |  |
|  |     Rust Channel |                v                               |  |
|  |  +---------------+--+     +-------+----------------------------+  |  |
|  |  | In-Process       |     | Storage Layer (rusqlite)           |  |  |
|  |  | Watchers (Rust)  |     |                                    |  |  |
|  |  | - Linux X11      |     | - raw_events                       |  |  |
|  |  | - Linux Wayland  |     | - blocks                           |  |  |
|  |  | - Idle / AFK     |     | - classification_rules             |  |  |
|  |  +------------------+     +------------------------------------+  |  |
|  +-------------------------------------------------------------------+  |
+-------------------------------------------------------------------------+
```

### Justification for Architecture B over Architecture A (Local HTTP API)

The original draft architecture proposed an internal loopback HTTP API (`axum` running on `127.0.0.1`) connecting the UI, watchers, and core. Following deep Phase 0 discovery, this approach was rejected for the core application:

1. **Security & Attack Surface**: Binding an open HTTP port on `127.0.0.1` exposes a local attack surface. Any unprivileged process or browser script running on the machine could attempt port scanning, request injection, or activity exfiltration unless complex token authentication is maintained.
2. **Resource & Runtime Simplicity**: Running an HTTP server incurs serialization overhead, socket lifecycle management, and additional dependency weight. Tauri's native IPC provides direct, type-safe, asynchronous communication between the Svelte frontend and Rust backend with zero network configuration.
3. **In-Process Watchers**: By implementing the MVP activity watchers directly in Rust, watchers communicate with the pipeline via zero-copy in-memory channels (`tokio::sync::mpsc`) without crossing process or network boundaries.
4. **Future Integration Stance**: External integrations (such as browser extensions in v0.3) will use standard OS-level mechanisms like Browser Native Messaging (`stdio` JSON streams) or an opt-in, explicitly bounded local interface only when those features are implemented. The core desktop app does not depend on an HTTP server.

---

## 3. Subsystem Breakdown

### 3.1. User Interface (Frontend Layer)

- **Technology**: Svelte 5, Vite, Tailwind CSS.
- **Rendering**: System native webview (WebKitGTK on Linux, WebView2 on Windows, WebKit on macOS) managed by Tauri 2.x.
- **Key Responsibilities**:
  - Render the horizontal day timeline and category breakdowns.
  - Present Focus, Neutral, and Drift state summaries.
  - Handle user overrides on misclassified blocks.
  - Provide settings for tracking pause/resume, application exclusions, and local AI preferences.
  - Provide one-click data inspection and complete data erasure.
- **Reactivity Model**: Svelte 5 Runes for fine-grained state updates without virtual DOM diffing.

### 3.2. Application Core & Tauri Bridge (Rust)

- **Technology**: Rust (stable), Tauri 2.x APIs.
- **Key Responsibilities**:
  - Manage system tray lifecycle (`TrayIconBuilder`), minimizing to tray on window close.
  - Expose IPC command handlers (`#[tauri::command]`) for querying blocks, updating settings, applying user overrides, and wiping data.
  - Coordinate graceful startup and shutdown of background watcher tasks.
  - Manage autostart configuration (`tauri-plugin-autostart`).

### 3.3. Activity Capture Pipeline (Watchers)

- **Technology**: Rust asynchronous tasks running on the `tokio` runtime.
- **Watcher Trait**:
  ```rust
  pub trait ActivityWatcher: Send + Sync {
      fn start(&self, tx: tokio::sync::mpsc::Sender<RawEvent>) -> Result<(), WatcherError>;
      fn stop(&self) -> Result<(), WatcherError>;
      fn name(&self) -> &'static str;
  }
  ```
- **MVP Implementations (Linux-First)**:
  - **X11 Watcher**: Uses `x11rb` to listen for `PropertyNotify` events on `_NET_ACTIVE_WINDOW` and extracts window titles (`_NET_WM_NAME`) and process names (`WM_CLASS`).
  - **Wayland Watcher**: Implements `wlr-foreign-toplevel-management-unstable-v1` via `wayland-client` for wlroots-based compositors (Sway, Hyprland, Wayfire).
  - **AFK / Idle Watcher**: Uses `XScreenSaverQueryInfo` on X11 and `ext-idle-notify-v1` on Wayland to detect when user input has ceased for longer than the configured threshold (default 5 minutes).
- **Graceful Degradation**: If an environment or permission check fails, the watcher logs an informational error and disables itself without crashing the application.

### 3.4. Aggregation Pipeline

- Converts discrete `RawEvent` records into fixed **3-minute `TimeBlock`s**.
- Rapid switching within a block is handled by determining the **dominant activity** (plurality of active duration).
- Periods identified as AFK pause block generation, preventing false "drift" accumulation while the user is away from the keyboard.

### 3.5. Classification Engine (3-Tier Cascade)

```
Incoming 3-Minute Block
          |
          v
+-----------------------------+
| Tier 1: Rules & Cache       |---> Match Found? ---> Return Label
| (Deterministic, 0ms, 0 RAM) |                       (Focus / Neutral / Drift)
+--------------+--------------+
               | No Match
               v
+-----------------------------+
| Tier 2: Local AI (Ollama)   |---> Ollama Available? ---> Query Qwen 2.5:3b
| (Optional, User Configured) |                            via localhost:11434
+--------------+--------------+
               | Disabled / Unavailable
               v
+-----------------------------+
| Tier 3: Optional Cloud BYOK |---> Key Provided? ---> Query Cloud Provider
| (User Opt-in Only)          |                        via HTTPS
+--------------+--------------+
               | Not Configured
               v
+-----------------------------+
| Default / Unclassified      |---> Flag as "Needs Review" in UI
+-----------------------------+
```

1. **Tier 1 (Deterministic Rules)**: Evaluates user rules and default rules stored in SQLite. Handles roughly 60-70% of routine developer activities (IDEs, terminals, communication tools, common domains) with zero CPU or memory penalty.
2. **Tier 2 (Local AI via Ollama)**: Queries `qwen2.5:3b` running in a local Ollama instance on `http://127.0.0.1:11434`. Uses structured JSON schemas via grammar-constrained output. Ollama is configured with a 5-minute keepalive so the model unloads automatically during idle periods.
3. **Tier 3 (Optional BYOK Cloud)**: Direct HTTPS call to user-selected OpenAI-compatible endpoint (OpenRouter, Groq, Google AI Studio) using the user's personal API key.

### 3.6. Storage Layer

- **Technology**: In-process SQLite via `rusqlite`.
- **Location**: Standard platform data directory (e.g., `~/.local/share/tendly/tendly.db` on Linux).
- **Concurrency**: WAL (Write-Ahead Logging) mode enabled. Single writer serialized through Rust services, concurrent reads supported.
- **Key Tables**:
  - `raw_events`: Append-only log of detected window changes and idle states.
  - `blocks`: Aggregated 3-minute buckets containing dominant app, title, classification, confidence, and user override.
  - `classification_rules`: User and system matching rules for Tier 1 classification.
  - `app_settings`: Key-value configuration for user preferences and state.

---

## 4. Platform Isolation Strategy

Platform-specific system calls are isolated behind modular Rust modules:

```
src-tauri/src/capture/
├── mod.rs          // Trait definitions, event channels, manager
├── x11.rs          // Linux X11 implementation (x11rb, libXss)
├── wayland.rs      // Linux Wayland implementation (wayland-client)
├── afk.rs          // Idle detection dispatch
├── windows.rs      // Windows implementation (win32, WinEventHook - v0.6)
└── macos.rs        // macOS implementation (Accessibility API - v0.7)
```

Target compilation flags (`#[cfg(target_os = "...")]`) ensure platform dependencies are compiled only on their respective operating systems.

---

## 5. Security & Privacy Architecture

- **No Remote Network Listeners**: No ports are opened to external network interfaces.
- **Zero Default Telemetry**: No crash reporting, analytics, or background metrics are compiled into or transmitted by the application.
- **File Permissions**: The database file is created with restricted permissions (`0600` on Unix platforms) accessible only by the current user.
- **Data Erasure**: The data deletion function issues SQL truncation followed by an immediate physical `VACUUM` to overwrite disk space.

---

## 6. Document History and Traceability

- **Original Architecture**: Created in initial repository planning (CLI-first, polyglot subprocess watchers, HTTP-centric). Preserved at [docs/archive/ARCHITECTURE-original.md](archive/ARCHITECTURE-original.md).
- **Validated Phase 0 Architecture**: Approved on 2026-09-11 following four parallel research evaluations and architectural review. Established Tauri 2.x, Svelte 5, in-process Rust watchers, and Architecture B as the authoritative standard.
