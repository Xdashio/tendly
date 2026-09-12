<script lang="ts">
  interface Props {
    variant?: 'default' | 'success' | 'warning' | 'neutral' | 'error';
    text: string;
  }

  let { variant = 'default', text }: Props = $props();

  const variantClasses = {
    default: 'bg-neutral-800 text-neutral-300 border-neutral-700',
    success: 'bg-emerald-950/60 text-emerald-300 border-emerald-800/60',
    warning: 'bg-amber-950/60 text-amber-300 border-amber-800/60',
    neutral: 'bg-neutral-900 text-neutral-400 border-neutral-800',
    error: 'bg-rose-950/60 text-rose-300 border-rose-800/60',
  };
</script>

<span
  class="inline-flex items-center px-2 py-0.5 rounded text-xs font-mono border {variantClasses[variant]}"
>
  {text}
</span>
