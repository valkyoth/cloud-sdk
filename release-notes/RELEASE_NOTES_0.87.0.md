# cloud-sdk 0.87.0 Release Notes

Status: implementation stop; pentest required.

Release date: unreleased

Security-Review: PENDING
Pentest: PENDING
Publication: DEFERRED TO v0.90.0

## Overview

v0.87 implements the active Hetzner Robot traffic query and continues the
v0.86-v0.90 cumulative train. This internal milestone will be tagged after its
incremental pentest and green CI/CodeQL but publishes no crate.

## Robot Traffic

- Added bounded repeated canonical IP/subnet targets and protected exact day,
  month, and year intervals.
- Added atomic sensitive form preparation for `POST /traffic`, optional
  `single_values=true`, explicit replayability, and caller-policy-only retry.
- Added an explicit provider-neutral read-only POST-query constructor without
  weakening ordinary POST authorization.
- Added direct incremental decoding with duplicate/unknown-key rejection,
  request-bound type/ranges/targets, canonical subnet CIDRs, sorted sparse
  periods, and exact non-negative decimal tokens.
- Source-locked the 200/hour quota, success shape, and operation-specific
  `INVALID_INPUT`, `NOT_FOUND`, and `INTERNAL_ERROR` failures.
- Added focused boundary, hostile shape, chunk-split, source contract, and core
  policy regression tests.

## Versions

| Crate | Published | v0.87 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.85.0` | `0.87.0` | deferred |
| `cloud-sdk-hetzner` | `0.44.0` | `0.44.0` | deferred |
| `cloud-sdk-reqwest` | `0.35.2` | `0.35.2` | unchanged |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.4` | `0.30.4` | unchanged |

## Evidence

- [`docs/PUBLIC_API_REVIEW_0.87.0.md`](../docs/PUBLIC_API_REVIEW_0.87.0.md)
- [`docs/DEPENDENCY_REVIEW_0.87.0.md`](../docs/DEPENDENCY_REVIEW_0.87.0.md)
- [`docs/THREAT_MODEL_DELTA_0.87.0.md`](../docs/THREAT_MODEL_DELTA_0.87.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.87.0.md`](../docs/REJECTED_ABSTRACTIONS_0.87.0.md)
- [`docs/MIGRATION_0.87.0.md`](../docs/MIGRATION_0.87.0.md)

## Stop Gate

Run the incremental pentest against the exact implementation-stop commit.
After remediation and retest, add the report, run `scripts/release_0_87_gate.sh`,
and require green GitHub CI/CodeQL on the unchanged evidence commit before
tagging. Do not publish crates for v0.87.
