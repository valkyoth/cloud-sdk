# cloud-sdk 0.63.0 Milestone Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-08

Security-Review: PASS
Pentest: PASS
Publication: DEFERRED TO v0.65.0

## Overview

v0.63 completes source-derived ordinary Hetzner Cloud resource models. It is
an internal tag and publishes no crate. The provider package version remains
0.39.1 while these changes accumulate for the v0.65.0 public checkpoint.

## Cloud Models

- Added dedicated models for firewalls, floating IPs, images, ISOs, load
  balancers and types, networks, placement groups, primary IPs, servers and
  types, and volumes.
- Made pricing source-complete and retained every nested source field.
- Added `CloudResource` and `CloudResourceKind` so ordinary Cloud responses no
  longer pass through the common-identity fallback.
- Added `CloudObject`, `CloudValue`, and `CloudNumber` to retain source-known
  and bounded future fields without coercing integer identities through
  floating point.
- Added dedicated single and list checked-success variants and a dedicated
  composite resource slot.

## Source Evidence

- Generated a 535-row field contract and complete fixtures from the exact
  pinned Hetzner Cloud OpenAPI document.
- Enforced required fields, nullability, exact JSON types, numeric, text, and
  list bounds, RFC 3339 date-times, exact decimal syntax, integer/double
  formats, source patterns, plus load-balancer discriminated unions. Source
  string lengths use Unicode scalar counts under a separate hard byte ceiling.
- Preserved unknown future enum strings and fields after source-known
  validation instead of rejecting additive upstream evolution.
- Bound both generated files to live upstream drift detection and added
  canonical-schema-equality, deterministic fixture, adversarial model, full
  operation matrix, and fuzz-corpus evidence.

## Security Hardening

- Redacted identifiers, metadata, topology, pricing, and unknown future values
  from every complete Cloud model's `Debug` output, including composite
  results.
- Removed infallible recursive `Clone` from complete Cloud and pricing trees;
  fallible `try_clone` methods now preserve checked-allocation failures.
- Made unknown source formats, patterns, and unsupported numeric or collection
  constraints stop schema generation instead of silently weakening validation.
- Made schema composition fail closed on unknown keywords, semantic `allOf`
  siblings, and unsafe overlapping properties while retaining checked
  discriminator-enum intersections used by the pinned specification.
- Restricted `oneOf` admission to the implemented discriminated array-item
  union path so direct object unions cannot silently lose branch constraints.

## Dependency Maintenance

- Updated the optional, exact `base64-ng` Basic-auth encoder dependency from
  1.3.9 to 2.0.1 with default features disabled and no new transitive package.
- Retained exact feature-boundary, bounded-output, source-clearing, and
  authorization-vector tests around the encoder integration.

## Versions

| Crate | Source version | Cumulative change | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.63.0` | metadata | deferred to v0.65.0 |
| `cloud-sdk-hetzner` | `0.39.1` | code | deferred |
| `cloud-sdk-reqwest` | `0.33.0` | code | deferred |
| `cloud-sdk-sanitization` | `0.18.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.29.1` | code | deferred |

## Release Evidence

- [`docs/PUBLIC_API_REVIEW_0.63.0.md`](../docs/PUBLIC_API_REVIEW_0.63.0.md)
- [`docs/DEPENDENCY_REVIEW_0.63.0.md`](../docs/DEPENDENCY_REVIEW_0.63.0.md)
- [`docs/THREAT_MODEL_DELTA_0.63.0.md`](../docs/THREAT_MODEL_DELTA_0.63.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.63.0.md`](../docs/REJECTED_ABSTRACTIONS_0.63.0.md)
- [`docs/MIGRATION_0.63.0.md`](../docs/MIGRATION_0.63.0.md)
- [`security/pentest/v0.63.0.md`](../security/pentest/v0.63.0.md)

## Release Gate

The incremental pentest and final retest passed. Run
`scripts/release_0_63_gate.sh` on the clean evidence commit. GitHub CI and
CodeQL must then be green on that unchanged commit before the signed internal
tag. Do not publish crates.
