# cloud-sdk 0.38.0 Release Notes

Status: implementation stop reached; pentest required before release.

Release date: pending

## Overview

v0.38 makes response cleanup a mandatory provider-neutral core property.
Checked response guards now own the complete body, metadata, request-ID, and
decoder staging lifecycle instead of trusting each transport to supply the
baseline sanitizer.

## Mandatory Cleanup

- Routed all core-owned clearing through the admitted volatile primitive.
- Removed ordinary first-party zero-fill cleanup implementations.
- Made `ResponseBuffer::new` clear complete storage without a transport hook.
- Kept platform sanitizers additive through
  `ResponseBuffer::with_additive_sanitizer`.
- Added a final-clear drop guard covering no-op, recontaminating, and panicking
  additive hooks.

## Complete Workspace

- Added fixed cleanup-owning decoder, cursor, and provider-link scratch.
- Made response headers, content type, metadata, and commits non-`Copy`.
- Migrated Hetzner direct JSON decoding to guard-owned scratch.
- Cleared complete caller storage and staging on success, rejection, decode
  error, cancellation, and unwind where supported.

## Request Identifiers

- Added explicit `Retain`, `Protected`, and `Discard` operation policy.
- Added bounded protected request-ID admission from response headers.
- Added non-`Copy`, non-`Clone`, redacted retained metadata.
- Added atomic transfer with immediate source cleanup on success or rejection.
- Classified every current Hetzner operation as protected.

## Dependency Direction

`cloud-sdk` now depends on `cloud-sdk-sanitization`, which depends only on the
admitted `sanitization 2.0.3`. Publication order was inverted accordingly.
No external package was added or upgraded.

## Versions

| Crate | Version | Change |
| --- | --- | --- |
| `cloud-sdk-sanitization` | `0.16.0` | dependency inversion and cleanup primitive |
| `cloud-sdk` | `0.38.0` | mandatory cleanup and complete workspace ownership |
| `cloud-sdk-hetzner` | `0.31.0` | checked decoder workspace migration |
| `cloud-sdk-reqwest` | `0.26.0` | non-Copy response metadata migration |
| `cloud-sdk-testkit` | `0.23.0` | mandatory cleanup fixtures |

## Verification

- `scripts/check_response_cleanup.sh`
- `scripts/check_response_provenance.sh`
- `scripts/check_sanitization_boundary.sh`
- `scripts/check_reqwest_boundary.sh`
- `scripts/check_testkit_boundary.sh`
- `scripts/checks.sh`
- `scripts/release_0_38_gate.sh` after pentest evidence is committed
- default, no_std, all-feature, clippy, doctest, package, deny, audit, platform,
  MSRV, fuzz, and SBOM gates

## Guarantee Limits

Cleanup does not cover process abort, `mem::forget` or deliberately leaked
guards, immutable/external copies, TLS and allocator internals, kernel/device
buffers, swap, crash dumps, or remote systems. Zero read-back is an integrity
check and not proof that an optional additive hook executed.

## Migration

See [`docs/MIGRATION_0.38.0.md`](../docs/MIGRATION_0.38.0.md),
[`docs/PUBLIC_API_REVIEW_0.38.0.md`](../docs/PUBLIC_API_REVIEW_0.38.0.md), and
[`docs/DEPENDENCY_REVIEW_0.38.0.md`](../docs/DEPENDENCY_REVIEW_0.38.0.md).

## Release Gate

```text
v0.38.0 implementation stop reached. Run pentest for this exact commit.
```
