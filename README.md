<div align="center">

# 🛡️ ʙᴀᴅɢᴇꜱ 🛡️

A silly badges API, based on [Devin's Badges](https://github.com/intergrav/devins-badges).

<a href="https://www.rust-lang.org"><img alt="Built with Rust" src="https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=f74c00&icon=rust&style=cozy"></a>
<a href="https://badges.hoppou.dev"><img alt="Deployed on Cloudflare" src="https://badges.hoppou.dev/badge?title=Deployed%20on&label=Cloudflare&color=f38020&icon=cloudflare&style=cozy"></a>
<a href="https://github.com/HoppouDev/badges/actions/workflows/rust.yml"><img alt="CI Passing" src="https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?style=cozy"></a>

</div>

## Usage

Badges are served at `https://badges.hoppou.dev`, as SVG by default or rasterised with `format`. Every parameter is listed below.

### Custom badge

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

### GitHub Actions status

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

### Image formats

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

## Layout

Two Cloudflare Workers share `badges.hoppou.dev`:

| Directory   | Worker        | Serves                                                                         |
| ----------- | ------------- | ------------------------------------------------------------------------------ |
| `backend/`  | `badges`      | The badge API (Rust), on the routes `/badge*` and `/ci/*`                      |
| `frontend/` | `badges-docs` | Everything else (SvelteKit); it holds the Custom Domain, so it is the fallback |

Routes run before a Custom Domain, so badge paths reach the API and all other paths fall through to the frontend. `badge*` also matches paths such as `/badges`, so frontend pages must not start with `/badge`.

## Development

The API is the `badges` crate in `backend/`, a member of the Cargo workspace at the repo root. Cargo commands run from the root; `wrangler` runs from `backend/`.

```sh
# Test
cargo test

# Render a badge locally (any format)
cargo run --example render -- "title=Built with" label=Sass color=cd6699 format=png > sass.png

# Preview the API locally; put GITHUB_TOKEN=... in backend/.dev.vars for /ci
cd backend && npx wrangler dev
```

The frontend uses pnpm:

```sh
cd frontend
pnpm install
pnpm dev
```

## Releasing

Pushing a `v*` tag deploys the API Worker and publishes a GitHub release with notes from [git-cliff](https://git-cliff.org). The tag must match the `backend/Cargo.toml` version, so cut releases with [cargo-release](https://github.com/crate-ci/cargo-release) (`cargo binstall cargo-release`), configured in `release.toml`. From the repo root, it bumps the version in `backend/Cargo.toml` and `Cargo.lock`, commits, tags and pushes, and only dry-runs without `--execute`:

```sh
# Suggested bump from conventional commits since the last tag
git cliff --bumped-version

cargo release minor --execute  # or patch, major, X.Y.Z
```

The workflow needs a `CLOUDFLARE_API_TOKEN` repository secret.

```sh
# From backend/
# Required for /ci
npx wrangler secret put GITHUB_TOKEN

# Manual API deploy, bypassing a release
npx wrangler deploy
```

The frontend is not part of releases; deploy it by hand:

```sh
cd frontend
pnpm build
pnpm exec wrangler deploy
```

Never commit tokens: `backend/.dev.vars` is ignored by git, and production uses the Worker secret.

## License

This project is licensed under the MIT License. See [LICENSE.md](LICENSE.md).

The bundled Inter fonts are licensed under the SIL Open Font License ([backend/assets/fonts/LICENSE.txt](backend/assets/fonts/LICENSE.txt)). Icons come from [Simple Icons](https://simpleicons.org) (CC0), and the badge design is based on [Devin's Badges](https://github.com/intergrav/devins-badges) (CC0).
