<script lang="ts">
	import { type WithElementRef } from 'bits-ui';
	import { cn } from '#lib/utils.js';
	import type { HTMLAttributes } from 'svelte/elements';

	let {
		ref = $bindable(null),
		inset,
		children,
		class: className,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLElement>> & {
		inset?: boolean;
	} = $props();
</script>

<div
	bind:this={ref}
	data-slot="menubar-label"
	data-inset={inset}
	class={cn(
		'px-3.5 py-2 text-xs font-semibold tracking-wider text-muted-foreground uppercase data-inset:pl-9.5',
		className
	)}
	{...restProps}
>
	{@render children?.()}
</div>
