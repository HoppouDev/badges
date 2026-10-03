<script lang="ts">
	let { data } = $props();
</script>

<svelte:head>
	<title>{data.title}</title>
</svelte:head>

<!-- Generated at build time from our own markdown, which may not contain raw HTML.
     dark: follows the .dark class mode-watcher sets before first paint -->
<article class="mx-auto tw-prose px-4 py-8 dark:tw-prose-invert">
	<!-- eslint-disable-next-line svelte/no-at-html-tags -->
	{@html data.html}
</article>
