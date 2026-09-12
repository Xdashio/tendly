<script lang="ts">
  import { onMount } from 'svelte';
  import Header from './lib/components/Header.svelte';
  import Card from './lib/components/Card.svelte';
  import Badge from './lib/components/Badge.svelte';
  import LoadingState from './lib/components/LoadingState.svelte';
  import ErrorState from './lib/components/ErrorState.svelte';
  import EmptyState from './lib/components/EmptyState.svelte';
  import {
    getAppInfo,
    getAppStatus,
    getCaptureStatus,
    getDatabaseStats,
    toggleTrackingPause,
    isTauri,
    type AppInfo,
    type AppStatus,
    type CaptureStatus,
    type DatabaseStats,
  } from './lib/api';

  let appInfo = $state<AppInfo | null>(null);
  let appStatus = $state<AppStatus | null>(null);
  let captureStatus = $state<CaptureStatus | null>(null);
  let dbStats = $state<DatabaseStats | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let activeTab = $state<'overview' | 'capture' | 'storage' | 'architecture'>('overview');

  async function loadData() {
    loading = true;
    error = null;
    try {
      const [info, status, capture, db] = await Promise.all([
        getAppInfo(),
        getAppStatus(),
        getCaptureStatus(),
        getDatabaseStats(),
      ]);
      appInfo = info;
      appStatus = status;
      captureStatus = capture;
      dbStats = db;
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
      // Refresh capture statuses
      const updated = await getCaptureStatus();
      captureStatus = updated;
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }

  onMount(() => {
    loadData();
    // Periodic refresh of status every 3 seconds
    const interval = setInterval(async () => {
      try {
        const [status, capture, db] = await Promise.all([
          getAppStatus(),
          getCaptureStatus(),
          getDatabaseStats(),
        ]);
        appStatus = status;
        captureStatus = capture;
        dbStats = db;
      } catch {
        // Silently ignore background polling errors in browser mode
      }
    }, 3000);

    return () => clearInterval(interval);
  });
</script>

<div class="min-h-screen bg-neutral-950 text-neutral-100 flex flex-col font-sans selection:bg-neutral-800">
  <Header
    appName={appInfo?.name ?? 'Tendly'}
    version={appInfo?.version ?? '0.1.0'}
    environment={appInfo?.environment ?? 'native'}
    isPaused={appStatus?.is_tracking_paused ?? false}
    onTogglePause={handleTogglePause}
  />

  <nav class="border-b border-neutral-800/80 bg-neutral-900/40 px-6">
    <div class="flex space-x-6 text-xs font-mono">
      <button
        onclick={() => (activeTab = 'overview')}
        class="py-3 border-b-2 transition {activeTab === 'overview'
          ? 'border-neutral-200 text-neutral-100 font-semibold'
          : 'border-transparent text-neutral-400 hover:text-neutral-200'}"
      >
        System Overview
      </button>
      <button
        onclick={() => (activeTab = 'capture')}
        class="py-3 border-b-2 transition {activeTab === 'capture'
          ? 'border-neutral-200 text-neutral-100 font-semibold'
          : 'border-transparent text-neutral-400 hover:text-neutral-200'}"
      >
        Linux Watchers
      </button>
      <button
        onclick={() => (activeTab = 'storage')}
        class="py-3 border-b-2 transition {activeTab === 'storage'
          ? 'border-neutral-200 text-neutral-100 font-semibold'
          : 'border-transparent text-neutral-400 hover:text-neutral-200'}"
      >
        Database & Privacy
      </button>
      <button
        onclick={() => (activeTab = 'architecture')}
        class="py-3 border-b-2 transition {activeTab === 'architecture'
          ? 'border-neutral-200 text-neutral-100 font-semibold'
          : 'border-transparent text-neutral-400 hover:text-neutral-200'}"
      >
        Capture Architecture
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
    {:else if activeTab === 'overview'}
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
                  variant={appStatus?.is_tracking_paused ? 'warning' : 'success'}
                  text={appStatus?.is_tracking_paused ? 'Paused' : 'Active'}
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

        <Card title="Database Health" subtitle="Local SQLite persistence">
          <dl class="space-y-2 text-xs font-mono">
            <div class="flex justify-between py-1 border-b border-neutral-800">
              <dt class="text-neutral-400">Schema Version</dt>
              <dd class="text-neutral-200">v{appInfo?.schema_version}</dd>
            </div>
            <div class="flex justify-between py-1 border-b border-neutral-800">
              <dt class="text-neutral-400">Connection State</dt>
              <dd class="text-emerald-400">{appInfo?.database_status}</dd>
            </div>
            <div class="flex justify-between py-1">
              <dt class="text-neutral-400">Raw Events Stored</dt>
              <dd class="text-neutral-200">{dbStats?.raw_events_count ?? 0}</dd>
            </div>
          </dl>
        </Card>
      </div>

      <Card title="Linux Activity Capture Subsystem" subtitle="Phase 2 production capture status">
        <div class="space-y-4">
          <div class="grid grid-cols-1 md:grid-cols-3 gap-3 text-xs font-mono">
            {#each captureStatus?.watchers ?? [] as watcher}
              <div class="border border-neutral-800 p-3 rounded bg-neutral-950">
                <div class="flex justify-between items-center mb-2">
                  <span class="font-semibold text-neutral-200">{watcher.name}</span>
                  <Badge
                    variant={watcher.running ? (watcher.paused ? 'warning' : 'success') : (watcher.supported ? 'neutral' : 'error')}
                    text={watcher.running ? (watcher.paused ? 'Paused' : 'Capturing') : (watcher.supported ? 'Standby' : 'Unsupported')}
                  />
                </div>
                <div class="space-y-1 text-neutral-400 text-[11px]">
                  <div>Supported: <span class={watcher.supported ? 'text-emerald-400' : 'text-neutral-500'}>{watcher.supported ? 'Yes' : 'No'}</span></div>
                  <div>Running: <span class={watcher.running ? 'text-emerald-400' : 'text-neutral-500'}>{watcher.running ? 'Yes' : 'No'}</span></div>
                  <div>Paused: <span class={watcher.paused ? 'text-amber-400' : 'text-neutral-500'}>{watcher.paused ? 'Yes' : 'No'}</span></div>
                </div>
              </div>
            {/each}
          </div>
        </div>
      </Card>
    {:else if activeTab === 'capture'}
      <div class="space-y-5">
        <Card title="Registered Activity Watchers" subtitle="In-process Rust capture threads">
          <div class="space-y-4 text-xs font-mono">
            {#each captureStatus?.watchers ?? [] as watcher}
              <div class="border border-neutral-800 p-4 rounded bg-neutral-950 space-y-2">
                <div class="flex justify-between items-center border-b border-neutral-800/80 pb-2">
                  <span class="text-sm font-semibold text-neutral-100">{watcher.name}</span>
                  <Badge
                    variant={watcher.running ? (watcher.paused ? 'warning' : 'success') : (watcher.supported ? 'neutral' : 'error')}
                    text={watcher.running ? (watcher.paused ? 'Paused' : 'Running') : (watcher.supported ? 'Supported' : 'Unsupported')}
                  />
                </div>
                <div class="grid grid-cols-2 md:grid-cols-4 gap-2 text-neutral-400 pt-1">
                  <div>Environment Supported: <span class={watcher.supported ? 'text-emerald-400' : 'text-neutral-500'}>{watcher.supported ? 'Yes' : 'No'}</span></div>
                  <div>Worker Thread: <span class={watcher.running ? 'text-emerald-400' : 'text-neutral-500'}>{watcher.running ? 'Active' : 'Inactive'}</span></div>
                  <div>Capture State: <span class={watcher.paused ? 'text-amber-400' : 'text-emerald-400'}>{watcher.paused ? 'Paused' : 'Active'}</span></div>
                  <div>Error Code: <span class="text-neutral-400">{watcher.last_error ?? 'None'}</span></div>
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
              <div class="text-xl font-semibold {captureStatus?.is_tracking_paused ? 'text-amber-400' : 'text-emerald-400'} mt-1">
                {captureStatus?.is_tracking_paused ? 'PAUSED' : 'TRACKING'}
              </div>
            </div>
          </div>
        </Card>
      </div>
    {:else if activeTab === 'storage'}
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
                <dt class="text-neutral-500 text-[11px] uppercase">Raw Events</dt>
                <dd class="text-base font-semibold text-neutral-200 mt-1">{dbStats?.raw_events_count}</dd>
              </div>
              <div class="border border-neutral-800 p-3 rounded bg-neutral-950/40">
                <dt class="text-neutral-500 text-[11px] uppercase">Classified Blocks</dt>
                <dd class="text-base font-semibold text-neutral-200 mt-1">{dbStats?.blocks_count}</dd>
              </div>
            </div>
          </dl>
        </Card>

        <Card title="Privacy Model Enforcement" subtitle="Guarantees active in this build">
          <div class="space-y-2 text-xs text-neutral-300 font-mono">
            <p class="py-1 border-b border-neutral-800/80">Local-only storage: Zero outbound telemetry, zero background network listeners.</p>
            <p class="py-1 border-b border-neutral-800/80">Physical deletion guarantee: Nuclear wipe triggers SQLite VACUUM.</p>
            <p class="py-1 border-b border-neutral-800/80">Logging policy: Raw window titles and URLs are strictly omitted from application logs.</p>
            <p class="py-1">Local permissions: Database file restricted to user permissions (0600 on Unix).</p>
          </div>
        </Card>
      </div>
    {:else if activeTab === 'architecture'}
      <Card title="Linux Activity Capture Pipeline" subtitle="Implemented in Phase 2">
        <div class="space-y-4 text-xs font-mono text-neutral-300 leading-relaxed">
          <div class="p-3 rounded bg-neutral-950 border border-neutral-800">
            <div class="font-semibold text-neutral-100 mb-1">X11 Activity Watcher (`x11rb`)</div>
            <p class="text-neutral-400">
              Observes active window via `_NET_ACTIVE_WINDOW` and application name via `WM_CLASS`. Retrieves window title from `_NET_WM_NAME`.
              Runs hybrid event-driven loop with 2s heartbeat.
            </p>
          </div>

          <div class="p-3 rounded bg-neutral-950 border border-neutral-800">
            <div class="font-semibold text-neutral-100 mb-1">Wayland wlroots Watcher (`wayland.rs`)</div>
            <p class="text-neutral-400">
              Direct integration for Hyprland (`.socket2.sock`) and wlroots foreign toplevel management protocol.
              Emits state changes with zero polling latency.
            </p>
          </div>

          <div class="p-3 rounded bg-neutral-950 border border-neutral-800">
            <div class="font-semibold text-neutral-100 mb-1">AFK / Idle Detection (`afk.rs`)</div>
            <p class="text-neutral-400">
              Tracks user inactivity threshold (default 300s) via XScreenSaver protocol and GNOME Mutter D-Bus `IdleMonitor`.
              Emits clean transitions into and out of AFK state with zero redundant event flooding.
            </p>
          </div>

          <div class="p-3 rounded bg-neutral-950 border border-neutral-800">
            <div class="font-semibold text-neutral-100 mb-1">In-Memory Deduplication Filter (`pipeline.rs`)</div>
            <p class="text-neutral-400">
              Suppresses identical events occurring within a 60-second window, emitting only state changes and periodic checkpoints
              directly to SQLite without burning disk I/O.
            </p>
          </div>
        </div>
      </Card>
    {/if}
  </main>

  <footer class="border-t border-neutral-800/80 bg-neutral-950 px-6 py-3 text-xs font-mono text-neutral-500 flex justify-between">
    <span>Tendly Foundation Phase 2</span>
    <span>MPL-2.0 Open Source</span>
  </footer>
</div>
