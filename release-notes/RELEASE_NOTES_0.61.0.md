# cloud-sdk 0.61.0 Milestone Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-07

Security-Review: PASS
Pentest: PASS
Publication: DEFERRED TO v0.65.0

## Overview

v0.61 completes the unpublished OVHcloud API v2 architecture probe by
executing all ten source-locked read operations through unchanged neutral
contracts. It adds no supported provider and selects no crate for crates.io.

This milestone requires an incremental pentest from signed v0.60.0, the full
local and GitHub gates, and a normal signed tag. Publication remains deferred
to the v0.65.0 cumulative checkpoint.

## Execution Harness

- Added the exact `publish = false` `ovhcloud-v2-probe` workspace harness.
- Bound its catalog one-to-one to all ten source-locked candidates.
- Executed every operation through blocking, Send-async, and local-async
  prepared-request paths using provider-neutral testkit transports.
- Required read-only and safe metadata, no known cost, no automatic retry,
  empty request bodies, exact endpoint identity, and bounded JSON responses.
- Sent `X-Pagination-Size` only for the five cursor collections and admitted
  only reviewed content-type and next-cursor response metadata.
- Kept provider identities, routes, fixtures, and live configuration out of
  every reusable crate's production source.

## Optional Live Smoke

- Added one ignored, feature-gated EU `GET /iam/policy` smoke requiring the
  exact `account:apiovh:iam/policy/get` action.
- Required explicit read-only mode and a private regular token file on Unix;
  non-Unix live execution fails closed pending equivalent ACL checks.
- Rejected custom endpoints, destructive mode, retries, symlinks, broad file
  permissions, replacement during open, and oversized token/response data.
- Protected the complete token read buffer, including partial-read failures,
  with the first-party sanitization boundary and kept credentials absent from
  CI and release gates.

## Versions

| Crate | Source version | Cumulative change | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.61.0` | metadata | deferred to v0.65.0 |
| `cloud-sdk-hetzner` | `0.39.1` | dependency | deferred |
| `cloud-sdk-reqwest` | `0.33.0` | dependency | deferred |
| `cloud-sdk-sanitization` | `0.18.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.29.1` | dependency | deferred |

The nonpublishable `ovhcloud-v2-probe` harness follows the workspace milestone
version but is not part of the independent crate publication plan.

## Documentation

- [`provider-probes/ovhcloud-v2/README.md`](../provider-probes/ovhcloud-v2/README.md)
- [`provider-probes/ovhcloud-v2/THREAT_MODEL.md`](../provider-probes/ovhcloud-v2/THREAT_MODEL.md)
- [`docs/MIGRATION_0.61.0.md`](../docs/MIGRATION_0.61.0.md)
- [`docs/PUBLIC_API_REVIEW_0.61.0.md`](../docs/PUBLIC_API_REVIEW_0.61.0.md)
- [`docs/DEPENDENCY_REVIEW_0.61.0.md`](../docs/DEPENDENCY_REVIEW_0.61.0.md)
- [`security/pentest/v0.61.0.md`](../security/pentest/v0.61.0.md)

## Release Gate

The incremental pentest and final retest passed. Run
`scripts/release_0_61_gate.sh` on the clean evidence commit. GitHub CI and
CodeQL must then be green on that unchanged commit before the signed internal
tag. Do not publish crates for this milestone.
