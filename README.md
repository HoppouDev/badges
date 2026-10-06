<div align="center">

<img src="assets/logo-large.svg" width="524">

A silly badges API.

<br>

<a href="https://www.rust-lang.org"><img alt="Built with Rust" src="https://badges.hoppou.dev/badge?style=pill&size=cozy&title=Built%20with&label=Rust&color=f74c00&icon=simple:rust"></a>
<a href="https://github.com/HoppouDev/badges/actions/workflows/rust.yml"><img alt="CI status" src="https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?style=pill&size=cozy"></a>
<a href="https://badges.hoppou.dev/docs"><img alt="Read the docs" src="https://badges.hoppou.dev/badge?style=pill&size=cozy&title=Read%20the&label=Docs&color=60a5fa&icon=lucide:book-open"></a>

</div>

## Usage

Badges are served at `https://badges.hoppou.dev`. The [docs](https://badges.hoppou.dev/docs) cover
every parameter, with examples:

```http
GET /badge?style=pill&title=Built%20with&label=Rust&color=f74c00&icon=simple:rust
GET /ci/{owner}/{repo}/{workflow}?style=pill
```

## Layout

Two Cloudflare Workers share `badges.hoppou.dev`:

| Directory   | Worker        | Serves                                                                         |
| ----------- | ------------- | ------------------------------------------------------------------------------ |
| `backend/`  | `badges`      | The badge API (Rust), on the routes `/badge*` and `/ci/*`                      |
| `frontend/` | `badges-docs` | Everything else (SvelteKit). It holds the Custom Domain, so it is the fallback |

Routes run before a Custom Domain, so badge paths reach the API and all other paths fall through to
the frontend. `badge*` also matches paths such as `/badges`, so frontend pages must not start with
`/badge`.

## Development

The API is the `badges` crate in `backend/`, a member of the Cargo workspace at the repo root. Cargo
commands run from the root, and `wrangler` runs from `backend/`.

Git hooks run through [prek](https://prek.j178.dev), configured in `prek.toml`. Before each commit
they:

- fix trailing whitespace and missing final newlines, and reject large files
- format TOML with tombi and Rust with rustfmt
- lint and format JS, TS and Svelte with [ESLint](https://eslint.org) (`frontend/eslint.config.js`)
- lint and format JSON, CSS and HTML with [Biome](https://biomejs.dev) (`biome.json`)
- lint and rewrap Markdown with [rumdl](https://github.com/rvben/rumdl) (`.rumdl.toml`)
- run clippy

A formatter that changes a file fails the commit, so review and stage its edits, then commit again.
On `git commit` without a conventional `-m`, [koji](https://github.com/cococonscious/koji) prompts
for one, with scopes autocompleted from history (`.koji.toml`).

Install the tools and hooks once per clone. ESLint and Biome come from the frontend's dependencies:

```sh
cargo install cargo-binstall  # once, fetches prebuilt binaries for the line below
cargo binstall prek koji rumdl
uv tool install tombi  # or npm i -g tombi (the crates.io crate is only a placeholder)
(cd frontend && pnpm install)
prek install
```

```sh
# Test
cargo test

# Render a badge locally (any format)
cargo run --example render -- "title=Built with" label=Sass color=cd6699 format=png > sass.png

# Render a workflow status badge in any state, without GitHub (or use /ci/...?state=running)
cargo run --example render -- state=running style=pill > running.svg

# Preview the API locally (put GITHUB_TOKEN=... in backend/.dev.vars for /ci)
cd backend && npx wrangler dev
```

The frontend is a SvelteKit app in `frontend/` that uses pnpm. Its
[README](frontend/README.md) covers its commands and how to write docs pages.

## Releasing

Pushing a `v*` tag deploys both Workers (the API from `backend/`, the frontend from `frontend/`)
and, once both succeed, publishes a GitHub release with notes from
[cocogitto](https://docs.cocogitto.io), rendered by `changelog.tera`. The tag must match the
`backend/Cargo.toml` version, so cut releases with `cog bump` from the repo root, configured in
`cog.toml`. It refuses to run off `main`, with uncommitted changes or with non-conventional commits
since the last tag. It then sets the version in `backend/Cargo.toml` and `Cargo.lock` through
[cargo-release](https://github.com/crate-ci/cargo-release)
(`cargo binstall cocogitto cargo-release`), commits, tags and pushes:

```sh
# Preview the next version and its release notes
cog bump --auto --dry-run
cog changelog "$(git describe --tags --abbrev=0).."

cog bump --auto  # or --patch, --minor, --major, --version X.Y.Z
```

The workflow needs a `CLOUDFLARE_API_TOKEN` repository secret.

```sh
# From backend/
# Required for /ci
npx wrangler secret put GITHUB_TOKEN

# Manual API deploy, bypassing a release
npx wrangler deploy
```

To deploy the frontend outside a release:

```sh
cd frontend
pnpm build
pnpm exec wrangler deploy
```

Never commit tokens: `backend/.dev.vars` is ignored by git, and production uses the Worker secret.

## License

This project is licensed under the MIT License. See [LICENSE.md](LICENSE.md).

The bundled Inter and JetBrains Mono fonts are licensed under the SIL Open Font License
([Inter](backend/assets/fonts/LICENSE.txt),
[JetBrains Mono](backend/assets/fonts/LICENSE-JetBrainsMono.txt)). Icons come from
[Simple Icons](https://simpleicons.org) (CC0) and [Lucide](https://lucide.dev) (ISC,
[bundled marks](backend/assets/icons/LICENSE-lucide.txt)).
