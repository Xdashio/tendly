<script lang="ts">
  import Badge from './Badge.svelte';

  interface Props {
    appName: string;
    version: string;
    environment: string;
    isPaused: boolean;
    onTogglePause: () => void;
  }

  let { appName, version, environment, isPaused, onTogglePause }: Props = $props();
</script>

<header class="border-b border-neutral-800 bg-neutral-900/80 px-6 py-4 backdrop-blur">
  <div class="flex items-center justify-between">
    <div class="flex items-center space-x-3">
      <div class="h-7 w-7 rounded bg-neutral-100 text-neutral-950 font-mono font-bold flex items-center justify-center text-sm">
        T
      </div>
      <div>
        <div class="flex items-center space-x-2">
          <h1 class="text-base font-semibold font-mono tracking-tight text-neutral-100">{appName}</h1>
          <span class="text-xs font-mono text-neutral-500">v{version}</span>
          <Badge variant="neutral" text={environment} />
        </div>
        <p class="text-xs text-neutral-400">Privacy-First Time Awareness</p>
      </div>
    </div>

    <div class="flex items-center space-x-3">
      <Badge
        variant={isPaused ? 'warning' : 'success'}
        text={isPaused ? 'Tracking Paused' : 'Tracking Active'}
      />
      <button
        onclick={onTogglePause}
        class="rounded border border-neutral-700 bg-neutral-800 px-3 py-1.5 text-xs font-mono text-neutral-200 hover:bg-neutral-700 transition"
      >
        {isPaused ? 'Resume Tracking' : 'Pause Tracking'}
      </button>
    </div>
  </div>
</header>
