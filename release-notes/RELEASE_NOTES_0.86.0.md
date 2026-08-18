# cloud-sdk 0.86.0 Release Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-13

Security-Review: PASS
Pentest: PASS
Publication: DEFERRED TO v0.90.0

## Overview

v0.86 implements all five active Hetzner Robot reverse-DNS operations and
begins the v0.86-v0.90 cumulative train. This internal milestone will be tagged
after its incremental pentest and green CI/CodeQL but publishes no crate.

## Robot Reverse DNS

- Added exact list, filtered-list, get, set, update-or-create, and delete
  requests bound to official Robot operation metadata and quotas.
- Added bounded canonical lowercase PTR names and reused protected canonical
  IPv4/IPv6 identities.
- Added atomic optional-query and sensitive-form preparation with cleanup on
  failure.
- Added exact request-bound mutation/destructive permits and denied automatic
  retry for every non-idempotent mutation.
- Added bounded strict models and decoders that reject duplicate list
  identities and bind mutation acknowledgements to the requested IP and PTR.
- Kept raw decoders internal and made filtered-list decoding require checked IP
  inventory because the provider response omits its server association.
- Rejected unverifiable empty filtered responses, returned non-empty matches as
  a distinct membership-only type, and bounded verification through a sorted
  assignment index tested at the 4,096-by-4,096 input boundary and at most 13
  comparisons per lookup.
- Restricted the Python checker to immutable source-contract validation;
  executable Rust tests own semantic and complexity assurance.
- Required the documented empty `200` delete response and narrowed every
  source-locked provider failure by operation and status.
- Added an immutable five-operation source fixture, mutation-resistant checker,
  direct/shared permit tests, compile-fail provenance, deterministic seeds, and
  a bounded fuzz target.

## Versions

| Crate | Published | v0.86 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.85.0` | `0.86.0` | deferred |
| `cloud-sdk-hetzner` | `0.44.0` | `0.44.0` | deferred |
| `cloud-sdk-reqwest` | `0.35.2` | `0.35.2` | unchanged |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.4` | `0.30.4` | unchanged |

## Evidence

- [`docs/PUBLIC_API_REVIEW.md#v0860`](../docs/PUBLIC_API_REVIEW.md#v0860)
- [`docs/DEPENDENCY_REVIEW.md#v0860`](../docs/DEPENDENCY_REVIEW.md#v0860)
- [`docs/THREAT_MODEL_DELTA.md#v0860`](../docs/THREAT_MODEL_DELTA.md#v0860)
- [`docs/REJECTED_ABSTRACTIONS.md#v0860`](../docs/REJECTED_ABSTRACTIONS.md#v0860)
- [`docs/MIGRATION.md#v0860`](../docs/MIGRATION.md#v0860)
- [`security/pentest/v0.86.0.md`](../security/pentest/v0.86.0.md)

## Stop Gate

The incremental pentest and final retest passed. Run
`scripts/release_0_86_gate.sh` on the clean evidence commit and require green
GitHub CI/CodeQL on that unchanged commit before tagging. Do not publish crates
for v0.86.
