---
title: Icons
section: Badges
order: 14
---

`icon` adds an icon before the text. It takes an icon set and a name, or an image URL.

| Value           | Icon                                                             |
| --------------- | ---------------------------------------------------------------- |
| `simple:<slug>` | A filled brand logo from [Simple Icons](https://simpleicons.org) |
| `lucide:<name>` | An outlined icon from [Lucide](https://lucide.dev/icons)         |
| `https://…`     | An image, see [Image URLs](#image-urls)                          |

The set and name are case-insensitive. A name without a set, such as `icon=rust`, returns `400`, and
one the set doesn't have returns `404`.

Devin badges colour `simple:` and `lucide:` icons with `color`, or with `iconColor` if you set one.
Pill badges paint them with the gradient like the rest of the badge.

## Simple Icons

Use `simple:` and the slug from [simpleicons.org](https://simpleicons.org): the brand name in
lowercase, without spaces or punctuation.

![GitHub](https://badges.hoppou.dev/badge?title=Hosted%20on&label=GitHub&color=ffffff&icon=simple:github)
![Docker](https://badges.hoppou.dev/badge?title=Ships%20with&label=Docker&color=2496ed&icon=simple:docker)
![Firefox](https://badges.hoppou.dev/badge?style=pill&title=works%20in&label=firefox&color=ff7139&icon=simple:firefoxbrowser)

```http
GET /badge?title=Ships%20with&label=Docker&color=2496ed&icon=simple:docker
```

## Lucide

Use `lucide:` and the icon's name from [lucide.dev](https://lucide.dev/icons), such as `rocket` or
`circle-check`.

![Rocket](https://badges.hoppou.dev/badge?title=Launched&label=Today&color=f472b6&icon=lucide:rocket)
![Shield](https://badges.hoppou.dev/badge?title=Security&label=Audited&color=34d399&icon=lucide:shield-check)
![Online](https://badges.hoppou.dev/badge?style=pill&title=status&label=online&color=34d399&icon=lucide:circle-check)

```http
GET /badge?title=Launched&label=Today&color=f472b6&icon=lucide:rocket
```

## Image URLs

`icon` also takes a PNG, JPEG, GIF or WebP image. The image keeps its own colours, in both styles.

![Avatar](https://badges.hoppou.dev/badge?title=Made%20by&label=Hoppou&color=60a5fa&icon=https%3A%2F%2Favatars.githubusercontent.com%2Fu%2F155388180%3Fs%3D64%26v%3D4)

```http
GET /badge?title=Made%20by&label=Hoppou&color=60a5fa&icon=https%3A%2F%2Favatars.githubusercontent.com%2Fu%2F155388180%3Fs%3D64%26v%3D4
```

Encode the URL when it has its own query string, as above. The image must:

- use `https` on one of the allowed hosts: `raw.githubusercontent.com`,
  `avatars.githubusercontent.com`, `user-images.githubusercontent.com`, `cdn.jsdelivr.net` or
  `unpkg.com`
- be at most 64 KiB and 2048 pixels on each side
- answer directly, without redirecting
