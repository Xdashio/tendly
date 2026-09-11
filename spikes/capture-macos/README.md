# Spike: macOS window capture (UNTESTED)

This was written without access to Mac hardware, ahead of need, because
macOS is a first-class v1.0 target per
[docs/PROBLEM_STATEMENT.md](../../docs/PROBLEM_STATEMENT.md#platforms).
**Nobody has run this yet.** Treat it as a first draft to validate, not a
working watcher.

## Run it

```bash
pip install pyobjc-framework-Quartz pyobjc-framework-Cocoa
python3 capture_macos.py --interval 2
```

The first time it tries to read a window title from another app, macOS
should prompt for Accessibility permission (System Settings > Privacy &
Security > Accessibility).

## What we're trying to learn

1. Does the permission flow work as expected?
2. Does frontmost-app detection work even *without* granting Accessibility
   permission (it should — that's the intended graceful-degrade path)?
3. Are app/title values clean across a few different apps (native, and an
   Electron app like VS Code or Slack, which sometimes behave differently)?
4. CPU cost.
5. Whether anything about your macOS version's permission model doesn't
   match what this script assumes — Apple tightens this periodically and
   this script may already be slightly out of date by the time you run it.

Once this is validated (or fixed) on real hardware, it becomes the basis
for the real `watcher-macos`.
