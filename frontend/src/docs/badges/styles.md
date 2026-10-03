---
title: Styles
section: Badges
order: 11
---

Pick a style with `style`. Both work for [custom badges](/docs/badges) and [workflow status badges](/docs/ci).

## Cozy

The default: 56px tall, with the title stacked above the label.

![Built with Rust](https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=f74c00&icon=rust&style=cozy)

```http
GET /badge?title=Built%20with&label=Rust&color=f74c00&icon=rust&style=cozy
```

## Compact

40px tall, with the title and label on one line. It suits rows of badges and dense READMEs.

![Built with Rust](https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=f74c00&icon=rust&style=compact)

```http
GET /badge?title=Built%20with&label=Rust&color=f74c00&icon=rust&style=compact
```

## Side by side

| Cozy                                                                                                     | Compact                                                                                                                |
| -------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| ![Built with Sass](https://badges.hoppou.dev/badge?title=Built%20with&label=Sass&color=cd6699&icon=sass) | ![Built with Sass](https://badges.hoppou.dev/badge?title=Built%20with&label=Sass&color=cd6699&icon=sass&style=compact) |
| ![Python](https://badges.hoppou.dev/badge?label=Python&color=3776ab&icon=python)                         | ![Python](https://badges.hoppou.dev/badge?label=Python&color=3776ab&icon=python&style=compact)                         |
| ![CI](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml)                                            | ![CI](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?style=compact)                                            |

Style names are case-insensitive, and anything other than `cozy` or `compact` returns `400`.
