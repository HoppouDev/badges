---
title: Overview
section: Getting started
order: 0
---

Badges for READMEs and websites, served from `https://badges.hoppou.dev`. Every badge is a plain
URL, so it works anywhere an image does, issue templates included.

![Built with Rust](https://badges.hoppou.dev/badge?style=pill&title=Built%20with&label=Rust&color=f74c00&icon=simple:rust)
![Deployed on Cloudflare](https://badges.hoppou.dev/badge?style=pill&title=Deployed%20on&label=Cloudflare&color=f38020&icon=simple:cloudflare)
![CI status](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?style=pill)

```markdown
![Built with Rust](https://badges.hoppou.dev/badge?style=pill&title=Built%20with&label=Rust&color=f74c00&icon=simple:rust)
```

## Two kinds of badge

- [Custom badges](/docs/badges) at `/badge` show any text you like, with your own colours and icon.
- [Workflow status badges](/docs/ci) at `/ci/{owner}/{repo}/{workflow}` show whether a GitHub
  Actions workflow is passing.

## Two styles

Pick one with `style`, and a size with `size=cozy` (the default) or `size=compact`. See
[Styles and sizes](/docs/badges/styles).

| Style   | Badge                                                                                                           |
| ------- | --------------------------------------------------------------------------------------------------------------- |
| `devin` | ![Devin](https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=f74c00&icon=simple:rust)           |
| `pill`  | ![Pill](https://badges.hoppou.dev/badge?style=pill&title=Built%20with&label=Rust&color=f74c00&icon=simple:rust) |

- **`devin`**, the default, is a gradient card in the style of
  [Devin's Badges](https://github.com/intergrav/devins-badges).
- **`pill`** is a rounded chip painted with one gradient, on a transparent background.

Both follow the viewer's light or dark mode, so one URL suits both GitHub themes. See
[Themes](/docs/badges/themes).

Icons come from [Simple Icons](https://simpleicons.org) with `icon=simple:<slug>` or
[Lucide](https://lucide.dev/icons) with `icon=lucide:<name>`, or from your own image URL. Badges are
SVG by default, or PNG, AVIF or WebP with `format`.

## Where to next

- [Embedding](/docs/embedding) shows the Markdown and HTML to put a badge on a page.
- [Colours](/docs/badges/colours), [Icons](/docs/badges/icons) and [Themes](/docs/badges/themes)
  cover the look of a badge.
- [Image formats](/docs/badges/formats) lists the raster formats and when to use them.
- [Errors and limits](/docs/reference/limits) lists what the API rejects and how long badges are
  cached.
