# Tendly Time-Block Semantics and Aggregation Specification

## 1. Overview and Core Philosophy

Tendly captures raw, immutable observations from operating system activity watchers (`RawEvent`) and transforms them into deterministic, classified units of time (`TimeBlock`).

### Source of Truth vs. Derived Interpretation
- **RawEvents are the primary source of truth.** They are immutable observations recorded with UTC timestamps.
- **TimeBlocks are derived data.** They represent an aggregated, structured interpretation of raw events over fixed or activity-bounded time windows.
- **Idempotent Historical Reconstruction:** If `blocks` are wiped, corrupted, or re-aggregated with new parameters, Tendly can deterministically rebuild all historical `TimeBlock`s from `raw_events`.

---

## 2. Model Evaluation: Fixed Buckets vs. Activity Segments vs. Hybrid

Three models were evaluated for Tendly's time architecture:

| Criterion | Model A: Pure Fixed Buckets | Model B: Pure Variable Segments | Model C: Hybrid (Adopted) |
|---|---|---|---|
| **Structure** | Strictly fixed 3-minute windows | Arbitrary length (1s to 8h) | ActivitySegments aggregated into bounded 3-minute TimeBlocks |
| **Flow Preservation** | Fragments continuous 2-hour coding into 40 blocks | Single 4-hour block obscures mid-session drift | Segments preserve true duration; blocks provide uniform classification units |
| **Classification Fitness** | Predictable prompt size for LLM/rules | Overly large context or micro-prompts | Ideal: 3 minutes provides stable context without prompt bloat |
| **Timeline Visualization** | Grid/heatmap alignment is trivial | Irregular widths complicate grid rendering | Uniform block grid with precise segment drill-down |
| **Idempotency** | Highly deterministic based on epoch math | Dependent on stream boundary cuts | Deterministic alignment to epoch intervals `[T, T + 180s)` |

### Decision: Model C (Hybrid Segment-First Bucketing)
1. **RawEvent Processing**: Consecutive raw events are normalized into contiguous **`ActivitySegment`**s (`start_ms`, `end_ms`, `app`, `title`, `state: Active | Afk | Unknown`).
2. **TimeBlock Aggregation**: The timeline is partitioned into canonical **3-minute epochs** (180,000 ms, aligned to `T - (T % 180_000)`).
3. **Plurality Attribution**: For each 3-minute block, the dominant application and window title are determined by the plurality of active duration within that window.
4. **First-Class Activity States**: A block is typed as `Active`, `Afk`, or `Unknown`:
   - `Active`: User was actively interacting with applications.
   - `Afk`: User was idle beyond the configured threshold.
   - `Unknown`: No observation data exists (computer sleep, shutdown, watcher pause).

---

## 3. Detailed Boundary and Event Semantics

### Question 1: What starts a block?
A block starts at a deterministic 3-minute epoch boundary: `start_ms = T - (T % 180_000)` in UTC milliseconds.

### Question 2: What ends a block?
A block ends exactly 180,000 ms after its start: `end_ms = start_ms + 180_000`.

### Question 3: Can blocks overlap?
**Never.** Blocks represent non-overlapping adjacent intervals:
$$\text{For any block } i, \quad end\_ms_i \le start\_ms_{i+1}$$

### Question 4: Can blocks have gaps?
**Yes.** When the computer is powered off, suspended, or tracking is paused, no events exist. Rather than falsely attributing continuous activity across hours of sleep, the aggregator marks intervals exceeding the maximum observation gap (180 seconds) as `Unknown` or leaves them unrecorded.

### Question 5: How are AFK periods represented?
AFK is a first-class state, not an application:
- `activity_type = "afk"`
- `dominant_app = "system"`
- `dominant_title = "afk"`
- `confidence = 1.0`
- AFK blocks are never classified as Focus, Neutral, or Drift.

### Question 6: How are application changes handled?
Given an event $E_1$ at $T_1$ with app $A_1$, followed by $E_2$ at $T_2$ with app $A_2$:
- The interval $[T_1, \min(T_2, T_1 + \text{gap\_timeout}))$ is attributed to $A_1$.
- At $T_2$, attribution immediately switches to $A_2$.
- Within a 3-minute block overlapping both, the duration spent in $A_1$ and $A_2$ are compared; the one with greater duration becomes `dominant_app`.

### Question 7: How are window title changes handled?
- If the application remains the same but the title changes (e.g., switching browser tabs or editing a different file in VS Code):
- The time spent in each title is tracked separately under that application.
- The title that occupied the greatest duration within the `dominant_app` becomes `dominant_title`.

### Question 8: What happens when the same application remains active?
- Phase 2 capture emits periodic heartbeat events every 60 seconds.
- The aggregator continues attributing active time to that application across block boundaries without interruption or state resets.

### Question 9: What happens when activity disappears without an AFK event?
If no event arrives for longer than `MAX_OBSERVATION_GAP` (180,000 ms = 3 minutes) without an explicit AFK event:
- The previous active state is capped at $T_{\text{last}} + \text{HEARTBEAT\_GRACE}$ (60,000 ms).
- The remaining elapsed time is attributed to `Unknown`.
- **Invariable Rule:** Unknown time is never converted to active work or AFK.

### Question 10: What happens when a watcher temporarily fails or reconnects?
- The period prior to failure is attributed up to the last valid observation.
- The gap until reconnection is recorded as `Unknown`.
- Reconnection starts a fresh segment immediately upon the first new event.

### Question 11: What happens across computer sleep / suspend / resume?
- A laptop lid close or system suspend creates a sudden time jump of minutes or hours between two events.
- Because delta exceeds `MAX_OBSERVATION_GAP`, the aggregator immediately closes the active segment at suspend time.
- The sleep period is represented as unobserved / unknown time.
- Resume generates a new active event, beginning a new active segment.

