#!/usr/bin/env python3
"""
Spike: capture the active window (app + title) on Linux/X11.

Purpose: prove we can reliably get (app, title) on a poll loop with no
special permissions beyond a normal X11 session, and see how noisy/clean
the data is in practice.

Requirements:
    - X11 session (not Wayland — see ../capture-wayland/ for that)
    - `wmctrl` and `xprop` installed:
        Debian/Ubuntu: sudo apt install wmctrl x11-utils
        Fedora:        sudo dnf install wmctrl xorg-x11-utils
        Arch:           sudo pacman -S wmctrl xorg-xprop

Usage:
    python3 capture_x11.py [--interval 2] [--duration 60]

Output:
    Prints one JSON line per detected change to stdout, and also appends
    to capture_x11_log.jsonl in the current directory so you can review
    the whole session afterward.

What to report back:
    1. Does it run without errors on your distro/WM?
    2. Do app + title look correct and useful (not empty/garbled)?
    3. How noisy is it — does switching within the same app (e.g. tabs)
       fire way more events than expected?
    4. CPU usage of this script over a few minutes (`top`/`htop`).
"""

import argparse
import json
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path

LOG_FILE = Path(__file__).parent / "capture_x11_log.jsonl"


def get_active_window():
    """Returns (app_class, title) for the currently active X11 window, or (None, None)."""
    try:
        # Get the active window ID
        win_id_raw = subprocess.check_output(
            ["xprop", "-root", "_NET_ACTIVE_WINDOW"],
            stderr=subprocess.DEVNULL,
            text=True,
        )
        # Format: _NET_ACTIVE_WINDOW(WINDOW): window id # 0x1234567
        win_id = win_id_raw.strip().split()[-1]
        if win_id in ("0x0", "0x00000000"):
            return None, None

        # Get window class (app identifier)
        class_raw = subprocess.check_output(
            ["xprop", "-id", win_id, "WM_CLASS"],
            stderr=subprocess.DEVNULL,
            text=True,
        )
        # Format: WM_CLASS(STRING) = "instance", "Class"
        app = None
        if "=" in class_raw:
            parts = class_raw.split("=", 1)[1].strip()
            names = [p.strip().strip('"') for p in parts.split(",")]
            app = names[-1] if names else None

        # Get window title
        title_raw = subprocess.check_output(
            ["xprop", "-id", win_id, "_NET_WM_NAME"],
            stderr=subprocess.DEVNULL,
            text=True,
        )
        title = None
        if "=" in title_raw:
            title = title_raw.split("=", 1)[1].strip().strip('"')
        else:
            # fallback to legacy WM_NAME if _NET_WM_NAME is unset
            legacy = subprocess.check_output(
                ["xprop", "-id", win_id, "WM_NAME"],
                stderr=subprocess.DEVNULL,
                text=True,
            )
            if "=" in legacy:
                title = legacy.split("=", 1)[1].strip().strip('"')

        return app, title
    except (subprocess.CalledProcessError, FileNotFoundError, IndexError) as e:
        print(f"[warn] failed to read active window: {e}")
        return None, None


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--interval", type=float, default=2.0, help="poll interval in seconds")
    parser.add_argument("--duration", type=float, default=None, help="stop after N seconds (default: run until Ctrl+C)")
    args = parser.parse_args()

    print(f"Watching active window every {args.interval}s. Ctrl+C to stop.")
    print(f"Logging to {LOG_FILE}")

    last = (None, None)
    start = time.time()

    with open(LOG_FILE, "a") as log:
        try:
            while True:
                app, title = get_active_window()
                if (app, title) != last and (app or title):
                    event = {
                        "timestamp": datetime.now(timezone.utc).isoformat(),
                        "watcher_id": "watcher-x11-spike",
                        "app": app,
                        "title": title,
                        "platform": "linux",
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
