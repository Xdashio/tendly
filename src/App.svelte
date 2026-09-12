<script lang="ts">
  import { onMount } from "svelte";
  import Header from "./lib/components/Header.svelte";
  import Card from "./lib/components/Card.svelte";
  import Badge from "./lib/components/Badge.svelte";
  import LoadingState from "./lib/components/LoadingState.svelte";
  import ErrorState from "./lib/components/ErrorState.svelte";
  import EmptyState from "./lib/components/EmptyState.svelte";
  import Timeline from "./lib/components/Timeline.svelte";
  import {
    getAppInfo,
    getAppStatus,
    getCaptureStatus,
    getDatabaseStats,
    getCurrentActivity,
    getRecentTimeBlocks,
    reprocessTimeBlocks,
    toggleTrackingPause,
    isTauri,
    type AppInfo,
    type AppStatus,
    type CaptureStatus,
    type DatabaseStats,
    type CurrentActivityState,
    type TimeBlock,
  } from "./lib/api";

  let appInfo = $state<AppInfo | null>(null);
  let appStatus = $state<AppStatus | null>(null);
  let captureStatus = $state<CaptureStatus | null>(null);
  let dbStats = $state<DatabaseStats | null>(null);
  let currentActivity = $state<CurrentActivityState | null>(null);
  let recentBlocks = $state<TimeBlock[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let activeTab = $state<"timeline" | "overview" | "blocks" | "capture" | "storage" | "architecture">("timeline");
  let isReprocessing = $state(false);
  let reprocessMessage = $state<string | null>(null);

  async function loadData() {
    loading = true;
    error = null;
    try {
      const [info, status, capture, db, activity, blocks] = await Promise.all([
        getAppInfo(),
        getAppStatus(),
        getCaptureStatus(),
        getDatabaseStats(),
        getCurrentActivity(),
        getRecentTimeBlocks(25),
      ]);
      appInfo = info;
      appStatus = status;
      captureStatus = capture;
      dbStats = db;
      currentActivity = activity;
      recentBlocks = blocks;
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      loading = false;
    }
  }

  async function handleTogglePause() {
    try {
      const nextState = await toggleTrackingPause();
      if (appStatus) {
        appStatus.is_tracking_paused = nextState;
      }
      if (captureStatus) {
        captureStatus.is_tracking_paused = nextState;
      }
      const updated = await getCaptureStatus();
      captureStatus = updated;
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }

  async function handleReprocess() {
    isReprocessing = true;
    reprocessMessage = null;
    try {
      const count = await reprocessTimeBlocks();
      reprocessMessage = `Successfully processed ${count} time blocks from raw history.`;
      const [blocks, db] = await Promise.all([getRecentTimeBlocks(25), getDatabaseStats()]);
      recentBlocks = blocks;
      dbStats = db;
    } catch (err) {
      reprocessMessage = `Reprocessing error: ${err instanceof Error ? err.message : String(err)}`;
    } finally {
      isReprocessing = false;
    }
  }

  function formatTime(timestampMs: number): string {
    const d = new Date(timestampMs);
    return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
  }

  function formatDuration(ms: number): string {
    const totalSeconds = Math.floor(ms / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes}m ${seconds.toString().padStart(2, "0")}s`;
  }

  function getActivityBadgeVariant(type: string): "default" | "success" | "warning" | "neutral" | "error" {
    switch (type) {
      case "active":
        return "success";
      case "afk":
        return "warning";
      case "unknown":
        return "neutral";
      default:
        return "default";
    }
  }

  onMount(() => {
    loadData();
    const interval = setInterval(async () => {
      try {
        const [status, capture, db, activity, blocks] = await Promise.all([
          getAppStatus(),
          getCaptureStatus(),
          getDatabaseStats(),
          getCurrentActivity(),
          getRecentTimeBlocks(25),
        ]);
        appStatus = status;
        captureStatus = capture;
        dbStats = db;
        currentActivity = activity;
        recentBlocks = blocks;
      } catch {
        // Silently ignore background polling errors in browser mode
      }
    }, 3000);

    return () => clearInterval(interval);
  });
</script>

<div class="min-h-screen bg-neutral-950 text-neutral-100 flex flex-col font-sans selection:bg-neutral-800">
  <Header
    appName={appInfo?.name ?? "Tendly"}
    version={appInfo?.version ?? "0.1.0"}
    environment={appInfo?.environment ?? "native"}
    isPaused={appStatus?.is_tracking_paused ?? false}
    onTogglePause={handleTogglePause}
  />

  <nav class="border-b border-neutral-800/80 bg-neutral-900/40 px-6">
    <div class="flex space-x-6 text-xs font-mono overflow-x-auto">
      <button
        onclick={() => (activeTab = "timeline")}
        class="py-3 border-b-2 transition shrink-0 {activeTab === "timeline"
          ? "border-neutral-200 text-neutral-100 font-semibold"
          : "border-transparent text-neutral-400 hover:text-neutral-200"}"
      >
        Activity Timeline
      </button>
      <button
        onclick={() => (activeTab = "overview")}
        class="py-3 border-b-2 transition shrink-0 {activeTab === "overview"
          ? "border-neutral-200 text-neutral-100 font-semibold"
          : "border-transparent text-neutral-400 hover:text-neutral-200"}"
      >
        System Overview
      </button>
      <button
        onclick={() => (activeTab = "blocks")}
        class="py-3 border-b-2 transition shrink-0 {activeTab === "blocks"
          ? "border-neutral-200 text-neutral-100 font-semibold"
          : "border-transparent text-neutral-400 hover:text-neutral-200"}"
      >
        Time Blocks
      </button>
      <button
        onclick={() => (activeTab = "capture")}
        class="py-3 border-b-2 transition shrink-0 {activeTab === "capture"
          ? "border-neutral-200 text-neutral-100 font-semibold"
          : "border-transparent text-neutral-400 hover:text-neutral-200"}"
      >
        Linux Watchers
      </button>
      <button
        onclick={() => (activeTab = "storage")}
        class="py-3 border-b-2 transition shrink-0 {activeTab === "storage"
          ? "border-neutral-200 text-neutral-100 font-semibold"
          : "border-transparent text-neutral-400 hover:text-neutral-200"}"
      >
        Database & Storage
      </button>
      <button
        onclick={() => (activeTab = "architecture")}
        class="py-3 border-b-2 transition shrink-0 {activeTab === "architecture"
          ? "border-neutral-200 text-neutral-100 font-semibold"
          : "border-transparent text-neutral-400 hover:text-neutral-200"}"
      >
        Pipeline Architecture
      </button>
    </div>
  </nav>

  <main class="flex-1 p-6 max-w-6xl w-full mx-auto space-y-6">
    {#if !isTauri()}
      <div class="rounded border border-amber-800/50 bg-amber-950/20 px-4 py-3 text-xs font-mono text-amber-300">
        Notice: Running in web browser preview mode. Native Tauri IPC is active when launched inside the desktop container (run via npm run tauri dev).
      </div>
    {/if}

    {#if loading}
      <LoadingState message="Connecting to Tendly Rust Core via Tauri IPC..." />
    {:else if error}
      <ErrorState title="IPC Bridge Error" message={error} onRetry={loadData} />
    {:else if activeTab === "timeline"}
      <Timeline {currentActivity} />
    {:else if activeTab === "overview"}
      <div class="rounded border border-neutral-800 bg-neutral-900/60 p-5 font-mono">
        <div class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 border-b border-neutral-800/80 pb-4">
          <div>
            <div class="text-[11px] uppercase tracking-wider text-neutral-400 font-medium">Live Activity State</div>
            <div class="text-lg font-semibold text-neutral-100 mt-0.5 flex items-center gap-3">
              <span>{currentActivity ? currentActivity.active_app : "No active input detected"}</span>
              {#if currentActivity}
                <Badge
                  variant={getActivityBadgeVariant(currentActivity.activity_type)}
                  text={currentActivity.activity_type.toUpperCase()}
                />
              {/if}
            </div>
          </div>
          <div class="text-right">
            <div class="text-[11px] uppercase tracking-wider text-neutral-500">Elapsed in State</div>
            <div class="text-base text-neutral-300 font-semibold mt-0.5">
              {currentActivity ? `${currentActivity.elapsed_in_state_seconds}s` : "0s"}
            </div>
          </div>
        </div>
        <div class="pt-3 text-xs text-neutral-400 flex flex-col md:flex-row justify-between gap-2">
          <div class="truncate max-w-2xl">
            <span class="text-neutral-500">Active Window:</span>
            <span class="text-neutral-200 ml-1">{currentActivity?.active_title || "None"}</span>
          </div>
          <div class="text-neutral-500 shrink-0">
            Current Block: {currentActivity?.current_block ? `${formatTime(currentActivity.current_block.start_ms)} (${currentActivity.current_block.dominant_app})` : "Aggregating in epoch"}
          </div>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-3 gap-5">
        <Card title="Application Identity" subtitle="Metadata reported by Rust core">
          <dl class="space-y-2 text-xs font-mono">
            <div class="flex justify-between py-1 border-b border-neutral-800">
              <dt class="text-neutral-400">Binary Name</dt>
              <dd class="text-neutral-200">{appInfo?.name}</dd>
            </div>
            <div class="flex justify-between py-1 border-b border-neutral-800">
              <dt class="text-neutral-400">Version</dt>
              <dd class="text-neutral-200">{appInfo?.version}</dd>
            </div>
            <div class="flex justify-between py-1 border-b border-neutral-800">
              <dt class="text-neutral-400">Target OS</dt>
              <dd class="text-neutral-200">{appInfo?.os}</dd>
            </div>
            <div class="flex justify-between py-1">
              <dt class="text-neutral-400">Environment</dt>
              <dd class="text-neutral-200">{appInfo?.environment}</dd>
            </div>
          </dl>
        </Card>

        <Card title="Runtime Status" subtitle="In-process core state">
          <dl class="space-y-2 text-xs font-mono">
            <div class="flex justify-between py-1 border-b border-neutral-800">
              <dt class="text-neutral-400">Tracking Status</dt>
              <dd>
                <Badge
                  variant={appStatus?.is_tracking_paused ? "warning" : "success"}
                  text={appStatus?.is_tracking_paused ? "Paused" : "Active"}
                />
              </dd>
            </div>
            <div class="flex justify-between py-1 border-b border-neutral-800">
              <dt class="text-neutral-400">Active Watchers</dt>
              <dd class="text-neutral-200">{captureStatus?.active_watchers_count ?? 0}</dd>
            </div>
            <div class="flex justify-between py-1">
              <dt class="text-neutral-400">Process Uptime</dt>
              <dd class="text-neutral-200">{appStatus?.uptime_seconds}s</dd>
            </div>
          </dl>
        </Card>

        <Card title="Storage Aggregation" subtitle="Deterministic SQLite blocks">
          <dl class="space-y-2 text-xs font-mono">
            <div class="flex justify-between py-1 border-b border-neutral-800">
              <dt class="text-neutral-400">Schema Version</dt>
              <dd class="text-neutral-200">v{appInfo?.schema_version}</dd>
            </div>
            <div class="flex justify-between py-1 border-b border-neutral-800">
              <dt class="text-neutral-400">Raw Events Stored</dt>
              <dd class="text-neutral-200">{dbStats?.raw_events_count ?? 0}</dd>
            </div>
            <div class="flex justify-between py-1">
              <dt class="text-neutral-400">Time Blocks Derived</dt>
              <dd class="text-emerald-400 font-semibold">{dbStats?.blocks_count ?? 0}</dd>
            </div>
          </dl>
        </Card>
      </div>

      <Card title="Recent Activity Time Blocks" subtitle="Most recent 3-minute aggregated intervals">
        {#if recentBlocks.length === 0}
          <EmptyState
            title="No Time Blocks Generated Yet"
            description="As you use your computer, raw window events are automatically aggregated into standard 3-minute epoch blocks."
          />
        {:else}
          <div class="overflow-x-auto">
            <table class="w-full text-left text-xs font-mono">
              <thead class="border-b border-neutral-800 text-neutral-400 uppercase text-[11px]">
                <tr>
                  <th class="py-2.5 px-3">Interval</th>
                  <th class="py-2.5 px-3">Duration</th>
                  <th class="py-2.5 px-3">Type</th>
                  <th class="py-2.5 px-3">Dominant App</th>
                  <th class="py-2.5 px-3">Dominant Title</th>
                  <th class="py-2.5 px-3 text-right">Status</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-neutral-800/60">
                {#each recentBlocks.slice(0, 5) as block}
                  <tr class="hover:bg-neutral-900/40 transition">
                    <td class="py-2.5 px-3 text-neutral-300 font-medium">
                      {formatTime(block.start_ms)} - {formatTime(block.end_ms)}
                    </td>
                    <td class="py-2.5 px-3 text-neutral-400">
                      {formatDuration(block.duration_ms)}
                    </td>
                    <td class="py-2.5 px-3">
                      <Badge
                        variant={getActivityBadgeVariant(block.activity_type)}
                        text={block.activity_type.toUpperCase()}
                      />
                    </td>
                    <td class="py-2.5 px-3 text-neutral-200 font-semibold">
                      {block.dominant_app}
                    </td>
                    <td class="py-2.5 px-3 text-neutral-400 max-w-xs truncate" title={block.dominant_title}>
                      {block.dominant_title}
                    </td>
                    <td class="py-2.5 px-3 text-right text-neutral-500">
                      Unclassified
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </Card>
    {:else if activeTab === "blocks"}
      <div class="space-y-5">
        <div class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 bg-neutral-900/60 border border-neutral-800 p-4 rounded font-mono text-xs">
          <div>
            <div class="font-semibold text-neutral-200 text-sm">Derived Time Blocks (Model C Hybrid)</div>
            <div class="text-neutral-400 mt-1">
              Raw events are continuously partitioned into 3-minute epoch blocks. Time blocks are completely derived and can be reprocessed idempotently at any time.
            </div>
          </div>
          <button
            onclick={handleReprocess}
            disabled={isReprocessing}
            class="shrink-0 px-4 py-2 bg-neutral-800 hover:bg-neutral-700 disabled:opacity-50 text-neutral-200 border border-neutral-700 rounded transition font-mono font-medium"
          >
            {isReprocessing ? "Reprocessing History..." : "Reprocess All History"}
          </button>
        </div>

        {#if reprocessMessage}
          <div class="p-3 rounded border border-neutral-800 bg-neutral-900 font-mono text-xs text-neutral-300">
            {reprocessMessage}
          </div>
        {/if}

        <Card title="All Recent Time Blocks" subtitle="Sorted chronologically (most recent first)">
          {#if recentBlocks.length === 0}
            <EmptyState
              title="No Time Blocks in Database"
              description="Keep Tendly running or click Reprocess All History if raw events already exist in the database."
            />
          {:else}
            <div class="overflow-x-auto">
              <table class="w-full text-left text-xs font-mono">
                <thead class="border-b border-neutral-800 text-neutral-400 uppercase text-[11px]">
                  <tr>
                    <th class="py-2.5 px-3">Interval</th>
                    <th class="py-2.5 px-3">Duration</th>
                    <th class="py-2.5 px-3">Type</th>
                    <th class="py-2.5 px-3">Dominant App</th>
                    <th class="py-2.5 px-3">Dominant Title</th>
                    <th class="py-2.5 px-3">Block ID</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-neutral-800/60">
                  {#each recentBlocks as block}
                    <tr class="hover:bg-neutral-900/40 transition">
                      <td class="py-2.5 px-3 text-neutral-300 font-medium whitespace-nowrap">
                        {formatTime(block.start_ms)} - {formatTime(block.end_ms)}
                      </td>
                      <td class="py-2.5 px-3 text-neutral-400 whitespace-nowrap">
                        {formatDuration(block.duration_ms)}
                      </td>
                      <td class="py-2.5 px-3">
                        <Badge
                          variant={getActivityBadgeVariant(block.activity_type)}
                          text={block.activity_type.toUpperCase()}
                        />
                      </td>
                      <td class="py-2.5 px-3 text-neutral-200 font-semibold whitespace-nowrap">
                        {block.dominant_app}
                      </td>
                      <td class="py-2.5 px-3 text-neutral-400 max-w-sm truncate" title={block.dominant_title}>
                        {block.dominant_title}
                      </td>
                      <td class="py-2.5 px-3 text-neutral-600 text-[10px] font-mono whitespace-nowrap">
                        {block.id.slice(0, 8)}...
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {/if}
        </Card>
      </div>
    {:else if activeTab === "capture"}
      <div class="space-y-5">
        <Card title="Registered Activity Watchers" subtitle="In-process Rust capture threads">
          <div class="space-y-4 text-xs font-mono">
            {#each captureStatus?.watchers ?? [] as watcher}
              <div class="border border-neutral-800 p-4 rounded bg-neutral-950 space-y-2">
                <div class="flex justify-between items-center border-b border-neutral-800/80 pb-2">
                  <span class="text-sm font-semibold text-neutral-100">{watcher.name}</span>
                  <Badge
                    variant={watcher.running ? (watcher.paused ? "warning" : "success") : (watcher.supported ? "neutral" : "error")}
                    text={watcher.running ? (watcher.paused ? "Paused" : "Running") : (watcher.supported ? "Supported" : "Unsupported")}
                  />
                </div>
                <div class="grid grid-cols-2 md:grid-cols-4 gap-2 text-neutral-400 pt-1">
                  <div>Environment Supported: <span class={watcher.supported ? "text-emerald-400" : "text-neutral-500"}>{watcher.supported ? "Yes" : "No"}</span></div>
                  <div>Worker Thread: <span class={watcher.running ? "text-emerald-400" : "text-neutral-500"}>{watcher.running ? "Active" : "Inactive"}</span></div>
                  <div>Capture State: <span class={watcher.paused ? "text-amber-400" : "text-emerald-400"}>{watcher.paused ? "Paused" : "Active"}</span></div>
                  <div>Error Code: <span class="text-neutral-400">{watcher.last_error ?? "None"}</span></div>
                </div>
              </div>
            {/each}
          </div>
        </Card>

        <Card title="Capture Stream Statistics" subtitle="Real activity metrics">
          <div class="grid grid-cols-1 md:grid-cols-3 gap-4 text-xs font-mono">
            <div class="border border-neutral-800 p-3 rounded bg-neutral-950">
              <div class="text-neutral-500 text-[11px] uppercase">Raw Events Persisted</div>
              <div class="text-xl font-semibold text-neutral-100 mt-1">{dbStats?.raw_events_count ?? 0}</div>
            </div>
            <div class="border border-neutral-800 p-3 rounded bg-neutral-950">
              <div class="text-neutral-500 text-[11px] uppercase">Active Watcher Count</div>
              <div class="text-xl font-semibold text-neutral-100 mt-1">{captureStatus?.active_watchers_count ?? 0}</div>
            </div>
            <div class="border border-neutral-800 p-3 rounded bg-neutral-950">
              <div class="text-neutral-500 text-[11px] uppercase">Tracking State</div>
              <div class="text-xl font-semibold {captureStatus?.is_tracking_paused ? "text-amber-400" : "text-emerald-400"} mt-1">
                {captureStatus?.is_tracking_paused ? "PAUSED" : "TRACKING"}
              </div>
            </div>
          </div>
        </Card>
      </div>
    {:else if activeTab === "storage"}
      <div class="space-y-5">
        <Card title="Local Storage Details" subtitle="Strictly on-device SQLite database">
          <dl class="space-y-3 text-xs font-mono">
            <div class="py-2 border-b border-neutral-800">
              <dt class="text-neutral-400 mb-1">Filesystem Path</dt>
              <dd class="text-neutral-200 bg-neutral-950 p-2 rounded border border-neutral-800 break-all">
                {dbStats?.database_path}
              </dd>
            </div>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4 pt-1">
              <div class="border border-neutral-800 p-3 rounded bg-neutral-950/40">
                <dt class="text-neutral-500 text-[11px] uppercase">Database Size</dt>
                <dd class="text-base font-semibold text-neutral-200 mt-1">{dbStats?.database_size_bytes} bytes</dd>
              </div>
              <div class="border border-neutral-800 p-3 rounded bg-neutral-950/40">
                <dt class="text-neutral-500 text-[11px] uppercase">Raw Events (Canonical)</dt>
                <dd class="text-base font-semibold text-neutral-200 mt-1">{dbStats?.raw_events_count}</dd>
              </div>
              <div class="border border-neutral-800 p-3 rounded bg-neutral-950/40">
                <dt class="text-neutral-500 text-[11px] uppercase">Derived Time Blocks</dt>
                <dd class="text-base font-semibold text-emerald-400 mt-1">{dbStats?.blocks_count}</dd>
              </div>
            </div>
          </dl>
        </Card>

        <Card title="Storage Invariants & Integrity" subtitle="Guarantees established in Phase 3">
          <div class="space-y-2 text-xs text-neutral-300 font-mono">
            <p class="py-1 border-b border-neutral-800/80">Canonical Source of Truth: Raw events are immutable and preserved. Time blocks are purely derived.</p>
            <p class="py-1 border-b border-neutral-800/80">Idempotent Reprocessing: Rebuilding history from raw events replaces derived blocks deterministically with zero duplicates.</p>
            <p class="py-1 border-b border-neutral-800/80">Gap Integrity: Unobserved gaps > 180s (system sleep, suspend) are explicitly recorded as Unknown and never assumed active.</p>
            <p class="py-1">Local Isolation: SQLite database file is stored locally under ~/.local/share/tendly/tendly.db with Unix 0600 permissions.</p>
          </div>
        </Card>
      </div>
    {:else if activeTab === "architecture"}
      <Card title="Activity Processing & Aggregation Pipeline" subtitle="Model C Hybrid Processing (Phase 3)">
        <div class="space-y-4 text-xs font-mono text-neutral-300 leading-relaxed">
          <div class="p-3 rounded bg-neutral-950 border border-neutral-800">
            <div class="font-semibold text-neutral-100 mb-1">1. Canonical Raw Observation Stream (RawEvent)</div>
            <p class="text-neutral-400">
              In-process watchers (X11, wlroots Wayland, AFK) write atomic observations to SQLite table raw_events.
              Events are immutable, timestamped in epoch milliseconds, and store window title, application name, and idle status.
            </p>
          </div>

          <div class="p-3 rounded bg-neutral-950 border border-neutral-800">
            <div class="font-semibold text-neutral-100 mb-1">2. Segment Reconstruction (segment.rs)</div>
            <p class="text-neutral-400">
              Chronologically sorts raw events, resolves event boundaries, limits active runs before unobserved gaps at 60s grace,
              creates explicit Unknown segments for sleep/suspend periods, and coalesces contiguous runs of identical activity.
            </p>
          </div>

          <div class="p-3 rounded bg-neutral-950 border border-neutral-800">
            <div class="font-semibold text-neutral-100 mb-1">3. Epoch Interval Bucketing (aggregator.rs)</div>
            <p class="text-neutral-400">
              Partitions continuous segments into standard 3-minute epoch blocks aligned to T - (T % 180_000).
              Calculates duration per application and window title within each epoch block.
            </p>
          </div>

          <div class="p-3 rounded bg-neutral-950 border border-neutral-800">
            <div class="font-semibold text-neutral-100 mb-1">4. Plurality Attribution & Persistence</div>
            <p class="text-neutral-400">
              The application and window title with the plurality of active duration is attributed as dominant.
              Block IDs are deterministically generated via UUID v5 from timeblock:[start_ms], guaranteeing complete idempotency on rebuild.
            </p>
          </div>
        </div>
      </Card>
    {/if}
  </main>

  <footer class="border-t border-neutral-800/80 bg-neutral-950 px-6 py-3 text-xs font-mono text-neutral-500 flex justify-between">
    <span>Tendly Foundation Phase 3</span>
    <span>MPL-2.0 Open Source</span>
  </footer>
</div>
