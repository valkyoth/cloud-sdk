# v0.51.0 Public API Review

Date: 2026-08-04

Scope: provider-neutral plan confirmation, execution authority, conservative
delivery classification, and typed Hetzner execution gating.

## Added API

`cloud_sdk::operation` exports bounded plan-confirm values, exact and
strong-digest fingerprint builders, direct mutation/destructive/cost permits,
explicit shared atomic permits, lifecycle tokens and dispositions, and
payload-free errors. `cloud_sdk::transport` exports `DeliveryClassified` and
`TransportFailure<E>`.

All constructors require complete policy input. There is no permissive
`Default` for authority, no weak digest selection, and no API that extracts a
reusable state-changing request from a permit attempt.

## Changed API Behavior

Direct `PreparedRequest` execution rejects state-changing or cost-bearing
metadata before transport. Hetzner typed direct execution is available only
for generated sealed read-only markers. This is an intentional pre-1.0 safety
break documented in `MIGRATION_0.51.0.md`.

## Ownership And Concurrency

Direct permits are non-`Copy` and non-`Clone`. Shared handle clones reference
one caller-owned `SharedPermitState`; atomic lifecycle, remaining budget,
recovery generation, and monotonic wall-clock observation are never copied.
Only one attempt can transition from ready to in-flight.

## Failure Contract

Public errors implement payload-free `Display`, redacted `Debug`, and
`core::error::Error`. Transport payloads are never formatted. Unknown delivery
is conservatively possibly sent. Failed, panicking, or dropped canonical and
digest construction clears caller output; successful guards clear on drop.

## Compatibility Boundaries

The API remains allocation-free and `no_std`, uses portable core atomics, and
adds no dependency or feature. Callers continue to own clocks, entropy,
pricing, reconciliation reads, retries, scheduling, storage, and transport.
