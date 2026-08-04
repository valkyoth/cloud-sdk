# Pager And Action Workflow Drivers

`cloud-sdk` provides pure state drivers for multi-request workflows. Core does
not read clocks, sleep, execute transport, allocate, spawn tasks, or retry a
mutation.

## Pager Contract

`PagerDriver<S>` wraps a `PageStrategy` and admits exactly one request token
before one decoded response observation. Calling `next_request` twice fails
with `ResponsePending`; observing without an admitted request fails with
`UnexpectedObservation`. Cancellation is explicit and terminal.

`NumberedPagination` and `OffsetPagination` implement `PageStrategy`. Their
existing request, item, snapshot, navigation, and traversal checks remain
transactional. A rejected response leaves the request pending and does not
advance provider state. Provider-specific cursor, marker, and raw-link state
continues to use its dedicated exact-byte ownership contracts.

## Action Contract

`ActionPoller` is a two-phase state machine:

1. `next_request(PollControl, MonotonicInstant)` admits a request, returns the
   remaining delay, cancels, or times out.
2. `observe(...)` accepts exactly one provider response for that request.
3. A running response asks a separate `PollBackoff` for a delay and validates
   it against per-delay, cumulative-delay, observation, and elapsed limits.
4. Success, provider failure, cancellation, exhaustion, rollback, and policy
   failure are terminal.

The caller selects `ActionPollLimits` before the first request. A running
observation at the hard observation bound fails closed even if custom backoff
would continue. Zero delay is rejected, so policy cannot create an implicit
busy loop.

## Progress Policy

`ProgressPolicy::Nondecreasing` rejects regression and every reset.
`ExplicitResets` admits only `ProgressObservation::Reset` and enforces a hard
reset count. `Unordered` validates percent bounds while leaving ordering to the
provider. Backoff receives the resulting `ProgressChange`; the built-in
`ExponentialBackoff` resets after initial, advanced, or explicitly reset
progress and remains capped.

## Time Domains

`MonotonicInstant` and `MonotonicDuration` exclusively drive local delay,
timeout, and elapsed budgets. Backward monotonic observations terminate the
workflow. A delay that reaches the elapsed deadline is rejected before another
request can be admitted.

`ProviderTimeObservation` carries only provider-reported wall-clock observation
and expiry timestamps. It may be passed to backoff as telemetry, but the driver
never converts it into local elapsed time. Provider wall-clock rollback cannot
extend a workflow.

## Error And Data Handling

Structural errors use payload-free `Display` and `Debug`. `ActionObserveError`
retains a caller policy error for programmatic handling but redacts it from
both formats. `ActionUpdate::Failed(E)` and `ActionPollStep::Failed(E)` also
redact `E` from Debug. Providers remain responsible for ensuring their typed
error values own and clear any sensitive payload.

Neither driver is `Copy` or `Clone`. Drop the owner to abandon a workflow.
Transport cancellation and uncertain mutation delivery remain governed by the
operation permit and delivery-phase contracts.
