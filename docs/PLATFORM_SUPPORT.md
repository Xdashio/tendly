# Platform Support & Integration Strategy

This document outlines the cross-platform capabilities, underlying mechanisms, permission requirements, and limitations for Tendly, a privacy-first AI-assisted time-awareness application. Tendly gathers window titles, application names, and idle (AFK) states locally across operating systems.

## 1. Overview

Tendly requires passive, low-latency, low-overhead monitoring of the active window and system idle state. Unlike traditional monolithic trackers, Tendly aims to rely on native system APIs where possible, fallback to compositors/desktop environments on Linux, and gracefully degrade when tracking is impossible (due to permissions, compositor limits, or OS restrictions). 

No data is sent off-device for activity tracking.

## 2. Linux

The Linux desktop is bifurcated between the legacy X11 server and the modern Wayland protocol. Wayland adoption is the default for major distributions (Ubuntu 24.04, Fedora 40/41, RHEL 10), representing 53-60% of the desktop market share as of 2025-2026.

### X11
X11 (Xorg) is now in maintenance-only mode but still present in many setups.
- **Active Window**: Tracked via the `_NET_ACTIVE_WINDOW` property using `XGetWindowProperty`. It is event-driven by subscribing to `PropertyNotify` on the root window.
- **Window Title**: Extracted from `_NET_WM_NAME` (UTF-8) with a fallback to `WM_NAME` (Latin-1).
- **Application Name**: Determined via `WM_CLASS`, then resolving `_NET_WM_PID` to read `/proc/<pid>/comm`.
- **AFK/Idle**: Queried using `XScreenSaverQueryInfo` via `libXss`, returning milliseconds since last input.
- **Permissions**: None required for a standard X11 user.
- **Tooling**: Can be observed manually via `xprop`, `wmctrl`, and `xdotool`.

### Wayland
Wayland delegates window management and input to individual compositors, leading to fragmentation in how active windows and idle states are exposed. 

#### wlroots (Sway, Wayfire, River)
- **Protocol**: `wlr-foreign-toplevel-management-unstable-v1`
- **Capability**: Fully event-driven active window tracking, providing the title, app ID, and activated state.
- **Idle**: Supported via the `ext-idle-notify-v1` protocol.

#### Hyprland
- **Protocol**: Implements `wlr` protocols but also provides a powerful IPC mechanism.
- **Capability**: Can use `hyprctl activewindow -j` for window state and title extraction, allowing a dual-approach strategy if the wlr protocol fails.

#### COSMIC (System76)
- **Protocol**: Supports the Wayland staging protocol `ext-foreign-toplevel-list-v1`.
- **Capability**: Provides standard staging capabilities for window list and active state.

#### KDE Plasma 6
- **Protocol**: KDE explicitly does *not* support the `wlr` or `ext` toplevel protocols natively for third parties without KWin scripts.
- **Capability**: Requires loading a specific KWin script via D-Bus to expose the active window and title to Tendly. 
- **Idle**: Fully supports `ext-idle-notify-v1`.

#### GNOME (Mutter)
- **Protocol**: GNOME refuses implementation of all foreign toplevel protocols. The historic `org.gnome.Shell.Eval` interface is permanently removed.
- **Capability**: The **only** route to obtain active window information is via a custom GNOME Shell Extension communicating over D-Bus.
- **Idle**: Requires querying the `org.gnome.Mutter.IdleMonitor` D-Bus interface.

### Known Gaps in Linux
- GNOME tracking requires users to install a shell extension.
- KDE requires users to load a KWin script.
- The `ext-foreign-toplevel-list-v1` protocol, despite being in wayland-protocols staging, is actively avoided by GNOME and KDE.
- Strict Snap/Flatpak sandbox environments may block D-Bus or Wayland socket access, breaking tracking entirely unless appropriate holes are punched in the manifest.

## 3. Windows

