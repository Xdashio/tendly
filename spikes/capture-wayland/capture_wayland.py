#!/usr/bin/env python3
"""
Spike: capture the active window (app + title) on Linux/Wayland.

This is the highest-risk spike in the whole project (see ARCHITECTURE.md
and the original planning discussion) — Wayland compositors vary widely
in whether they expose active-window info to arbitrary clients at all,
for good security reasons. This script tries several compositor-specific
approaches and reports which one (if any) works on your setup.

Known landscape (accurate as of the time this was written — verify
against current docs, this moves fast):
    - Sway / wlroots-based compositors: support the wlr-foreign-toplevel-
      management protocol. This is the best-supported path.
    - Hyprland: has its own IPC (`hyprctl activewindow`) — easiest path
      if you're on Hyprland.
    - GNOME (Mutter): does NOT expose foreign-toplevel to arbitrary
      clients by default, for privacy/security reasons. Typically
      requires a GNOME Shell extension (e.g. "Window Calls" or similar)
      that exposes window info over D-Bus. This is the gap we most need
      real-world data on.
    - KDE (KWin): has partial protocol support depending on version;
      also has a scripting/D-Bus route.

Usage:
    python3 capture_wayland.py

The script will detect your compositor where possible and tell you which
method it tried and whether it worked.

What to report back:
    1. Which compositor/DE are you on?
    2. Did ANY method work out of the box?
    3. If not, what would be required (e.g. installing a specific GNOME
       extension) — and is that an acceptable ask for end users, or does
       it need to be part of our install docs / an automated setup step?
    4. If Hyprland or Sway: does the IPC output look clean and usable?
"""

import json
import os
import shutil
import subprocess
import sys
from datetime import datetime, timezone


def detect_compositor():
    desktop = os.environ.get("XDG_CURRENT_DESKTOP", "").lower()
    session = os.environ.get("XDG_SESSION_DESKTOP", "").lower()
    wayland_display = os.environ.get("WAYLAND_DISPLAY", "")

    if not wayland_display:
        print("[warn] WAYLAND_DISPLAY is not set — are you actually running Wayland?")

    guess = desktop or session or "unknown"
    return guess


def try_hyprland():
    if not shutil.which("hyprctl"):
        return None
    try:
        out = subprocess.check_output(["hyprctl", "activewindow", "-j"], text=True)
        data = json.loads(out)
        return {"app": data.get("class"), "title": data.get("title"), "method": "hyprctl"}
    except Exception as e:
        print(f"[hyprctl attempt failed] {e}")
        return None


def try_sway():
    if not shutil.which("swaymsg"):
        return None
    try:
        out = subprocess.check_output(["swaymsg", "-t", "get_tree"], text=True)
        tree = json.loads(out)

        def find_focused(node):
            if node.get("focused"):
                return node
            for child in node.get("nodes", []) + node.get("floating_nodes", []):
                found = find_focused(child)
                if found:
                    return found
            return None

        focused = find_focused(tree)
        if focused:
            return {
                "app": focused.get("app_id") or focused.get("window_properties", {}).get("class"),
                "title": focused.get("name"),
                "method": "swaymsg",
            }
        return None
    except Exception as e:
        print(f"[swaymsg attempt failed] {e}")
        return None


def try_gnome_dbus():
    # GNOME typically requires a shell extension exposing a D-Bus method.
    # This is a best-effort probe for a couple of commonly-used extensions'
    # D-Bus interfaces. Very likely to fail on a stock GNOME install —
    # that's expected and is itself the data point we need.
    if not shutil.which("gdbus"):
        return None
    candidates = [
        # (bus name, object path, interface.method) — extension-dependent, may not exist
        ("org.gnome.Shell", "/org/gnome/Shell/Extensions/Windows",
         "org.gnome.Shell.Extensions.Windows.List"),
    ]
    for bus, path, method in candidates:
        try:
            out = subprocess.check_output(
                ["gdbus", "call", "--session", "--dest", bus, "--object-path", path,
                 "--method", method],
                text=True, stderr=subprocess.DEVNULL,
            )
            return {"app": None, "title": None, "method": f"gnome-dbus:{method}", "raw": out}
        except Exception:
            continue
    return None


def main():
    compositor = detect_compositor()
    print(f"Detected desktop/compositor hint: {compositor}")
    print("Trying available methods...\n")

    result = try_hyprland() or try_sway() or try_gnome_dbus()

    if result:
        print(f"SUCCESS via {result['method']}:")
        event = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "watcher_id": "watcher-wayland-spike",
            "app": result.get("app"),
            "title": result.get("title"),
            "platform": "linux",
            "method": result["method"],
        }
        print(json.dumps(event, indent=2))
    else:
        print("No method succeeded on this system.")
        print("This is itself valuable data — please report:")
        print(f"  - XDG_CURRENT_DESKTOP={os.environ.get('XDG_CURRENT_DESKTOP')}")
        print(f"  - XDG_SESSION_DESKTOP={os.environ.get('XDG_SESSION_DESKTOP')}")
        print(f"  - WAYLAND_DISPLAY={os.environ.get('WAYLAND_DISPLAY')}")
        sys.exit(1)


if __name__ == "__main__":
    main()
