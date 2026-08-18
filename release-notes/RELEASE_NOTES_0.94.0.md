# cloud-sdk 0.94.0 Release Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-17

Security-Review: PASS
Pentest: PASS
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
  closes the current generation after one wire attempt; later calls fail before
  transport use until credentials are replaced or explicitly reconfirmed.
- Added atomic exclusive dispatch admission per credential generation. A second
  concurrent call fails before transport access, and an unclassified guard drop
  or cancellation closes the generation.
- Clarified that dispatch serialization and rejection state are scoped to one
  `RobotClient`; independently constructed clients or processes using the same
  credential require a shared client or external credential-keyed coordinator.
- Preserved final HTTP status on later transport response-processing failures.
  An observed `401`, or any indeterminate post-dispatch failure, closes Robot
  credential reuse instead of permitting another lockout attempt.
- Preserved exact unexpected HTTP status codes across blocking, `Send` async,
  and local-async neutral execution.
- Retained the typed official-endpoint verification cause when replacement
  transport validation fails.
- Added exhaustive compile-time operation coverage plus deterministic endpoint,
  response, lockout, reconfirmation, cancellation, and concurrency tests.
- Added no dependency edge, default feature, unsafe code, runtime, custom
  endpoint, automatic retry, or implicit multi-request workflow.
- Updated the admitted optional transport graph to `aws-lc-rs 1.18.0`,
  `aws-lc-sys 0.44.0`, and `http-body-util 0.1.5`, refreshed compatible locked
  transitives, and retained explicit exclusion of every FIPS package.
- Updated the isolated fuzz compiler to `nightly-2026-08-17`; Rust 1.97.1,
  `actions/checkout v7.0.1`, and all pinned Cargo security tools remain current.

Robot exposes no Cloud-style paginated collections or asynchronous action
resources. Bounded lists and transactions therefore remain one typed request;
billable-order reconciliation remains the explicit v0.93 workflow rather than
an invented pager or action abstraction.

## Versions

| Crate | Published | v0.94 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.90.0` | `0.94.0` | deferred |
| `cloud-sdk-hetzner` | `0.45.0` | `0.45.0` | deferred |
| `cloud-sdk-reqwest` | `0.35.3` | `0.35.3` | code; deferred |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.5` | `0.30.5` | code; deferred |

## Evidence

- [`docs/PUBLIC_API_REVIEW.md#v0940`](../docs/PUBLIC_API_REVIEW.md#v0940)
- [`docs/DEPENDENCY_REVIEW.md#v0940`](../docs/DEPENDENCY_REVIEW.md#v0940)
- [`docs/THREAT_MODEL_DELTA.md#v0940`](../docs/THREAT_MODEL_DELTA.md#v0940)
- [`docs/REJECTED_ABSTRACTIONS.md#v0940`](../docs/REJECTED_ABSTRACTIONS.md#v0940)
- [`docs/MIGRATION.md#v0940`](../docs/MIGRATION.md#v0940)
- [`security/pentest/v0.94.0.md`](../security/pentest/v0.94.0.md)

## Stop Gate

The incremental pentest and final remediation retest are green. Run
`scripts/release_0_94_gate.sh`, then require green GitHub CI and CodeQL on the
unchanged evidence commit before tagging. Do not publish crates for v0.94.
