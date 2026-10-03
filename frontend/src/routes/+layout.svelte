<script lang="ts">
	import './layout.css';
	import favicon from '#lib/assets/favicon.svg';
	import ModeToggle from '#lib/components/mode-toggle.svelte';
	import { ModeWatcher } from 'mode-watcher';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<ModeWatcher />

<header class="flex justify-end px-4 pt-4">
	<ModeToggle />
</header>

{@render children()}
