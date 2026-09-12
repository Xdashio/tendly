<script lang="ts">
  import Badge from "./Badge.svelte";
  import EmptyState from "./EmptyState.svelte";
  import LoadingState from "./LoadingState.svelte";
  import ErrorState from "./ErrorState.svelte";
  import {
    getDailyTimeline,
    getSessionDetails,
    type DailyTimeline,
    type ActivitySession,
    type SessionDetails,
    type CurrentActivityState,
  } from "../api";

  interface Props {
    currentActivity: CurrentActivityState | null;
  }

  let { currentActivity }: Props = $props();

  function getTodayString(): string {
    const d = new Date();
    const year = d.getFullYear();
    const month = String(d.getMonth() + 1).padStart(2, "0");
    const day = String(d.getDate()).padStart(2, "0");
    return `${year}-${month}-${day}`;
  }

  let selectedDate = $state<string>(getTodayString());
  let timeline = $state<DailyTimeline | null>(null);
  let loading = $state<boolean>(true);
  let error = $state<string | null>(null);

  // Drill-down cache and expanded session state
  let expandedSessionId = $state<string | null>(null);
  let sessionDetailsCache = $state<Record<string, SessionDetails>>({});
  let loadingDetails = $state<boolean>(false);

  function getDayRange(dateStr: string): { startMs: number; endMs: number } {
    const [year, month, day] = dateStr.split("-").map(Number);
    const start = new Date(year, month - 1, day, 0, 0, 0, 0);
    const end = new Date(year, month - 1, day + 1, 0, 0, 0, 0);
    return { startMs: start.getTime(), endMs: end.getTime() };
  }

  export async function loadTimeline() {
    loading = true;
    error = null;
    try {
      const { startMs, endMs } = getDayRange(selectedDate);
      const res = await getDailyTimeline(startMs, endMs);
      timeline = res;
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      loading = false;
    }
  }

  function changeDateByOffset(offsetDays: number) {
    const [year, month, day] = selectedDate.split("-").map(Number);
    const d = new Date(year, month - 1, day + offsetDays);
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, "0");
    const dt = String(d.getDate()).padStart(2, "0");
    selectedDate = `${y}-${m}-${dt}`;
  }

  function jumpToToday() {
    selectedDate = getTodayString();
  }

  $effect(() => {
    // Re-load timeline whenever selectedDate changes
    if (selectedDate) {
      loadTimeline();
    }
  });

  async function toggleSessionExpansion(session: ActivitySession) {
    if (expandedSessionId === session.id) {
      expandedSessionId = null;
      return;
    }

    expandedSessionId = session.id;

    if (!sessionDetailsCache[session.id] && session.dominant_app !== "unrecorded") {
      loadingDetails = true;
      try {
        const details = await getSessionDetails(session.id, session.start_ms, session.end_ms);
        sessionDetailsCache[session.id] = details;
      } catch (err) {
        console.error("Failed to load session details", err);
      } finally {
        loadingDetails = false;
      }
    }
  }

  function formatTime(timestampMs: number): string {
    const d = new Date(timestampMs);
    return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
  }

  function formatDuration(ms: number): string {
    if (ms <= 0) return "0m";
    const totalMinutes = Math.round(ms / 60000);
    if (totalMinutes < 60) {
      return `${totalMinutes}m`;
    }
    const hours = Math.floor(totalMinutes / 60);
    const minutes = totalMinutes % 60;
    return minutes > 0 ? `${hours}h ${minutes}m` : `${hours}h`;
  }

  function formatDetailedDuration(ms: number): string {
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

  const isToday = $derived(selectedDate === getTodayString());
  const isFutureDate = $derived(selectedDate > getTodayString());

  $effect(() => {
    if (!isToday) return;

    const timer = setInterval(async () => {
      try {
        const { startMs, endMs } = getDayRange(selectedDate);
        const res = await getDailyTimeline(startMs, endMs);
        timeline = res;
      } catch {
        // Silently ignore background refresh errors
      }
    }, 15000);

    return () => clearInterval(timer);
  });
</script>

<div class="space-y-6">
  <!-- Date Navigation & Summary Bar -->
  <div class="rounded border border-neutral-800 bg-neutral-900/60 p-4 font-mono">
    <div class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
      <div class="flex items-center gap-2">
        <button
          onclick={() => changeDateByOffset(-1)}
          class="px-2.5 py-1 text-xs rounded border border-neutral-700 bg-neutral-800 hover:bg-neutral-700 text-neutral-200 transition"
          title="Previous Day"
        >
          &lt; Prev
        </button>
        <input
          type="date"
          bind:value={selectedDate}
          class="px-2.5 py-1 text-xs rounded border border-neutral-700 bg-neutral-950 text-neutral-200 focus:outline-none focus:border-neutral-500"
        />
        <button
          onclick={() => changeDateByOffset(1)}
          disabled={selectedDate >= getTodayString()}
          class="px-2.5 py-1 text-xs rounded border transition {selectedDate >= getTodayString()
            ? 'border-neutral-800 bg-neutral-900/50 text-neutral-600 cursor-not-allowed'
            : 'border-neutral-700 bg-neutral-800 hover:bg-neutral-700 text-neutral-200'}"
          title="Next Day"
        >
          Next &gt;
        </button>
        {#if !isToday}
          <button
            onclick={jumpToToday}
            class="px-2.5 py-1 text-xs rounded border border-neutral-600 bg-neutral-700 hover:bg-neutral-600 text-neutral-100 transition"
          >
            Today
          </button>
        {/if}
      </div>

      <!-- Daily Totals -->
      <div class="flex flex-wrap items-center gap-4 text-xs">
        <div class="flex items-center gap-1.5">
          <span class="text-neutral-500">Active:</span>
          <span class="font-semibold text-emerald-400">{formatDuration(timeline?.total_active_ms ?? 0)}</span>
        </div>
        <div class="flex items-center gap-1.5">
          <span class="text-neutral-500">Idle/AFK:</span>
          <span class="font-semibold text-amber-400">{formatDuration(timeline?.total_afk_ms ?? 0)}</span>
        </div>
        {#if (timeline?.total_unknown_ms ?? 0) > 0}
          <div class="flex items-center gap-1.5">
            <span class="text-neutral-500">Unrecorded:</span>
            <span class="font-semibold text-neutral-400">{formatDuration(timeline?.total_unknown_ms ?? 0)}</span>
          </div>
        {/if}
        <div class="flex items-center gap-1.5 border-l border-neutral-800 pl-4">
          <span class="text-neutral-500">Blocks:</span>
          <span class="text-neutral-300">{timeline?.block_count ?? 0}</span>
        </div>
      </div>
    </div>
  </div>

  <!-- Live Activity Indicator (Only on Today) -->
  {#if isToday && currentActivity}
    <div class="rounded border border-neutral-800 bg-neutral-900/60 px-4 py-3 text-xs font-mono flex items-center justify-between">
      <div class="flex items-center gap-2.5">
        {#if currentActivity.activity_type === "unknown"}
          <span class="inline-block w-2 h-2 rounded-full bg-neutral-500"></span>
          <span class="text-neutral-500">Current status:</span>
          <span class="text-neutral-400 italic">No active window input detected</span>
        {:else if currentActivity.activity_type === "afk"}
          <span class="inline-block w-2 h-2 rounded-full bg-amber-400"></span>
          <span class="text-neutral-400">Current status:</span>
          <span class="font-semibold text-amber-300">Away from keyboard (AFK)</span>
        {:else}
          <span class="inline-block w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
          <span class="text-neutral-400">Current activity:</span>
          <span class="font-semibold text-neutral-200">{currentActivity.active_app}</span>
          <span class="text-neutral-500 truncate max-w-md">({currentActivity.active_title})</span>
        {/if}
      </div>
      <div class="text-neutral-400 shrink-0">
        {currentActivity.elapsed_in_state_seconds}s in state
      </div>
    </div>
  {/if}

  <!-- Timeline Body -->
  {#if loading}
    <LoadingState message="Loading activity timeline for {selectedDate}..." />
  {:else if error}
    <ErrorState title="Timeline Error" message={error} onRetry={loadTimeline} />
  {:else if isFutureDate}
    <EmptyState
      title="Future date: {selectedDate}"
      description="Tendly cannot record activity for future dates. Return to Today or select a past date."
    />
  {:else if !timeline || timeline.sessions.length === 0}
    <EmptyState
      title="No recorded activity for {selectedDate}"
      description="Tendly did not record any desktop activity during this 24-hour window. Ensure watchers are active."
    />
  {:else}
    <div class="space-y-3 font-mono">
      {#each timeline.sessions as session (session.id)}
        {#if session.dominant_app === "unrecorded"}
          <!-- Unrecorded Gap Session -->
          <div class="rounded border border-dashed border-neutral-800/80 bg-neutral-950/40 px-4 py-2.5 text-xs text-neutral-500 flex justify-between items-center">
            <div class="flex items-center gap-3">
              <span class="text-neutral-600">{formatTime(session.start_ms)} - {formatTime(session.end_ms)}</span>
              <span class="italic">No recorded activity</span>
            </div>
            <span>{formatDuration(session.duration_ms)}</span>
          </div>
        {:else}
          <!-- Recorded Activity Session Card -->
          <div class="rounded border border-neutral-800 bg-neutral-900/50 hover:bg-neutral-900/80 transition duration-150 overflow-hidden">
            <div
              role="button"
              tabindex="0"
              onclick={() => toggleSessionExpansion(session)}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') toggleSessionExpansion(session); }}
              class="p-4 cursor-pointer flex flex-col md:flex-row justify-between items-start md:items-center gap-3 select-none"
            >
              <div class="space-y-1 min-w-0 flex-1">
                <div class="flex items-center gap-2.5">
                  <span class="text-sm font-semibold text-neutral-100">{session.dominant_app}</span>
                  <Badge
                    variant={getActivityBadgeVariant(session.activity_type)}
                    text={session.activity_type.toUpperCase()}
                  />
                  <span class="text-xs text-neutral-400 font-medium">
                    {formatDuration(session.duration_ms)}
                  </span>
                  {#if session.block_count > 1}
                    <span class="text-[11px] text-neutral-500">
                      ({session.block_count} blocks)
                    </span>
                  {/if}
                </div>
                <div class="text-xs text-neutral-400 truncate max-w-2xl">
                  {session.dominant_title}
                </div>

                <!-- Secondary Activity Inline Preview -->
                {#if session.has_secondary_activity && session.secondary_apps.length > 0}
                  <div class="pt-1 text-[11px] text-neutral-500 flex flex-wrap gap-2">
                    <span>Includes:</span>
                    {#each session.secondary_apps as sec}
                      <span class="text-neutral-400">
                        {sec.app} ({formatDuration(sec.duration_ms)})
                      </span>
                    {/each}
                  </div>
                {/if}
              </div>

              <div class="flex items-center gap-4 text-xs text-neutral-400 shrink-0 self-end md:self-center">
                <div class="text-right">
                  <div>{formatTime(session.start_ms)} - {formatTime(session.end_ms)}</div>
                </div>
                <button
                  type="button"
                  class="px-2 py-1 text-[11px] rounded border border-neutral-700 bg-neutral-800 hover:bg-neutral-700 text-neutral-300"
                >
                  {expandedSessionId === session.id ? "Hide Details [-]" : "Inspect [+]"}
                </button>
              </div>
            </div>

            <!-- Expanded Drill-Down Detail -->
            {#if expandedSessionId === session.id}
              <div class="border-t border-neutral-800/80 bg-neutral-950/60 p-4 space-y-4 text-xs">
                {#if loadingDetails && !sessionDetailsCache[session.id]}
                  <div class="py-3 text-center text-neutral-500 text-xs">
                    Reconstructing sub-minute composition from raw events...
                  </div>
                {:else}
                  {@const details = sessionDetailsCache[session.id]}
                  {#if details}
                    <!-- App Breakdown Summary -->
                    <div class="space-y-2">
                      <div class="text-[11px] uppercase tracking-wider text-neutral-400 font-semibold">
                        Session Application Breakdown
                      </div>
                      <div class="space-y-1.5">
                        {#each details.app_breakdown as item}
                          {@const pct = Math.round((item.duration_ms / session.duration_ms) * 100)}
                          <div class="flex items-center gap-3">
                            <span class="w-24 truncate text-neutral-300">{item.app}</span>
                            <div class="flex-1 h-1.5 rounded-full bg-neutral-800 overflow-hidden">
                              <div
                                class="h-full bg-neutral-400"
                                style="width: {pct}%"
                              ></div>
                            </div>
                            <span class="w-16 text-right text-neutral-400">{formatDetailedDuration(item.duration_ms)}</span>
                            <span class="w-10 text-right text-neutral-500 text-[11px]">{pct}%</span>
                          </div>
                        {/each}
                      </div>
                    </div>

                    <!-- Exact Sub-Minute Activity Segments -->
                    <div class="space-y-2 pt-2 border-t border-neutral-800/60">
                      <div class="text-[11px] uppercase tracking-wider text-neutral-400 font-semibold">
                        Underlying Activity Segments ({details.segments.length} continuous runs)
                      </div>
                      <div class="max-h-56 overflow-y-auto rounded border border-neutral-800/60 bg-neutral-900/30 divide-y divide-neutral-800/40">
                        {#each details.segments as seg}
                          <div class="px-3 py-2 flex flex-col md:flex-row justify-between items-start md:items-center gap-2 hover:bg-neutral-800/20 text-[11px]">
                            <div class="flex items-center gap-2 truncate flex-1 min-w-0">
                              <span class="font-semibold text-neutral-300 shrink-0">{seg.app}</span>
                              <span class="text-neutral-400 truncate">{seg.title}</span>
                            </div>
                            <div class="flex items-center gap-3 text-neutral-500 shrink-0">
                              <span>{formatTime(seg.start_ms)} - {formatTime(seg.end_ms)}</span>
                              <span class="font-medium text-neutral-400">{formatDetailedDuration(seg.end_ms - seg.start_ms)}</span>
                            </div>
                          </div>
                        {/each}
                      </div>
                    </div>

                    <!-- Underlying TimeBlocks -->
                    {#if session.time_blocks && session.time_blocks.length > 0}
                      <div class="space-y-2 pt-2 border-t border-neutral-800/60">
                        <div class="text-[11px] uppercase tracking-wider text-neutral-400 font-semibold">
                          Analytical 3-Minute TimeBlocks ({session.time_blocks.length} blocks)
                        </div>
                        <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-2">
                          {#each session.time_blocks as block}
                            <div class="p-2 rounded border border-neutral-800/60 bg-neutral-900/40 text-[11px]">
                              <div class="flex justify-between text-neutral-400">
                                <span>{formatTime(block.start_ms)}</span>
                                <span>3m epoch</span>
                              </div>
                              <div class="mt-1 font-semibold text-neutral-200 truncate">
                                {block.dominant_app}
                              </div>
                              <div class="text-neutral-500 truncate text-[10px]">
                                {block.dominant_title}
                              </div>
                            </div>
                          {/each}
                        </div>
                      </div>
                    {/if}
                  {/if}
                {/if}
              </div>
            {/if}
          </div>
        {/if}
      {/each}
    </div>
  {/if}
</div>
