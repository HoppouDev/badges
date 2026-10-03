<script lang="ts">
	import { cn } from '#lib/utils.js';
	import { mode } from 'mode-watcher';

	let { data } = $props();
</script>

<svelte:head>
	<title>{data.title}</title>
</svelte:head>

<!-- Generated at build time from our own markdown, which may not contain raw HTML -->
<article class={cn('mx-auto tw-prose px-4 py-8', mode.current === 'dark' && 'tw-prose-invert')}>
	<!-- eslint-disable-next-line svelte/no-at-html-tags -->
	{@html data.html}
</article>
