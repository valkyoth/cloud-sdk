# cloud-sdk 0.46.0 Release Notes

Status: implementation stop reached; pentest required.

Release date: pending

## Overview

v0.46 adds exact request replay identity, fresh local idempotency binding, and
one bounded provider-neutral retry owner without changing the default no_std
dependency boundary.

## Retry And Idempotency

- Added a versioned, domain-separated canonical fingerprint over provider,
  service, operation, method, endpoint identity, exact path/query, prepared
  headers, body, and optional account scope.
- Added caller-buffer exact fingerprints and caller-supplied SHA-256, SHA-384,
  SHA-512, or BLAKE3 digest contracts with exact output validation.
- Added cleanup-owning, redacted caller-buffer canonical and digest storage.
- Bound prepared request policy and fingerprint identity into private-field
  `RetrySubject` values and reject endpoints outside prepared service policy.
- Added non-cloneable fresh `IdempotencyIntent` guards and fingerprint-bound
  `IdempotencyBinding` values; valid intent bytes remain in one borrowed
  caller location and clear on drop, while invalid sources clear immediately.
- Added nonzero total attempts, cumulative requested-delay, and monotonic
  elapsed budgets under one non-cloneable `RetryController`.
- Added conservative delivery-phase and `429`/`5xx` response decisions with
  exact replay mismatch, rollback, overflow, and exhaustion rejection.
- Reject identical-wire replays when service, operation, authentication,
  response, raw-response, operation-identity, or body-replay policies differ.
- Included requested delay in the hard elapsed deadline and added one-use
  `RetryPermit` blocking and async execution. A permit exclusively borrows
  controller clock state, advances it after sleep, and never returns a
  reusable prepared request.
- Used reviewed `subtle` fixed-time primitives for equal-length fingerprint
  comparison and all-zero intent validation.
- Added explicit prepared-body replayability. Hetzner's transactionally
  prepared immutable byte snapshots are replayable; nonempty custom provider
  bodies fail closed until explicitly marked.
- Kept reqwest transport calls single-attempt and made its release
  dependency-only.
- Updated the optional transport graph's transitive `ipnet` dependency from
  `2.12.0` to `2.12.1` after registry freshness review.
- Added known SHA-256, canonical field, cleanup, intent, mutation, delivery,
  endless-transient, attempt, delay, elapsed, rollback, identical-wire policy
  laundering, compile-fail fan-out, and blocking/async permit tests.

## Versions

| Crate | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.46.0` | retry, fingerprint, and idempotency code |
| `cloud-sdk-hetzner` | `0.36.0` | prepared replayability source lock |
| `cloud-sdk-reqwest` | `0.31.1` | dependency-only patch |
| `cloud-sdk-sanitization` | `0.16.0` | unchanged; not published |
| `cloud-sdk-testkit` | `0.26.0` | replayability record code |

## Documentation

- [`docs/RETRY_AND_IDEMPOTENCY.md`](../docs/RETRY_AND_IDEMPOTENCY.md)
- [`docs/MIGRATION_0.46.0.md`](../docs/MIGRATION_0.46.0.md)
- [`docs/PUBLIC_API_REVIEW_0.46.0.md`](../docs/PUBLIC_API_REVIEW_0.46.0.md)
- [`docs/DEPENDENCY_REVIEW_0.46.0.md`](../docs/DEPENDENCY_REVIEW_0.46.0.md)

## Pentest

Pentest is required for the exact implementation-stop commit. Temporary
findings belong in root `PENTEST.md` and must be removed after remediation.

## Release Gate

```text
v0.46.0 implementation stop reached. Run pentest for this exact commit.
```
