---
title: Errors and limits
section: Reference
order: 30
---

## Errors

Errors come back as plain text with an HTTP status, so a broken badge URL shows its reason when opened directly.

| Status | When                                                                                                           |
| ------ | -------------------------------------------------------------------------------------------------------------- |
| `400`  | A parameter is missing or invalid: no `label`, an unknown `style` or `format`, a bad colour, a disallowed icon |
| `404`  | The icon, repository or workflow doesn't exist; private repositories also answer `404`                         |
| `502`  | GitHub or the icon host returned an error                                                                      |
| `503`  | GitHub's rate limit was reached; try again shortly                                                             |
| `504`  | GitHub or the icon host took too long                                                                          |

## Limits

| Limit               | Value                                                     |
| ------------------- | --------------------------------------------------------- |
| `title` and `label` | 64 characters each, in characters the badge font supports |
| Remote icon size    | 64 KiB, and at most 2048 pixels on each side              |
| Remote icon hosts   | See [Icons](/docs/badges/icons)                           |
| Repositories        | Public only                                               |

## Caching

| Response                                       | Browsers  | Cloudflare edge |
| ---------------------------------------------- | --------- | --------------- |
| Custom badges                                  | 1 day     | 7 days          |
| Workflow status badges, including their errors | 1 minute  | 1 minute        |
| Invalid custom badge requests (`4xx`)          | 5 minutes | 5 minutes       |
| Other custom badge errors (`5xx`)              | never     | never           |

A custom badge never changes for a given URL, so there's nothing to purge: change the URL to get a new badge.
