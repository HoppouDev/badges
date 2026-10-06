---
title: Colours
section: Badges
order: 13
---

Colours accept hex with or without `#` (encode `#` as `%23`), `rgb()`, `hsl()` or CSS names such as
`rebeccapurple`. The two [styles](/docs/badges/styles) use them differently.

| Parameter    | Devin                           | Pill                  |
| ------------ | ------------------------------- | --------------------- |
| `color`      | Label and icon                  | Gradient start        |
| `color2`     | Label gradient's bottom         | Gradient end          |
| `titleColor` | Title                           | Not used              |
| `iconColor`  | Icon                            | Not used              |
| `bg`         | Background, or its gradient top | Fill behind the badge |
| `bg2`        | Background gradient's bottom    | Not used              |

## Devin

### Accent

`color` sets the label and icon. The background is derived from it: a dark, tinted gradient for
saturated colours, and a neutral one for greys.

![Rust](https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=f74c00&icon=simple:rust)
![Go](https://badges.hoppou.dev/badge?title=Built%20with&label=Go&color=00add8&icon=simple:go)
![Kotlin](https://badges.hoppou.dev/badge?title=Built%20with&label=Kotlin&color=7f52ff&icon=simple:kotlin)
![Grey](https://badges.hoppou.dev/badge?title=Built%20with&label=Plain%20text&color=a3a3a3)

### Label gradient

`color2` blends the label from `color` at the top into `color2` at the bottom.

![Gradient](https://badges.hoppou.dev/badge?title=Built%20with&label=Gradients&color=f472b6&color2=8b5cf6)

```http
GET /badge?title=Built%20with&label=Gradients&color=f472b6&color2=8b5cf6
```

### Title and icon

`titleColor` changes the title, which is otherwise a light grey, and `iconColor` changes a `simple:`
or `lucide:` icon, which otherwise matches `color`.

![Title colour](https://badges.hoppou.dev/badge?title=Powered%20by&label=Coffee&color=d97706&titleColor=fcd34d)
![Icon colour](https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=ffffff&icon=simple:rust&iconColor=f74c00)

```http
GET /badge?title=Powered%20by&label=Coffee&color=d97706&titleColor=fcd34d
```

### Background

`bg` alone gives a flat background; add `bg2` for a gradient from `bg` at the top to `bg2` at the
bottom. `bg2` alone keeps the derived top and replaces only the bottom.

| Badge                                                                                                          | Query                               |
| -------------------------------------------------------------------------------------------------------------- | ----------------------------------- |
| ![Flat](https://badges.hoppou.dev/badge?title=Flat&label=Background&color=ffffff&bg=1e293b)                    | `color=ffffff&bg=1e293b`            |
| ![Gradient](https://badges.hoppou.dev/badge?title=Gradient&label=Background&color=ffffff&bg=4338ca&bg2=0f172a) | `color=ffffff&bg=4338ca&bg2=0f172a` |
| ![CSS names](https://badges.hoppou.dev/badge?title=CSS&label=Names&color=gold&bg=darkslategray)                | `color=gold&bg=darkslategray`       |

## Pill

A pill badge is drawn in one colour and painted with a single gradient across its whole width, from
`color` on the left to `color2` on the right. The outline, chip, icon, title and label all take the
colour of the gradient where they sit, so the text shades along with it. Without `color2`, the
gradient ends on `color` with its hue turned a little.

![Pink to violet](https://badges.hoppou.dev/badge?style=pill&title=gradient&label=pink%20to%20violet&color=f472b6&color2=8b5cf6)
![Hue turned](https://badges.hoppou.dev/badge?style=pill&title=gradient&label=one%20colour&color=34d399)

```http
GET /badge?style=pill&title=gradient&label=pink%20to%20violet&color=f472b6&color2=8b5cf6
```

The [theme](/docs/badges/themes) tones both ends to stay readable on the page. The background is
transparent, apart from a faint tint of the gradient; `bg` fills it with a solid colour instead. An
image icon keeps its own colours.
