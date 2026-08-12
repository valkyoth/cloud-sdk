# cloud-sdk 0.83.0 Milestone Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-12

Security-Review: PASS
Pentest: PASS
Publication: DEFERRED TO v0.85.0

## Overview

v0.83 adds all four active Hetzner Robot failover operations. This is an
internal source milestone; no crate is selected for crates.io publication.

## Robot Failover

- Added exact failover list, detail, reroute, and active-route delete request
  preparation.
- Added protected canonical route, owner, and destination addresses with
  redacted diagnostics.
- Added strict bounded response models with contiguous-mask, host-bit,
  address-family, duplicate-route, and exact-field validation.
- Enforced the 2 MiB list and 16 KiB item limits again inside the exported
  decoders, independently of the generic checked-response provenance.
- Added exact request association and mutation acknowledgement checks:
  reroute must echo its destination and deletion must return a JSON failover
  object with `active_server_ip: null`.
- Classified reroute as a sensitive non-idempotent mutation and deletion as a
  non-idempotent destructive operation. Neither is automatically retried.
- Added request-bound direct/shared mutation and destructive permits across
  blocking, Send-async, and local-async execution. Sensitive reroute plans
  require a strong digest.
- Added exact operation-specific `404`, `409`, and `500` provider failure
  classification, plus reroute-only invalid-input admission.
- Added a bounded immutable source fixture, mutation-resistant checker,
  deterministic contract tests, and four-path response fuzzing. Exact
  minus-one, exact, and plus-one list-body boundaries are deterministic fuzz
  corpus paths, and smoke fuzzing admits the full selector-plus-body size.

## Reviewed Source Inconsistency

The official field table describes `active_server_ip` as a string, while the
official DELETE example returns `null`. Deletion therefore requires that exact
JSON null acknowledgement. The earlier roadmap phrase `no-content policy` was
incorrect; `204` and empty-body success remain rejected.

## Versions

| Crate | Published | v0.83 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.80.0` | `0.83.0` | deferred to v0.85.0 |
| `cloud-sdk-hetzner` | `0.43.0` | `0.43.0` | code accumulated, no publication |
| `cloud-sdk-reqwest` | `0.35.1` | `0.35.1` | unchanged |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.3` | `0.30.3` | unchanged |

## Evidence

- [`docs/PUBLIC_API_REVIEW_0.83.0.md`](../docs/PUBLIC_API_REVIEW_0.83.0.md)
- [`docs/DEPENDENCY_REVIEW_0.83.0.md`](../docs/DEPENDENCY_REVIEW_0.83.0.md)
- [`docs/THREAT_MODEL_DELTA_0.83.0.md`](../docs/THREAT_MODEL_DELTA_0.83.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.83.0.md`](../docs/REJECTED_ABSTRACTIONS_0.83.0.md)
- [`docs/MIGRATION_0.83.0.md`](../docs/MIGRATION_0.83.0.md)
- [`security/pentest/v0.83.0.md`](../security/pentest/v0.83.0.md)
