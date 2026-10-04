---
title: Colours
section: Badges
order: 12
---

Colours accept hex with or without `#` (encode `#` as `%23`), `rgb()`, `hsl()` or CSS names such as `rebeccapurple`. This page shows the Devin style; [pill colours](#pill-colours) work a little differently.

## Accent

`color` sets the label and icon. The background is derived from it: a dark, tinted gradient for saturated colours, and a neutral one for greys.

![Rust](https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=f74c00&icon=simple:rust)
![Go](https://badges.hoppou.dev/badge?title=Built%20with&label=Go&color=00add8&icon=simple:go)
![Kotlin](https://badges.hoppou.dev/badge?title=Built%20with&label=Kotlin&color=7f52ff&icon=simple:kotlin)
![Grey](https://badges.hoppou.dev/badge?title=Built%20with&label=Plain%20text&color=a3a3a3)

## Label gradient

`color2` blends the label vertically from `color` into a second colour.

![Gradient](https://badges.hoppou.dev/badge?title=Built%20with&label=Gradients&color=f472b6&color2=8b5cf6)

```http
GET /badge?title=Built%20with&label=Gradients&color=f472b6&color2=8b5cf6
```

## Title colour

`titleColor` changes the small top line, which is otherwise a light grey.

![Title colour](https://badges.hoppou.dev/badge?title=Powered%20by&label=Coffee&color=d97706&titleColor=fcd34d)

```http
GET /badge?title=Powered%20by&label=Coffee&color=d97706&titleColor=fcd34d
```

## Background

`bg` alone gives a flat background; add `bg2` for a gradient from `bg` at the top to `bg2` at the bottom. `bg2` alone keeps the derived top and replaces only the bottom.

| Badge                                                                                                          | Query                               |
| -------------------------------------------------------------------------------------------------------------- | ----------------------------------- |
| ![Flat](https://badges.hoppou.dev/badge?title=Flat&label=Background&color=ffffff&bg=1e293b)                    | `color=ffffff&bg=1e293b`            |
| ![Gradient](https://badges.hoppou.dev/badge?title=Gradient&label=Background&color=ffffff&bg=4338ca&bg2=0f172a) | `color=ffffff&bg=4338ca&bg2=0f172a` |
| ![CSS names](https://badges.hoppou.dev/badge?title=CSS&label=Names&color=gold&bg=darkslategray)                | `color=gold&bg=darkslategray`       |

## Pill colours

A pill badge is drawn in a single colour and painted with one gradient that runs across its whole width, from `color` on the left to `color2` on the right. The outline, chip, mark or icon, title and label all take their colour from wherever they sit along it, so the text is part of the gradient too.

| Parameter | Pill use                                                                |
| --------- | ----------------------------------------------------------------------- |
| `color`   | Gradient start, at the left edge                                        |
| `color2`  | Gradient end, at the right edge (default: `color` with its hue turned)  |
| `bg`      | Fill behind the badge (default transparent); `theme=auto` then suits it |

`titleColor`, `iconColor` and `bg2` have no effect, and an image icon keeps its own colours. `color` and `color2` are lightened on the dark theme and darkened on the light one, as far as needed to stay readable.

![Gradient pill](https://badges.hoppou.dev/badge?style=pill&title=gradient&label=pink%20to%20violet&color=f472b6&color2=8b5cf6)

```http
GET /badge?style=pill&title=gradient&label=pink%20to%20violet&color=f472b6&color2=8b5cf6
```
