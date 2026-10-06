# Frontend

The docs site at `https://badges.hoppou.dev/docs`, built with SvelteKit and deployed as the
`badges-docs` Cloudflare Worker. See the [root README](../README.md) for how it shares the domain
with the badge API.

## Development

```sh
pnpm install
pnpm dev      # dev server
pnpm check    # type check
pnpm lint     # ESLint and Biome
pnpm test     # unit tests
pnpm build    # production build
pnpm preview  # serve the production build with wrangler
```

## Writing docs

The docs are Markdown files in `src/docs/`, served under `/docs`. `index.md` is `/docs` and `foo.md`
is `/docs/foo`. They are turned into HTML with [remark](https://github.com/remarkjs/remark) and
[remark-rehype](https://github.com/remarkjs/remark-rehype) and prerendered at build time. `/`
redirects to `/docs`.

Each doc starts with a frontmatter block:

```markdown
---
title: Overview
section: Getting started
order: 0
---
```

- `title` is the page's heading, its browser title and its sidebar link, so docs don't write their
  own top-level heading.
- `section` is the sidebar group the page sits under. Sections follow the order of their first page.
- `order` is optional and positions the page in the sidebar (default `0`, lowest first, ties sorted
  by title).

The build fails if:

- `title` or `section` is missing
- `order` is not a finite number
- a doc has a top-level heading (`# Title`, or `Title` underlined with `===`)
- two files map to the same URL, such as `foo.md` and `foo/index.md`

An image's Markdown title becomes a caption below it. This works for an image on its own line or
alone in a table cell. A titled image anywhere else fails the build.

```markdown
![Built with Rust](https://badges.hoppou.dev/badge?label=Rust "Built with Rust")
```
