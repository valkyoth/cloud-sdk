# cloud-sdk 0.53.0 Milestone Notes

Status: internal tagged development milestone.

Release date: pending

Security-Review: PASS
Pentest: DEFERRED TO v0.55.0
Publication: DEFERRED TO v0.55.0

## Overview

v0.53 adds pure provider-neutral drivers for sequenced pagination and bounded
action polling. Core still owns no transport, clock, sleep, executor, allocator,
or retry loop.

This milestone receives the complete local and GitHub release gates and a
normal signed tag. Its changes remain inside the cumulative v0.55 pentest and
crates.io publication range.

## Pager Driver

- Added one `PageStrategy` contract with borrowed response observations.
- Added a non-cloneable `PagerDriver` that admits exactly one request before
  one response and keeps cancellation independent from provider state.
- Implemented the contract for numbered and offset pagination while retaining
  transactional request, item, snapshot, and traversal budgets.
- Kept direct strategy use available and left opaque cursor, marker, and raw
  provider-link ownership unchanged.

## Action Driver

- Replaced combined `PollPolicy` decisions with separate `PollControl` and
  `PollBackoff` contracts.
- Added unconditional observation, per-delay, cumulative-delay, and monotonic
  elapsed limits selected before the first request.
- Added explicit nondecreasing, bounded-reset, and unordered provider progress
  policies plus deterministic capped exponential backoff.
- Kept provider timestamps in a distinct wall-clock telemetry type that cannot
  affect local delay or timeout budgets.
- Redacted generic backoff and provider failure values from Debug output and
  kept their carrier enums non-`Copy` and non-`Clone`.

## Versions

| Crate | Source version | Publication |
| --- | --- | --- |
| `cloud-sdk` | `0.53.0` | deferred to v0.55.0 |
| `cloud-sdk-hetzner` | `0.38.0` | retained; cumulative changes deferred |
| `cloud-sdk-reqwest` | `0.32.3` | retained; dependency accumulation deferred |
| `cloud-sdk-sanitization` | `0.17.0` | unchanged |
| `cloud-sdk-testkit` | `0.28.2` | retained; cumulative changes deferred |

## Documentation

- [`docs/WORKFLOW_DRIVERS.md`](../docs/WORKFLOW_DRIVERS.md)
- [`docs/MIGRATION.md#v0530`](../docs/MIGRATION.md#v0530)
- [`docs/PUBLIC_API_REVIEW.md#v0530`](../docs/PUBLIC_API_REVIEW.md#v0530)
- [`docs/DEPENDENCY_REVIEW.md#v0530`](../docs/DEPENDENCY_REVIEW.md#v0530)

## Release Gate

Tag only after the clean local v0.53 gate plus GitHub CI and CodeQL pass on the
final milestone commit. No v0.53 pentest report or crates.io publication is
created under the ordinary cumulative cadence.
