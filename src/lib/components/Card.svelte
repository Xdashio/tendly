<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title?: string;
    subtitle?: string;
    children: Snippet;
  }

  let { title, subtitle, children }: Props = $props();
</script>

<div class="rounded-lg border border-neutral-800 bg-neutral-900/60 p-5 shadow-sm">
  {#if title}
    <div class="mb-4">
      <h3 class="text-sm font-medium tracking-wide uppercase text-neutral-400 font-mono">
        {title}
      </h3>
      {#if subtitle}
        <p class="mt-1 text-xs text-neutral-400">{subtitle}</p>
      {/if}
    </div>
  {/if}
  <div>
    {@render children()}
  </div>
</div>
