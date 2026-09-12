# Linux Activity Capture Architecture

## 1. Overview and Scope

This document specifies the technical design, protocol mechanics, and operational guarantees for Tendly's Linux activity capture subsystem in Phase 2.

Tendly's primary persona is software developers working on Linux desktop environments. Due to the architectural differences between X11 and Wayland compositors, activity capture requires modular, platform-specific watchers that observe desktop state changes and emit canonical `RawEvent` records into an in-process aggregation pipeline.

### Supported Linux Environments in Phase 2 MVP

| Environment | Compositor / Display Server | Window Tracking | Title Tracking | AFK Detection | Status |
|---|---|---|---|---|---|
| **Native X11** | X.Org Server, XFCE, i3, bspwm, etc. | `_NET_ACTIVE_WINDOW` + `WM_CLASS` | `_NET_WM_NAME` / `WM_NAME` | `XScreenSaverQueryInfo` | Supported |
| **XWayland** | XWayland on Wayland compositors | X11 client windows only | X11 client windows only | Varies by compositor | Partial / Fallback |
| **wlroots Wayland** | Sway, Wayfire, River, labwc | `zwlr_foreign_toplevel_manager_v1` | `zwlr_foreign_toplevel_manager_v1` | `ext_idle_notifier_v1` | Supported |
| **Hyprland** | Hyprland (wlroots-adjacent) | `zwlr_foreign_toplevel_manager_v1` + IPC | `zwlr_foreign_toplevel_manager_v1` + IPC | `ext_idle_notifier_v1` | Supported |
| **GNOME Wayland** | Mutter | Out of scope for Phase 2 MVP (requires Shell Extension) | Out of scope for Phase 2 MVP | `org.gnome.Mutter.IdleMonitor` D-Bus | Documented limitation |
| **KDE Wayland** | KWin (Plasma 6) | Out of scope for Phase 2 MVP (requires KWin Script) | Out of scope for Phase 2 MVP | `ext_idle_notifier_v1` / D-Bus | Documented limitation |

---

## 2. X11 Capture Strategy

### Protocol Mechanics
Under X11, window management follows the Extended Window Manager Hints (EWMH) specification:

1. **Active Window Identification**:
   - Query the root window property `_NET_ACTIVE_WINDOW` (type `WINDOW`, format 32).
   - A window ID of `0` indicates no active window is focused (desktop or root focus).
2. **Application Identification**:
   - Query `WM_CLASS` (type `STRING`) on the active window. This returns two null-separated strings: `instance_name` and `class_name` (e.g. `"code\0Code"`).
   - If `WM_CLASS` is missing, query `_NET_WM_PID` (CARDINAL 32) and resolve the process executable name from `/proc/<pid>/comm`.
3. **Window Title Retrieval**:
   - Query `_NET_WM_NAME` (type `UTF8_STRING`).
   - Fall back to `WM_NAME` (type `STRING` / COMPOUND_TEXT) if `_NET_WM_NAME` is not set.
   - Sanitize string data: enforce valid UTF-8, strip non-printable control characters, trim excessive length (max 512 bytes).

### Event vs. Polling Model
- **Hybrid Event-Driven Architecture**:
  - The watcher registers `PropertyChangeMask` on the root window to receive `PropertyNotify` events when `_NET_ACTIVE_WINDOW` changes.
  - When a new active window is focused, the watcher subscribes to `PropertyChangeMask` on that target window to receive immediate updates when its title (`_NET_WM_NAME`) changes (e.g. browser tab navigation or terminal directory change).
  - A periodic heartbeat check runs every 2.0 seconds. This guarantees that window destruction race conditions, unmapped windows, or dropped X11 events do not cause tracking stalls.

### Lifecycle, Connection, and Failure Handling
- Direct connection via `x11rb` protocol crate.
- If the X11 server disconnects (e.g. session restart or sleep), the watcher catches the socket error, logs a sanitized warning, enters a degraded retry state with exponential backoff (1s, 2s, 4s, up to 30s), and re-establishes registration automatically.
- No panic or crash propagates to the Tauri application shell.

### Permissions
- Standard unprivileged user session. No elevated privileges or special capabilities required.

---

## 3. Wayland Capture Strategy (wlroots)

### Wayland Security Model
Under the Wayland architecture, compositors isolate clients. A regular Wayland client cannot query other clients' surface titles or keystrokes through the core Wayland protocol (`wl_compositor`, `wl_surface`).

### Protocol Specification: `wlr-foreign-toplevel-management-unstable-v1`
To provide window-management capabilities (such as taskbars, docks, and time trackers), the wlroots ecosystem defines the `zwlr_foreign_toplevel_manager_v1` protocol extension:

1. **Registration**:
   - Client binds `zwlr_foreign_toplevel_manager_v1` from the `wl_registry`.
   - The compositor emits `toplevel` events providing a `zwlr_foreign_toplevel_handle_v1` proxy for every mapped window.
2. **Handle State Tracking**:
   - `title(title: String)`: Sent when window title changes.
   - `app_id(app_id: String)`: Sent when the application desktop/process identifier is established.
   - `state(states: Vec<u32>)`: Array of bitflags. Value `2` corresponds to `ZWLR_FOREIGN_TOPLEVEL_HANDLE_V1_STATE_ACTIVATED`.
   - `closed()`: Handle is destroyed when window closes.
