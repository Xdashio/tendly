# Spike: X11 window capture

Proves out active-window capture on Linux/X11 before we build the real
`watcher-x11` (see [../../docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md#1-capture-layer--watchers)).

## Run it

```bash
sudo apt install wmctrl x11-utils   # or your distro's equivalent
python3 capture_x11.py --interval 2
```

Let it run in the background while you use your computer normally for
10–15 minutes — switch apps, browser tabs, terminals, editors.

## What we're trying to learn

1. Does this run cleanly with no special permissions on a normal X11 session?
2. Is `app`/`title` data clean enough to be useful, or noisy/garbled?
3. Event volume — is 2s polling too chatty, or fine?
4. CPU cost of the poll loop over time.

Report back with the answers (and the `capture_x11_log.jsonl` output if
anything looks weird) and we'll adjust the real watcher design accordingly.
