# cloud-sdk 0.45.0 Release Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-01

## Overview

v0.45 moves quota interpretation out of transports and adds bounded,
provider-neutral quota and pure retry-delay policy.

## Quota And Retry

- Added fixed-capacity multi-bucket quota state with distinct relative,
  absolute, and unknown reset semantics.
- Added bounded informational extensions with redacted values.
- Added exact `Retry-After` delay-seconds, IMF-fixdate, RFC 850, and asctime
  parsing, including leap-year, leap-second, weekday, obsolete-year, and
  overflow validation.
- Added pure delay decisions with caller-owned time, rollback detection,
  stale-timestamp policy, explicit metadata conflict policy, and hard maximum
  delay clamping or rejection.
- Moved Hetzner's complete three-header decoder into `cloud-sdk-hetzner` and
  retained quota on checked successes and typed provider errors.
- Added `decode_response_at` for externally observed wall time and retained the
  existing single-bucket `rate_limit` compatibility view.
- Removed the obsolete reqwest-owned provider parser and its public transport
  error variant. Reqwest continues to retain bounded admitted headers only.
- Updated the exact rustls patch pin to `0.23.43` and re-ran ordinary,
  deterministic-root, FIPS, and feature-unification boundary checks.
- Added focused boundary, duplicate, partial, unknown-state, multiple-bucket,
  conflict, rollback, maximum, response-success, and response-error tests.
- Made large fixed-capacity quota aggregates non-`Copy`, changed read-only
  accessors to borrow, and boxed quota before checked success/error branching.
- Corrected RFC 850 two-digit-year resolution to compare the complete date and
  time at the exact 50-year boundary.
- Made redacted quota extensions non-`Copy` and volatile-cleared their complete
  fixed-capacity storage when the final owner is dropped; documented that this
  is best-effort cleanup for metadata and not stable secret storage across
  moves.
- Added a quota-gate Clippy denial for large types passed by value.
- Added a release check requiring the dependency review to inventory every
  root lockfile package-version change against the previous release tag.
- Added a dedicated quota, Retry-After, decision-policy, and provider-header
  fuzz target.

## Versions

| Crate | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.45.0` | quota and delay strategy family |
| `cloud-sdk-hetzner` | `0.35.0` | provider-owned quota decoder |
| `cloud-sdk-reqwest` | `0.31.0` | remove provider-specific decoder path |
| `cloud-sdk-sanitization` | `0.16.0` | unchanged; not published |
| `cloud-sdk-testkit` | `0.25.2` | dependency-only patch |

## Documentation

- [`docs/QUOTA_AND_RETRY.md`](../docs/QUOTA_AND_RETRY.md)
- [`docs/MIGRATION.md#v0450`](../docs/MIGRATION.md#v0450)
- [`docs/PUBLIC_API_REVIEW.md#v0450`](../docs/PUBLIC_API_REVIEW.md#v0450)
- [`docs/DEPENDENCY_REVIEW.md#v0450`](../docs/DEPENDENCY_REVIEW.md#v0450)

## Pentest

The permanent [v0.45.0 pentest report](../security/pentest/v0.45.0.md) records
the iterative review, completed remediation, and green final retest of commit
`13b00163493dfef13a49189135f457592a9435cf`.

## Release Gate

```text
v0.45.0 release candidate. Tag only after the local release gate and GitHub
checks pass on the final release commit.
```
