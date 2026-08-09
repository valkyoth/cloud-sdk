# cloud-sdk 0.70.0 Release Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-09

Security-Review: PASS
Pentest: PASS
Publication: PENDING

## Overview

v0.70 completes named Hetzner Cloud client workflows and publishes the
cumulative v0.66-v0.70 work. All 139 active Cloud operations now have
source-locked blocking, `Send` async, and local-async client methods while the
default provider graph remains transport-free, runtime-free, and `no_std`.

## Cloud Client Methods

- Added exhaustive generated methods for 55 read-only, 37 mutation, 37
  destructive, and 10 cost-bearing Cloud operations.
- Read methods consume one caller-owned workspace lease, prepare and verify the
  official endpoint, send one attempt, enforce status/content/size policy, and
  return a checked owned response.
- State-changing methods separate cleanup-owning preparation from execution and
  accept only a matching typed plan-confirm permit attempt.
- Preserved blocking, `Send` async, and local-async parity without client-owned
  retries, clocks, executors, queues, storage, or concurrency policy.
- Added a 139-row descriptor registry and generation checks tied to exact
  permit and pagination classifications.
- Added deterministic read and mutation scenarios plus an ignored read-only
  live client probe using the existing credential-isolated smoke harness.
- Clear complete state-changing response-body and response-header storage
  synchronously when Send-async or local-async execution futures are created,
  including futures dropped without their first poll.
- Added core and named-client regressions proving unpolled futures perform no
  transport call, clear both complete buffers, and leave permit authority in
  the conservative reconciliation state.

## Cumulative Checkpoint

- Includes complete certificate and SSH-key response models from v0.66.
- Includes complete Console Storage Box response models from v0.67.
- Includes exact request, response, error, endpoint, retry, permit, quota, and
  pagination bindings for all 208 active pre-Robot operations from v0.68.
- Includes service-typed endpoint-trust construction, checked read-only
  execution, and complete caller-owned workspace profiles from v0.69.
- Keeps the 13 deprecated operations explicitly unavailable.
- Updates the existing optional SHA-256 implementation from `sha2 0.10.9` to
  current `0.11.0`, removing its obsolete duplicate digest stack. The newer
  AWS-LC set remains rejected under the documented FIPS/source-build decision.

## Versions

| Crate | Previous published | v0.70 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.65.0` | `0.70.0` | yes |
| `cloud-sdk-hetzner` | `0.40.0` | `0.41.0` | yes, cumulative code |
| `cloud-sdk-reqwest` | `0.34.0` | `0.34.1` | yes, dependency-only |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | no, unchanged |
| `cloud-sdk-testkit` | `0.30.0` | `0.30.1` | yes, dependency-only |

## Release Evidence

- [`docs/PUBLIC_API_REVIEW_0.70.0.md`](../docs/PUBLIC_API_REVIEW_0.70.0.md)
- [`docs/DEPENDENCY_REVIEW_0.70.0.md`](../docs/DEPENDENCY_REVIEW_0.70.0.md)
- [`docs/THREAT_MODEL_DELTA_0.70.0.md`](../docs/THREAT_MODEL_DELTA_0.70.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.70.0.md`](../docs/REJECTED_ABSTRACTIONS_0.70.0.md)
- [`docs/MIGRATION_0.70.0.md`](../docs/MIGRATION_0.70.0.md)
- [`security/pentest/v0.70.0.md`](../security/pentest/v0.70.0.md)

## Release Gate

The incremental pentest and final retest passed. Run
`scripts/release_0_70_gate.sh` on the clean final evidence commit, then require
green GitHub CI and CodeQL on that unchanged commit before the signed tag and
crates.io publication.
