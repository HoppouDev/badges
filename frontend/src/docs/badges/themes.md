---
title: Themes
section: Badges
order: 12
---

[Pill badges](/docs/badges/styles#pill) have a transparent background, so they take on the page behind them. `theme` picks colours that suit it. Devin badges carry their own dark background and ignore `theme`.

| Theme            | Badge                                                                                                 | For                                      |
| ---------------- | ----------------------------------------------------------------------------------------------------- | ---------------------------------------- |
| `auto` (default) | ![Auto](https://badges.hoppou.dev/badge?style=pill&title=license&label=MIT&color=fbbf24)              | Pages that can be either, such as GitHub |
| `dark`           | ![Dark](https://badges.hoppou.dev/badge?style=pill&theme=dark&title=license&label=MIT&color=fbbf24)   | Dark pages                               |
| `light`          | ![Light](https://badges.hoppou.dev/badge?style=pill&theme=light&title=license&label=MIT&color=fbbf24) | Light pages                              |

Each theme tones `color` and `color2`: lighter on dark pages and darker on light ones, only as far as needed to stay readable. So any colour works on both.

## auto

An `auto` SVG carries both sets of colours and switches with the viewer's light or dark mode, so one URL suits both GitHub themes. Switch this site between light and dark with the sun and moon button to see it.

`auto` follows the viewer's browser or system setting, which isn't always the theme GitHub itself is set to. To show a fixed badge for each GitHub theme instead, use a `<picture>`:

```html
<picture>
	<source
		media="(prefers-color-scheme: light)"
		srcset="https://badges.hoppou.dev/badge?style=pill&theme=light&label=MIT"
	/>
	<img alt="License MIT" src="https://badges.hoppou.dev/badge?style=pill&theme=dark&label=MIT" />
</picture>
```

[Image formats](/docs/badges/formats) other than SVG can't switch, so they render `auto` as `dark`.

## With a background

A pill with its own `bg` no longer sits on the page, so `auto` picks whichever theme suits that fill instead of switching, in every format.

![Dark fill](https://badges.hoppou.dev/badge?style=pill&title=fill&label=dark&color=fbbf24&bg=1e1e2e)
![Light fill](https://badges.hoppou.dev/badge?style=pill&title=fill&label=light&color=fbbf24&bg=fef3c7)

Theme names are case-insensitive, and an unknown one returns `400`.
