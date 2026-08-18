# cloud-sdk 0.34.0 Release Notes

Status: release candidate; pentest, final retest, and local release checks
passed. GitHub checks remain required before tagging.

Release date: 2026-07-26

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
- Bounded raw endpoint input before URL parsing and rejected any base-path
  bytes that the URL parser could normalize or remove.
- Kept DNS resolution and egress filtering outside provider-neutral core.

## Provider And Adapter Migration

- Added exact fixed Hetzner Cloud and Console Storage policies.
- Routed prepared operations and diagnostic verification through the same
  canonical Hetzner endpoint identity constructor.
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

## Pentest

The iterative v0.34 pentest identified three Low endpoint-hardening findings
and one informational unreachable IPv4 parser guard. Raw configured paths now
reject every byte class that the URL parser could normalize, complete endpoint
input is bounded before allocation, and prepared Hetzner operations use the
canonical provider-owned policy source. The final retest passed commit
`54d522fbf3717ebeea26f4fe65e0894dd951ad01`.

The optional exact composite boundary is also covered: a valid 253-byte host,
port `65535`, and 1,024-byte path produce the admitted 1,291-byte maximum, while
one additional byte fails with `InputTooLong`. See the
[`v0.34.0` pentest report](../security/pentest/v0.34.0.md).

## Migration

See [`docs/MIGRATION.md#v0340`](../docs/MIGRATION.md#v0340),
[`docs/PUBLIC_API_REVIEW.md#v0340`](../docs/PUBLIC_API_REVIEW.md#v0340), and
[`docs/DEPENDENCY_REVIEW.md#v0340`](../docs/DEPENDENCY_REVIEW.md#v0340).

## Release Gate

```text
v0.34.0 pentest stop passed. Tag only after the clean local release gate and
GitHub checks pass on the final release commit.
```
