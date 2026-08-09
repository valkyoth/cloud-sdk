# cloud-sdk 0.67.0 Milestone Notes

Status: release candidate; pentest and retest passed.

Release date: 2026-08-09

Security-Review: PASS
Pentest: PASS
Publication: DEFERRED TO v0.70.0

## Overview

v0.67 completes source-derived Hetzner Console Storage Box response models. It
is an internal milestone and publishes no crate. The provider package remains
at 0.40.0 while changes accumulate for v0.70.0.

## Console Models

- Added dedicated box, type, snapshot, subaccount, statistics, access,
  pagination, and partial create-reference models.
- Routed list, singleton, update, and create-composite operations through
  source-specific `HetznerSuccess` and `StorageBoxResource` variants.
- Replaced public dynamic fields with read-only accessors, cleanup-owning text,
  canonical UTC timestamps, and redacted aggregate diagnostics.
- Enforced status/nullability coherence, source integer and text bounds,
  snapshot and home-directory character policy, decimal syntax, collection
  limits, and page-size consistency.

## Security And Verification

- Extended deterministic model evidence from 595 to 718 field-contract rows by
  combining exact pinned Cloud and Console specifications.
- Corrected source normalization so legitimate API fields named `description`
  are retained instead of being confused with OpenAPI documentation keywords.
- Added singleton/page/composite routing, late-failure, nullability, timestamp,
  exact-list-bound, and cross-chunk large-response tests.
- Added named fuzz seeds and an ignored credential-gated read-only live smoke
  for boxes and types that does not require owned storage inventory.
- Added `scripts/check_storage_response_models.sh` to the ordinary and final
  release gates. No dependency or default-feature boundary changed.
- Redacted Console resource identifiers in direct, page, composite, and
  `HetznerSuccess` diagnostics and removed structural equality from dynamic
  Console aggregates.
- Bound typed singleton, parent-scoped list, and create-reference responses to
  identifiers captured directly from endpoint values; cross-resource replay
  now fails with a payload-free model error.
- Carried those identities through exact and strong-digest plan confirmation,
  direct/shared mutation, destructive, and cost permits, and all three
  execution modes. Authorized typed execution now returns an associated
  checked response instead of erasing provider provenance.
- Removed the endpoint identity-policy default. Every endpoint adapter must
  declare its policy, AST coverage rejects omissions, and source-locked tests
  enumerate all current ID-bearing Storage Box variants.
- Sanitized invalid owned timestamp allocations before rejection and made the
  Console specification mandatory for the model-generator CLI.

## Versions

| Crate | Source version | Cumulative change | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.67.0` | metadata | deferred to v0.70.0 |
| `cloud-sdk-hetzner` | `0.40.0` | code | deferred |
| `cloud-sdk-reqwest` | `0.34.0` | unchanged | no |
| `cloud-sdk-sanitization` | `0.18.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.0` | unchanged | no |

## Release Evidence

- [`docs/PUBLIC_API_REVIEW_0.67.0.md`](../docs/PUBLIC_API_REVIEW_0.67.0.md)
- [`docs/DEPENDENCY_REVIEW_0.67.0.md`](../docs/DEPENDENCY_REVIEW_0.67.0.md)
- [`docs/THREAT_MODEL_DELTA_0.67.0.md`](../docs/THREAT_MODEL_DELTA_0.67.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.67.0.md`](../docs/REJECTED_ABSTRACTIONS_0.67.0.md)
- [`docs/MIGRATION_0.67.0.md`](../docs/MIGRATION_0.67.0.md)

## Release Gate

The incremental pentest against signed v0.66.0 and final retest are green. Run
`scripts/release_0_67_gate.sh` on the clean evidence commit. GitHub CI and
CodeQL must be green on that unchanged commit before the signed internal tag.
Do not publish crates.
