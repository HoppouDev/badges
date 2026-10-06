---
title: Errors and limits
section: Reference
order: 30
---

## Errors

Errors come back as plain text with an HTTP status, so a broken badge URL shows its reason when
opened directly.

| Status | When                                                                                                                                                                                        |
| ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `400`  | A parameter is missing or invalid: no `label`, an unknown `style`, `size`, `theme`, `format` or `state`, a bad colour, an icon without a set, a disallowed image host, a renamed repository |
| `404`  | The icon, repository or workflow doesn't exist. Private repositories also answer `404`                                                                                                      |
| `502`  | GitHub or the icon host returned an error                                                                                                                                                   |
| `503`  | GitHub's rate limit was reached, so try again shortly                                                                                                                                       |
| `504`  | GitHub or the icon host took too long                                                                                                                                                       |

## Limits

| Limit               | Value                                                  |
| ------------------- | ------------------------------------------------------ |
| `title` and `label` | 64 characters each, in characters the badge fonts have |
| Image icon size     | 64 KiB, and at most 2048 pixels on each side           |
| Image icon hosts    | See [Icons](/docs/badges/icons#image-urls)             |
| Repositories        | Public only                                            |

## Caching

| Response                                       | Browsers  | Cloudflare edge |
| ---------------------------------------------- | --------- | --------------- |
| Custom badges                                  | 1 day     | 7 days          |
| Workflow status badges, including their errors | 1 minute  | 1 minute        |
| Invalid custom badge requests (`4xx`)          | 5 minutes | 5 minutes       |
| Other custom badge errors (`5xx`)              | never     | never           |

A custom badge is cached for its URL. After the service is updated, an old copy can keep showing in
browsers for up to a day. Changing the URL, for example by adding `&v=2`, gets the new badge at
once.
