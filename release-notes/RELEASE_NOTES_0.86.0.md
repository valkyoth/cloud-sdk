# cloud-sdk 0.86.0 Release Notes

Status: implementation stop; pentest required.

Release date: pending

Security-Review: PENDING
Pentest: PENDING
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

- [`docs/PUBLIC_API_REVIEW_0.86.0.md`](../docs/PUBLIC_API_REVIEW_0.86.0.md)
- [`docs/DEPENDENCY_REVIEW_0.86.0.md`](../docs/DEPENDENCY_REVIEW_0.86.0.md)
- [`docs/THREAT_MODEL_DELTA_0.86.0.md`](../docs/THREAT_MODEL_DELTA_0.86.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.86.0.md`](../docs/REJECTED_ABSTRACTIONS_0.86.0.md)
- [`docs/MIGRATION_0.86.0.md`](../docs/MIGRATION_0.86.0.md)

## Stop Gate

Run the incremental pentest against the exact implementation commit. After any
finding is fixed and retested, add `security/pentest/v0.86.0.md`, run
`scripts/release_0_86_gate.sh` on the clean evidence commit, and require green
GitHub CI/CodeQL before tagging. Do not publish crates for v0.86.
