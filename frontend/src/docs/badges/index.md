---
title: Custom badges
section: Badges
order: 10
---

A custom badge shows any text you like, with your own colours and icon.

```http
GET /badge?title=Built%20with&label=Sass&color=cd6699&icon=simple:sass
```

![Built with Sass](https://badges.hoppou.dev/badge?title=Built%20with&label=Sass&color=cd6699&icon=simple:sass)
![Built with Sass](https://badges.hoppou.dev/badge?style=pill&title=Built%20with&label=Sass&color=cd6699&icon=simple:sass)

`/badge.svg` works too, for tools that expect an image URL to end in `.svg`.

## Parameters

Only `label` is required.

| Parameter    | Description                                                                         |
| ------------ | ----------------------------------------------------------------------------------- |
| `label`      | The main text                                                                       |
| `title`      | Smaller text before the label, omit it for a label-only badge                       |
| `style`      | `devin` (default) or `pill`, see [Styles and sizes](/docs/badges/styles)            |
| `size`       | `cozy` (default) or `compact`                                                       |
| `theme`      | `auto` (default), `dark` or `light`, see [Themes](/docs/badges/themes)              |
| `color`      | Accent colour (default `f1f1f1`), see [Colours](/docs/badges/colours)               |
| `color2`     | Second accent colour                                                                |
| `titleColor` | Devin only: title colour                                                            |
| `bg`, `bg2`  | Background colours                                                                  |
| `icon`       | `simple:<slug>`, `lucide:<name>` or an image URL, see [Icons](/docs/badges/icons)   |
| `iconColor`  | Devin only: icon colour                                                             |
| `format`     | `svg` (default), `png`, `avif` or `webp`, see [Image formats](/docs/badges/formats) |

Encode spaces and other special characters in values, such as `%20` for a space and `%23` for `#`.

## Title and label

With a `title`, Devin badges stack it above the label and pill badges lead with it. Without one, the
label stands alone.

| Badge                                                                                                   | Query                                                  |
| ------------------------------------------------------------------------------------------------------- | ------------------------------------------------------ |
| ![Made with love](https://badges.hoppou.dev/badge?title=Made%20with&label=Love&color=e11d48)            | `title=Made%20with&label=Love&color=e11d48`            |
| ![Love](https://badges.hoppou.dev/badge?label=Love&color=e11d48&icon=simple:githubsponsors)             | `label=Love&color=e11d48&icon=simple:githubsponsors`   |
| ![Made with love](https://badges.hoppou.dev/badge?style=pill&title=made%20with&label=love&color=e11d48) | `style=pill&title=made%20with&label=love&color=e11d48` |
| ![Love](https://badges.hoppou.dev/badge?style=pill&label=love&color=e11d48&icon=lucide:heart)           | `style=pill&label=love&color=e11d48&icon=lucide:heart` |

The `title` and `label` are each limited to 64 characters. Text is drawn with the badge's own fonts,
which cover Latin letters, with accents, and common punctuation. Any other character, such as an
emoji or a Cyrillic letter, returns `400`.

## Examples

![Built with TypeScript](https://badges.hoppou.dev/badge?title=Built%20with&label=TypeScript&color=3178c6&icon=simple:typescript)
![Powered by Svelte](https://badges.hoppou.dev/badge?title=Powered%20by&label=Svelte&color=ff3e00&icon=simple:svelte)
![Runs on Linux](https://badges.hoppou.dev/badge?title=Runs%20on&label=Linux&color=fcc624&icon=simple:linux)
![License MIT](https://badges.hoppou.dev/badge?title=License&label=MIT&color=22c55e)

![Styled with Tailwind](https://badges.hoppou.dev/badge?style=pill&title=styled%20with&label=tailwind&color=06b6d4&icon=simple:tailwindcss)
![Release](https://badges.hoppou.dev/badge?style=pill&title=release&label=v2.4.1&color=60a5fa&icon=lucide:tag)
![Docs](https://badges.hoppou.dev/badge?style=pill&title=read%20the&label=docs&color=c084fc&icon=lucide:book-open)
