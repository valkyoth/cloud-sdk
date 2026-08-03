# Migrating To v0.47

v0.47 adds local asynchronous execution for browser WASM, embedded, and
single-threaded executors without changing existing blocking or `Send` async
contracts.

## Dependency Versions

```toml
[dependencies]
cloud-sdk = "0.47.0"
cloud-sdk-hetzner = "0.36.1"
cloud-sdk-reqwest = { version = "0.31.2", features = ["async-rustls"] }
cloud-sdk-sanitization = "0.16.0"
cloud-sdk-testkit = "0.27.0"
```

`cloud-sdk-sanitization` is unchanged and is not published. Hetzner and
reqwest receive dependency-only patches. Testkit adds local-only mock code.

## Existing Async Implementations

No implementation change is required for types that already implement
`AsyncTransport`, `AsyncAuthenticatedTransport`, or `AsyncRawHttpExecutor`.
Blanket compatibility makes each type implement the corresponding local trait.
Their futures remain `Send` and may be used through either API.

## Local-Only Implementations

Use `LocalAsyncTransport::send_local`,
`LocalAsyncAuthenticatedTransport::send_authenticated_local`, or
`LocalAsyncRawHttpExecutor::execute_local` when a future cannot be `Send`.
These methods own no executor and may borrow `!Sync` platform state.

Prepared requests, operation-bound provider links, and retry permits expose
`execute_local_async`. Their endpoint, authentication, response, cleanup, and
one-use retry policy is identical to the existing cross-thread path.

## Cancellation Migration

Treat every dropped async future as `DeliveryPhase::PossiblySent` unless the
transport returns stronger evidence before cancellation. The constant
`ASYNC_CANCELLATION_DELIVERY_PHASE` makes this policy explicit. Cancellation
leaves no committed response and clears partial response body and header
storage, but cannot recall request bytes already accepted by the network.

Do not infer `NotSent` from a dropped future and do not automatically repeat a
mutation. Use the retry policy from v0.46 or reconcile provider state.

## Testkit

`LocalMockTransport` is a no-allocation, deliberately `!Sync` ordered mock. Use
it to compile and execute local-only provider workflows. Existing
`MockTransport` remains blocking and `Send` async compatible and also gains
the local contract through blanket compatibility.

See [`LOCAL_ASYNC.md`](LOCAL_ASYNC.md) for the complete contract and security
boundaries.
