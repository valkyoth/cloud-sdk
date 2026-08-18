# cloud-sdk 0.81.0 Milestone Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-12

Security-Review: PASS
Pentest: PASS
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
  authorization evidence, reject permit validity beyond that evidence, and
  recheck expiry with the generic clock sample immediately before dispatch.
- Installed evidence scratch/output cleanup guards before encoding, algorithm
  selection, and digest callbacks, including unwind-enabled panic paths.
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

- [`docs/PUBLIC_API_REVIEW.md#v0810`](../docs/PUBLIC_API_REVIEW.md#v0810)
- [`docs/DEPENDENCY_REVIEW.md#v0810`](../docs/DEPENDENCY_REVIEW.md#v0810)
- [`docs/THREAT_MODEL_DELTA.md#v0810`](../docs/THREAT_MODEL_DELTA.md#v0810)
- [`docs/REJECTED_ABSTRACTIONS.md#v0810`](../docs/REJECTED_ABSTRACTIONS.md#v0810)
- [`docs/MIGRATION.md#v0810`](../docs/MIGRATION.md#v0810)
- [`security/pentest/v0.81.0.md`](../security/pentest/v0.81.0.md)

## Release Gate

Run `scripts/release_0_81_gate.sh` on the clean final evidence commit. GitHub
CI and CodeQL must be green on that unchanged commit before signing the
internal tag. Do not publish crates for this milestone.
