---
title: Colours
section: Badges
order: 12
---

Colours accept hex with or without `#` (encode `#` as `%23`), `rgb()`, `hsl()` or CSS names such as `rebeccapurple`.

## Accent

`color` sets the label and icon. The background is derived from it: a dark, tinted gradient for saturated colours, and a neutral one for greys.

![Rust](https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=f74c00&icon=rust)
![Go](https://badges.hoppou.dev/badge?title=Built%20with&label=Go&color=00add8&icon=go)
![Kotlin](https://badges.hoppou.dev/badge?title=Built%20with&label=Kotlin&color=7f52ff&icon=kotlin)
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
