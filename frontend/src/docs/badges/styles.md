---
title: Styles and sizes
section: Badges
order: 11
---

`style` picks a badge's design and `size` its height. Both work for [custom badges](/docs/badges)
and [workflow status badges](/docs/ci).

| Style   | Cozy (default)                                                                                                                | Compact                                                                                                                                    |
| ------- | ----------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `devin` | ![Built with Sass](https://badges.hoppou.dev/badge?title=Built%20with&label=Sass&color=cd6699&icon=simple:sass)               | ![Built with Sass](https://badges.hoppou.dev/badge?title=Built%20with&label=Sass&color=cd6699&icon=simple:sass&size=compact)               |
| `pill`  | ![Build passing](https://badges.hoppou.dev/badge?style=pill&title=build&label=passing&color=34d399&icon=simple:githubactions) | ![Build passing](https://badges.hoppou.dev/badge?style=pill&size=compact&title=build&label=passing&color=34d399&icon=simple:githubactions) |

## Devin

The default, after [Devin's Badges](https://github.com/intergrav/devins-badges): a gradient card
with a faint border and a bold label. Its background is derived from `color`. In the dark theme it's
the original dark design, and the light theme gives a pale variant for light pages. See
[Themes](/docs/badges/themes).

| Cozy                                                                                    | Compact                                                                                              |
| --------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| ![Python](https://badges.hoppou.dev/badge?label=Python&color=3776ab&icon=simple:python) | ![Python](https://badges.hoppou.dev/badge?label=Python&color=3776ab&icon=simple:python&size=compact) |
| ![CI](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml)                           | ![CI](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?size=compact)                           |

- **Cozy** is 56px tall, with the title stacked above the label.
- **Compact** is 40px tall, with the title and label on one line.

## Pill

A rounded chip with monospace text on a transparent background. One gradient paints the whole badge,
text included, from `color` on the left to `color2` on the right (see
[Pill colours](/docs/badges/colours#pill)). It adapts to light and dark pages, see
[Themes](/docs/badges/themes).

```http
GET /badge?style=pill&title=release&label=v2.4.1&color=60a5fa&icon=lucide:tag
```

| Cozy                                                                                                           | Compact                                                                                                                     |
| -------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| ![Release](https://badges.hoppou.dev/badge?style=pill&title=release&label=v2.4.1&color=60a5fa&icon=lucide:tag) | ![Release](https://badges.hoppou.dev/badge?style=pill&size=compact&title=release&label=v2.4.1&color=60a5fa&icon=lucide:tag) |
| ![Operational](https://badges.hoppou.dev/badge?style=pill&label=operational&color=34d399)                      | ![Operational](https://badges.hoppou.dev/badge?style=pill&size=compact&label=operational&color=34d399)                      |
| ![CI](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?style=pill)                                       | ![CI](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?style=pill&size=compact)                                       |

- **Cozy** is 44px tall. The icon and the title, in capitals, sit in a tinted chip on the left, and
  the label follows.
- **Compact** is 34px tall. The icon, if there is one, leads the title and label on one line.

## Older URLs

Before styles existed, `style` chose the size, so `style=cozy` and `style=compact` still mean the
Devin style at that size. An explicit `size` wins over them.

Names are case-insensitive, and an unknown `style` or `size` returns `400`.
