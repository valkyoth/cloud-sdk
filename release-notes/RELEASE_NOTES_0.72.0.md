# cloud-sdk 0.72.0 Milestone Notes

Status: implementation stop reached; pentest and retest required before tagging.

Release date: 2026-08-10

Security-Review: PENDING
Pentest: PENDING
Publication: DEFERRED TO v0.75.0

## Overview

v0.72 completes named Hetzner Security client workflows for all 14 active
certificate and SSH-key operations. This is an internal milestone; no crate is
selected for crates.io publication.

## Security Client Methods

- Added generated named methods for seven read-only, five mutation, and two
  destructive Security operations.
- Preserved blocking, `Send` async, and local-async parity, four numbered list
  policies, checked actions/resources, and exact Security service identity.
- Kept state-changing requests behind cleanup-owning preparation and exact
  plan-confirm permits, with no client-owned retries, rollback, or authority.
- Added paginated read execution, uploaded private-key redaction and cleanup,
  permit-authorized create execution, unpolled cancellation cleanup, a
  compile-checked example, and named read-only live smoke paths.
- Documented SSH-key rotation as create, verify, and separately authorize old-
  key deletion rather than an unsafe synthetic atomic workflow.

## Versions

| Crate | Published | v0.72 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.70.0` | `0.72.0` | deferred to v0.75.0 |
| `cloud-sdk-hetzner` | `0.41.0` | `0.41.0` | code accumulated, no publication |
| `cloud-sdk-reqwest` | `0.34.1` | `0.34.1` | unchanged |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.1` | `0.30.1` | unchanged |

## Release Evidence

- [`docs/PUBLIC_API_REVIEW_0.72.0.md`](../docs/PUBLIC_API_REVIEW_0.72.0.md)
- [`docs/DEPENDENCY_REVIEW_0.72.0.md`](../docs/DEPENDENCY_REVIEW_0.72.0.md)
- [`docs/THREAT_MODEL_DELTA_0.72.0.md`](../docs/THREAT_MODEL_DELTA_0.72.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.72.0.md`](../docs/REJECTED_ABSTRACTIONS_0.72.0.md)
- [`docs/MIGRATION_0.72.0.md`](../docs/MIGRATION_0.72.0.md)

## Release Gate

Run the incremental pentest from signed v0.71.0 after this implementation
commit. After remediation and a green retest, add the pentest report, run
`scripts/release_0_72_gate.sh` on the clean final evidence commit, and require
green GitHub CI and CodeQL before the signed internal tag. Do not publish
crates.
