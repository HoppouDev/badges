---
title: Workflow status
section: GitHub Actions
order: 20
---

A workflow status badge shows whether a GitHub Actions workflow is passing.

```http
GET /ci/{owner}/{repo}/{workflow}
```

![CI](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml)
![CI](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?style=pill)

```markdown
![CI](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?style=pill)
```

`{workflow}` is the workflow's file name, such as `rust.yml`, or its numeric id. Only public
repositories are served.

## Parameters

| Parameter | Description                                                                         |
| --------- | ----------------------------------------------------------------------------------- |
| `title`   | Text before the state (default `CI`, or empty for a state-only badge)               |
| `branch`  | Branch to report (default: the repository's default branch)                         |
| `event`   | Triggering event to report (default `push`)                                         |
| `state`   | Show this [state](#states) without asking GitHub, see [Previewing](#previewing)     |
| `style`   | `devin` (default) or `pill`, see [Styles and sizes](/docs/badges/styles)            |
| `size`    | `cozy` (default) or `compact`                                                       |
| `theme`   | `auto` (default), `dark` or `light`, see [Themes](/docs/badges/themes)              |
| `format`  | `svg` (default), `png`, `avif` or `webp`, see [Image formats](/docs/badges/formats) |

The badge reports the latest run triggered by `event`, so pull requests from forks can't change a
badge that follows `push` runs.

| Badge                                                                                    | Query                    |
| ---------------------------------------------------------------------------------------- | ------------------------ |
| ![Titled](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?title=Tests)            | `title=Tests`            |
| ![State only](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?title=)             | `title=`                 |
| ![Branch](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?branch=main&title=main) | `branch=main&title=main` |
| ![Compact](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?size=compact)          | `size=compact`           |

## States

States are coloured like GitHub's own status icons. Devin badges show the GitHub Actions logo, and
pill badges show a mark for the state instead.

| State     | The run                   | Devin                                                                                | Pill                                                                                            |
| --------- | ------------------------- | ------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------- |
| Passing   | Succeeded                 | ![Passing](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?state=passing)     | ![Passing](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?state=passing&style=pill)     |
| Failing   | Failed or timed out       | ![Failing](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?state=failing)     | ![Failing](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?state=failing&style=pill)     |
| Running   | Is queued or in progress  | ![Running](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?state=running)     | ![Running](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?state=running&style=pill)     |
| Cancelled | Was cancelled             | ![Cancelled](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?state=cancelled) | ![Cancelled](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?state=cancelled&style=pill) |
| Skipped   | Was skipped or neutral    | ![Skipped](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?state=skipped)     | ![Skipped](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?state=skipped&style=pill)     |
| Unknown   | None yet, or unrecognised | ![Unknown](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?state=unknown)     | ![Unknown](https://badges.hoppou.dev/ci/HoppouDev/badges/rust.yml?state=unknown&style=pill)     |

The pill's running icon turns. It stands still in raster [image formats](/docs/badges/formats) and
for viewers who ask for reduced motion.

## Previewing

`state` shows a state without waiting for a run: `passing`, `failing`, `running`, `cancelled`,
`skipped` or `unknown`. GitHub isn't asked, so the repository and workflow aren't checked either,
only their spelling. The table above is made this way.

```http
GET /ci/HoppouDev/badges/rust.yml?style=pill&state=running
```

## Freshness

Status badges are cached for a minute, so a new run can take that long to show. A repository that
has been renamed returns `400`, so use its new name.
