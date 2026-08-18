# cloud-sdk 0.51.0 Milestone Notes

Status: internal tagged development milestone.

Release date: pending

Security-Review: PASS
Pentest: DEFERRED TO v0.55.0
Publication: DEFERRED TO v0.55.0

## Overview

v0.51 enforces plan-confirm authority at the execution boundary for mutation,
destructive, and cost-bearing requests. This milestone receives the complete
local and GitHub release gates and a normal signed tag. Its changes are
included in the cumulative v0.55 pentest and crates.io publication.

## Execution Authority

- Added distinct versioned exact and caller-hashed plan fingerprints over the
  complete request, endpoint, account/tenant, review context, validity,
  replay, attempt, idempotency, scope, and cost policy.
- Added non-copyable direct permits and explicit shared atomic permits whose
  clones retain one state, attempt budget, clock observation, and generation.
- Added generation-bound proven-not-sent recovery and operation-specific
  reconciliation for uncertain delivery.
- Added no-op, stale plan, scope, cost ceiling, expiry, rollback, replay,
  idempotency, and concurrent-spend rejection.
- Added cleanup and redaction for canonical inputs, digests, errors, and panic
  paths without changing the default no_std graph.
- Added conservative `DeliveryClassified` transport failures for blocking,
  Send-async, and local-async execution.
- Made authenticated request construction and extraction internal, and removed
  reusable prepared-request access from permit attempts.
- Retained and enforced the exact confirmed endpoint at authorized dispatch,
  including within multi-endpoint official policies.
- Added caller-owned `PermitClock` dispatch sampling, exclusive expiry, and
  fail-closed spending of attempts that expire before transport access.

## Provider Integration

- Direct provider-neutral prepared execution now rejects state-changing and
  cost-bearing metadata before transport and clears response storage.
- Hetzner `Prepared<O>` exposes direct execution only for source-locked
  read-only operation markers; explicit type erasure cannot bypass the neutral
  permit check.
- The operation-association generator now owns the read-only marker binding.

## Versions

| Crate | Source version | Publication |
| --- | --- | --- |
| `cloud-sdk` | `0.51.0` | deferred to v0.55.0 |
| `cloud-sdk-hetzner` | `0.38.0` | retained; accumulated code deferred |
| `cloud-sdk-reqwest` | `0.32.3` | retained; dependency accumulation deferred |
| `cloud-sdk-sanitization` | `0.17.0` | unchanged |
| `cloud-sdk-testkit` | `0.28.2` | retained; accumulated test code deferred |

## Documentation

- [`docs/EXECUTION_PERMITS.md`](../docs/EXECUTION_PERMITS.md)
- [`docs/MIGRATION.md#v0510`](../docs/MIGRATION.md#v0510)
- [`docs/PUBLIC_API_REVIEW.md#v0510`](../docs/PUBLIC_API_REVIEW.md#v0510)
- [`docs/DEPENDENCY_REVIEW.md#v0510`](../docs/DEPENDENCY_REVIEW.md#v0510)

## Release Gate

Tag only after the clean local v0.51 gate plus GitHub CI and CodeQL pass on the
final milestone commit. No v0.51 pentest report or crates.io publication is
created under the ordinary cumulative cadence.
