#!/usr/bin/env python3
"""
Spike: capture the active window (app + title) on macOS.

UNTESTED - written ahead of having Mac hardware to run it on, per the
plan to keep macOS a first-class target from the start (see
docs/PROBLEM_STATEMENT.md). This needs a real run-and-report pass before
we trust it at all. Treat every line of this as a hypothesis, not a fact.

Requirements:
    pip install pyobjc-framework-Quartz pyobjc-framework-Cocoa

Permissions:
    macOS will require you to grant this script (or your terminal app)
    "Accessibility" permission the first time it tries to read another
    app's window title — System Settings > Privacy & Security >
    Accessibility. Getting the title (not just the frontmost app name)
    needs this; getting just the frontmost app name does not.

Usage:
    python3 capture_macos.py [--interval 2] [--duration 60]

What to report back:
    1. Does the Accessibility permission prompt appear as expected, and
       does granting it actually make title capture work?
    2. Does frontmost-app detection (NSWorkspace) work even WITHOUT
       Accessibility permission granted (it should — this is the
       degrade-gracefully fallback per the Watcher contract in
       ARCHITECTURE.md)?
    3. Do app + title look correct across a few different apps (Safari,
       a terminal, an Electron app like VS Code or Slack)?
    4. CPU usage over a few minutes.
    5. Anything about Sequoia/Sonoma-era permission changes that this
       script doesn't account for — macOS tightens this periodically.
"""

import argparse
import json
import time
from datetime import datetime, timezone
from pathlib import Path

try:
    from AppKit import NSWorkspace
    from Quartz import (
        CGWindowListCopyWindowInfo,
        kCGWindowListOptionOnScreenOnly,
        kCGNullWindowID,
    )
except ImportError:
    print("Missing dependency. Run:")
    print("  pip install pyobjc-framework-Quartz pyobjc-framework-Cocoa")
    raise SystemExit(1)

LOG_FILE = Path(__file__).parent / "capture_macos_log.jsonl"


def get_frontmost_app():
    """Works without Accessibility permission — the graceful-degrade fallback."""
    active_app = NSWorkspace.sharedWorkspace().activeApplication()
    if active_app:
        return active_app.get("NSApplicationName")
    return None


def get_active_window_title(app_name):
    """
    Best-effort window title lookup via CGWindowListCopyWindowInfo.
    Requires Accessibility permission to return titles for apps other
    than the one this script itself belongs to, on modern macOS.
    """
    window_list = CGWindowListCopyWindowInfo(
        kCGWindowListOptionOnScreenOnly, kCGNullWindowID
    )
    for window in window_list:
        owner = window.get("kCGWindowOwnerName")
        layer = window.get("kCGWindowLayer", 0)
        if owner == app_name and layer == 0:
            return window.get("kCGWindowName")
    return None


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--interval", type=float, default=2.0)
    parser.add_argument("--duration", type=float, default=None)
    args = parser.parse_args()

    print(f"Watching active window every {args.interval}s. Ctrl+C to stop.")
    print(f"Logging to {LOG_FILE}")
    print("NOTE: first run will likely trigger a macOS Accessibility permission prompt.\n")

    last = (None, None)
    start = time.time()

    with open(LOG_FILE, "a") as log:
        try:
            while True:
                app = get_frontmost_app()
                title = get_active_window_title(app) if app else None

                if (app, title) != last and app:
                    event = {
                        "timestamp": datetime.now(timezone.utc).isoformat(),
                        "watcher_id": "watcher-macos-spike",
                        "app": app,
                        "title": title,
                        "platform": "macos",
                    }
                    line = json.dumps(event)
                    print(line)
                    log.write(line + "\n")
                    log.flush()
                    last = (app, title)

                if args.duration and (time.time() - start) > args.duration:
                    break

                time.sleep(args.interval)
        except KeyboardInterrupt:
            print("\nStopped.")


if __name__ == "__main__":
    main()
