# v0.47.0 Public API Review

Date: 2026-08-03

Scope: provider-neutral local asynchronous execution and cancellation policy.

## Added API

`LocalAsyncTransport`, `LocalAsyncAuthenticatedTransport`, and
`LocalAsyncRawHttpExecutor` mirror the existing cross-thread contracts without
requiring returned futures to implement `Send`. Their methods are explicitly
named `send_local`, `send_authenticated_local`, and `execute_local` to avoid
ambiguous method resolution when a cross-thread implementation also receives
the local blanket implementation.

`PreparedRequest::execute_local_async`,
`ValidatedProviderLink::execute_local_async`, and
`RetryPermit::execute_local_async` preserve existing endpoint, authentication,
response-policy, provider-link, and one-use retry controls for local futures.

`ASYNC_CANCELLATION_DELIVERY_PHASE` exposes the mandatory conservative
`PossiblySent` classification. `LocalMockTransport` gives testkit a
no-allocation, intentionally `!Sync` basic and authenticated fixture.

## Compatibility

Blanket implementations adapt every `AsyncTransport`,
`AsyncAuthenticatedTransport`, and `AsyncRawHttpExecutor` to its local
counterpart. Existing downstream implementations require no source change.
No blanket implementation converts local futures into `Send` futures.

Blocking and cross-thread method signatures are unchanged. Reqwest remains a
Tokio-backed cross-thread adapter and receives local compatibility through the
blanket implementation; no executor or browser support is implied.

## Cancellation And Cleanup

Dropping a local or cross-thread future cancels observation but cannot prove
that request bytes were not delivered. The response remains uncommitted.
Implementations must hold a cleanup-owning `ResponseAttempt` while mutating
body or header storage; cancellation drops that guard and clears partial
state. Prepared execution additionally owns a cleanup-owning `ResponseBuffer`.

Regression tests poll a genuinely local future until it writes sensitive body
and header bytes, drop it while pending, and prove the next attempt observes
cleared storage. Separate tests keep two local futures cooperatively
outstanding, execute prepared requests and retry permits through a `!Sync`
mock, and prove cross-thread transports satisfy local traits automatically.

## Security Boundaries

The SDK owns no executor, task, wake policy, clock, delay, timeout, entropy,
transport implementation, or concurrency limit. `&self` permits concurrency
only when the implementation and caller support it. Cancellation remains
possibly sent, so mutation repetition still requires the v0.46 retry and
idempotency policy or provider reconciliation.