- **Window Tracking**: 
  - Active Window: `GetForegroundWindow()` combined with `GetWindowTextW()`.
  - Process ID: `GetWindowThreadProcessId()`.
  - Process Name: `QueryFullProcessImageNameW()` using the `PROCESS_QUERY_LIMITED_INFORMATION` flag (less privileged than standard query).
  - Event-Driven Loop: Achieved using `SetWinEventHook` listening to `EVENT_SYSTEM_FOREGROUND` and `EVENT_OBJECT_NAMECHANGE`.
- **Idle/AFK**: 
  - Tracked via `GetLastInputInfo()` which is session-specific. The 32-bit tick count wraparound (~49.7 days) is handled via unsigned subtraction.
  - Session Locks: Handled by `WTSRegisterSessionNotification()` subscribing to `WTS_SESSION_LOCK` and `WTS_SESSION_UNLOCK`.
- **UWP Edge Cases**: Universal Windows Platform (UWP) apps report `ApplicationFrameHost.exe` as the foreground window. Tendly must recursively use `EnumChildWindows` to locate the actual underlying application process.
- **Win11 Changes**: Windows 11 24H2 introduced aggressive background throttling (EcoQoS) and heightened EDR scrutiny of WinEvent hooks. Tendly's event loop must be profiled to avoid triggering EDR heuristics or being completely frozen by EcoQoS.
- **Permissions**: Functions for a standard interactive user session. Tracking elevated (Admin) windows requires the app to be manifested with `uiAccess=true` and signed.

## 4. macOS

- **App Tracking**: Tracked using `NSWorkspace.shared.frontmostApplication`. Requires no special permissions.
- **Title Tracking**: Extracting window titles uses the Accessibility API (`AXUIElementCopyAttributeValue` with `kAXFocusedWindowAttribute` followed by `kAXTitleAttribute`).
- **Permissions**: Reading titles requires the user to explicitly grant permission in **System Settings > Privacy & Security > Accessibility**.
- **macOS 15 Sequoia Changes**: We strictly avoid `CGWindowListCopyWindowInfo`. macOS 15 introduced a nag screen requiring users to re-authorize Screen Recording permissions monthly. The Accessibility API *does not* trigger this monthly nag, making it the superior choice.
- **Idle/AFK**: 
  - `CGEventSourceSecondsSinceLastEventType(.combinedSessionState, .anyInputEventType)` is used, but in macOS 14+ this *also* requires Accessibility permissions. 
  - Fallback: IOKit `HIDIdleTime` property provides idle time without permissions, though with some quirkiness across hardware types.
  - Session Events: Tracked using `screensDidSleepNotification` and `com.apple.screenIsLocked`.
- **Distribution Restrictions**: Sandboxed apps on the Mac App Store **cannot** use the Accessibility API to read other apps' titles. Tendly must be distributed outside the App Store, using a Developer ID certificate and Apple Notarization.

## 5. Browser Extensions

To gather granular context (e.g., specific website names or active tabs), Tendly relies on browser extensions.
- **Architecture**: Chrome dictates Manifest V3 (MV3); MV2 is completely deprecated. Service Workers in MV3 are strictly killed after 30 seconds of inactivity.
- **Keepalive Strategy**: Extensions rely on event-driven updates (`tabs.onActivated`, `tabs.onUpdated`, `windows.onFocusChanged`). The Native Messaging host connection (`chrome.runtime.connectNative`) implicitly keeps the Service Worker alive while connected.
- **Permissions**:
  - Requires `"tabs"` for broad URL tracking (triggers a manual review in Web Stores).
  - Requires `"idle"` for tracking browser-specific AFK.
  - Requires `"nativeMessaging"` for IPC to the Tendly daemon.
  - The `"activeTab"` permission is insufficient as it requires explicit user clicks per tab, failing the passive tracking requirement.
- **Incognito**: Extensions are disabled by default in private/incognito modes. The user must manually enable tracking in extension settings.
- **Store Policies**: To be listed on the Chrome Web Store, the extension must adhere strictly to a single-purpose policy, aggressive data minimization, and require a published privacy policy.

