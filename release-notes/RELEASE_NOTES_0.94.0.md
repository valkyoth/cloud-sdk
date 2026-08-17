# cloud-sdk 0.94.0 Release Notes

Status: implementation stop; incremental pentest required.

Release date: 2026-08-17

Security-Review: PENDING
Pentest: PENDING
Publication: DEFERRED TO v0.95.0

## Overview

v0.94 exposes every active Hetzner Robot operation through a sealed typed
client contract. It continues the v0.91-v0.95 cumulative train and publishes
no crate.

## Robot Clients

- Added `RobotClient<T>`, fixed to the exact official Robot HTTPS endpoint and
  the credential binding captured at construction.
- Added typed operation contracts for all 89 active non-deprecated Robot
  operations, with a public source-locked method inventory.
- Added direct blocking, `Send` async, and local-async execution for all 45
  read-only operations.
- Kept every state-changing operation behind its existing specialized permit
  or a new sealed permit family for the nine server and boot mutations that
  did not already have operation-specific evidence.
- Reused request-bound checked decoders and cleanup-owning response guards.
  The client never returns an unchecked success payload.
- Added one shared credential-attempt lifecycle. An exact or malformed `401`
  closes the current generation after one wire attempt; later calls fail
  before transport use until credentials are replaced or explicitly
  reconfirmed.
- Revalidated queued attempts immediately before dispatch so a concurrent
  rejection cannot authorize a second request from a stale generation.
- Preserved exact unexpected HTTP status codes across blocking, `Send` async,
  and local-async neutral execution.
- Added exhaustive compile-time operation coverage plus deterministic endpoint,
  response, lockout, reconfirmation, cancellation, and concurrency tests.
- Added no dependency, default feature, unsafe code, runtime, custom endpoint,
  automatic retry, or implicit multi-request workflow.

Robot exposes no Cloud-style paginated collections or asynchronous action
resources. Bounded lists and transactions therefore remain one typed request;
billable-order reconciliation remains the explicit v0.93 workflow rather than
an invented pager or action abstraction.

## Versions

| Crate | Published | v0.94 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.90.0` | `0.94.0` | deferred |
| `cloud-sdk-hetzner` | `0.45.0` | `0.45.0` | deferred |
| `cloud-sdk-reqwest` | `0.35.3` | `0.35.3` | unchanged |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.5` | `0.30.5` | unchanged |

## Evidence

- [`docs/PUBLIC_API_REVIEW_0.94.0.md`](../docs/PUBLIC_API_REVIEW_0.94.0.md)
- [`docs/DEPENDENCY_REVIEW_0.94.0.md`](../docs/DEPENDENCY_REVIEW_0.94.0.md)
- [`docs/THREAT_MODEL_DELTA_0.94.0.md`](../docs/THREAT_MODEL_DELTA_0.94.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.94.0.md`](../docs/REJECTED_ABSTRACTIONS_0.94.0.md)
- [`docs/MIGRATION_0.94.0.md`](../docs/MIGRATION_0.94.0.md)

## Stop Gate

Run the incremental pentest for the exact implementation commit. After its
findings and retest are resolved, record the report, run
`scripts/release_0_94_gate.sh`, and require green GitHub CI and CodeQL on the
unchanged evidence commit before tagging. Do not publish crates for v0.94.
