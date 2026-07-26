# cloud-sdk 0.32.0 Release Notes

Status: implementation complete; pentest required before release.

Release date: unreleased

## Overview

v0.32 removes closed Hetzner-shaped provider and API-family enums from the
provider-neutral core. Providers now own bounded canonical identities and
zero-sized service markers, so adding a provider does not require editing a
central registry.

## Provider-Neutral Identity

- Added allocation-free `ProviderId` and `ServiceId` static token types.
- Bounded both domains to 63 bytes with lowercase ASCII, digit, and canonical
  internal-hyphen validation.
- Added open `ProviderMarker` and `ServiceMarker` traits with explicit service
  ownership.
- Added `ProviderService::from_marker` and validated-ID direct construction.
- Replaced `provider()` and `family()` accessors with `provider_id()` and
  `service_id()`.
- Removed the closed `Provider`, `ApiFamily`, and catch-all `Extended` variants.

## Hetzner Migration

- Added provider-owned Hetzner, Cloud, DNS, security, and Console Storage
  markers and canonical IDs.
- Migrated all prepared operations and checked response bindings without
  changing methods, targets, bodies, endpoints, operation IDs, or response
  policies.
- Kept exact official endpoint verification independent from provider/service
  metadata.

## Verification

- External-crate provider implementation with no core registry edit.
- Compile-fail coverage for forged IDs and incomplete service ownership.
- Identifier syntax and length boundary corpus.
- Hetzner service mismatch regression coverage.
- `scripts/check_provider_identities.sh`
- `scripts/checks.sh`
- `scripts/release_0_32_gate.sh` once pentest evidence is committed.

## Dependency Freshness

- Updated the first-party `sanitization` boundary from `1.2.5` to `2.0.3`
  after reviewing retained Rust 1.90, `no_std`, volatile cleanup, owned secret,
  and closure-scoped access contracts.
- Preserved `cloud-sdk-sanitization::sanitize_bytes` while adapting its
  implementation to upstream `sanitization::wipe::bytes`.
- Updated Tokio to `1.53.1` and the isolated source checker to `syn 3.0.3`.
- Refreshed every independent lockfile and SPDX inventory.
- No new dependency or default feature enters `cloud-sdk`.

## Versions

| Crate | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.32.0` | Extensible provider/service identity foundation |
| `cloud-sdk-hetzner` | `0.25.0` | Provider-owned Hetzner identity migration |
| `cloud-sdk-reqwest` | `0.20.3` | Dependency-only patch |
| `cloud-sdk-sanitization` | `0.15.0` | Sanitization 2.x wrapper adaptation |
| `cloud-sdk-testkit` | `0.18.3` | Dependency-only patch |

## Migration

See [`docs/MIGRATION_0.32.0.md`](../docs/MIGRATION_0.32.0.md).

## Release Gate

```text
v0.32.0 implementation stop reached. Run pentest for this exact commit.
```
