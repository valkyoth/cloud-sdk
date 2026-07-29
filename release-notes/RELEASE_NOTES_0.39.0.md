# cloud-sdk 0.39.0 Release Notes

Status: implementation stop reached; pentest and final release checks are
required before tagging.

Release date: pending

## Overview

v0.39 makes request encoding transactional. Provider paths, queries, and JSON
bodies are measured from an immutable snapshot, written only after exact
capacity admission, and replayed against the exact output. The release also
adds cleanup-owning request preparation storage and named bounded profiles.

## Atomic Encoding

- Added provider-neutral `SnapshotEncoder` measure, write, and exact verify
  passes.
- Used checked arithmetic and explicit aggregate limits.
- Left every undersized destination byte-for-byte unchanged.
- Cleared only the exact admitted prefix if a later pass drifts.
- Compared bounded output directly without `Hash` or another digest.
- Made legacy single-value percent, integer, and JSON writes individually
  atomic.

## Provider Migration

- Migrated complete Hetzner Cloud, DNS, security, Storage Box, catalog,
  server, action, and shared query paths.
- Intentionally validated complete static-path output to keep future dynamic
  helper call sites inside the canonical path boundary.
- Migrated prepared JSON bodies behind a sealed sensitive-string interface.
- Removed duplicated mutable query cursors and shared one generic no_std
  encoder.
- Preserved provider-specific validation and payload-free errors.

## Preparation Storage

- Added non-`Copy` `PreparationStorageGuard` over complete target and body
  buffers.
- Bound prepared-request lifetime to cleanup ownership.
- Cleared complete target and body storage before every preparation attempt so
  reused buffers cannot retain a longer earlier request in their tails.
- Added `EMBEDDED`, `DEFAULT`, and `LARGE` capacity profiles.
- Added opt-in fallible `OwnedPreparationStorage` under `alloc`.
- Kept the default graph allocation-free, transport-free, and `no_std`.

## Versions

| Crate | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.39.0` | transactional encoder and preparation profiles |
| `cloud-sdk-hetzner` | `0.32.0` | provider request writer migration |
| `cloud-sdk-reqwest` | `0.26.1` | dependency-only core range update |
| `cloud-sdk-sanitization` | `0.16.0` | unchanged; not published |
| `cloud-sdk-testkit` | `0.23.1` | dependency-only core range update |

## Verification

- Core arithmetic, exact replay, drift cleanup, aggregate-cap, every
  undersized-capacity, guard cleanup, and allocation-failure tests
- Hetzner path/query/body atomicity and preparation cleanup tests
- Fuzzed buffer and provider request writers with unchanged-on-error checks
- `scripts/check_atomic_encoders.sh`
- `scripts/checks.sh`
- `scripts/release_0_39_gate.sh` after pentest evidence is committed
- default, no_std, all-feature, Clippy, doctest, package, deny, audit, platform,
  MSRV, fuzz, and SBOM gates

## Dependency Review

No external package was added or upgraded. See
[`docs/DEPENDENCY_REVIEW_0.39.0.md`](../docs/DEPENDENCY_REVIEW_0.39.0.md).

## Migration

See [`docs/MIGRATION_0.39.0.md`](../docs/MIGRATION_0.39.0.md) and
[`docs/PUBLIC_API_REVIEW_0.39.0.md`](../docs/PUBLIC_API_REVIEW_0.39.0.md).

## Pentest

Pending for the exact implementation-stop commit.

## Release Gate

```text
v0.39.0 implementation stop reached. Run pentest for this exact commit.
```
