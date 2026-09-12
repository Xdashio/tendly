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

## 5. Idempotency and Database Integrity

1. **Unique Constraint**: Each `TimeBlock` has a deterministic natural key or unique start time:
   `UNIQUE(start_ms)` ensures that running aggregation over the same time range multiple times will `INSERT OR REPLACE` identical blocks without duplication.
2. **Transactional Writes**: Blocks for an aggregated window are committed in an explicit SQLite transaction.
3. **Vacuum / Rebuild Safe**: Dropping the `blocks` table and re-running historical aggregation from `raw_events` reproduces the exact same set of `TimeBlock`s.
