# Classification Specification

## 1. Classification Philosophy
Tendly's classification system operates on the principle that activity is **contextual, not app-name-based**. The same application can represent Focus, Neutral, or Drift depending entirely on the content and the user's current goals. 

**Core Test Cases:**
- **YouTube:** Watching a biology lecture for a class = Focus; Watching algorithmically suggested cat videos = Drift.
- **Firefox/Chrome:** Browsing Stack Overflow or reading documentation = Focus; Scrolling BuzzFeed or Reddit = Drift.
- **VS Code:** Writing project code = Focus.

## 2. The 3-Tier Cascade

Tendly uses a cascading architecture to classify `TimeBlock`s. This balances speed, accuracy, resource usage, and privacy.

### Tier 1: Rules & Cache (0ms, 0 RAM)
- **Mechanism:** Deterministic matching using `ClassificationRule`s against the `TimeBlock`'s dominant app/title/url.
- **Rulesets:**
  - **System/AFK:** Screen saver, lock screen, sleep -> `Inactive` (no TimeBlocks generated).
  - **Unambiguous Focus:** IDEs (VS Code, IntelliJ), terminals (Alacritty, Kitty), professional design tools (Figma).
  - **Unambiguous Neutral:** Communication tools (Slack, Discord, MS Teams, Zoom).
  - **Unambiguous Drift:** Known gaming clients (Steam), social media apps (Twitter/X Desktop).
  - **Known Domains:** github.com -> Focus, netflix.com -> Drift.
- **Priority:** User-created rules (`source = "user"`) have the highest priority, followed by default shipped rules (`source = "default"`).
- **Ambiguity:** Browsers with unknown domains or titles, or generalized apps (like Notion or Obsidian) that aren't matched by rules are passed to Tier 2.

### Tier 2: Local AI (Ollama)
- **Mechanism:** Local LLM inference via Ollama.
- **Model:** `qwen2.5:3b` (Q4_K_M quant, ~1.9GB download, ~2.5GB RAM).
- **Fallback Model:** `qwen2.5:1.5b` (For machines with ≤8GB RAM, ~1.4GB RAM).
- **Input:** The `TimeBlock`'s dominant app, title, and url + user profile + trajectory context (last 3-5 blocks).
- **Output:** Structured JSON containing `{classification, category, confidence}` via Ollama's structured output format feature.
- **Performance:** Latency ranges from ~0.5s on Apple Silicon to ~1.5s on modern x86 CPUs.
- **Memory Lifecycle:** Ollama is run with a `keep_alive` of 5 minutes. It auto-unloads when the user goes AFK to free up system memory.
- **Invocation:** Only invoked for ambiguous blocks that Tier 1 could not classify.

### Tier 3: BYOK Cloud (Optional)
- **Mechanism:** Cloud LLM API inference.
- **Providers:** OpenRouter, Groq (free tier), Google AI Studio (free tier).
- **Contract:** Uses the exact same prompt and JSON output schema as Tier 2.
- **Cost:** ~$0.13 - $0.50/month for typical usage on paid providers.
- **Privacy:** Strictly **Opt-In**. The user must explicitly provide an API key and consent to sending their activity data off-device.

## 3. Classification vs. Event Frequency
**Should the AI classify every event, every block, or only ambiguous blocks?**
- **Only ambiguous blocks.** Tier 1 handles approximately 60-70% of all blocks deterministically based on default rules and user habits. Tier 2 (or Tier 3) only handles the remaining 30-40%. This dramatically reduces AI API calls or local GPU/CPU load.
- **Batch Mode:** When waking from sleep or catching up after AFK, the system batches up to 10 unclassified blocks into a single prompt for efficiency.

## 4. Context Window
The prompt provided to the LLM must contain sufficient context without compromising privacy by dumping raw event logs.
- **User Profile:** Includes the user's role and current goals (editable text in settings). E.g., *"I am a software engineer working on a Rust desktop app."*
- **Trajectory:** Includes the classifications of the last 3-5 blocks to understand momentum (e.g., if the user was deeply focused on code, a quick switch to a browser to search something is likely still Focus).
- **No Raw Data:** Only the `dominant_app`, `dominant_title`, and `dominant_url` are sent.

## 5. Confidence and Uncertainty
- The LLM outputs a `confidence` score (float `0.0` - `1.0`).
- If `confidence < 0.5`, the UI flags the `TimeBlock` as "Needs Review".
- User corrections on low-confidence blocks immediately improve future rule matching by generating a Tier 1 rule.

## 6. Evaluation
The model is evaluated against the 8 test cases defined in `spikes/ai-classification/test_cases.json`.
- **Key Tests:**
  - Case 2 vs 3: Same app, different content (e.g., VS Code editing config vs VS Code playing a text adventure).
  - Case 5 vs 6: YouTube educational content vs YouTube entertainment.
- **Target:** >85% accuracy on the test suite using `qwen2.5:3b`.

## 7. Prompt Template

```text
You are an activity classification engine. Categorize the user's current activity into exactly one of three labels: "focus", "neutral", or "drift".

User Profile:
Role: {{user_role}}
Current Goals: {{user_goals}}

Recent Context (Last {{history_count}} blocks):
{{#each history}}
- {{this.time}}: {{this.app}} ({{this.title}}) -> {{this.classification}}
{{/each}}

Current Activity to Classify:
App: {{app}}
Title: {{title}}
URL: {{url}}

Rules:
1. "focus" = Deep work, active creation, or learning related to goals.
2. "neutral" = Necessary administrative tasks, communication, or ambiguous utility.
3. "drift" = Distraction, entertainment, or irrelevant browsing.

Output JSON strictly conforming to this schema:
{
  "classification": "focus|neutral|drift",
  "category": "String (e.g., 'Software Development', 'Social Media', 'Communication')",
  "confidence": "Float between 0.0 and 1.0"
}
```

## 8. Code & Schema Definitions

### SQLite Schema: Classification Rules

```sql
CREATE TABLE classification_rules (
    id TEXT PRIMARY KEY,
    priority INTEGER NOT NULL DEFAULT 0,
    match_field TEXT NOT NULL, -- 'app', 'domain', 'title_contains'
    pattern TEXT NOT NULL,
    classification TEXT NOT NULL, -- 'focus', 'neutral', 'drift'
    category TEXT,
    source TEXT NOT NULL, -- 'user', 'default'
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_rules_priority ON classification_rules(priority DESC);
```

### Rust Interface (Backend Traits)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Classification {
    #[serde(rename = "focus")]
    Focus,
    #[serde(rename = "neutral")]
    Neutral,
    #[serde(rename = "drift")]
    Drift,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AIClassificationResult {
    pub classification: Classification,
    pub category: String,
    pub confidence: f32,
}

#[derive(Debug)]
pub struct ClassificationContext {
    pub user_role: String,
    pub user_goals: String,
    pub recent_blocks: Vec<TimeBlockSummary>,
}

#[async_trait::async_trait]
pub trait Classifier {
    /// Attempt to classify a TimeBlock. Returns None if it cannot (e.g., Tier 1 misses).
    async fn classify(
        &self,
        block: &TimeBlock,
        context: &ClassificationContext,
    ) -> Result<Option<AIClassificationResult>, anyhow::Error>;
    
    /// Batch classification for catching up.
    async fn classify_batch(
        &self,
        blocks: &[TimeBlock],
        context: &ClassificationContext,
    ) -> Result<Vec<Option<AIClassificationResult>>, anyhow::Error>;
}
```
