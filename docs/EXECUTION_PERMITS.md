# Plan-Confirm Execution Permits

`cloud-sdk` requires explicit execution authority for every prepared request
whose metadata is mutating, destructive, or cost-bearing. Read-only requests
retain direct blocking, Send-async, and local-async execution. Type erasure
does not bypass the neutral runtime check.

## Security Purpose

A permit proves that a caller reviewed one exact plan, not merely an operation
name. The versioned `cloud-sdk/plan-confirm/v1` canonical input binds:

- provider, service, operation, method, and canonical endpoint identity;
- exact path, query, selected header names/values/sensitivity, and body;
- account, tenant, and caller review context;
- permit scope, issuance, expiry, replay policy, and attempt budget;
- exact idempotency identity when reconciliation can authorize repetition; and
- currency, observed price, and spending ceiling for cost-bearing requests.

Use `build_canonical_plan` when bounded exact comparison is appropriate. Use
`build_plan_digest` only with a caller-supplied collision-resistant
`FingerprintHasher` implementing one of the admitted algorithms. Rust `Hash`,
CRC, truncated checksums, and caller-asserted detached digests are not plan
identity. Canonical and digest storage is redacted and volatile-cleared on
failure, panic, and drop.

## Construction

1. Prepare the provider operation into cleanup-owning caller buffers.
2. Inspect the final request and current provider state. Reject a no-op.
3. Obtain caller-owned time, account/tenant identity, review context, price,
   spending ceiling, and fresh idempotency entropy where required.
4. Construct `PlanConfirmation` and build its exact or strong-digest identity.
5. Select `MutationPermit`, `DestructivePermit`, or `CostPermit` from the
   fingerprint's `PlanSubject`. A scope mismatch fails before execution.
6. Call `begin`, then consume the returned `PermitAttempt` through
   `execute_blocking`, `execute_async`, or `execute_local_async`.

The SDK has no clock, random source, price feed, account inventory, or remote
state oracle. The caller must obtain trustworthy values and compare desired
state with current state. `PlanChange::ChangesState` is an assertion reviewed
by the caller; the SDK cannot infer a provider-side no-op from wire bytes.

## Direct And Shared Authority

Direct permits are neither `Copy` nor `Clone` and allow one in-flight attempt.
Use them unless authority genuinely must be shared.

`SharedPermitState` supports explicit shared handles. Every clone references
the same caller-owned atomic state, remaining attempt budget, clock
observation, and recovery generation. Cloning never creates new authority;
dropping a handle never restores budget. One compare-and-exchange transition
owns the only in-flight attempt, so concurrent clones cannot double-spend.

The shared state uses portable core atomics and remains allocation-free and
`no_std`. The caller owns the state for longer than every handle and controls
thread/task admission.

## Delivery And Repetition

| Outcome | Permit state | Required action |
| --- | --- | --- |
| Checked success | `Spent` | None; authority cannot be reused. |
| Proven `NotSent` | `Recoverable` or `Spent` | Use the exact generation token only when replay policy and remaining budget allow it. |
| `PossiblySent` | `PendingReconciliation` | Query provider state with operation-specific logic before any repetition. |
| `ResponseStarted` but execution failed | `PendingReconciliation` | Treat delivery as uncertain and reconcile. |
| Attempt dropped or cancelled | `PendingReconciliation` | Never infer not-sent from cancellation. |

Manual `PermitAttempt::complete(DeliveryPhase::NotSent)` is sound only behind
a transport boundary that proves no request bytes reached the peer. Unknown
state is `PossiblySent`. Prefer the execute methods, which require a
`DeliveryClassified` transport error and apply the transition automatically.

`recover_not_sent` accepts only the token for the current recoverable
generation. `reconcile_not_applied` additionally requires
`ReplayPolicy::ReconcileThenRetry`, the exact plan fingerprint, and the exact
idempotency identity. Calling it asserts that an operation-specific provider
read proved the uncertain attempt was not applied. A generic transport error,
timeout, missing response body, or repeated GET alone is not proof.

Attempt budgets include every transport attempt. Exhaustion, stale tokens,
fingerprint mismatch, idempotency mismatch, expiry, not-yet-valid time, and a
caller clock moving backward all fail closed. A backward observation never
extends validity.

## Cost Authority

Cost permits require an exact three-letter uppercase currency code, decimal
scale, nonzero observed price, and spending ceiling in the same `u128` scaled
units. Floating-point prices are deliberately excluded. The observed price
must not exceed the ceiling when the plan is built, and every value is bound
into its identity.

Provider prices can change between confirmation and execution. Choose a short
validity interval, re-fetch pricing where appropriate, and rebuild the plan
when any price or request input changes. The SDK does not guarantee provider
billing behavior.

## Secret And Cleanup Boundaries

Account, tenant, review context, idempotency identity, request body, and
canonical input may be sensitive. Diagnostics redact these values, but
borrowed source memory remains caller-owned. Generate idempotency bytes with a
CSPRNG, keep mutable sources in `cloud-sdk-sanitization` guards where
appropriate, rotate according to application policy, and clear caller copies
after the permit and transport lifetimes end.

Cleanup cannot cover deliberately leaked values, immutable external copies,
allocator/TLS/kernel buffers, process abort, swap, crash dumps, or remote
provider systems.
