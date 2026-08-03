# v0.47.0 Public API Review

Date: 2026-08-03

Scope: provider-neutral local asynchronous execution and cancellation policy.

## Added API

`LocalAsyncTransport`, `LocalAsyncAuthenticatedTransport`, and
`LocalAsyncRawHttpExecutor` accept non-committing `AsyncResponseStaging` and
return `ResponseCompletion` without requiring returned futures to implement
`Send`. The existing Send async traits now use the same staging and completion
types. `drive_async`, `drive_async_authenticated`, `drive_async_raw`, and their
local counterparts own the cleanup transaction and perform final commitment
after `Ready(Ok)`. `AsyncExecutionError<E>` separates transport failure from
SDK response-transaction failure.

`PreparedRequest::execute_local_async`,
`ValidatedProviderLink::execute_local_async`, and
`RetryPermit::execute_local_async` preserve existing endpoint, authentication,
response-policy, provider-link, and one-use retry controls for local futures.

`ASYNC_CANCELLATION_DELIVERY_PHASE` exposes the mandatory conservative
`PossiblySent` classification. `LocalMockTransport` gives testkit a
no-allocation, intentionally `!Sync` basic and authenticated fixture.

## Compatibility

Blanket implementations adapt every updated `AsyncTransport`,
`AsyncAuthenticatedTransport`, and `AsyncRawHttpExecutor` to its local
counterpart. Downstream Send async implementations must migrate from
`ResponseWriter` plus `Result<(), E>` to `AsyncResponseStaging` plus
`Result<ResponseCompletion, E>` and callers must use the corresponding driver.
No blanket implementation converts local futures into `Send` futures.

Blocking method signatures are unchanged. Reqwest remains a Tokio-backed
cross-thread adapter and receives local compatibility through the blanket
implementation; no executor or browser support is implied.

## Cancellation And Cleanup

Dropping any async driver future cancels observation but cannot prove that
request bytes were not delivered. Implementations cannot access `commit()`.
Core holds a cleanup-owning `ResponseAttempt` while the implementation mutates
body or header staging, and commits returned completion metadata only after
success. Send and local implementations have no safe access to that attempt or
its commit operation.

Regression tests poll a genuinely local future until it writes sensitive body
and header bytes, drop it while pending, and prove the next attempt observes
cleared storage. Another test stages sensitive bytes through a Send transport,
suspends, cancels, and proves all bytes are cleared. Separate tests keep two local futures cooperatively
outstanding, execute prepared requests and retry permits through a `!Sync`
mock, and prove cross-thread transports satisfy local traits automatically.

## Security Boundaries

The SDK owns no executor, task, wake policy, clock, delay, timeout, entropy,
transport implementation, or concurrency limit. `&self` permits concurrency
only when the implementation and caller support it. Cancellation remains
possibly sent, so mutation repetition still requires the v0.46 retry and
idempotency policy or provider reconciliation.
