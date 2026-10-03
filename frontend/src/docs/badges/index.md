---
title: Custom badges
section: Badges
order: 10
---

```http
GET /badge?title=Built%20with&label=Sass&color=cd6699&icon=sass
```

![Built with Sass](https://badges.hoppou.dev/badge?title=Built%20with&label=Sass&color=cd6699&icon=sass)

`/badge.svg` is an alias for `/badge`, for tools that expect an image URL to end in `.svg`.

## Parameters

| Parameter    | Description                                                                                   |
| ------------ | --------------------------------------------------------------------------------------------- |
| `label`      | Bold bottom line (required)                                                                   |
| `title`      | Small top line; omit for a single-line badge                                                  |
| `color`      | Accent colour for the label and icon (default `f1f1f1`)                                       |
| `color2`     | Second label colour, making a vertical gradient                                               |
| `titleColor` | Title colour (default `e8e8e8`)                                                               |
| `bg`         | Background gradient top; alone it gives a flat background (default derived from `color`)      |
| `bg2`        | Background gradient bottom (default derived from `color`)                                     |
| `icon`       | [Simple Icons](https://simpleicons.org) slug or an image URL; see [Icons](/docs/badges/icons) |
| `iconColor`  | Simple Icons fill (default `color`)                                                           |
| `style`      | `cozy` (default) or `compact`; see [Styles](/docs/badges/styles)                              |
| `format`     | `svg` (default), `png`, `avif`, `webp` or `jpeg`; see [Image formats](/docs/badges/formats)   |

The `title` and `label` are each limited to 64 characters.

## Title and label

With a `title`, the badge stacks it above the label. Without one, the label is the only line.

| Badge                                                                                        | Query                                         |
| -------------------------------------------------------------------------------------------- | --------------------------------------------- |
| ![Made with love](https://badges.hoppou.dev/badge?title=Made%20with&label=Love&color=e11d48) | `title=Made%20with&label=Love&color=e11d48`   |
| ![Love](https://badges.hoppou.dev/badge?label=Love&color=e11d48)                             | `label=Love&color=e11d48`                     |
| ![Love](https://badges.hoppou.dev/badge?label=Love&color=e11d48&icon=githubsponsors)         | `label=Love&color=e11d48&icon=githubsponsors` |

## Examples

![Built with TypeScript](https://badges.hoppou.dev/badge?title=Built%20with&label=TypeScript&color=3178c6&icon=typescript)
![Powered by Svelte](https://badges.hoppou.dev/badge?title=Powered%20by&label=Svelte&color=ff3e00&icon=svelte)
![Styled with Tailwind](https://badges.hoppou.dev/badge?title=Styled%20with&label=Tailwind&color=06b6d4&icon=tailwindcss)
![Runs on Linux](https://badges.hoppou.dev/badge?title=Runs%20on&label=Linux&color=fcc624&icon=linux)
![License MIT](https://badges.hoppou.dev/badge?title=License&label=MIT&color=22c55e)
