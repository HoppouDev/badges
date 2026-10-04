---
title: Custom badges
section: Badges
order: 10
---

```http
GET /badge?title=Built%20with&label=Sass&color=cd6699&icon=simple:sass
```

![Built with Sass](https://badges.hoppou.dev/badge?title=Built%20with&label=Sass&color=cd6699&icon=simple:sass)

`/badge.svg` is an alias for `/badge`, for tools that expect an image URL to end in `.svg`.

## Parameters

| Parameter    | Description                                                                                                                                                      |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `label`      | Bold bottom line (required)                                                                                                                                      |
| `title`      | Small top line; omit for a single-line badge                                                                                                                     |
| `color`      | Accent colour for the label and icon, or the pill gradient's start (default `f1f1f1`); see [Colours](/docs/badges/colours)                                       |
| `color2`     | Second accent: the label's vertical gradient (Devin) or the gradient's end (pill)                                                                                |
| `titleColor` | Devin title colour (default `e8e8e8`)                                                                                                                            |
| `bg`         | Background; for Devin, the gradient top, and alone a flat background (default from `color`)                                                                      |
| `bg2`        | Devin background gradient bottom (default derived from `color`)                                                                                                  |
| `icon`       | `simple:<slug>` ([Simple Icons](https://simpleicons.org)), `lucide:<name>` ([Lucide](https://lucide.dev/icons)) or an image URL; see [Icons](/docs/badges/icons) |
| `iconColor`  | Devin colour for `simple:` and `lucide:` icons (default `color`)                                                                                                 |
| `style`      | `devin` (default) or `pill`; see [Styles and sizes](/docs/badges/styles)                                                                                         |
| `size`       | `cozy` (default) or `compact`                                                                                                                                    |
| `theme`      | Pill colours: `auto` (default), `dark` or `light`; see [Themes](/docs/badges/styles#themes)                                                                      |
| `format`     | `svg` (default), `png`, `avif` or `webp`; see [Image formats](/docs/badges/formats)                                                                              |

The `title` and `label` are each limited to 64 characters.

## Title and label

With a `title`, the badge stacks it above the label. Without one, the label is the only line.

| Badge                                                                                        | Query                                                |
| -------------------------------------------------------------------------------------------- | ---------------------------------------------------- |
| ![Made with love](https://badges.hoppou.dev/badge?title=Made%20with&label=Love&color=e11d48) | `title=Made%20with&label=Love&color=e11d48`          |
| ![Love](https://badges.hoppou.dev/badge?label=Love&color=e11d48)                             | `label=Love&color=e11d48`                            |
| ![Love](https://badges.hoppou.dev/badge?label=Love&color=e11d48&icon=simple:githubsponsors)  | `label=Love&color=e11d48&icon=simple:githubsponsors` |

## Examples

![Built with TypeScript](https://badges.hoppou.dev/badge?title=Built%20with&label=TypeScript&color=3178c6&icon=simple:typescript)
![Powered by Svelte](https://badges.hoppou.dev/badge?title=Powered%20by&label=Svelte&color=ff3e00&icon=simple:svelte)
![Styled with Tailwind](https://badges.hoppou.dev/badge?title=Styled%20with&label=Tailwind&color=06b6d4&icon=simple:tailwindcss)
![Runs on Linux](https://badges.hoppou.dev/badge?title=Runs%20on&label=Linux&color=fcc624&icon=simple:linux)
![License MIT](https://badges.hoppou.dev/badge?title=License&label=MIT&color=22c55e)
