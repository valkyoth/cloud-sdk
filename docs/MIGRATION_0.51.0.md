# Migrating Source Users To v0.51

v0.51 is an internal tagged development milestone and is not separately
published to crates.io. Published users remain on v0.50 until the cumulative
v0.55 checkpoint. Git users tracking source must account for the execution
boundary changes below.

## State-Changing Execution

`PreparedRequest::execute_blocking`, `execute_async`, and
`execute_local_async` now return
`PreparedExecutionError::AuthorizationRequired` before transport for mutation,
destructive, and cost-bearing metadata. Complete response body and header
storage is cleared on this rejection.

Build a `PlanConfirmation`, call `build_canonical_plan` or
`build_plan_digest`, construct the permit matching `PlanSubject::scope`, then
execute through the one-use `PermitAttempt`. See
[`EXECUTION_PERMITS.md`](EXECUTION_PERMITS.md).

## Typed Hetzner Operations

`cloud_sdk_hetzner::association::Prepared<O>` exposes direct execution only
for sealed read-only `NoPermit` operation markers. Mutation, destructive, and
cost markers retain response validation and explicit access to the neutral
prepared request, but state-changing execution requires the neutral permit.
Explicit `into_untyped` erasure does not restore direct execution.

## Transport Errors

Permit execution requires transport errors implementing `DeliveryClassified`.
Use `TransportFailure<E>` or implement the trait conservatively. Only failures
proven to precede all request delivery are `NotSent`; unknown state is
`PossiblySent`, and an observed response head is `ResponseStarted`.

Custom code manually completing an attempt must use `NotSent` only with the
same proof. Cancellation and timeout do not establish that proof.

## Retry Integration

The existing retry controller does not replace execution authority. A
state-changing retry must satisfy both the retry/idempotency policy and the
plan-confirm recovery or reconciliation state. Existing direct prepared retry
execution will fail closed until integrated with the permit lifecycle.

## Compatibility

No default feature, allocation, network, TLS, runtime, filesystem, clock, or
random dependency was added. The default provider-neutral and Hetzner graphs
remain allocation-free and `no_std`.
