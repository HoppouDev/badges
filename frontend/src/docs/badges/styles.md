---
title: Styles and sizes
section: Badges
order: 11
---

Every badge has a `style`, its design, and a `size`. Pill badges also have a `theme`. All three work for [custom badges](/docs/badges) and [workflow status badges](/docs/ci).

| Parameter | Values                                                                |
| --------- | --------------------------------------------------------------------- |
| `style`   | `devin` (default) or `pill`                                           |
| `size`    | `cozy` (default) or `compact`                                         |
| `theme`   | Pill only: `auto` (default), `dark` or `light`; see [Themes](#themes) |

## Devin

The default: [Devin's Badges](https://github.com/intergrav/devins-badges), a dark gradient card with a bold label.

| Cozy                                                                                                            | Compact                                                                                                                      |
| --------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| ![Built with Sass](https://badges.hoppou.dev/badge?title=Built%20with&label=Sass&color=cd6699&icon=simple:sass) | ![Built with Sass](https://badges.hoppou.dev/badge?title=Built%20with&label=Sass&color=cd6699&icon=simple:sass&size=compact) |
| ![Python](https://badges.hoppou.dev/badge?label=Python&color=3776ab&icon=simple:python)                         | ![Python](https://badges.hoppou.dev/badge?label=Python&color=3776ab&icon=simple:python&size=compact)                         |
| ![CI](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml)                                                   | ![CI](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?size=compact)                                                   |

Cozy is 56px tall with the title stacked above the label; compact is 40px with both on one line.

## Pill

A rounded chip with an outline and text in 16px JetBrains Mono, on a transparent background so it sits on whatever is behind it. The whole badge, text included, is painted with one gradient from `color` on the left to `color2` on the right; without `color2` the end is `color` with its hue turned a little. See [Pill colours](/docs/badges/colours#pill-colours).

| Cozy                                                                                                                          | Compact                                                                                                                                    |
| ----------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| ![Build passing](https://badges.hoppou.dev/badge?style=pill&title=build&label=passing&color=34d399&icon=simple:githubactions) | ![Build passing](https://badges.hoppou.dev/badge?style=pill&size=compact&title=build&label=passing&color=34d399&icon=simple:githubactions) |
| ![Release](https://badges.hoppou.dev/badge?style=pill&title=release&label=v2.4.1&color=60a5fa)                                | ![Release](https://badges.hoppou.dev/badge?style=pill&size=compact&title=release&label=v2.4.1&color=60a5fa)                                |
| ![Operational](https://badges.hoppou.dev/badge?style=pill&label=operational&color=34d399)                                     | ![Operational](https://badges.hoppou.dev/badge?style=pill&size=compact&label=operational&color=34d399)                                     |
| ![CI](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?style=pill)                                                      | ![CI](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?style=pill&size=compact)                                                      |

```http
GET /badge?style=pill&title=build&label=passing&color=34d399&icon=simple:githubactions
```

- **Cozy** is 44px tall. The icon and the title, in capitals, sit in a tinted chip on the left, and the label follows.
- **Compact** is 34px tall. A dot, or the icon if there is one, leads the title and label on one line.

## Themes

The pill background is transparent, so `theme` picks gradient tones that suit the page behind it: `dark` for dark pages and `light` for light ones. `theme=auto`, the default, carries both and follows the viewer's colour scheme, so one URL suits both GitHub themes. With a fixed `bg`, `auto` instead picks whichever suits that fill.

| Theme   | Badge                                                                                                 |
| ------- | ----------------------------------------------------------------------------------------------------- |
| `dark`  | ![Dark](https://badges.hoppou.dev/badge?style=pill&theme=dark&title=license&label=MIT&color=fbbf24)   |
| `light` | ![Light](https://badges.hoppou.dev/badge?style=pill&theme=light&title=license&label=MIT&color=fbbf24) |
| `auto`  | ![Auto](https://badges.hoppou.dev/badge?style=pill&title=license&label=MIT&color=fbbf24)              |

Colours are lightened on the dark theme and darkened on the light one, as far as needed to stay readable, so any `color` works on both.

`auto` follows the viewer's browser or system preference, which isn't always the theme GitHub itself is set to. [Image formats](/docs/badges/formats) other than SVG can't switch, so `auto` renders them dark. To pin each GitHub theme to its own badge, use a `<picture>`:

```html
<picture>
	<source
		media="(prefers-color-scheme: light)"
		srcset="https://badges.hoppou.dev/badge?style=pill&theme=light&label=MIT"
	/>
	<img alt="License MIT" src="https://badges.hoppou.dev/badge?style=pill&theme=dark&label=MIT" />
</picture>
```

## Older URLs

Before styles existed, `style` chose the size, so `style=cozy` and `style=compact` still mean the Devin style at that size. An explicit `size` wins over them.

Names are case-insensitive, and an unknown `style`, `size` or `theme` returns `400`.
