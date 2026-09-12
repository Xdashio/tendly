#!/usr/bin/env python3
"""Audit the repository to ensure no emojis exist in source code or documentation."""

import os
import re
import sys

EMOJI_PATTERN = re.compile(
    r"[\U0001F300-\U0001F9FF\U00002600-\U000026FF\U00002700-\U000027BF]"
)

CHECK_DIRS = ["src", "src-tauri/src", "src-tauri/tests", "docs"]
EXCLUDE_DIRS = {".git", "node_modules", "target", "dist"}

def audit_file(filepath):
    try:
        with open(filepath, "r", encoding="utf-8", errors="replace") as f:
            for line_idx, line in enumerate(f, start=1):
                match = EMOJI_PATTERN.search(line)
                if match:
                    print(f"Emoji found in {filepath}:{line_idx}: {match.group()}")
                    return False
    except Exception as e:
        print(f"Error reading {filepath}: {e}")
    return True

def main():
    root_dir = os.path.dirname(os.path.abspath(__file__))
    failed = False
    files_checked = 0

    for check_dir in CHECK_DIRS:
        abs_check_dir = os.path.join(root_dir, check_dir)
        if not os.path.exists(abs_check_dir):
            continue
        for root, dirs, files in os.walk(abs_check_dir):
            dirs[:] = [d for d in dirs if d not in EXCLUDE_DIRS]
            for file in files:
                if file.endswith((".rs", ".ts", ".svelte", ".md", ".json", ".toml", ".css", ".html")):
                    filepath = os.path.join(root, file)
                    files_checked += 1
                    if not audit_file(filepath):
                        failed = True

    print(f"Emoji audit complete. {files_checked} files checked.")
    if failed:
        print("FAIL: Emojis detected in repository.")
        sys.exit(1)
    else:
        print("PASS: No emojis detected.")
        sys.exit(0)

if __name__ == "__main__":
    main()
