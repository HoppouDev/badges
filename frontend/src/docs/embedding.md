---
title: Embedding
section: Getting started
order: 1
---

A badge is an image URL. Encode spaces and other special characters in the query string (`%20` for a space).

## Markdown

```markdown
![Built with Svelte](https://badges.hoppou.dev/badge?title=Built%20with&label=Svelte&color=ff3e00&icon=svelte)
```

![Built with Svelte](https://badges.hoppou.dev/badge?title=Built%20with&label=Svelte&color=ff3e00&icon=svelte)

Wrap it in a link to make it clickable:

```markdown
[![Built with Svelte](https://badges.hoppou.dev/badge?title=Built%20with&label=Svelte&color=ff3e00&icon=svelte)](https://svelte.dev)
```

## HTML

```html
<a href="https://svelte.dev">
	<img
		alt="Built with Svelte"
		src="https://badges.hoppou.dev/badge?title=Built%20with&label=Svelte&color=ff3e00&icon=svelte"
	/>
</a>
```

GitHub READMEs render HTML too, which is handy for centring a row of badges:

```html
<div align="center">
	<img
		alt="Built with Rust"
		src="https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=f74c00&icon=rust"
	/>
	<img alt="CI status" src="https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml" />
</div>
```

## Alt text

Badges are images, so always give them alt text. SVG badges also carry their text as an accessible label, but PNG and other [raster formats](/docs/badges/formats) do not.
