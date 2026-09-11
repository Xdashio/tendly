# Tendly Activity Model

## Overview
The Tendly activity model defines how raw user interactions are captured, aggregated, and classified into meaningful context. The system is designed to provide high-resolution activity tracking while respecting privacy and preserving user control.

## Core Concepts

### 1. RawEvent
An atomic observation from an OS-level or application-level watcher.
- **Characteristics:** Immutable. Never modified after creation. Never deleted except by explicit user action.
- **Fields:**
  - `id`: UUID (Primary Key)
  - `source`: String enum (`"x11" | "wayland" | "windows" | "macos" | "browser" | "afk"`)
  - `timestamp_ms`: BigInt (Unix epoch milliseconds)
  - `app`: String (Process name or WM_CLASS)
  - `title`: String (Window title)
  - `url`: String (Nullable, domain or full URL if captured by browser extension)
  - `idle_ms`: Integer (Nullable, milliseconds since last mouse/keyboard input)
  - `raw_json`: Text (Nullable, for extensible watcher-specific data)

### 2. ActivitySegment
A contiguous period where the same application + title was active.
- **Characteristics:** Computed, not stored (derived on-demand from `RawEvent`s). Derived by merging consecutive events with the same `app` and `title`.
- **Fields:**
  - `start_ms`: BigInt
  - `end_ms`: BigInt
  - `app`: String
  - `title`: String
  - `url`: String (Nullable)
  - `source`: String
  - `event_count`: Integer (Number of merged `RawEvent`s)

### 3. TimeBlock
A fixed-duration bucket for classification (default 3 minutes). The core unit that gets classified, containing the "dominant" activity (the activity that occupied the plurality of time within the block).
- **Fields:**
  - `id`: UUID (Primary Key)
  - `start_ms`: BigInt
  - `end_ms`: BigInt
  - `dominant_app`: String
  - `dominant_title`: String
  - `dominant_url`: String (Nullable)
  - `classification`: String (`"focus" | "neutral" | "drift"`, Nullable)
  - `category`: String (e.g., "Software Development", "Social Media")
  - `confidence`: Float (0.0 - 1.0)
  - `classified_by`: String (`"tier1_rule" | "tier2_local_ai" | "tier3_cloud_ai" | "user_correction"`)
  - `user_override`: String (`"focus" | "neutral" | "drift"`, Nullable)

### 4. AFKPeriod
A period of detected inactivity, derived from idle events exceeding a configured threshold (default 5 minutes). During AFK periods, no `TimeBlock`s are generated.

### 5. Application
Metadata about a tracked application. Identified by the process name (`app` field) or `WM_CLASS`. Users can add applications to an exclusion list to prevent tracking.

### 6. BrowserContext (Post-MVP)
Additional context gathered from the Tendly browser extension.
- **Characteristics:** Stripped to domain-only by default for privacy.
- **Fields:** `url`, `domain`, `title`, `browser`, `is_incognito`

### 7. Classification
The tripartite label assigned to a `TimeBlock`.
- **Values:** `"focus" | "neutral" | "drift"`
- Unclassified blocks have a `null` value (stored as "focus", "neutral", "drift" or NULL in DB).

### 8. UserCorrection
When a user overrides a classification in the UI.
- **Characteristics:** Stored directly as the `user_override` field on the `TimeBlock`. Optionally prompts the creation of a `ClassificationRule` for future matching.

### 9. ClassificationRule
User-defined or default rules for Tier 1 matching.
- **Fields:**
  - `id`: UUID (Primary Key)
  - `priority`: Integer (Higher number = higher priority)
  - `match_field`: String enum (`"app" | "domain" | "title_contains"`)
  - `pattern`: String (Regex or exact match string)
  - `classification`: String (`"focus" | "neutral" | "drift"`)
  - `category`: String
  - `source`: String enum (`"user" | "default"`)
  - `created_at`: BigInt

## Relationships Diagram

```mermaid
flowchart TD
    subgraph Capture
        R[RawEvents]
    end

    subgraph Aggregation
        R -- Merged by app/title --> S(ActivitySegments)
        S -- Bucketed (e.g., 3 mins) --> T[TimeBlocks]
    end

    subgraph Rules
        CR[ClassificationRules]
        CR -- Tier 1 --> T
    end

    subgraph AI
        AI[Local/Cloud AI]
        AI -- Tier 2/3 --> T
    end

    subgraph Output
        T -- Produces --> C(Classification)
    end
    
    subgraph User
        UC(UserCorrection)
        UC -- Overrides --> T
        UC -- Generates --> CR
    end
```

## Data Lifecycle

1. **Creation:** A watcher (OS, Wayland, X11, browser) generates a `RawEvent` representing atomic state. This is immediately stored in the local SQLite database.
2. **Bucketing:** On a set interval (default 3 minutes), raw events are aggregated into `ActivitySegment`s, which are then bucketed into a single `TimeBlock` representing the dominant activity for that window.
3. **Classification:** The unclassified `TimeBlock` enters the 3-Tier Classification Cascade (Rules -> Local AI -> Cloud AI).
4. **Correction:** If the user disagrees with a classification, they issue a `UserCorrection`. This updates the `user_override` field on the `TimeBlock` and can optionally generate a new user-defined `ClassificationRule` with high priority.
5. **Retention & Deletion:** All raw and aggregated data is retained indefinitely on the user's local machine until explicitly deleted. Users have full control to wipe all data or delete specific date ranges.
