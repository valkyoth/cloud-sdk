# cloud-sdk 0.34.0 Release Notes

Status: implementation complete; exact-commit pentest required before release.

Release date: unreleased

## Overview

v0.34 makes credential destinations provider-owned and explicit. Prepared
operations carry an endpoint policy rather than one static identity, and the
reqwest boundary requires provider-policy admission or an explicit
trusted-operator acknowledgement before constructing a credential endpoint.

## Endpoint Policy

- Added fixed, bounded official-set, region-derived, and acknowledged-custom
  policy classes.
- Removed the `'static` endpoint restriction from `ProviderService`.
- Bound prepared blocking and async execution to exact policy verification.
- Added canonical lowercase DNS/A-label, IPv4, and bracketed IPv6 identity.
- Compared IPv6 by parsed address bits and rejected zone identifiers.
- Rejected userinfo, trailing DNS dots, percent-encoded hosts, Unicode host
  input, uppercase DNS, ambiguous IPv4, and non-canonical ports.
- Kept DNS resolution and egress filtering outside provider-neutral core.

## Provider And Adapter Migration

- Added exact fixed Hetzner Cloud and Console Storage policies.
- Added a bounded two-endpoint Hetzner diagnostic policy.
- Added `HttpsEndpoint::new_with_policy`.
- Changed `new_custom` to require
  `CustomEndpointAcknowledgement::trusted_operator_configuration()`.
- Preserved redirect denial, response-origin verification, HTTPS-only
  production clients, and credential rotation behavior.
- Preserved endpoint-policy lifetimes in testkit prepared-request records.
- Refreshed seven transitive lock entries to their latest
  Rust-1.90-compatible releases and regenerated all complete SPDX SBOMs.

## Versions

| Crate | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.34.0` | Endpoint policy algebra and canonical identity |
| `cloud-sdk-hetzner` | `0.27.0` | Provider-owned official policies |
| `cloud-sdk-reqwest` | `0.22.0` | Policy-aware endpoint construction |
| `cloud-sdk-sanitization` | `0.15.2` | Dependency-only patch |
| `cloud-sdk-testkit` | `0.19.0` | Policy-aware prepared records |

## Verification

- `scripts/check_endpoint_policy.sh`
- `scripts/checks.sh`
- `scripts/release_0_34_gate.sh` after pentest evidence is committed
- Rust `1.90.0` through `1.96.1` and pinned stable checks
- Default, no_std, all-feature, clippy, doctest, package, deny, audit, and SBOM
  gates

## Migration

See [`docs/MIGRATION_0.34.0.md`](../docs/MIGRATION_0.34.0.md),
[`docs/PUBLIC_API_REVIEW_0.34.0.md`](../docs/PUBLIC_API_REVIEW_0.34.0.md), and
[`docs/DEPENDENCY_REVIEW_0.34.0.md`](../docs/DEPENDENCY_REVIEW_0.34.0.md).

## Release Gate

```text
v0.34.0 implementation stop reached. Run pentest for this exact commit.
```
