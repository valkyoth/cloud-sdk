# cloud-sdk 0.50.0 Release Notes

Status: release candidate; pentest and final retest passed.

Release date: pending

## Overview

v0.50 adds exhaustive compile-time associations for all 208 active Hetzner
operations. Endpoint, query, body, prepared request, response family, and
safety policy can now retain one nominal operation marker through preparation
and execution.

## Operation Associations

- Added one sealed operation marker for every active source-locked operation.
- Added nominal endpoint, query, and body wrappers that reject wrong bindings.
- Added typed `AssociatedOperation` construction for absent, queried, and JSON
  request shapes.
- Added `Prepared<O>` with checked blocking, Send-async, and local-async
  execution and explicit operation type erasure.
- Associated service, official endpoint, auth class/scope, method, query/body,
  request headers/media, success status, response body/media/caps, pagination,
  quota, retry, streaming, success/error models, and permit class.
- Added inspectable non-secret descriptors and forward-compatible policy enums.
- Added clear-before-validation exact policy checking and cleanup-owning
  `prepare_typed_guarded` before typed request serialization.
- Snapshot canonical operation ID, method, metadata, request-ID handling, body
  replayability, authentication scope, checked-response policy, and complete
  raw-response policy into one token consumed directly by request assembly.
- Added a stateful-endpoint regression proving assembly does not recalculate
  security-relevant endpoint policy after successful validation.
- Generated all bindings from a strict reviewed 208-row classification
  manifest plus exact API fingerprint, body, and response locks.
- Added freshness, regression, compile-fail, service-identity, and behavioral
  tests without changing the default no_std or dependency graph.
- Added const `OperationId::new` and compile-time `operation_id!` literals.

## Versions

| Crate | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.50.0` | const operation identifier code |
| `cloud-sdk-hetzner` | `0.38.0` | operation association code |
| `cloud-sdk-reqwest` | `0.32.3` | dependency-only patch |
| `cloud-sdk-sanitization` | `0.17.0` | unchanged; not published |
| `cloud-sdk-testkit` | `0.28.2` | dependency-only patch |

## Documentation

- [`docs/OPERATION_ASSOCIATIONS.md`](../docs/OPERATION_ASSOCIATIONS.md)
- [`docs/MIGRATION.md#v0500`](../docs/MIGRATION.md#v0500)
- [`docs/PUBLIC_API_REVIEW.md#v0500`](../docs/PUBLIC_API_REVIEW.md#v0500)
- [`docs/DEPENDENCY_REVIEW.md#v0500`](../docs/DEPENDENCY_REVIEW.md#v0500)

## Pentest

Pentest and final retest passed. The permanent report is committed at
[`security/pentest/v0.50.0.md`](../security/pentest/v0.50.0.md).

## Release Gate

v0.50.0 release candidate. Tag only after the clean local release gate and
GitHub CI and CodeQL default setup pass on the final release commit.
