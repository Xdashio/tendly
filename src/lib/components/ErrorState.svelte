<script lang="ts">
  interface Props {
    title?: string;
    message: string;
    onRetry?: () => void;
  }

  let { title = 'Encountered Error', message, onRetry }: Props = $props();
</script>

<div class="rounded-lg border border-red-900/60 bg-red-950/20 p-5 text-neutral-200">
  <div class="flex items-start justify-between">
    <div>
      <h4 class="text-sm font-medium font-mono text-red-400">{title}</h4>
      <p class="mt-1 text-xs text-neutral-400">{message}</p>
    </div>
    {#if onRetry}
      <button
        onclick={onRetry}
        class="rounded border border-red-800/80 bg-red-900/40 px-3 py-1 text-xs font-mono text-red-200 hover:bg-red-800/60 transition"
      >
        Retry
      </button>
    {/if}
  </div>
</div>
