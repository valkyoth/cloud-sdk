# cloud-sdk 0.81.0 Milestone Notes

Status: implementation stop reached; pentest required.

Release date: pending

Security-Review: PENDING
Pentest: PENDING
Publication: DEFERRED TO v0.85.0

## Overview

v0.81 adds all six active Hetzner Robot subnet and subnet-MAC operations. This
is an internal source milestone; no crate is selected for crates.io
publication.

## Robot Subnets

- Added list, detail, traffic update, MAC read, explicit MAC assignment, and
  default-MAC restoration requests.
- Added exact endpoint, Basic scope, operation ID, metadata, content type,
  response size, and automatic-retry policies.
- Added bounded strict subnet models with nullable assignment, failover/lock
  state, traffic policy, family-valid masks, and same-network gateways.
- Preserved documented host-bits-set route identities while exposing derived
  mathematical network and IPv4 broadcast values.
- Added a nonempty bounded canonical IP-to-MAC choice map and exact selected
  MAC acknowledgement.
- Made default-MAC restoration constructible only from consumed checked subnet
  and MAC snapshots, bounded observation timestamps, and a same-resource
  external-lock lease; success must return the assigned server's mapped MAC.
- Bound server, MAC, freshness, and lock generation into digest-only
  authorization evidence and reject stale evidence at permit entry.
- Made traffic policy/update aggregates non-copyable, redacted, and
  drop-cleared; validated late preparation policy before writing caller storage.
- Added request-associated decoding for every documented subnet failure,
  including operation-specific `404` and `500` codes.
- Added request-bound direct/shared permits across blocking, Send-async, and
  local-async execution plus cleanup and redaction tests.
- Added an exact source fixture/checker and direct checked-response fuzz target.
- Hardened the source checker to compare every operation field, status/code
  pair, exact quota, response field, source inconsistency, and security policy,
  with mutation tests and compiled Rust contract enforcement.

## Reviewed Source Inconsistencies

The official list example admits `server_ip: null` while the table says
string. Detail masks are integers while MAC masks are decimal strings. Official
subnet route examples may have host bits set. These are narrowly source-locked
rather than normalized away.

## Versions

| Crate | Published | v0.81 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.80.0` | `0.81.0` | deferred to v0.85.0 |
| `cloud-sdk-hetzner` | `0.43.0` | `0.43.0` | code accumulated, no publication |
| `cloud-sdk-reqwest` | `0.35.1` | `0.35.1` | unchanged |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.3` | `0.30.3` | unchanged |

## Release Evidence

- [`docs/PUBLIC_API_REVIEW_0.81.0.md`](../docs/PUBLIC_API_REVIEW_0.81.0.md)
- [`docs/DEPENDENCY_REVIEW_0.81.0.md`](../docs/DEPENDENCY_REVIEW_0.81.0.md)
- [`docs/THREAT_MODEL_DELTA_0.81.0.md`](../docs/THREAT_MODEL_DELTA_0.81.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.81.0.md`](../docs/REJECTED_ABSTRACTIONS_0.81.0.md)
- [`docs/MIGRATION_0.81.0.md`](../docs/MIGRATION_0.81.0.md)

## Stop Gate

`v0.81.0 implementation stop reached. Complete the pentest and full release
gate for this exact commit; defer crates.io publication to v0.85.0.`
