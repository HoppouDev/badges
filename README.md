<div align="center">

# 🛡️ ʙᴀᴅɢᴇꜱ 🛡️

A silly badges API, based on [Devin's Badges](https://github.com/intergrav/devins-badges).

<a href="https://www.rust-lang.org"><img alt="Built with Rust" src="https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=f74c00&icon=rust&style=cozy"></a>
<a href="https://badges.hoppou.dev"><img alt="Deployed on Cloudflare" src="https://badges.hoppou.dev/badge?title=Deployed%20on&label=Cloudflare&color=f38020&icon=cloudflare&style=cozy"></a>
<a href="https://github.com/HoppouDev/badges/actions/workflows/rust.yml"><img alt="CI Passing" src="https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?style=cozy"></a>

</div>

## Usage

Badges are SVGs served by a Cloudflare Worker at `https://badges.hoppou.dev`. The full parameter list is served at `/`.

### Custom badge

```http
GET /badge?title=Built%20with&label=Sass&color=cd6699&icon=sass
```

| Parameter | Description                                                                                        |
| --------- | -------------------------------------------------------------------------------------------------- |
| `label`   | Bold bottom line (required)                                                                        |
| `title`   | Small top line                                                                                     |
| `color`   | Accent colour for the label and icon                                                               |
| `icon`    | [Simple Icons](https://simpleicons.org) slug, or an https PNG/JPEG/GIF/WebP URL on an allowed host |
| `style`   | `cozy` (default, 56px, title above label) or `compact` (40px, one line)                            |

Colours accept hex with or without `#`, `rgb()`, `hsl()` or CSS names.

### GitHub Actions status

```http
GET /ci/{owner}/{repo}/{workflow}?branch=main&title=CI
```

Shows Passing, Failing, Running, Cancelled, Skipped or Unknown for the latest `push` run, in either `style`. Only public repositories are served.

## Development

```sh
# Test
cargo test

# Preview the API locally put GITHUB_TOKEN=... and in .dev.vars for /ci
npx wrangler dev
```

## Deploying

```sh
# Required for /ci
npx wrangler secret put GITHUB_TOKEN

# Deploy to Cloudflare Workers
npx wrangler deploy
```

Never commit tokens: `.dev.vars` is ignored by git, and production uses the Worker secret.

## License

This project is licensed under the MIT License. See [LICENSE.md](LICENSE.md).

The bundled Inter fonts are licensed under the SIL Open Font License ([assets/fonts/LICENSE.txt](assets/fonts/LICENSE.txt)). Icons come from [Simple Icons](https://simpleicons.org) (CC0), and the badge design is based on [Devin's Badges](https://github.com/intergrav/devins-badges) (CC0).
