# cloud-sdk 0.78.0 Milestone Notes

Status: implementation complete; pentest required.

Release date: 2026-08-11

Security-Review: PENDING
Pentest: REQUIRED
Publication: DEFERRED TO v0.80.0

## Overview

v0.78 implements the first Hetzner Robot endpoint family: server list, get,
and rename. This is an internal milestone; no crate is selected for crates.io
publication.

## Robot Servers

- Added canonical positive server-number identity with no deprecated IP alias.
- Added `alloc`-gated preparation for list, get, and explicit rename intent,
  bound to the official Robot origin, Basic scope, operation metadata, form
  media type, and checked response policy. Preparation does not allocate after
  fallible stable identity construction.
- Added bounded summaries and details with capability flags, finite status,
  calendar-valid dates, canonical addresses/subnets, nullable subnets, and
  linked Storage Box identity.
- Added strict checked decoding with exact fields, duplicate rejection,
  response identity binding, protected text, and cleanup-owning request decode
  methods.
- Added non-`Copy`, stable-allocation-backed owners for server identities,
  topology, billing dates, states, cancellation flags, and capabilities, with
  static redacted diagnostics and closure-scoped inspection. Owner moves no
  longer relocate classified bytes.
- Removed ordinary retained scalar payloads from strict JSON numbers and
  Booleans. Robot identities now remain canonical protected decimal bytes,
  request paths copy those bytes directly, and address/subnet/date decoding
  uses bounded clear-on-drop scratch before stable protected ownership.
- Replaced quadratic duplicate scans with sorted public index scratch and
  in-place protected comparisons, eliminating copied identity arrays, and
  added a direct checked-response Robot server fuzz target with exact-bound
  deterministic seeds.
- Added source-contract and regression checks for all three operations, fields,
  statuses, nullability, update input, and deprecated aliases.
- Added provider-neutral form media-type constants without changing the
  default dependency graph.

## Versions

| Crate | Published | v0.78 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.75.0` | `0.78.0` | deferred to v0.80.0 |
| `cloud-sdk-hetzner` | `0.42.0` | `0.42.0` | code accumulated, no publication |
| `cloud-sdk-reqwest` | `0.35.0` | `0.35.0` | unchanged |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | code accumulated, no publication |
| `cloud-sdk-testkit` | `0.30.2` | `0.30.2` | unchanged |

## Release Evidence

- [`docs/PUBLIC_API_REVIEW_0.78.0.md`](../docs/PUBLIC_API_REVIEW_0.78.0.md)
- [`docs/DEPENDENCY_REVIEW_0.78.0.md`](../docs/DEPENDENCY_REVIEW_0.78.0.md)
- [`docs/THREAT_MODEL_DELTA_0.78.0.md`](../docs/THREAT_MODEL_DELTA_0.78.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.78.0.md`](../docs/REJECTED_ABSTRACTIONS_0.78.0.md)
- [`docs/MIGRATION_0.78.0.md`](../docs/MIGRATION_0.78.0.md)

## Release Gate

Run `scripts/release_0_78_gate.sh` only after the pentest report is committed.
GitHub CI and CodeQL must be green on the unchanged final evidence commit
before the signed internal tag. Do not publish crates.