3. **Active Window Determination**:
   - The watcher maintains an in-memory dictionary of active handles.
   - The window with the `ACTIVATED` flag represents the currently focused surface.
   - When the focused handle changes, or its title changes while active, the watcher emits an event.

### Hyprland Native IPC
For Hyprland environments, Hyprland exposes an event socket at `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket2.sock`. The socket streams newline-delimited events:
`activewindow>>[class],[title]`
The Wayland watcher supports this IPC directly as a high-performance backend.

### Non-Supported Wayland Environments (GNOME & KDE)
- **GNOME Mutter**: Upstream GNOME refuses to implement foreign toplevel protocols. Passive window tracking under GNOME Wayland requires a GNOME Shell Extension communicating over D-Bus or loopback socket.
- **KDE Plasma 6**: KWin does not implement `wlr-foreign-toplevel`. It requires a loaded KWin script exposing window changes over D-Bus.
- Both are explicitly classified as out-of-scope for the Phase 2 Linux MVP.

---

## 4. AFK (Idle Detection) Strategy

### Philosophy: Separate Watcher vs. Aggregation Layer
AFK detection functions as an independent observation stream. It observes user inactivity and informs the aggregation pipeline whether recorded foreground activity represents active engagement or an abandoned desk.

### Linux Idle Mechanisms

1. **Native X11**:
   - `x11rb::protocol::screensaver::query_info` invokes `XScreenSaverQueryInfo`.
   - Returns `idle` in milliseconds since the last keyboard or pointer hardware event.
   - Polled every 5 seconds.
2. **Wayland wlroots (`ext-idle-notify-v1`)**:
   - Client creates an `ext_idle_notification_v1` with a configurable timeout (default 300,000 ms = 5 minutes).
   - Compositor emits `idled` event when inactivity is reached.
   - Compositor emits `resumed` event on subsequent user input.
3. **Desktop Environment D-Bus Fallback**:
   - Under GNOME/Mutter sessions, `org.gnome.Mutter.IdleMonitor` at `/org/gnome/Mutter/IdleMonitor/Core` provides `GetIdletime() -> uint64` ms.

### AFK Event Semantics
- **Inactivity Threshold**: Default 300 seconds (5 minutes), configurable via `AppConfig`.
- **State Transition Events**:
  - Inactivity reached: Emits a single `RawEvent` with `source = RawEventSource::Afk`, `app = "system"`, `title = "afk"`, `idle_ms = Some(elapsed)`.
  - Activity resumed: Emits a single `RawEvent` with `source = RawEventSource::Afk`, `app = "system"`, `title = "active"`, `idle_ms = Some(0)`.
- **Flood Prevention**: No continuous redundant AFK records are generated while the user remains away. Only entry and exit transitions are recorded.

---

## 5. RawEvent Pipeline and Deduplication

```
[OS Watcher: X11 / Wayland / AFK]
              |
              v (mpsc::channel)
      [WatcherManager]
              |
              v
    [Deduplication Filter]
    - State-change trigger
    - Debounce timer (500ms)
    - Heartbeat checkpoint (60s)
              |
              v
   [Database Persistence Worker]
    - Bounded channel
    - Batch insert in transaction
              |
              v
      [SQLite: raw_events]
```

### Event Generation Semantics
An event is persisted only when:
1. The active application (`app`) changes.
2. The active window title (`title`) changes for the current application (debounced by 500ms to avoid recording rapid typing or transient title flickers).
3. The user enters or exits AFK status.
4. A periodic state checkpoint occurs (every 60 seconds) to ensure time blocks can reconstruct continuity even if the user stays on the same window for hours.

### Suppression of Redundant Events
If a window remains active without changes, high-frequency polling events are dropped in-memory by the deduplication filter. This keeps SQLite storage minimal:
- Average event rate during active work: ~5 to 20 events per hour.
- Active typing in one window: 1 checkpoint event per 60 seconds.
- Total database growth: < 500 KB per week of typical engineering work.

---

## 6. Privacy & Data Minimization Guarantees

1. **No Content Capture**: Only application identifier (`WM_CLASS` / `app_id`) and window title are collected.
2. **Zero Keystroke / Screen Data**: No raw keypresses, mouse trajectories, screenshots, or clipboard text are monitored or stored.
3. **Safe Logging Policy**: Application logging strictly redacts window titles and user activity. Diagnostic logs output event counters and state transitions only.
4. **Local Isolation**: All activity is persisted to `~/.local/share/tendly/tendly.db` with Unix file permissions `0600`.

---

## 7. Failure Recovery and Resource Footprint

- **Degraded Operation**: If X11 or Wayland is disconnected, the watcher reports degraded state and enters an exponential retry loop without terminating the main application.
- **Resource Constraints**:
  - Memory: In-process watcher tasks allocate < 5 MB of heap.
  - CPU: Polling intervals are strictly bounded (2s heartbeat, 5s AFK). Average CPU overhead is < 0.1% on modern x86_64 systems.
  - File descriptors: 1 X11/Wayland socket connection, 1 SQLite file lock.
