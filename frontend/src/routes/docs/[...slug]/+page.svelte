<script lang="ts">
	let { data } = $props();
</script>

<svelte:head>
	<title>{data.title}</title>
</svelte:head>

<!-- Generated at build time from our own markdown, which may not contain raw HTML.
     tw-prose colours are Catppuccin's, which switch with the theme, so there's no invert.
     w-full: with auto margins in a flex column the article would otherwise size to its
     widest code line instead of the column, pushing the page sideways.
     wrap-break-word: long URLs and host names break instead of running off small screens;
     td wrap-anywhere: table cells need it too, as tables otherwise size to their longest word;
     td img/figure my-0 and align-middle: badges in tables sit tight and line up with their text;
     p img inline: badges written on consecutive lines sit in a row, as on GitHub, instead of
     stacking as blocks (Tailwind's default for images) -->
<article
	class="mx-auto tw-prose w-full px-4 py-8 wrap-break-word [&_p_img]:my-1 [&_p_img]:inline [&_p_img]:align-middle [&_td]:wrap-anywhere [&_td_figure]:my-0 [&_td_img]:my-0 [&_td,&_th]:align-middle"
>
	<h1>{data.title}</h1>
	<!-- eslint-disable-next-line svelte/no-at-html-tags -->
	{@html data.html}
</article>
