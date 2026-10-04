<script lang="ts">
	import './layout.css';
	import favicon from '#lib/assets/favicon.svg';
	import { ModeWatcher } from 'mode-watcher';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<!-- mocha: Catppuccin's dark flavour, set before first paint along with dark -->
<ModeWatcher darkClassNames={['dark', 'mocha']} />
{@render children()}
