---
title: Themes
section: Badges
order: 12
---

`theme` picks colours that suit the page a badge sits on, in both [styles](/docs/badges/styles). By
default, `auto`, a badge follows the viewer's light or dark mode.

| Theme            | Devin                                                                                                        | Pill                                                                                                  | For                                      |
| ---------------- | ------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------- | ---------------------------------------- |
| `auto` (default) | ![Auto](https://badges.hoppou.dev/badge?title=License&label=MIT&color=fbbf24&icon=lucide:scale)              | ![Auto](https://badges.hoppou.dev/badge?style=pill&title=license&label=MIT&color=fbbf24)              | Pages that can be either, such as GitHub |
| `dark`           | ![Dark](https://badges.hoppou.dev/badge?theme=dark&title=License&label=MIT&color=fbbf24&icon=lucide:scale)   | ![Dark](https://badges.hoppou.dev/badge?style=pill&theme=dark&title=license&label=MIT&color=fbbf24)   | Dark pages                               |
| `light`          | ![Light](https://badges.hoppou.dev/badge?theme=light&title=License&label=MIT&color=fbbf24&icon=lucide:scale) | ![Light](https://badges.hoppou.dev/badge?style=pill&theme=light&title=license&label=MIT&color=fbbf24) | Light pages                              |

- **Devin** `dark` is the original [Devin's Badges](https://github.com/intergrav/devins-badges)
  design. `light` is a pale variant: a near-white background tinted with `color`, a faint dark
  border and a softer shadow.
- **Pill** badges have a transparent background, so the theme only changes the colours drawn over
  the page.

In the light theme, `color`, `color2`, `titleColor` and `iconColor` are darkened as far as needed to
stay readable on a light background; the pill's dark theme lightens them likewise. So any colour
works in both.

## auto

An `auto` SVG carries both sets of colours and switches with the viewer's light or dark mode, so one
URL suits both GitHub themes. Switch this site between light and dark with the sun and moon button
to see it.

`auto` follows the viewer's browser or system setting, which isn't always the theme GitHub itself is
set to. To show a fixed badge for each GitHub theme instead, use a `<picture>`:

```html
<picture>
	<source
		media="(prefers-color-scheme: light)"
		srcset="https://badges.hoppou.dev/badge?theme=light&title=License&label=MIT"
	/>
	<img alt="License MIT" src="https://badges.hoppou.dev/badge?theme=dark&title=License&label=MIT" />
</picture>
```

[Image formats](/docs/badges/formats) other than SVG can't switch, so they render `auto` as `dark`.
For a Devin badge that's the original design.

## With a background

A badge with its own `bg` brings its own backdrop, so `auto` picks whichever theme suits that fill
instead of switching, in every format.

![Dark fill](https://badges.hoppou.dev/badge?style=pill&title=fill&label=dark&color=fbbf24&bg=1e1e2e)
![Light fill](https://badges.hoppou.dev/badge?style=pill&title=fill&label=light&color=fbbf24&bg=fef3c7)

Theme names are case-insensitive, and an unknown one returns `400`.
