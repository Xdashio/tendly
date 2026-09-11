<script lang="ts">
  interface Props {
    title: string;
    description: string;
  }

  let { title, description }: Props = $props();
</script>

<div class="rounded-lg border border-dashed border-neutral-800 p-8 text-center">
  <p class="text-sm font-medium font-mono text-neutral-300">{title}</p>
  <p class="mt-1 text-xs text-neutral-500 max-w-sm mx-auto">{description}</p>
</div>
