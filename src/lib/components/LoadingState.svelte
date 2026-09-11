<script lang="ts">
  interface Props {
    message?: string;
  }

  let { message = 'Loading application state...' }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center p-12 text-center">
  <div class="h-6 w-6 animate-spin rounded-full border-2 border-neutral-700 border-t-neutral-200"></div>
  <p class="mt-4 text-xs font-mono text-neutral-400">{message}</p>
</div>
