---
title: Overview
section: Getting started
order: 0
---

Cozy badges in the style of [Devin's Badges](https://github.com/intergrav/devins-badges), served from `https://badges.hoppou.dev`. Every badge is a plain URL, so it works anywhere an image does: READMEs, websites, issue templates.

![Built with Rust](https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=f74c00&icon=rust)
![Deployed on Cloudflare](https://badges.hoppou.dev/badge?title=Deployed%20on&label=Cloudflare&color=f38020&icon=cloudflare)
![CI status](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml)

```markdown
![Built with Rust](https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=f74c00&icon=rust)
```

## Two kinds of badge

- [Custom badges](/docs/badges) at `/badge` show any text you like, with your own colours and icon.
- [Workflow status badges](/docs/ci) at `/ci/{owner}/{repo}/{workflow}` show whether a GitHub Actions workflow is passing.

Both come in two [styles](/docs/badges/styles) and several [image formats](/docs/badges/formats).

## Where to next

- [Embedding](/docs/embedding) shows the Markdown and HTML to put a badge on a page.
- [Colours](/docs/badges/colours) and [Icons](/docs/badges/icons) cover the look of a badge.
- [Errors and limits](/docs/reference/limits) lists what the API rejects and how long badges are cached.
