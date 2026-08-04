# cloud-sdk 0.54.0 Milestone Notes

Status: internal tagged development milestone.

Release date: pending

Security-Review: PASS
Pentest: DEFERRED TO v0.55.0
Publication: DEFERRED TO v0.55.0

## Overview

v0.54 adds allocation-free structured lifecycle diagnostics to the
provider-neutral client kernel. Observation remains explicit and disabled by
ordinary execution methods. Core never logs or retains telemetry.

This milestone receives the complete local and GitHub release gates and a
normal signed tag. Its changes remain inside the cumulative v0.55 pentest and
crates.io publication range.

## Diagnostic Boundary

- Added finite preparation, authorization, endpoint, transport, response, and
  decode error categories.
- Added bounded provider, service, operation, impact, retry, status, and
  request-ID disposition context.
- Excluded credentials, request targets, headers, bodies, messages, cursors,
  generic errors, resource IDs, and request-ID bytes from events.
- Kept discarded request-ID presence indistinguishable.

## Observation

- Added observed blocking, cross-thread async, and local async client methods.
- Kept ordinary methods on a zero-state no-op observer.
- Ignored observer return errors without imposing formatting bounds or changing
  client results.
- Added failure-sequence, cross-mode, cleanup, reentrancy, maximum-bound,
  downstream-error, and redaction snapshot tests.

## Versions

| Crate | Source version | Publication |
| --- | --- | --- |
| `cloud-sdk` | `0.54.0` | deferred to v0.55.0 |
| `cloud-sdk-hetzner` | `0.38.0` | retained; cumulative changes deferred |
| `cloud-sdk-reqwest` | `0.32.3` | retained; dependency accumulation deferred |
| `cloud-sdk-sanitization` | `0.17.0` | unchanged |
| `cloud-sdk-testkit` | `0.28.2` | retained; cumulative changes deferred |

## Documentation

- [`docs/DIAGNOSTICS.md`](../docs/DIAGNOSTICS.md)
- [`docs/MIGRATION_0.54.0.md`](../docs/MIGRATION_0.54.0.md)
- [`docs/PUBLIC_API_REVIEW_0.54.0.md`](../docs/PUBLIC_API_REVIEW_0.54.0.md)
- [`docs/DEPENDENCY_REVIEW_0.54.0.md`](../docs/DEPENDENCY_REVIEW_0.54.0.md)

## Release Gate

Tag only after the clean local v0.54 gate plus GitHub CI and CodeQL pass on the
final milestone commit. No v0.54 pentest report or crates.io publication is
created under the ordinary cumulative cadence.
