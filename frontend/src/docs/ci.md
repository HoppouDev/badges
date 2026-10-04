---
title: Workflow status
section: GitHub Actions
order: 20
---

Show whether a GitHub Actions workflow is passing:

```http
GET /ci/{owner}/{repo}/{workflow}
```

![CI](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml)

```markdown
![CI](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml)
```

`{workflow}` is the workflow's file name, such as `rust.yml`, or its numeric id. Only public repositories are served.

## Parameters

| Parameter | Description                                                              |
| --------- | ------------------------------------------------------------------------ |
| `title`   | Top line (default `CI`; empty for a single-line badge)                   |
| `branch`  | Branch to report (default: the repository's default branch)              |
| `event`   | Triggering event to report (default `push`)                              |
| `style`   | `devin` (default) or `pill`; see [Styles and sizes](/docs/badges/styles) |
| `size`    | `cozy` (default) or `compact`                                            |
| `theme`   | Pill colours: `auto` (default), `dark` or `light`                        |
| `format`  | Same as for [custom badges](/docs/badges/formats)                        |
| `state`   | Show this [state](#states) without asking GitHub; for previews and tests |

Only runs triggered by `event` count, so pull requests from forks can't change a badge that follows `push` runs.

## States

The badge reports the latest matching run, coloured like GitHub's own status icons. Devin badges show the GitHub Actions icon; [pill badges](/docs/badges/styles#pill) show a status mark in its place.

| State     | Meaning                                        | Pill mark                      |
| --------- | ---------------------------------------------- | ------------------------------ |
| Passing   | The run succeeded                              | Lucide's `check`               |
| Failing   | The run failed, timed out or failed to start   | Lucide's `x`                   |
| Running   | The run is queued or in progress               | Lucide's `refresh-cw`, turning |
| Cancelled | The run was cancelled                          | Dot                            |
| Skipped   | The run was skipped or finished as neutral     | Dot                            |
| Unknown   | No matching run yet, or an unrecognised result | Dot                            |

The Running icon only turns in SVG; other [image formats](/docs/badges/formats) show it standing still, and so do viewers who ask for reduced motion.

To preview a state without waiting for a run, add `state` (`passing`, `failing`, `running`, `cancelled`, `skipped` or `unknown`). GitHub isn't asked, so the repository and workflow aren't checked either, only their spelling:

```http
GET /ci/HoppouDev/badges/rust.yml?style=pill&state=running
```

## Examples

| Badge                                                                                    | Query                    |
| ---------------------------------------------------------------------------------------- | ------------------------ |
| ![Default](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml)                       | (none)                   |
| ![Titled](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?title=Tests)            | `title=Tests`            |
| ![Single line](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?title=)            | `title=`                 |
| ![Compact](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?size=compact)          | `size=compact`           |
| ![Pill](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?style=pill)               | `style=pill`             |
| ![Branch](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?branch=main&title=main) | `branch=main&title=main` |

Status badges are cached for a minute, so a new run can take that long to show.