## 6. Platform Support Matrix

| OS / Protocol | Active App | Window Title | Idle (AFK) | Event-Driven | Status |
|---|---|---|---|---|---|
| **Linux (X11)** | Yes | Yes | Yes | Yes | Supported / Tested |
| **Linux (wlroots)** | Yes | Yes | Yes | Yes | Supported / Tested |
| **Linux (Hyprland)**| Yes | Yes | Yes | Yes | Supported / Tested |
| **Linux (COSMIC)** | Yes | Yes | Unknown | Yes | Untested |
| **Linux (KDE 6)** | Requires Script | Requires Script | Yes | Yes | Untested / Complex |
| **Linux (GNOME)** | Requires Ext | Requires Ext | Yes | Yes | Untested / Complex |
| **Windows 10/11** | Yes | Yes | Yes | Yes | Supported / Tested |
| **macOS 14+** | Yes | Yes (w/ Perm) | Yes | Yes | Supported / Tested |

## 7. MVP Platform Scope

Given the fragmentation of the Linux desktop ecosystem and the complexities of permissions on modern platforms, the MVP scope for Tendly is:
- **Linux**: X11 (native) + Wayland/wlroots environments (Sway, Hyprland). GNOME and KDE are explicitly out-of-scope for the MVP due to the requirement for external scripts/extensions.
- **Windows**: Windows 10/11 standard user sessions (skipping tracking of Admin windows via `uiAccess` for MVP).
- **macOS**: Non-App Store distribution utilizing the Accessibility API.
- **Browsers**: Chrome/Firefox MV3 extension.

*Honest limitation*: Linux users on GNOME (the default for Ubuntu/Fedora) running Wayland will experience zero tracking in the MVP.

## 8. API Reference

### Linux (X11)
```c
// Requires libX11, libXss
Atom active_window_atom = XInternAtom(display, "_NET_ACTIVE_WINDOW", False);
XGetWindowProperty(display, root, active_window_atom, ...);
XScreenSaverInfo *info = XScreenSaverAllocInfo();
XScreenSaverQueryInfo(display, root, info); // info->idle gives AFK ms
```

### Windows
```c
// Requires User32.dll
HWND foreground = GetForegroundWindow();
GetWindowTextW(foreground, title_buffer, max_count);

// Event Hook
SetWinEventHook(EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND, NULL, WinEventProc, 0, 0, WINEVENT_OUTOFCONTEXT);

// Idle
LASTINPUTINFO lii;
lii.cbSize = sizeof(LASTINPUTINFO);
GetLastInputInfo(&lii);
```

### macOS
```swift
// Requires AppKit, ApplicationServices (Accessibility API)
let app = NSWorkspace.shared.frontmostApplication
let element = AXUIElementCreateApplication(app.processIdentifier)
var focusedWindow: CFTypeRef?
AXUIElementCopyAttributeValue(element, kAXFocusedWindowAttribute as CFString, &focusedWindow)
```

## 9. Known Limitations

- **GNOME Wayland Black Hole**: There is currently no native standard Wayland protocol supported by GNOME for foreign toplevel management. Without a dedicated GNOME extension, Tendly is entirely blind to window titles and apps on GNOME Wayland.
- **macOS Accessibility Friction**: macOS users must dig into settings to allow Accessibility permissions. If they deny or ignore this, Tendly can only track the Application Name, not the Document/Window title, degrading contextual AI effectiveness.
- **Windows Admin Windows**: When a standard user opens an application as Administrator, the event hooks drop. Tendly will not track elevated windows unless the tracking process itself is elevated or `uiAccess` manifested.
- **Browser Granularity vs Store Policy**: Using the broad `"tabs"` permission guarantees a delayed, manual review process on the Chrome Web Store.
- **Linux Flatpaks**: Distributing Tendly as a Flatpak would block access to `/proc`, `libXss`, and un-sandboxed D-Bus calls, severely breaking tracking unless permissions are explicitly overridden by the user.
