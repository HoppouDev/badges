---
title: Image formats
section: Badges
order: 15
---

Badges are SVG by default. Add `format` to get a raster image instead, for places that don't accept SVG.

```http
GET /badge?title=Built%20with&label=Rust&color=f74c00&icon=simple:rust&format=png
```

| Format | Badge                                                                                                            | Notes                                                     |
| ------ | ---------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------- |
| `svg`  | ![SVG](https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=f74c00&icon=simple:rust&format=svg)   | Vector, sharp at any size; the best choice where it works |
| `png`  | ![PNG](https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=f74c00&icon=simple:rust&format=png)   | Lossless, works everywhere                                |
| `webp` | ![WebP](https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=f74c00&icon=simple:rust&format=webp) | Lossless and smaller than PNG                             |
| `avif` | ![AVIF](https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=f74c00&icon=simple:rust&format=avif) | Smallest, lossy at high quality; slowest to make          |

All raster formats keep transparent corners and backgrounds.

## What SVG does that raster can't

Raster images are drawn once, at the badge's natural size, so they lose three things:

- **Sharpness:** they're as tall as the badge (Devin 56px cozy or 40px compact, pill 44px or 34px), so they can look soft on high-DPI screens.
- **Themes:** they can't follow the viewer's light or dark mode, so pill badges with `theme=auto` render as `dark`; see [Themes](/docs/badges/themes).
- **Animation:** the turning icon on a running [workflow status badge](/docs/ci#states) stands still.

`format` works on workflow status badges too. Format names are case-insensitive, and anything else returns `400`.
