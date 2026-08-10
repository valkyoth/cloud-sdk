# cloud-sdk 0.73.0 Milestone Notes

Status: release candidate; pentest passed with no findings.

Release date: 2026-08-10

Security-Review: PASS
Pentest: PASS
Publication: DEFERRED TO v0.75.0

## Overview

v0.73 completes named Hetzner Console Storage client workflows for all 31
active operations. This is an internal milestone; no crate is selected for
crates.io publication.

## Storage Client Methods

- Added generated named methods for 12 read-only, nine mutation, eight
  destructive, and two cost-authorized Storage operations.
- Preserved blocking, `Send` async, and local-async parity, four numbered list
  policies, checked boxes/types/snapshots/folders/subaccounts/actions, and
  exact Storage service identity.
- Kept state-changing requests behind cleanup-owning preparation and bound
  plan-confirm permits, with no client-owned retries, rollback, or authority.
- Added digest-only password reset planning, redaction, guarded preparation,
  permit-authorized execution, and unpolled cancellation cleanup evidence.
- Added a 32-item nested response larger than 32 KiB through every executor to
  exercise bounded incremental decoding, pagination, and quota retention.
- Added a compile-checked named Storage example and moved opt-in read-only live
  Storage smoke coverage onto the named official client methods.
- Updated capability documentation to claim named end-to-end workflows for all
  208 active pre-Robot operations while keeping custom execution unavailable.

## Versions

| Crate | Published | v0.73 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.70.0` | `0.73.0` | deferred to v0.75.0 |
| `cloud-sdk-hetzner` | `0.41.0` | `0.41.0` | code accumulated, no publication |
| `cloud-sdk-reqwest` | `0.34.1` | `0.34.1` | unchanged |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.1` | `0.30.1` | unchanged |

## Release Evidence

- [`docs/PUBLIC_API_REVIEW_0.73.0.md`](../docs/PUBLIC_API_REVIEW_0.73.0.md)
- [`docs/DEPENDENCY_REVIEW_0.73.0.md`](../docs/DEPENDENCY_REVIEW_0.73.0.md)
- [`docs/THREAT_MODEL_DELTA_0.73.0.md`](../docs/THREAT_MODEL_DELTA_0.73.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.73.0.md`](../docs/REJECTED_ABSTRACTIONS_0.73.0.md)
- [`docs/MIGRATION_0.73.0.md`](../docs/MIGRATION_0.73.0.md)
- [`security/pentest/v0.73.0.md`](../security/pentest/v0.73.0.md)

## Release Gate

Run `scripts/release_0_73_gate.sh` on the clean final evidence commit after the
incremental pentest and final retest. GitHub CI and CodeQL must be green on that
unchanged commit before the signed internal tag. Do not publish crates.
