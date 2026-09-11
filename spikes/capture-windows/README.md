# Spike: Windows window capture

Proves out active-window capture on Windows before building the real
`watcher-windows`. Uses only `ctypes` — no extra pip installs needed.

## Run it

```powershell
python capture_windows.py --interval 2
```

Let it run for 10–15 minutes of normal use — browser, terminal, IDE, a
UWP/Store app if you have one, to check for edge cases.

## What we're trying to learn

1. Runs without needing admin rights?
2. `app`/`title` clean and correct across normal apps?
3. Any apps (especially UWP/Store apps) where the title comes back empty
   or unhelpful?
4. CPU cost of the poll loop.

Report back the answers and the `capture_windows_log.jsonl` output if
anything looks off.
