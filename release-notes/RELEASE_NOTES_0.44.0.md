# cloud-sdk 0.44.0 Release Notes

Status: implementation stop reached; pentest required before release.

Release date: unreleased

## Overview

v0.44 separates numbered, offset, cursor, marker, and provider-link pagination
into explicit provider-neutral strategies with shared hard budgets.

## Pagination

- Added transactional request, item, state-size, and snapshot budgets.
- Added numbered and offset progression with drift and empty-continuation
  rejection.
- Added cleanup-owning opaque cursor and marker state.
- Added exact cursor history with cycle, digest-collision, and digest-change
  rejection.
- Added operation-bound absolute and origin-form provider next links.
- Preserved raw link query ordering, duplicates, plus signs, separators, and
  percent encoding without decode or re-encode.
- Prevented provider-link queries from entering structured target assembly.
- Migrated Hetzner pagination metadata to `NumberedPageMetadata`.

## Versions

| Crate | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.44.0` | pagination strategy family |
| `cloud-sdk-hetzner` | `0.34.0` | numbered metadata migration |
| `cloud-sdk-reqwest` | `0.30.1` | dependency-only patch |
| `cloud-sdk-sanitization` | `0.16.0` | unchanged; not published |
| `cloud-sdk-testkit` | `0.25.1` | dependency-only patch |

## Documentation

- [`docs/PAGINATION_STRATEGIES.md`](../docs/PAGINATION_STRATEGIES.md)
- [`docs/MIGRATION_0.44.0.md`](../docs/MIGRATION_0.44.0.md)
- [`docs/PUBLIC_API_REVIEW_0.44.0.md`](../docs/PUBLIC_API_REVIEW_0.44.0.md)
- [`docs/DEPENDENCY_REVIEW_0.44.0.md`](../docs/DEPENDENCY_REVIEW_0.44.0.md)

## Pentest

Pending for the exact implementation-stop commit.

## Release Gate

```text
v0.44.0 implementation stop reached. Run pentest for this exact commit.
```
