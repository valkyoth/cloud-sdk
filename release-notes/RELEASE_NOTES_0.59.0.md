# cloud-sdk 0.59.0 Milestone Notes

Status: implementation stop reached; incremental pentest required.

Release date: 2026-08-07

Security-Review: PENDING
Pentest: REQUIRED
Publication: DEFERRED TO v0.60.0

## Overview

v0.59 challenges neutral pagination and schema-version contracts against the
source-locked OVHcloud API v2 probe. It adds bounded sensitive header-cursor
decoding and explicit reviewed schema validation without publishing an
OVHcloud provider or adding a default dependency.

This milestone requires an incremental pentest from signed v0.58.0, the full
local and GitHub gates, and a normal signed tag. No crate is published until
the v0.60.0 checkpoint.

## Header Cursor Pagination

- Bound each cursor lifecycle to one `OperationId` plus distinct request
  cursor, request size, and response next-cursor header roles.
- Marked cursor request values sensitive and required sensitive raw response
  retention before decoding.
- Treated an absent next header as terminal exactly as the provider source
  defines, without body-size inference.
- Rejected empty, control-bearing, non-ASCII, oversized, duplicate, public,
  and insufficient-storage cursor metadata.
- Reused cleanup-owning `PaginationCursor` and exact `CursorHistory` cycle and
  collision checks.
- Returned an operation-bound continuation that cannot safely rebind its
  cursor to another header policy.

## Schema Validation

- Added canonical bounded `major.minor` schema versions.
- Bound an admitted major to exact reviewed SHA-256 source evidence.
- Rejected malformed, zero, overflowing, and unreviewed major versions.
- Kept schema overrides validation-only and absent from ordinary production
  request construction.
- Source-locked OVHcloud schema `1.0`, four paginated IAM operations, and the
  exact three cursor headers.

## Versions

| Crate | Source version | Cumulative change | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.59.0` | code | deferred to v0.60.0 |
| `cloud-sdk-hetzner` | `0.39.0` | dependency | deferred |
| `cloud-sdk-reqwest` | `0.32.4` | dependency | deferred |
| `cloud-sdk-sanitization` | `0.18.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.29.0` | dependency | deferred |

## Documentation

- [`provider-probes/ovhcloud-v2/README.md`](../provider-probes/ovhcloud-v2/README.md)
- [`docs/PAGINATION_STRATEGIES.md`](../docs/PAGINATION_STRATEGIES.md)
- [`docs/SCHEMA_VERSION_VALIDATION.md`](../docs/SCHEMA_VERSION_VALIDATION.md)
- [`docs/MIGRATION_0.59.0.md`](../docs/MIGRATION_0.59.0.md)
- [`docs/PUBLIC_API_REVIEW_0.59.0.md`](../docs/PUBLIC_API_REVIEW_0.59.0.md)
- [`docs/DEPENDENCY_REVIEW_0.59.0.md`](../docs/DEPENDENCY_REVIEW_0.59.0.md)

## Release Gate

Stop for the incremental pentest after the implementation commit. After a
green retest, add `security/pentest/v0.59.0.md`, change the review fields to
PASS, run `scripts/release_0_59_gate.sh` on the clean evidence commit, and wait
for GitHub CI and CodeQL before tagging. Do not publish crates for this
internal milestone.
