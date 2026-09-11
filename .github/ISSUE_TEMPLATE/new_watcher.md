---
name: New watcher proposal
about: Propose or claim work on a new watcher (OS, editor, browser, etc.)
title: "[Watcher] "
labels: watcher
assignees: ''
---

**What does this watcher observe?**
e.g. "active window on Wayland/KDE", "Neovim buffer/session", "Firefox tab
title + URL".

**Target platform(s)**

**Proposed mechanism**
The API/protocol/library you plan to use (e.g. `wlr-foreign-toplevel`,
`pyobjc` + Accessibility API, a specific browser extension API).

**Does this fit the Watcher contract?**
Confirm you've read
[docs/ARCHITECTURE.md#1-capture-layer--watchers](../../docs/ARCHITECTURE.md#1-capture-layer--watchers) —
in particular: runs as its own process, POSTs to the loopback Local API,
degrades silently if permissions are denied.

**Are you planning to implement this yourself?**
Let us know so we can avoid duplicate work, or pair you with someone.
