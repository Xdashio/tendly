# Tendly Timeline and Activity History Specification

## 1. Architectural Separation: Analytical Unit vs. User-Facing Timeline

Tendly enforces a strict separation between its analytical activity model and its presentation model:

```text
RawEvents (canonical, immutable observations)
    ↓
ActivitySegments (reconstructed actual contiguous runs)
    ↓
TimeBlocks (standardized 3-minute analytical epoch buckets)
    ↓
ActivitySessions (user-facing continuous activity runs)
    ↓
Desktop Timeline UI (clean, calm chronological history)
```

### Key Principles:
1. **TimeBlock is an internal analytical unit.** It partitions time into fixed 3-minute epoch intervals `[start_ms, end_ms)` where `start_ms = T - (T % 180_000)`. This structure is required for deterministic rules matching, token-budgeted AI classification prompts, and historical recalculation.
2. **ActivitySession is a derived presentation entity.** Users conceptualize activity as sessions (e.g., "VS Code for 45 minutes", "Firefox for 15 minutes"), not sixteen discrete 3-minute blocks. Sessions are derived in-memory on demand via deterministic coalescing.
3. **No Redundant Storage:** Derived sessions are never written to a secondary database table. They are computed dynamically from `TimeBlock`s and underlying `ActivitySegment`s using bounded, indexed range queries on demand.

---

## 2. Session Coalescing Rules

Two chronologically adjacent `TimeBlock`s belong to the same `ActivitySession` if and only if all three compatibility conditions are satisfied:

1. **Temporal Contiguity:**
   $$\text{block}[i].\text{end\_ms} == \text{block}[i+1].\text{start\_ms}$$
   Any time gap $\ge 180{,}000$ ms breaks the session and triggers gap evaluation.
2. **Homogeneous Dominant Application:**
   $$\text{block}[i].\text{dominant\_app} == \text{block}[i+1].\text{dominant\_app}$$
   An application switch (e.g. `code` -> `slack` -> `code`) always creates separate sessions.
3. **Homogeneous Activity Type:**
   $$\text{block}[i].\text{activity\_type} == \text{block}[i+1].\text{activity\_type}$$
   An activity state transition (e.g. `Active` -> `Afk`) always creates separate sessions.

### Plurality Title Attribution across Sessions:
When multiple blocks are coalesced into a single session, the overall `dominant_title` of the session is chosen by plurality frequency across the constituent `TimeBlock`s.

---

## 3. Secondary Activity Preservation

A 45-minute coding session in VS Code might contain brief sub-minute context switches (e.g., 3 minutes consulting documentation in Firefox, 2 minutes responding to a colleague on Slack).

### How Secondary Activity is Preserved and Surfaced:
1. **Lossless Retention:** Underlying sub-minute activities are preserved in the immutable `raw_events` table.
2. **Inline Summary:** Each `ActivitySession` contains `has_secondary_activity: bool` and `secondary_apps: Vec<AppDurationSummary>`, computed dynamically from raw event segments overlapping the session window.
3. **UI Transparency:** The timeline card clearly displays secondary activity (e.g., `Includes: firefox (3m), slack (2m)`).
4. **Drill-Down Inspection:** Clicking a session retrieves the exact composition via `get_session_details`, showing a full application breakdown bar and the exact sub-minute activity segments.

---

## 4. Timeline Semantics and Date Boundaries

### Strict Half-Open Intervals:
All timeline queries operate on strict half-open intervals:
$$\text{Query Window} = [\text{day\_start\_ms}, \text{day\_end\_ms})$$

- `day_start_ms`: `00:00:00.000` local time.
- `day_end_ms`: `00:00:00.000` of the subsequent calendar day.

### Midnight Handling:
- Because 3-minute epochs (180,000 ms) divide evenly into standard 24-hour days (86,400,000 ms), epoch boundaries align naturally with midnight.
- Activity at `23:59:59.999` belongs strictly to the current day.
- Activity at `00:00:00.000` belongs strictly to the following day.
- No session or block is duplicated across midnight boundaries.

### Gaps vs. Idle vs. Active:
The system makes a clear three-way distinction:
1. **Active Work:** User actively engaged with applications (`ActivityType::Active`).
2. **Recorded Idle (AFK):** User walked away from computer while system remained active (`ActivityType::Afk`).
3. **Unrecorded Gap:** No observation data exists (computer shut down, suspended, tracking paused) with duration $\ge 3$ minutes. Displayed in the timeline as an unrecorded gap card (`No recorded activity`).

---

## 5. Live Activity Integration

When viewing the current calendar day:
- The timeline shows a live status banner displaying the real-time active application, window title, and seconds elapsed in state.
- Enclosing window rule:
  $$\text{now\_ms} \ge \text{start\_ms} \quad \text{AND} \quad \text{now\_ms} < \text{end\_ms}$$
- Stale activity (>180s without active watcher updates) automatically transitions to `ActivityType::Unknown` and does not fabricate fake current blocks.
