---
title: Icons
section: Badges
order: 13
---

`icon` adds an icon to the left of the text. It takes an icon set prefix and a name, or an image URL:

| Value           | Icon                                                                          |
| --------------- | ----------------------------------------------------------------------------- |
| `simple:<slug>` | A brand logo from [Simple Icons](https://simpleicons.org), filled             |
| `lucide:<name>` | A line icon from [Lucide](https://lucide.dev/icons), drawn with round strokes |
| `https://…`     | An image; see [Image URLs](#image-urls)                                       |

A name without a prefix, such as `icon=rust`, returns `400`. The prefix and name are case-insensitive.

## Simple Icons

Use `simple:` and the slug from [simpleicons.org](https://simpleicons.org), the brand name in lowercase without spaces or punctuation. The icon is filled with the accent `color`, or with `iconColor` if you set one.

![GitHub](https://badges.hoppou.dev/badge?title=Hosted%20on&label=GitHub&color=ffffff&icon=simple:github)
![Docker](https://badges.hoppou.dev/badge?title=Ships%20with&label=Docker&color=2496ed&icon=simple:docker)
![Firefox](https://badges.hoppou.dev/badge?title=Works%20in&label=Firefox&color=ff7139&icon=simple:firefoxbrowser)

```http
GET /badge?title=Ships%20with&label=Docker&color=2496ed&icon=simple:docker
```

With `iconColor`, the icon and label can differ:

![Icon colour](https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=ffffff&icon=simple:rust&iconColor=f74c00)

```http
GET /badge?title=Built%20with&label=Rust&color=ffffff&icon=simple:rust&iconColor=f74c00
```

## Lucide

Use `lucide:` and the icon's name from [lucide.dev](https://lucide.dev/icons), such as `rocket` or `circle-check`. Lucide icons are outlines, coloured the same way as Simple Icons.

![Rocket](https://badges.hoppou.dev/badge?title=Launched&label=Today&color=f472b6&icon=lucide:rocket)
![Shield](https://badges.hoppou.dev/badge?title=Security&label=Audited&color=34d399&icon=lucide:shield-check)
![Pill](https://badges.hoppou.dev/badge?style=pill&title=status&label=online&color=34d399&icon=lucide:circle-check)

```http
GET /badge?title=Launched&label=Today&color=f472b6&icon=lucide:rocket
```

## Image URLs

`icon` also accepts an `https` PNG, JPEG, GIF or WebP URL on an allowed host. The image keeps its own colours, so `iconColor` does not apply.

![Avatar](https://badges.hoppou.dev/badge?title=Made%20by&label=Hoppou&color=60a5fa&icon=https%3A%2F%2Favatars.githubusercontent.com%2Fu%2F155388180%3Fs%3D64%26v%3D4)

```http
GET /badge?title=Made%20by&label=Hoppou&color=60a5fa&icon=https%3A%2F%2Favatars.githubusercontent.com%2Fu%2F155388180%3Fs%3D64%26v%3D4
```

Encode the URL when it has its own query string, as above. Remote icons must:

- use `https` on one of the allowed hosts: `raw.githubusercontent.com`, `avatars.githubusercontent.com`, `user-images.githubusercontent.com`, `cdn.jsdelivr.net` or `unpkg.com`
- be at most 64 KiB and 2048 pixels on each side
- answer directly, without redirecting
