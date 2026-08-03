# cloud-sdk 0.50.0 Release Notes

Status: implementation complete; pentest required.

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
- Rechecked generated policy against existing prepared runtime policy before a
  typed prepared value can be constructed.
- Generated all bindings from exact API fingerprint, body, and response locks.
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
- [`docs/MIGRATION_0.50.0.md`](../docs/MIGRATION_0.50.0.md)
- [`docs/PUBLIC_API_REVIEW_0.50.0.md`](../docs/PUBLIC_API_REVIEW_0.50.0.md)
- [`docs/DEPENDENCY_REVIEW_0.50.0.md`](../docs/DEPENDENCY_REVIEW_0.50.0.md)

## Pentest

Independent pentest is required for the exact implementation commit. The
permanent report will be committed at
`security/pentest/v0.50.0.md` after remediation and final retest.

## Release Gate

Do not tag v0.50.0 until pentest evidence is committed, the clean local release
gate passes, and GitHub CI and CodeQL default setup pass on the release commit.
