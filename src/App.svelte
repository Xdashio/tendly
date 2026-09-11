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
    getDatabaseStats,
    toggleTrackingPause,
    isTauri,
    type AppInfo,
    type AppStatus,
    type DatabaseStats,
  } from './lib/api';

  let appInfo = $state<AppInfo | null>(null);
  let appStatus = $state<AppStatus | null>(null);
  let dbStats = $state<DatabaseStats | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let activeTab = $state<'overview' | 'storage' | 'architecture'>('overview');

  async function loadData() {
    loading = true;
    error = null;
    try {
      const [info, status, db] = await Promise.all([
        getAppInfo(),
        getAppStatus(),
        getDatabaseStats(),
      ]);
      appInfo = info;
      appStatus = status;
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
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }

  onMount(() => {
    loadData();
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
        Foundation Architecture
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
              <dd class="text-neutral-200">{appStatus?.active_watchers_count} (Phase 1 scaffolding)</dd>
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
              <dt class="text-neutral-400">Recorded Blocks</dt>
              <dd class="text-neutral-200">{dbStats?.blocks_count ?? 0}</dd>
            </div>
          </dl>
        </Card>
      </div>

      <Card title="Activity Timeline" subtitle="Planned for Phase 2 implementation">
        <EmptyState
          title="No Activity Events Recorded"
          description="Phase 1 establishes the production architecture, IPC bridge, and database foundation. Activity capture watchers (X11, wlroots Wayland, AFK) will be connected in Phase 2."
        />
      </Card>
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
      <Card title="System Architecture Boundaries" subtitle="Validated in Phase 0, scaffolded in Phase 1">
        <div class="space-y-4 text-xs font-mono text-neutral-300 leading-relaxed">
          <div class="p-3 rounded bg-neutral-950 border border-neutral-800">
            <div class="font-semibold text-neutral-100 mb-1">Architecture B (Direct In-Process Model)</div>
            <p class="text-neutral-400">
              Svelte 5 UI communicates with the Rust Application Core strictly through Tauri IPC (`invoke` commands and events).
              No internal HTTP port is opened on 127.0.0.1, eliminating the local port-scanning attack surface.
            </p>
          </div>

          <div class="p-3 rounded bg-neutral-950 border border-neutral-800">
            <div class="font-semibold text-neutral-100 mb-1">In-Process Watcher Pipeline (Phase 2 Target)</div>
            <p class="text-neutral-400">
              OS-level watchers for Linux X11 (`x11rb`) and wlroots Wayland (`wayland-client`) will run as in-process Rust asynchronous
              tasks on Tokio, feeding `RawEvent` structures directly through in-memory channels (`mpsc`) into SQLite.
            </p>
          </div>

          <div class="p-3 rounded bg-neutral-950 border border-neutral-800">
            <div class="font-semibold text-neutral-100 mb-1">3-Tier Classification Cascade (Phase 2 / Phase 3)</div>
            <p class="text-neutral-400">
              Tier 1: Deterministic Rules & Cache (0ms, 0 RAM) ->
              Tier 2: Local AI via Ollama Qwen 2.5:3b (Optional) ->
              Tier 3: User BYOK Cloud AI (Optional).
            </p>
          </div>
        </div>
      </Card>
    {/if}
  </main>

  <footer class="border-t border-neutral-800/80 bg-neutral-950 px-6 py-3 text-xs font-mono text-neutral-500 flex justify-between">
    <span>Tendly Foundation Phase 1</span>
    <span>MPL-2.0 Open Source</span>
  </footer>
</div>
