# cloud-sdk 0.84.0 Milestone Notes

Status: implementation stop; pentest required.

Release date: pending

Security-Review: PENDING
Pentest: PENDING
Publication: DEFERRED TO v0.85.0

## Overview

v0.84 adds both active Hetzner Robot Wake-on-LAN operations. This is an
internal source milestone; no crate is selected for crates.io publication.

## Robot Wake-on-LAN

- Added exact server-number-only capability discovery and packet-send request
  preparation with official Basic-auth scope, methods, quotas, media types,
  operation IDs, and checked response policy.
- Added protected redacted WOL identity with strict canonical IPv4, IPv6
  network, positive server-number, exact-field, and request-association
  validation.
- Independently capped exported success decoding at 16 KiB.
- Required packet sending to use explicit `RobotWolIntent::Send` constructed
  from a 30-second authenticated discovery bound to credential lineage.
- Classified sending as non-idempotent mutation and disabled automatic retry.
- Added request-bound direct/shared mutation permits across blocking,
  Send-async, and local-async execution. Mandatory authorization evidence
  requires a strong plan digest and is revalidated at dispatch.
- Added exact operation-specific `SERVER_NOT_FOUND`, `WOL_NOT_AVAILABLE`, and
  send-only `WOL_FAILED` classification.
- Excluded the deprecated server-IP path alias and locked that absence in the
  source checker.
- Added a bounded immutable source fixture, mutation-resistant checker,
  focused identity/capability/permit tests, and compile-fail associations.

## Versions

| Crate | Published | v0.84 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.80.0` | `0.84.0` | deferred to v0.85.0 |
| `cloud-sdk-hetzner` | `0.43.0` | `0.43.0` | code accumulated, no publication |
| `cloud-sdk-reqwest` | `0.35.1` | `0.35.1` | unchanged |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.3` | `0.30.3` | unchanged |

## Evidence

- [`docs/PUBLIC_API_REVIEW_0.84.0.md`](../docs/PUBLIC_API_REVIEW_0.84.0.md)
- [`docs/DEPENDENCY_REVIEW_0.84.0.md`](../docs/DEPENDENCY_REVIEW_0.84.0.md)
- [`docs/THREAT_MODEL_DELTA_0.84.0.md`](../docs/THREAT_MODEL_DELTA_0.84.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.84.0.md`](../docs/REJECTED_ABSTRACTIONS_0.84.0.md)
- [`docs/MIGRATION_0.84.0.md`](../docs/MIGRATION_0.84.0.md)
- `security/pentest/v0.84.0.md` after the required pentest and retest
