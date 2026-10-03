---
title: Icons
section: Badges
order: 13
---

`icon` adds an icon to the left of the text. It takes either a Simple Icons slug or an image URL.

## Simple Icons

Use the slug from [simpleicons.org](https://simpleicons.org), the brand name in lowercase without spaces or punctuation. The icon is filled with the accent `color`, or with `iconColor` if you set one.

![GitHub](https://badges.hoppou.dev/badge?title=Hosted%20on&label=GitHub&color=ffffff&icon=github)
![Docker](https://badges.hoppou.dev/badge?title=Ships%20with&label=Docker&color=2496ed&icon=docker)
![Firefox](https://badges.hoppou.dev/badge?title=Works%20in&label=Firefox&color=ff7139&icon=firefoxbrowser)

```http
GET /badge?title=Ships%20with&label=Docker&color=2496ed&icon=docker
```

With `iconColor`, the icon and label can differ:

![Icon colour](https://badges.hoppou.dev/badge?title=Built%20with&label=Rust&color=ffffff&icon=rust&iconColor=f74c00)

```http
GET /badge?title=Built%20with&label=Rust&color=ffffff&icon=rust&iconColor=f74c00
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
