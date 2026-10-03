<script lang="ts">
	import DocsSidebar from '#lib/components/docs-sidebar.svelte';
	import ModeToggle from '#lib/components/mode-toggle.svelte';
	import { ButtonGroup } from '#lib/components/ui/button-group/index.js';
	import * as Sidebar from '#lib/components/ui/sidebar/index.js';

	let { data, children } = $props();
</script>

<Sidebar.Provider>
	<DocsSidebar nav={data.nav} />
	<!-- min-w-0: a flex item otherwise can't shrink below its widest line (long code).
	     px-11: a gutter one button wide (icon-lg, 44px) on each side, so the floating
	     buttons never cover text and need no outline or background -->
	<Sidebar.Inset class="min-w-0 px-11">
		{@render children()}
	</Sidebar.Inset>
	<!-- Sits in the left gutter; follows the inset's left edge as the sidebar opens.
	     The sidebar root is the preceding peer, and only exists as one from md up -->
	<ButtonGroup
		orientation="vertical"
		class="fixed top-0 left-0 z-40 transition-[left] duration-200 ease-linear md:peer-data-[state=expanded]:left-(--sidebar-width)"
	>
		<!-- Bare icons floating over the page; a hover fill would read as a stray box -->
		<Sidebar.Trigger size="icon-lg" class="hover:bg-transparent dark:hover:bg-transparent" />
		<ModeToggle
			variant="ghost"
			size="icon-lg"
			class="hover:bg-transparent dark:hover:bg-transparent"
		/>
	</ButtonGroup>
</Sidebar.Provider>
