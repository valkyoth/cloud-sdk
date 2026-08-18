# cloud-sdk 0.36.0 Release Notes

Status: release candidate; pentest and final retest passed. Local and GitHub
release checks remain required before tagging.

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
- Captured bounded response headers after the final async body-read suspension
  but before caller-visible body publication, keeping the 8 KiB arena out of
  suspended task state.
- Derived typed content and rate-limit metadata from that same collection.
- Preserved reqwest sensitivity, defaulted unknown response fields to
  sensitive, and allowed public classification only for reviewed content,
  length, date, and rate-limit metadata.
- Added exact testkit header matching, raw response fixtures, and redacted
  prepared header counts.
- Aligned typed Content-Type values with canonical no-trailing-space request
  header grammar.
- Removed ordinary equality from secret-capable header values, collections,
  transport responses, checked responses, and transitive testkit wrappers.

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

## Pentest

The iterative v0.36 pentest found four issues in async header lifetime,
Content-Type grammar, response-field sensitivity, and ordinary equality on
secret-capable values. Response headers are now captured after the final async
suspension, typed content values reject trailing whitespace, unknown response
fields default to sensitive under a narrow public allowlist, and public
secret-capable wrappers no longer expose structural equality.

The final retest passed commit
`7785ad754451c2ef4f2736ea0442d3fc3a4464db`. See the
[`v0.36.0` pentest report](../security/pentest/v0.36.0.md).

## Migration

See [`docs/MIGRATION.md#v0360`](../docs/MIGRATION.md#v0360),
[`docs/PUBLIC_API_REVIEW.md#v0360`](../docs/PUBLIC_API_REVIEW.md#v0360), and
[`docs/DEPENDENCY_REVIEW.md#v0360`](../docs/DEPENDENCY_REVIEW.md#v0360).

## Release Gate

```text
v0.36.0 pentest stop passed. Tag only after the clean local release gate and
GitHub checks pass on the final release commit.
```
