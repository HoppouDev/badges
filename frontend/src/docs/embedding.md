---
title: Embedding
section: Getting started
order: 1
---

A badge is an image URL, so it goes anywhere an image does. Encode spaces and other special
characters in the query string, such as `%20` for a space.

## Markdown

```markdown
![Built with Svelte](https://badges.hoppou.dev/badge?style=pill&title=Built%20with&label=Svelte&color=ff3e00&icon=simple:svelte)
```

![Built with Svelte](https://badges.hoppou.dev/badge?style=pill&title=Built%20with&label=Svelte&color=ff3e00&icon=simple:svelte)

Wrap it in a link to make it clickable:

```markdown
[![Built with Svelte](https://badges.hoppou.dev/badge?style=pill&title=Built%20with&label=Svelte&color=ff3e00&icon=simple:svelte)](https://svelte.dev)
```

## HTML

```html
<a href="https://svelte.dev">
	<img
		alt="Built with Svelte"
		src="https://badges.hoppou.dev/badge?style=pill&title=Built%20with&label=Svelte&color=ff3e00&icon=simple:svelte"
	/>
</a>
```

GitHub READMEs render HTML too, which is handy for centring a row of badges:

```html
<div align="center">
	<img
		alt="Built with Rust"
		src="https://badges.hoppou.dev/badge?style=pill&title=Built%20with&label=Rust&color=f74c00&icon=simple:rust"
	/>
	<img alt="CI status" src="https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?style=pill" />
</div>
```

## On GitHub

- Pill badges with the default `theme=auto` follow the viewer's light or dark mode. To match
  GitHub's own theme setting exactly, see [Themes](/docs/badges/themes#auto).
- GitHub serves README images through its own image proxy, which caches them. A badge can take a
  while to update after you change it; changing the URL shows the new one at once.

## Alt text

Badges are images, so always give them alt text. SVG badges also carry their text as an accessible
label, but PNG and other [raster formats](/docs/badges/formats) don't.
