# cloud-sdk 0.93.0 Release Notes

Status: implementation stop; incremental pentest required.

Release date: 2026-08-16

Security-Review: PENDING
Pentest: PENDING
Publication: DEFERRED TO v0.95.0

## Overview

v0.93 implements all three active billable Hetzner Robot order operations and
continues the v0.91-v0.95 cumulative train. This internal milestone will be
tagged only after its incremental pentest and green CI/CodeQL; it publishes no
crate.

## Robot Orders

- Added standard-server, Server Auction, and per-server addon order requests
  with exact source-locked paths, strict JSON `201`, typed provider failures,
  and the shared 20-request daily quota.
- Derived executable requests exclusively from current typed catalog plans.
- Added exact scale-4 gross recurring-plus-setup cost aggregation, checked
  addon quantities, caller ceilings, and required account scope.
- Added digest-only sensitive-body fingerprints covering request, cost,
  endpoint, account, expiry, replay, budget, and reconciliation identity.
- Added direct non-cloneable cost permits and delivery-aware attempt handling.
  `NotSent` recovery is separate from mandatory uncertain-send transaction
  reconciliation.
- Added strict request-bound success decoding, bounded guarded form encoding,
  provider-source fixtures, mutation-resistant contract checks, and static CI
  purchase-route rejection.
- Added no dependency, default feature, unsafe code, runtime, transport,
  custom endpoint, automatic retry, or high-level Robot client.

## Versions

| Crate | Published | v0.93 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.90.0` | `0.93.0` | deferred |
| `cloud-sdk-hetzner` | `0.45.0` | `0.45.0` | deferred |
| `cloud-sdk-reqwest` | `0.35.3` | `0.35.3` | unchanged |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.5` | `0.30.5` | unchanged |

## Evidence

- [`docs/PUBLIC_API_REVIEW_0.93.0.md`](../docs/PUBLIC_API_REVIEW_0.93.0.md)
- [`docs/DEPENDENCY_REVIEW_0.93.0.md`](../docs/DEPENDENCY_REVIEW_0.93.0.md)
- [`docs/THREAT_MODEL_DELTA_0.93.0.md`](../docs/THREAT_MODEL_DELTA_0.93.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.93.0.md`](../docs/REJECTED_ABSTRACTIONS_0.93.0.md)
- [`docs/MIGRATION_0.93.0.md`](../docs/MIGRATION_0.93.0.md)

## Stop Gate

Run the incremental pentest against the exact implementation commit. After
remediation and retest, add permanent pentest evidence, run
`scripts/release_0_93_gate.sh`, and require green GitHub CI and CodeQL on the
unchanged evidence commit before tagging. Do not publish crates for v0.93.
