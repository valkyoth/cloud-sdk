# cloud-sdk 0.64.0 Milestone Notes

Status: implementation stop reached; pentest required.

Release date: 2026-08-08

Security-Review: PASS
Pentest: PENDING
Publication: DEFERRED TO v0.65.0

## Overview

v0.64 completes Cloud actions, exact metrics, composite results, and shared
exact scalar models. It is an internal tag and publishes no crate. The provider
package version remains 0.39.1 while changes accumulate for v0.65.0.

## Cloud Special Models

- Added canonical UTC timestamps with calendar, leap-year, fraction, and
  uppercase `T`/`Z` validation.
- Added exact bounded decimal tokens preserving integer, fractional, exponent,
  and negative-zero forms without public binary-float coercion.
- Added bounded server/load-balancer metrics with positive exact steps,
  per-series and aggregate point limits, fallible copies, and redacted
  diagnostics.
- Completed action identifiers, commands, status, progress, start/finish,
  resources, nullable errors, protected messages, and retained future error
  codes.
- Preserved composite singular, collection, and follow-up actions separately
  and distinguished absent, null, and protected secret outputs.

## Security And Verification

- Limited every strict-JSON number token to 128 bytes before allocation and
  made lexical-storage allocation failure recoverable.
- Derived metric sign and zero classification from exact lexical tokens,
  covering underflow-sized exponents and signed zero without float coercion.
- Restricted borrowed, provider, action, and certificate error-code text to a
  shared bounded ASCII machine-identifier grammar.
- Enforced source-max action resource IDs and operation-specific secret
  nullability.
- Added exact-number, UTC/calendar, metrics amplification, action/error,
  composite-nullability, redaction, all-operation, dual-decoder fuzz, and named
  seed-route coverage.
- Retained exact pinned upstream operation and schema drift gates.

## Versions

| Crate | Source version | Cumulative change | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.64.0` | metadata | deferred to v0.65.0 |
| `cloud-sdk-hetzner` | `0.39.1` | code | deferred |
| `cloud-sdk-reqwest` | `0.33.0` | code | deferred |
| `cloud-sdk-sanitization` | `0.18.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.29.1` | code | deferred |

## Release Evidence

- [`docs/PUBLIC_API_REVIEW_0.64.0.md`](../docs/PUBLIC_API_REVIEW_0.64.0.md)
- [`docs/DEPENDENCY_REVIEW_0.64.0.md`](../docs/DEPENDENCY_REVIEW_0.64.0.md)
- [`docs/THREAT_MODEL_DELTA_0.64.0.md`](../docs/THREAT_MODEL_DELTA_0.64.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.64.0.md`](../docs/REJECTED_ABSTRACTIONS_0.64.0.md)
- [`docs/MIGRATION_0.64.0.md`](../docs/MIGRATION_0.64.0.md)

## Release Gate

Pentest this exact implementation-stop commit. After remediation and a green
retest, add the permanent v0.64 report and run `scripts/release_0_64_gate.sh`
on the clean evidence commit. GitHub CI and CodeQL must be green on that
unchanged commit before the signed internal tag. Do not publish crates.