### Question 12: What happens across midnight?
- All calculations are performed on UTC millisecond integers (`i64`).
- Midnight in UTC or local timezone does not introduce boundary discontinuities or duration calculation bugs.

### Question 13: What happens after application restart?
- When Tendly restarts, the aggregator identifies the timestamp of the latest persisted `TimeBlock`.
- It resumes aggregation from that timestamp forward, preserving historical integrity.

### Question 14: What happens if events arrive out of order?
- Before aggregation, `RawEvent` records are explicitly sorted by `timestamp_ms ASC, id ASC`.

### Question 15: What happens if timestamps are duplicated?
- If two distinct events share the exact same millisecond timestamp, deterministic sorting by `id` establishes a consistent sequence.

### Question 16: What happens if timestamps are invalid?
- An event with $timestamp\_ms \le 0$ or $timestamp\_ms > \text{now} + 300\_000$ ms (clock skew into the future) is dropped by the normalization validator.

---

## 4. Time Reconstruction Algorithm

```
Sorted RawEvents: [E0, E1, E2, ..., En]
       │
       ▼
[Segment Reconstructor]
- Attribute duration [Ti, Ti+1) to Ei
- If Ti+1 - Ti > MAX_GAP (180s):
    - Cap Ei at Ti + 60s
    - Insert UnknownSegment [Ti + 60s, Ti+1)
- If Ei.source == Afk:
    - Insert AfkSegment [Ti, Ti+1)
       │
       ▼
ActivitySegments: [S0, S1, S2, ...]
       │
       ▼
[Epoch Bounding (3-minute windows: 00:00, 03:00, 06:00...)]
- Slice segments by [Epoch_Start, Epoch_End)
- Sum duration per (app, title) within epoch
- Plurality winner = dominant_app, dominant_title
- Type = Active | Afk | Unknown
       │
       ▼
Deterministic TimeBlocks: [B0, B1, B2, ...] -> SQLite `blocks`
```

---

## 5. Information Preservation and Composition

### What Information Survives Plurality Attribution
In each 3-minute epoch block (`[start_ms, end_ms)`):
- `dominant_app` and `dominant_title`: The application and window title occupying the greatest active duration.
- `activity_type`: The dominant state (`active`, `afk`, `unknown`).
- `duration_ms`: Standardized duration (180,000 ms).

### What Information is Intentionally Summarized
- Sub-minute secondary activities (e.g., a 20-second documentation lookup in Firefox during 160 seconds of coding in VS Code) are summarized under the dominant activity for the analytical record.
- This prevents downstream classification engines from choking on high-frequency context switching.

### How Detailed Sub-Block Composition is Preserved Losslessly
- **No information is permanently lost.** `raw_events` is the immutable canonical source of truth.
- When downstream classification (Phase 6/7) or the user-facing timeline (Phase 4) requires the sub-minute breakdown of any block, `ActivityProcessor::get_block_composition(db, start_ms, end_ms)` reconstructs the exact contiguous `ActivitySegment`s on demand.
- Measured performance: Querying and reconstructing a 3-minute window from indexed `raw_events` takes ~0.02 ms.
- Storing denormalized JSON summaries inside `TimeBlock` was rejected to maintain single-source-of-truth integrity and avoid storage duplication.

---

## 6. Analytical Unit vs. User-Facing Timeline

### TimeBlock is Primarily an Internal Analytical Unit
- A `TimeBlock` (3-minute epoch) exists primarily for the classification and analytical engine.
- Discrete, fixed-width blocks provide uniform tokens for rules-based and LLM-based categorization without context-window fragmentation.

### User-Facing Representation (Phase 4 Progression)
- Users do not think in rigid 3-minute slices. Slicing a 45-minute continuous coding session into fifteen 3-minute cards creates artificial visual clutter.
- The user-facing timeline (Phase 4) presents **Activity Sessions / Runs** by visually coalescing adjacent compatible `TimeBlock`s that share the same dominant context (e.g., displaying `10:00 - 10:45 VS Code (45m)`).
- Users can click or expand any session to inspect the underlying 3-minute blocks and sub-minute segment composition.

---

## 7. Reprocessing and Classification Staleness

When historical raw events are reprocessed (due to late watcher backfill, synchronization, or algorithm updates), previously classified `TimeBlock` records are updated via `ON CONFLICT(start_ms) DO UPDATE`.

### Staleness Invalidation Rules
1. **Unchanged Derived Content** (`dominant_app`, `dominant_title`, and `activity_type` match):
   - Automatic classification, category, confidence, and classifier attribution are preserved.
   - User overrides are preserved.
2. **Changed Derived Content** (`dominant_app`, `dominant_title`, or `activity_type` altered):
   - **Automatic classification is strictly invalidated** (reset to `NULL`). A block originally classified as "Focus" under VS Code will never retain that label if reprocessing reveals the dominant activity was actually Slack.
   - **User Override Scoping**: A user override is preserved only if `dominant_app` remains identical. If the dominant application changed, the user override is invalidated (reset to `NULL`) to prevent manual labels from corrupting different applications.

---

## 8. Idempotency and Database Integrity

1. **Unique Constraint**: Each `TimeBlock` has a deterministic natural key or unique start time:
   `UNIQUE(start_ms)` ensures that running aggregation over the same time range multiple times will update identical blocks deterministically with zero duplicates.
2. **Deterministic UUID v5**: Block IDs are computed using UUID v5 from `timeblock:{start_ms}` in the URL namespace.
3. **Vacuum / Rebuild Safe**: Dropping the `blocks` table and re-running historical aggregation from `raw_events` reproduces the exact same set of `TimeBlock`s.
