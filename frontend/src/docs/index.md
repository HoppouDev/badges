# Badges

Cozy badges in the style of [Devin's Badges](https://github.com/intergrav/devins-badges), served from `https://badges.hoppou.dev` as SVG by default or rasterised with `format`.

## Custom badge

```http
GET /badge?title=Built%20with&label=Sass&color=cd6699&icon=sass
```

| Parameter    | Description                                                                                                                             |
| ------------ | --------------------------------------------------------------------------------------------------------------------------------------- |
| `label`      | Bold bottom line (required)                                                                                                             |
| `title`      | Small top line; omit for a single-line badge                                                                                            |
| `color`      | Accent colour for the label and icon (default `f1f1f1`)                                                                                 |
| `color2`     | Second label colour, making a vertical gradient                                                                                         |
| `titleColor` | Title colour (default `e8e8e8`)                                                                                                         |
| `bg`         | Background gradient top; alone it gives a flat background (default derived from `color`)                                                |
| `bg2`        | Background gradient bottom (default derived from `color`)                                                                               |
| `icon`       | [Simple Icons](https://simpleicons.org) slug, or an https PNG/JPEG/GIF/WebP URL on an allowed host (at most 64 KiB and 2048px per side) |
| `iconColor`  | Simple Icons fill (default `color`)                                                                                                     |
| `style`      | `cozy` (default, 56px, title above label) or `compact` (40px, one line)                                                                 |
| `format`     | `svg` (default), `png`, `avif`, `webp` or `jpeg`; see [Image formats](#image-formats)                                                   |

Colours accept hex with or without `#`, `rgb()`, `hsl()` or CSS names.

## GitHub Actions status

```http
GET /ci/{owner}/{repo}/{workflow}?branch=main&title=CI
```

`{workflow}` is a workflow file name such as `rust.yml` or a numeric workflow id. The badge is coloured by status and shows Passing, Failing, Running, Cancelled, Skipped or Unknown. Only public repositories are served.

| Parameter | Description                                                 |
| --------- | ----------------------------------------------------------- |
| `title`   | Top line (default `CI`; empty for a single-line badge)      |
| `branch`  | Branch to report (default: the repository's default branch) |
| `event`   | Triggering event to report (default `push`)                 |
| `style`   | `cozy` (default) or `compact`                               |
| `format`  | Same as for `/badge`                                        |

## Image formats

```http
GET /badge?title=Built%20with&label=Rust&color=f74c00&icon=rust&format=png
```

SVG is the default and stays sharp at any size. The other formats are rasterised from the same SVG at its natural size (56px tall for `cozy`, 40px for `compact`), so they can look soft on high-DPI screens.

| Format | Content type    | Notes                                                         |
| ------ | --------------- | ------------------------------------------------------------- |
| `svg`  | `image/svg+xml` | Vector, sharp at any size                                     |
| `png`  | `image/png`     | Lossless, transparent corners                                 |
| `webp` | `image/webp`    | Lossless, transparent corners                                 |
| `avif` | `image/avif`    | Lossy at high quality, transparent corners; slowest to encode |
| `jpeg` | `image/jpeg`    | Lossy at high quality; no transparency, so corners are white  |

Format names are case-insensitive, and anything else returns `400`.
