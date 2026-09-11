# Spike: Wayland window capture (highest-risk spike)

This is the one most likely to reveal a real problem — Wayland's security
model deliberately restricts arbitrary clients from reading window titles
across the compositor. We need real data on how bad this is in practice.

## Run it

```bash
python3 capture_wayland.py
```

No extra dependencies for Sway/Hyprland (uses their built-in IPC tools —
`swaymsg` / `hyprctl`, which ship with those compositors). GNOME support
depends on you already having a window-listing extension installed — the
script just probes for one, it won't install anything.

## What we're trying to learn

1. What compositor/DE are you actually running day to day?
2. Does it work out of the box (Sway, Hyprland — should)?
3. On GNOME specifically: does it fail as expected? If you install a
   window-listing extension (e.g. search GNOME Extensions for "Window
   Calls"), does the D-Bus probe start working?
4. Given what you find: is "install this GNOME extension" a reasonable
   thing to ask end users to do, or should GNOME support be a known,
   documented limitation for v1.0 rather than a blocker?

This spike result directly decides whether `watcher-wayland` is a v0.1-tier
feature or something we scope down and document as a known gap for
specific compositors.
