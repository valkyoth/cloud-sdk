# Migrating To v0.47

v0.47 adds local asynchronous execution for browser WASM, embedded, and
single-threaded executors. Pentest remediation also changes the `Send` async
traits to the same non-committing response contract. Blocking contracts remain
unchanged.

## Dependency Versions

```toml
[dependencies]
cloud-sdk = "0.47.0"
cloud-sdk-hetzner = "0.36.1"
cloud-sdk-reqwest = { version = "0.32.0", features = ["async-rustls"] }
cloud-sdk-sanitization = "0.16.0"
cloud-sdk-testkit = "0.27.0"
```

`cloud-sdk-sanitization` is unchanged and is not published. Hetzner and
Hetzner receives a dependency-only patch. Reqwest migrates the async response
contract, and testkit adds local-only mock code.

## Send Async Implementations

Implementations of `AsyncTransport`, `AsyncAuthenticatedTransport`, and
`AsyncRawHttpExecutor` must accept `AsyncResponseStaging` and return
`ResponseCompletion`. Call them through `drive_async`,
`drive_async_authenticated`, or `drive_async_raw`. These drivers own commitment
and cleanup across the await. Blanket compatibility still makes each Send type
implement the corresponding local trait.

## Local-Only Implementations

Implement `LocalAsyncTransport::send_local`,
`LocalAsyncAuthenticatedTransport::send_authenticated_local`, or
`LocalAsyncRawHttpExecutor::execute_local` when a future cannot be `Send`.
Implementations receive `AsyncResponseStaging` and return
`ResponseCompletion`; they cannot commit a response themselves.

Call low-level implementations through `drive_local`,
`drive_local_authenticated`, or `drive_local_raw`. These SDK-owned drivers hold
the cleanup transaction across the await and commit only after `Ready(Ok)`.
All low-level async drivers return `AsyncExecutionError<E>`, separating the
transport's redacted error from SDK response-transaction failure.

Prepared requests, operation-bound provider links, and retry permits expose
`execute_local_async`. Their endpoint, authentication, response, cleanup, and
one-use retry policy is identical to the existing cross-thread path.

## Cancellation Migration

Treat every dropped driver future as `DeliveryPhase::PossiblySent` unless the
transport returns stronger evidence before cancellation. The constant
`ASYNC_CANCELLATION_DELIVERY_PHASE` makes this policy explicit. Cancellation
leaves no committed response and clears complete partial response body and
header storage. Neither Send nor local async implementations receive a commit
capability. Cancellation cannot recall request bytes already accepted by the
network.

Do not infer `NotSent` from a dropped future and do not automatically repeat a
mutation. Use the retry policy from v0.46 or reconcile provider state.

## Testkit

`LocalMockTransport` is a no-allocation, deliberately `!Sync` ordered mock. Use
it to compile and execute local-only provider workflows. Existing
`MockTransport` remains blocking and `Send` async compatible and also gains
the local contract through blanket compatibility.

See [`LOCAL_ASYNC.md`](LOCAL_ASYNC.md) for the complete contract and security
boundaries.
