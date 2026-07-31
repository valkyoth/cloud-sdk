# cloud-sdk 0.43.0 Release Notes

Status: implementation stop reached; pentest required before release.

Release date: unreleased

## Overview

v0.43 completes the authenticated raw-wire migration for all 208 active
Hetzner operations. Preparation, authentication, bounded HTTP execution,
delivery phase, response provenance, and cleanup now form one mandatory path
with no compatibility fallback.

## Prepared Wire Contract

- Added mandatory authentication and raw response policies to
  `PreparedRequest`.
- Made prepared construction fail when protected or retainable request IDs are
  absent from raw header admission.
- Required authenticated transports for prepared blocking and async execution.
- Added an exact `authenticated_request` projection for adapter integration.
- Made admitted raw response headers bounded and owned by the policy.
- Preserved delivery phase when mapping transport errors.

## Hetzner Migration

- Bound Cloud, DNS, Security, and Storage operations to distinct service
  identities.
- Required the exact official endpoint plus provider/service/endpoint
  authentication scope.
- Forbade unowned audience, account, and tenant scope.
- Added independent success/error response bounds, exact media policy, admitted
  content type, protected request ID, complete quota metadata, and bounded
  informational responses.
- Migrated the live read-only smoke harness to prepared authenticated requests.
- Added a machine-checked zero-fallback gate for all 208 active operations.

## Reqwest And Testkit

- Routed bearer and Basic clients through the shared bounded raw Hyper engine.
- Kept authorization transport-owned and inserted it only after scope
  validation.
- Returned conservative `NotSent`, `PossiblySent`, or `ResponseStarted`
  failures from authenticated clients.
- Retained operation-admitted quota headers for later provider decoding.
- Rechecked informational rejection after final-response readiness to close the
  multithreaded completion race.
- Added authenticated mock execution and redacted auth/raw policy records.

## Versions

| Crate | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.43.0` | mandatory prepared authentication and raw policies |
| `cloud-sdk-hetzner` | `0.33.0` | all active operations migrated |
| `cloud-sdk-reqwest` | `0.30.0` | authenticated raw Hyper execution |
| `cloud-sdk-sanitization` | `0.16.0` | unchanged; not published |
| `cloud-sdk-testkit` | `0.25.0` | authenticated mocks and policy records |

## Documentation

- [`docs/MIGRATION_0.43.0.md`](../docs/MIGRATION_0.43.0.md)
- [`docs/PUBLIC_API_REVIEW_0.43.0.md`](../docs/PUBLIC_API_REVIEW_0.43.0.md)
- [`docs/DEPENDENCY_REVIEW_0.43.0.md`](../docs/DEPENDENCY_REVIEW_0.43.0.md)
- [`docs/RAW_HTTP_EXECUTOR.md`](../docs/RAW_HTTP_EXECUTOR.md)
- [`docs/AUTHENTICATION_POLICY.md`](../docs/AUTHENTICATION_POLICY.md)

## Pentest

Initial review found request-ID/quota admission gaps and a final-response race
in informational rejection. These are remediated with cross-policy validation,
complete Hetzner header admission, protected metadata regressions, and a
multithreaded race regression. Retest is required; final evidence will be
recorded in `security/pentest/v0.43.0.md`.

## Release Gate

```text
v0.43.0 implementation stop reached. Run pentest for this exact commit.
```
