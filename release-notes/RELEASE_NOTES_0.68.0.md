# cloud-sdk 0.68.0 Milestone Notes

Status: implementation complete; pentest required.

Release date: pending

Security-Review: PENDING
Pentest: PENDING
Publication: DEFERRED TO v0.70.0

## Overview

v0.68 proves complete typed request, response, error, and policy associations
for all 208 active pre-Robot Hetzner operations. It is an internal milestone
and publishes no crate.

## Complete Binding Evidence

- Added a canonical 28-column generated operation manifest covering exact API
  path, endpoint, authentication, request, response, bounds, workflow, permit,
  and response-identity policy.
- Cross-checked all 28 columns in all 208 rows against compiled descriptors and
  associated marker labels, plus independent fingerprint, association,
  request-body, response, provider-authentication, Markdown matrix, generated
  Rust marker, Rust AST, and compiled descriptor evidence.
- Source-locked path templates and exact/parent/none response-identity classes
  in executable descriptors. Written paths now fail closed against the template
  and `RequestPath`, reject raw or encoded query/fragment delimiters, and clear
  complete request storage on mismatch.
- Required the exact 91 JSON request-body operations while source-locking 12
  shared enum variants that reject bodies under typed policy.
- Explicitly excluded all 13 deprecated operations from executable endpoint,
  body, response, and marker registries.
- Added regression tests for schema truncation, representative cross-service
  policy rows, deprecated exclusion, fail-closed body variants, optimized
  Python rejection, identity classes, and wrong executable paths.
- Added compile-fail coverage for cross-operation body assembly. Existing
  compile-fail tests cover query, response, and permit mismatches.

## Versions

| Crate | Source version | Cumulative change | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.68.0` | metadata | deferred to v0.70.0 |
| `cloud-sdk-hetzner` | `0.40.0` | code | deferred |
| `cloud-sdk-reqwest` | `0.34.0` | unchanged | no |
| `cloud-sdk-sanitization` | `0.18.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.0` | unchanged | no |

## Release Evidence

- [`docs/PUBLIC_API_REVIEW_0.68.0.md`](../docs/PUBLIC_API_REVIEW_0.68.0.md)
- [`docs/DEPENDENCY_REVIEW_0.68.0.md`](../docs/DEPENDENCY_REVIEW_0.68.0.md)
- [`docs/THREAT_MODEL_DELTA_0.68.0.md`](../docs/THREAT_MODEL_DELTA_0.68.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.68.0.md`](../docs/REJECTED_ABSTRACTIONS_0.68.0.md)
- [`docs/MIGRATION_0.68.0.md`](../docs/MIGRATION_0.68.0.md)

## Release Gate

Run `scripts/release_0_68_gate.sh` on the clean final evidence commit after the
incremental pentest and retest. GitHub CI and CodeQL must be green on that
unchanged commit before the signed internal tag. Do not publish crates.
