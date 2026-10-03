<script lang="ts">
	import { afterNavigate } from '$app/navigation';
	import { page } from '$app/state';
	import * as Sidebar from '#lib/components/ui/sidebar/index.js';
	import type { NavSection } from '#lib/server/docs.ts';

	let { nav }: { nav: NavSection[] } = $props();

	// Needs the provider's context, so this can't live in the layout that creates it
	const sidebar = Sidebar.useSidebar();

	// The mobile sheet overlays the page; close it once a link has navigated
	afterNavigate(() => sidebar.setOpenMobile(false));
</script>

<!-- The right border otherwise takes the page's --border, not the sidebar's -->
<Sidebar.Root class="border-sidebar-border">
	<Sidebar.Content>
		{#each nav as section (section.title)}
			<Sidebar.Group>
				<Sidebar.GroupLabel>{section.title}</Sidebar.GroupLabel>
				<Sidebar.GroupContent>
					<Sidebar.Menu>
						{#each section.pages as link (link.href)}
							<Sidebar.MenuItem>
								<Sidebar.MenuButton isActive={page.url.pathname === link.href}>
									{#snippet child({ props })}
										<a href={link.href} {...props}>{link.title}</a>
									{/snippet}
								</Sidebar.MenuButton>
							</Sidebar.MenuItem>
						{/each}
					</Sidebar.Menu>
				</Sidebar.GroupContent>
			</Sidebar.Group>
		{/each}
	</Sidebar.Content>
</Sidebar.Root>
