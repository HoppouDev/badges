---
title: Image formats
section: Badges
order: 14
---

Badges are SVG by default. Add `format` to get a raster image instead, for places that don't accept SVG.

```http
GET /badge?title=Built%20with&label=Rust&color=f74c00&icon=simple:rust&format=png
```

| Format | Content type    | Badge                                                                                                            | Notes                                                         |
| ------ | --------------- | ---------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| `svg`  | `image/svg+xml` | ![SVG](https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=f74c00&icon=simple:rust&format=svg)   | Vector, sharp at any size                                     |
| `png`  | `image/png`     | ![PNG](https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=f74c00&icon=simple:rust&format=png)   | Lossless, transparent corners                                 |
| `webp` | `image/webp`    | ![WebP](https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=f74c00&icon=simple:rust&format=webp) | Lossless, transparent corners                                 |
| `avif` | `image/avif`    | ![AVIF](https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=f74c00&icon=simple:rust&format=avif) | Lossy at high quality, transparent corners; slowest to encode |

Raster formats are drawn at the badge's natural height (Devin: 56px cozy, 40px compact; pill: 44px cozy, 34px compact), so they can look soft on high-DPI screens. Prefer SVG where it works. A raster image can't follow the viewer's colour scheme, so pill badges with `theme=auto` use the dark theme.

`format` also works on [workflow status badges](/docs/ci). Format names are case-insensitive, and anything else returns `400`.
