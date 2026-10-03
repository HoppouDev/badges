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

| Parameter | Description                                                 |
| --------- | ----------------------------------------------------------- |
| `title`   | Top line (default `CI`; empty for a single-line badge)      |
| `branch`  | Branch to report (default: the repository's default branch) |
| `event`   | Triggering event to report (default `push`)                 |
| `style`   | `cozy` (default) or `compact`                               |
| `format`  | Same as for [custom badges](/docs/badges/formats)           |

Only runs triggered by `event` count, so pull requests from forks can't change a badge that follows `push` runs.

## States

The badge reports the latest matching run, coloured like GitHub's own status icons.

| State     | Meaning                                        |
| --------- | ---------------------------------------------- |
| Passing   | The run succeeded                              |
| Failing   | The run failed, timed out or failed to start   |
| Running   | The run is queued or in progress               |
| Cancelled | The run was cancelled                          |
| Skipped   | The run was skipped or finished as neutral     |
| Unknown   | No matching run yet, or an unrecognised result |

## Examples

| Badge                                                                                    | Query                    |
| ---------------------------------------------------------------------------------------- | ------------------------ |
| ![Default](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml)                       | (none)                   |
| ![Titled](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?title=Tests)            | `title=Tests`            |
| ![Single line](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?title=)            | `title=`                 |
| ![Compact](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?style=compact)         | `style=compact`          |
| ![Branch](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?branch=main&title=main) | `branch=main&title=main` |

Status badges are cached for a minute, so a new run can take that long to show.
