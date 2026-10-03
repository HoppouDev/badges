<script lang="ts">
	import { cn, type WithElementRef } from '#lib/utils.js';
	import type { HTMLAttributes } from 'svelte/elements';

	let {
		ref = $bindable(null),
		class: className,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> & {} = $props();
</script>

<div
	bind:this={ref}
	data-slot="select-label"
	class={cn(
		'px-3 py-2 text-xs font-semibold tracking-wider text-muted-foreground uppercase',
		className
	)}
	{...restProps}
>
	{@render children?.()}
</div>
