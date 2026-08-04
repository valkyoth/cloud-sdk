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

Every permit execute method now takes a caller-owned `PermitClock` before the
transport argument. The SDK samples it at actual blocking dispatch or first
async poll, and `expires_at` is exclusive. Queued attempts that expire are
spent before transport access.

The endpoint confirmed into the plan is enforced exactly during dispatch.
Another endpoint admitted by the same provider policy no longer matches.

## Authenticated Transport Capability

`AuthenticatedRequest::new`, `PreparedRequest::authenticated_request`, and
`PermitAttempt::prepared` are no longer public. Source users must execute
read-only requests through `PreparedRequest::execute_*` and state-changing
requests through `PermitAttempt::execute_*`. Transport implementations still
receive `AuthenticatedRequest`, but application and provider code cannot forge
or extract that wire capability.

## Typed Hetzner Operations

`cloud_sdk_hetzner::association::Prepared<O>` exposes direct execution only
for sealed read-only `NoPermit` operation markers. Mutation, destructive, and
cost markers retain response validation and explicit access to the neutral
prepared request, but state-changing execution requires the neutral permit.
Explicit `into_untyped` erasure does not restore direct execution.
The provider-neutral constructor also rejects read-only metadata for methods
other than `GET` and `HEAD`, and dispatch independently permit-gates every
other method.

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

No default feature, allocation, network, TLS, runtime, filesystem, system-clock,
or random dependency was added. `PermitClock` is a caller implementation. The
default provider-neutral and Hetzner graphs remain allocation-free and
`no_std`.
