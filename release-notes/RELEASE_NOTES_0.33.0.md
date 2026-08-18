# cloud-sdk 0.33.0 Release Notes

Status: release candidate; pentest, final retest, and local release checks
passed. GitHub checks remain before tagging.

Release date: 2026-07-26

## Overview

v0.33 completes the provider-neutral HTTP method domain without adding
allocation or weakening the origin-form transport contract. Providers can add
bounded canonical extension methods without a core enum change, while
operation safety remains explicit provider metadata.

## HTTP Method Domain

- Preserved GET, POST, PUT, and DELETE construction.
- Added PATCH, HEAD, and origin-form OPTIONS.
- Added static extension methods bounded to 32 uppercase HTTP token bytes.
- Rejected empty, oversized, lowercase, Unicode, separator, control, and known
  alias inputs.
- Denied CONNECT and TRACE.
- Kept `OPTIONS *`, protocol upgrade, and tunnelling outside the transport
  contract.
- Added payload-free `MethodError` diagnostics.

## Transport And Provider Migration

- Mapped every admitted method through blocking and async reqwest adapters.
- Added exact wire tests for PATCH, HEAD, OPTIONS, and PURGE.
- Added exact extension-method matching to the provider-neutral testkit.
- Replaced Hetzner method-derived safety inference with explicit operation
  classes while preserving all existing impact, semantics, retry, and cost
  behavior.
- Added a portable source gate preventing the removed inference helper from
  returning.

## Versions

| Crate | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.33.0` | Complete bounded HTTP method domain |
| `cloud-sdk-hetzner` | `0.26.0` | Explicit provider-owned operation classes |
| `cloud-sdk-reqwest` | `0.21.0` | Complete method transport mapping |
| `cloud-sdk-sanitization` | `0.15.1` | Dependency-only patch |
| `cloud-sdk-testkit` | `0.18.4` | Dependency-only patch and method regression |

## Verification

- `scripts/check_http_method_domain.sh`
- `scripts/checks.sh`
- `scripts/release_0_33_gate.sh` after pentest evidence is committed
- Rust `1.90.0` MSRV and current pinned stable checks
- Default, no_std, all-feature, clippy, doctest, package, deny, audit, and SBOM
  gates

## Migration

See [`docs/MIGRATION.md#v0330`](../docs/MIGRATION.md#v0330) and
[`docs/PUBLIC_API_REVIEW.md#v0330`](../docs/PUBLIC_API_REVIEW.md#v0330).
No external package changed; the review is in
[`docs/DEPENDENCY_REVIEW.md#v0330`](../docs/DEPENDENCY_REVIEW.md#v0330).

## Release Gate

```text
v0.33.0 implementation stop reached. Run pentest for this exact commit.
```
