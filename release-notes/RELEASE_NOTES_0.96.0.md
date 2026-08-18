# cloud-sdk 0.96.0 Release Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-18

Security-Review: PASS
Pentest: PASS
Publication: DEFERRED TO v0.100.0

## Overview

v0.96 makes the pinned Hetzner request contract executable at parameter level
and corrects server metrics to support every documented metric combination.
It starts the v0.96-v0.100 cumulative train and publishes no crate.

## Request Fidelity

- Added `SourceLockedQuery`, an allocation-free operation-bound query model
  generated from all 218 active query declarations across 47 operations.
- Enforced required fields, operation ownership, scalar/repeated cardinality,
  duplicate rejection, source enums, pagination limits, bounded text, metrics
  timestamps, positive steps, and atomic target encoding.
- Preserved Hetzner's documented comma-separated metrics exception while all
  ordinary form-exploded arrays encode as repeated query parameters.
- Added a 528-row request inventory covering 437 path/query declarations and
  91 request-body operations. Four deprecated Data Center query rows remain
  explicit exclusions.
- Bound inventory generation to local and fetched OpenAPI drift checks. An
  accepted lock refresh now remains red until its executable inventory is
  separately reviewed and regenerated.
- Added mutation tests for parameter additions, requiredness, scalar/array
  transitions, enums, fingerprints, and unsupported encoding/type changes.

## Request Corrections

- Replaced single server metric selection with a non-empty duplicate-free
  `ServerMetricTypes` bitset covering all seven combinations.
- Added real UTC calendar validation, increasing ranges, and positive numeric
  `ServerMetricsStep` values.
- Replaced the removed Primary IP `type` list filter with current `name` and
  `ip` filters.
- Added Image architecture, deprecation, label, name, and status convenience
  filters; repeated Image filters and every action-list filter are available
  through the operation-bound source query.
- Added compile-checked documentation and preparation tests for complete
  repeated filters and mismatch cleanup.

## Pentest Remediation

- Scoped dependency-review evidence to the explicitly selected release
  section and corrected the v0.96 comparison baseline to v0.95.0. Historical
  evidence can no longer satisfy a current dependency transition.
- Rejected every non-decimal byte in source-locked metrics timestamps rather
  than accepting punctuation whose ASCII offset exceeded nine.
- Added regressions for historical dependency-row reuse, missing or malformed
  release sections, and malformed metrics timestamp digits.
- Reused the canonical release-train checkpoint calculation in release
  readiness, with regressions proving v0.96 defers to v0.100 and rejects a
  premature v1.0 deferral.

## Versions

| Crate | Published | v0.96 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.95.0` | `0.96.0` | deferred |
| `cloud-sdk-hetzner` | `0.46.0` | `0.46.0` | code; deferred |
| `cloud-sdk-reqwest` | `0.36.0` | `0.36.0` | unchanged |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged |
| `cloud-sdk-testkit` | `0.31.0` | `0.31.0` | unchanged |

## Stop Gate

The incremental pentest and remediation retest are green. Run
`scripts/release_0_96_gate.sh` against the exact final evidence commit and
require green GitHub CI and CodeQL before tagging. Do not publish crates; the
cumulative public checkpoint is v0.100.0.

## Result

v0.96.0 is ready for its internal signed tag after the clean local release
gate and GitHub CI and CodeQL pass. No crate is selected for publication.
