# cloud-sdk 0.60.0 Release Notes

Status: release candidate; pentest and final retest passed.

Release date: pending

Security-Review: PASS
Pentest: PASS
Publication: PENDING

## Overview

v0.60 is the cumulative public checkpoint for v0.56 through v0.60. It
publishes provider-generic drift evidence, the neutral contracts challenged by
the unpublished OVHcloud API v2 probe, and bounded asynchronous-resource
models. It does not publish or claim an OVHcloud provider.

## Cumulative Changes

- Added strict provider-generic source locks, observations, canonical drift
  reports, ownership, and malicious-input regression coverage.
- Locked an excluded ten-operation OVHcloud API v2 read probe to five exact
  official sources without adding a package or credentialed execution path.
- Added exact regional API/token endpoint pairs, explicit-time OAuth lifetime
  decisions, and expiry-qualified atomic bearer rotation.
- Added prepared-request-bound sensitive header-cursor sessions and reviewed
  schema-version validation.
- Added allocation-free bounded identifiers, text, non-executable links,
  calendar-valid UTC timestamps, task/progress/error snapshots, and generic
  event batches.
- Preserved optional task links/messages, nullable error collections, semantic
  timestamp equality, and an explicit waiting-for-input polling disposition.
- Preserved successful task snapshots containing provider errors behind an
  explicit contradictory-success disposition instead of silently discarding
  the failure evidence or assuming undocumented provider semantics.
- Bound every task property type, nullability flag, and required flag into
  source conformance evidence.
- Bound task conformance to the production reads
  `/notification/contactMean/{contactMeanId}/task` and
  `/notification/contactMean/{contactMeanId}/task/{taskId}` plus the exact
  `common.Task` model family.
- Kept events fixture-only because the reviewed source surface does not claim
  an event endpoint.

## Versions

| Crate | Previous | v0.60 | Change | Publication |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.55.0` | `0.60.0` | cumulative code | yes |
| `cloud-sdk-hetzner` | `0.39.0` | `0.39.1` | core dependency | yes |
| `cloud-sdk-reqwest` | `0.32.4` | `0.33.0` | cumulative code | yes |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.29.0` | `0.29.1` | core dependency | yes |

## Documentation

- [`docs/ASYNC_RESOURCES.md`](../docs/ASYNC_RESOURCES.md)
- [`provider-probes/ovhcloud-v2/README.md`](../provider-probes/ovhcloud-v2/README.md)
- [`docs/MIGRATION.md#v0600`](../docs/MIGRATION.md#v0600)
- [`docs/PUBLIC_API_REVIEW.md#v0600`](../docs/PUBLIC_API_REVIEW.md#v0600)
- [`docs/DEPENDENCY_REVIEW.md#v0600`](../docs/DEPENDENCY_REVIEW.md#v0600)

## Release Gate

The incremental pentest from signed v0.59.0 and final retest passed. Run
`scripts/release_0_60_gate.sh` on the clean final evidence commit. GitHub CI
and CodeQL must then be green on that unchanged commit before tagging and
publishing the four selected crates in the verified release-tool order.
