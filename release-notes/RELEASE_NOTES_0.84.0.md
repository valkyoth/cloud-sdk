# cloud-sdk 0.84.0 Milestone Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-13

Security-Review: PASS
Pentest: PASS
Publication: DEFERRED TO v0.85.0

## Overview

v0.84 adds both active Hetzner Robot Wake-on-LAN operations. This is an
internal source milestone; no crate is selected for crates.io publication.

## Robot Wake-on-LAN

- Added exact server-number-only capability discovery and packet-send request
  preparation with official Basic-auth scope, methods, quotas, media types,
  operation IDs, and checked response policy.
- Exposed the source-locked 500 discovery/hour and 10 send/hour allowances on
  their request types for caller-owned account and credential limiters.
- Added protected redacted WOL identity with strict canonical IPv4, IPv6
  network, positive server-number, exact-field, and request-association
  validation. Send acknowledgements must preserve all three identity fields
  from authenticated discovery, not only the server number.
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

- [`docs/PUBLIC_API_REVIEW.md#v0840`](../docs/PUBLIC_API_REVIEW.md#v0840)
- [`docs/DEPENDENCY_REVIEW.md#v0840`](../docs/DEPENDENCY_REVIEW.md#v0840)
- [`docs/THREAT_MODEL_DELTA.md#v0840`](../docs/THREAT_MODEL_DELTA.md#v0840)
- [`docs/REJECTED_ABSTRACTIONS.md#v0840`](../docs/REJECTED_ABSTRACTIONS.md#v0840)
- [`docs/MIGRATION.md#v0840`](../docs/MIGRATION.md#v0840)
- [`security/pentest/v0.84.0.md`](../security/pentest/v0.84.0.md)
