# cloud-sdk 0.36.0 Release Notes

Status: implementation complete; pentest required before final release checks.

Release date: 2026-07-27

## Overview

v0.36 makes HTTP headers complete bounded transport values. Providers own
request policy explicitly, adapters cannot be tricked into accepting
caller-owned authority, framing, or authorization fields, and response
metadata is retained only within fixed count and byte limits.

## Bounded Header Model

- Added validated `HeaderName`, `HeaderValue`, `RequestHeader`, and
  `RequestHeaders`.
- Added public/sensitive classifications with payload-free diagnostics.
- Added typed Accept and Content-Type constructors.
- Added exact atomic HTTP/1 field-line encoding.
- Rejected controls, oversized values, count/aggregate overflow, and both
  identical and conflicting duplicates.
- Reserved Host, framing, authorization, proxy, and hop-by-hop ownership.
- Added fixed-capacity owned `ResponseHeaders` and ordered borrowed views.

## Provider, Adapter, And Testkit

- Moved Hetzner JSON Accept and Content-Type policy into every prepared
  request.
- Removed reqwest's implicit JSON Accept injection.
- Forwarded exact admitted request headers in blocking and async adapters.
- Bound Host and TLS SNI to the endpoint URL verified against
  `EndpointIdentity`.
- Captured bounded response headers before body reads and derived typed content
  and rate-limit metadata from that same collection.
- Added exact testkit header matching, raw response fixtures, and redacted
  prepared header counts.

## Versions

| Crate | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.36.0` | Bounded header contracts |
| `cloud-sdk-hetzner` | `0.29.0` | Explicit prepared headers |
| `cloud-sdk-reqwest` | `0.24.0` | Header forwarding and capture |
| `cloud-sdk-sanitization` | `0.15.4` | Dependency-only patch |
| `cloud-sdk-testkit` | `0.21.0` | Exact header fixtures |

## Verification

- `scripts/check_header_model.sh`
- `scripts/checks.sh`
- `scripts/release_0_36_gate.sh` after pentest evidence is committed
- default, no_std, all-feature, clippy, doctest, package, deny, audit, platform,
  MSRV, and SBOM gates

## Migration

See [`docs/MIGRATION_0.36.0.md`](../docs/MIGRATION_0.36.0.md),
[`docs/PUBLIC_API_REVIEW_0.36.0.md`](../docs/PUBLIC_API_REVIEW_0.36.0.md), and
[`docs/DEPENDENCY_REVIEW_0.36.0.md`](../docs/DEPENDENCY_REVIEW_0.36.0.md).

## Release Gate

```text
v0.36.0 implementation stop reached. Run pentest for this exact commit.
```
