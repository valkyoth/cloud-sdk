# cloud-sdk 0.35.0 Release Notes

Status: release candidate; pentest and final retest passed. Local and GitHub
release checks remain required before tagging.

Release date: 2026-07-27

## Overview

v0.35 makes `cloud-sdk` the single request-target validation authority.
Canonical paths and query dialects are separate, inspectable values, and
caller-buffer assembly preserves exact wire bytes without allocation.

## Canonical Request Targets

- Added `RequestPath`, `CanonicalQuery`, `FormQuery`, and `RequestQuery`.
- Distinguished absent from present-empty queries.
- Preserved pair order, duplicate keys, and missing versus empty values.
- Required uppercase percent hex and `%20` spaces in canonical queries.
- Kept form-style `+` behind the explicit `FormQuery` type.
- Added transactional `RequestTarget::assemble`.
- Added exact `path`, `query`, and `query_bytes` views.
- Documented that assembly initializes only `output[..target.len()]`; callers
  must not consume the untouched scratch-buffer tail.
- Rejected malformed percent triplets, hidden path separators and controls,
  dot segments, repeated slashes, fragments, backslashes, raw non-ASCII, and
  ambiguous structured-query syntax in core.

## Provider And Adapter Migration

- Aligned Hetzner `EndpointPath` with the canonical core grammar.
- Exposed exact prepared Hetzner query evidence through the core target.
- Removed reqwest's divergent target percent validator.
- Required reqwest URL composition to preserve exact admitted target bytes.
- Made testkit distinguish query presence and encoding dialect.
- Expanded the request-target fuzz harness and added a cross-adapter gate.
- Updated the immutable `actions/checkout` pin to signed release `v7.0.1` and
  the isolated fuzz toolchain to `nightly-2026-07-26`.
- Redacted encoded query keys and values from iterator and pair debug output.

## Versions

| Crate | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.35.0` | Canonical path/query model and assembly |
| `cloud-sdk-hetzner` | `0.28.0` | Canonical provider path integration |
| `cloud-sdk-reqwest` | `0.23.0` | Single-validator target composition |
| `cloud-sdk-sanitization` | `0.15.3` | Dependency-only patch |
| `cloud-sdk-testkit` | `0.20.0` | Exact query-state matching |

## Verification

- `scripts/check_request_targets.sh`
- `scripts/checks.sh`
- `scripts/release_0_35_gate.sh` after pentest evidence is committed
- Rust `1.90.0` through `1.96.1` and pinned stable checks
- Default, no_std, all-feature, clippy, doctest, package, deny, audit, fuzz,
  and SBOM gates

## Pentest

The iterative v0.35 pentest clarified the caller-owned scratch-buffer boundary
and found one Medium diagnostic disclosure. Successful assembly now documents
and tests that only `output[..target.len()]` is initialized, while the unused
tail remains untouched and outside every returned view. `QueryPair` and
`QueryPairs` now use explicit redacted `Debug` implementations so encoded
keys, values, and remaining query text cannot enter diagnostics.

The final retest passed commit
`7b25e7fc11139c5b664f62a8b245f3e6cf35a976`. See the
[`v0.35.0` pentest report](../security/pentest/v0.35.0.md).

## Migration

See [`docs/MIGRATION.md#v0350`](../docs/MIGRATION.md#v0350),
[`docs/PUBLIC_API_REVIEW.md#v0350`](../docs/PUBLIC_API_REVIEW.md#v0350), and
[`docs/DEPENDENCY_REVIEW.md#v0350`](../docs/DEPENDENCY_REVIEW.md#v0350).

## Release Gate

```text
v0.35.0 pentest stop passed. Tag only after the clean local release gate and
GitHub checks pass on the final release commit.
```
