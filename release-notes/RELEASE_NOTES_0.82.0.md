# cloud-sdk 0.82.0 Milestone Notes

Status: implementation stop; pentest required.

Release date: pending

Security-Review: PENDING
Pentest: PENDING
Publication: DEFERRED TO v0.85.0

## Overview

v0.82 adds all three active Hetzner Robot reset operations. This is an
internal source milestone; no crate is selected for crates.io publication.

## Robot Reset

- Added exact reset list, detail, and execute request preparation.
- Added finite software, hardware, power, long-power, and manual capabilities.
- Added bounded protected reset summary, detail, list, action, and operating
  status models with strict duplicate and identity rejection.
- Made execute construction require checked detail and an advertised type.
- Classified execution as sensitive, destructive, non-idempotent, and never
  automatically retryable.
- Restricted plan construction to execute requests and required request-bound
  strong-digest direct/shared destructive permits.
- Preserved exact association across blocking, Send-async, and local-async
  execution.
- Bound action success to checked IPv4, IPv6 network, optional server number,
  and exact selected type.
- Source-locked every operation, field, quota, failure pair, finite type,
  reviewed source inconsistency, and security policy.
- Added mutation-resistant source checks and direct list/detail/action response
  fuzzing.

## Reviewed Source Inconsistency

The official POST output table lists `server_number`, while its response
example omits it. Action decoding therefore permits only this field to be
absent and validates it against checked state whenever present.

## Versions

| Crate | Published | v0.82 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.80.0` | `0.82.0` | deferred to v0.85.0 |
| `cloud-sdk-hetzner` | `0.43.0` | `0.43.0` | code accumulated, no publication |
| `cloud-sdk-reqwest` | `0.35.1` | `0.35.1` | unchanged |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.3` | `0.30.3` | unchanged |

## Evidence

- [`docs/PUBLIC_API_REVIEW_0.82.0.md`](../docs/PUBLIC_API_REVIEW_0.82.0.md)
- [`docs/DEPENDENCY_REVIEW_0.82.0.md`](../docs/DEPENDENCY_REVIEW_0.82.0.md)
- [`docs/THREAT_MODEL_DELTA_0.82.0.md`](../docs/THREAT_MODEL_DELTA_0.82.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.82.0.md`](../docs/REJECTED_ABSTRACTIONS_0.82.0.md)
- [`docs/MIGRATION_0.82.0.md`](../docs/MIGRATION_0.82.0.md)
- Pentest report pending at `security/pentest/v0.82.0.md`.
