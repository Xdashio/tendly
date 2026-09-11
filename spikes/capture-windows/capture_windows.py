#!/usr/bin/env python3
"""
Spike: capture the active window (app + title) on Windows.

Uses raw ctypes (no pywin32 dependency needed) so this is a single file
that runs with a stock Python install on Windows.

Usage:
    python capture_windows.py [--interval 2] [--duration 60]

Output:
    Prints one JSON line per detected change to stdout, and appends to
    capture_windows_log.jsonl in the current directory.

What to report back:
    1. Does it run without errors / without needing admin rights?
    2. Do app + title look correct (app name resolves sensibly for
       browsers, terminals, IDEs)?
    3. Any apps where the title is empty/unhelpful (some UWP/Store apps
       are known to be trickier — good to know which ones, if any)?
    4. CPU usage of this script over a few minutes (Task Manager).
"""

import argparse
import ctypes
import json
import time
from ctypes import wintypes
from datetime import datetime, timezone
from pathlib import Path

user32 = ctypes.windll.user32
kernel32 = ctypes.windll.kernel32
psapi = ctypes.windll.psapi

LOG_FILE = Path(__file__).parent / "capture_windows_log.jsonl"


def get_active_window():
    """Returns (process_name, title) for the currently active window, or (None, None)."""
    hwnd = user32.GetForegroundWindow()
    if not hwnd:
        return None, None

    # Window title
    length = user32.GetWindowTextLengthW(hwnd)
    buf = ctypes.create_unicode_buffer(length + 1)
    user32.GetWindowTextW(hwnd, buf, length + 1)
    title = buf.value

    # Owning process name
    pid = wintypes.DWORD()
    user32.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))

    PROCESS_QUERY_LIMITED_INFORMATION = 0x1000
    h_process = kernel32.OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, False, pid.value)
    app_name = None
    if h_process:
        exe_buf = ctypes.create_unicode_buffer(260)
        size = wintypes.DWORD(260)
        # QueryFullProcessImageNameW is preferred over the deprecated GetModuleFileNameExW
        success = kernel32.QueryFullProcessImageNameW(h_process, 0, exe_buf, ctypes.byref(size))
        if success:
            full_path = exe_buf.value
            app_name = full_path.split("\\")[-1]
        kernel32.CloseHandle(h_process)

    return app_name, title


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
                        "watcher_id": "watcher-windows-spike",
                        "app": app,
                        "title": title,
                        "platform": "windows",
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
