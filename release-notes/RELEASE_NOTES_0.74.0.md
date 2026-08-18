# cloud-sdk 0.74.0 Milestone Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-10

Security-Review: PASS
Pentest: PASS
Publication: DEFERRED TO v0.75.0

## Overview

v0.74 establishes the complete reproducible source of truth for Hetzner Robot
before any Robot runtime operation is implemented. This is an internal
milestone; no crate is selected for crates.io publication.

## Robot Source Lock

- Locked all 105 operation headings from the official Robot document in exact
  source order under the already reviewed document SHA-256.
- Classified 89 operations as active and assigned each one to its planned
  implementation milestone from v0.78 through v0.93.
- Classified and excluded all 16 deprecated legacy `/storagebox` operations;
  the supported replacement is the existing Console Storage Box API.
- Preserved the v0.42 HTTPS, Basic authentication, form, error, invalid-input,
  quota, maintenance, lockout, and empty-body protocol fixture.
- Added a bounded redirect-rejecting fetch check that compares the exact
  upstream digest, all extracted HTTP headings, and every Storage Box
  deprecation marker to the committed lock.
- Bound the complete canonical operation policy to a separate reviewed
  SHA-256, so structurally valid ID or group/milestone swaps fail closed.
- Bounded lock reads before allocation and added a 90-second hard wall-clock
  fetch deadline in addition to per-operation network timeouts. Existing
  process-global real-time timers are rejected without being disabled.
- Added regression checks for count, identity, grouping, status, milestone,
  policy swapping, lockout retry, source-size, wall deadline, and redirect
  drift.
- Aligned the local upstream gate with the API matrix's explicit 221-operation
  OpenAPI total; Robot remains a separate 105-heading HTML source lock.
- Kept both Robot locks outside every publishable crate and added no runtime
  Robot module, credential, request, decoder, client, feature, or dependency.

## Versions

| Crate | Published | v0.74 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.70.0` | `0.74.0` | deferred to v0.75.0 |
| `cloud-sdk-hetzner` | `0.41.0` | `0.41.0` | accumulated code/evidence, no publication |
| `cloud-sdk-reqwest` | `0.34.1` | `0.34.1` | unchanged |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.1` | `0.30.1` | unchanged |

## Release Evidence

- [`docs/PUBLIC_API_REVIEW.md#v0740`](../docs/PUBLIC_API_REVIEW.md#v0740)
- [`docs/DEPENDENCY_REVIEW.md#v0740`](../docs/DEPENDENCY_REVIEW.md#v0740)
- [`docs/THREAT_MODEL_DELTA.md#v0740`](../docs/THREAT_MODEL_DELTA.md#v0740)
- [`docs/REJECTED_ABSTRACTIONS.md#v0740`](../docs/REJECTED_ABSTRACTIONS.md#v0740)
- [`docs/MIGRATION.md#v0740`](../docs/MIGRATION.md#v0740)
- [`security/pentest/v0.74.0.md`](../security/pentest/v0.74.0.md)

## Release Gate

Run `scripts/release_0_74_gate.sh` on the clean final evidence commit after the
incremental pentest and final retest. GitHub CI and CodeQL must be green on that
unchanged commit before the signed internal tag. Do not publish crates.
