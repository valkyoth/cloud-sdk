# Local Async Contract

`cloud-sdk` separates cross-thread asynchronous execution from local
single-threaded execution. Both contracts remain `no_std`, allocation-free,
runtime-neutral, and executor-neutral.

## Contract Families

| Layer | Cross-thread future | Local `!Send` future |
| --- | --- | --- |
| Basic transport | `AsyncTransport` | `LocalAsyncTransport` |
| Authenticated transport | `AsyncAuthenticatedTransport` | `LocalAsyncAuthenticatedTransport` |
| Raw HTTP executor | `AsyncRawHttpExecutor` | `LocalAsyncRawHttpExecutor` |
| Prepared request | `execute_async` | `execute_local_async` |
| Provider pagination link | `execute_async` | `execute_local_async` |
| Retry permit | `execute_async` | `execute_local_async` |

The local traits remove only the `Send` requirement from returned futures.
They do not add allocation, networking, TLS, clocks, task spawning, or an
executor. Browser WASM, embedded, and single-threaded applications still
provide a platform transport implementation and drive its future themselves.

Every cross-thread transport automatically implements its corresponding local
trait. Existing adapters therefore remain usable through the local API without
duplicate implementations. A transport that returns a genuinely `!Send`
future implements only the local trait.

## Cancellation

Dropping a future cancels caller observation. It does not prove that the
provider did not receive the request. The public
`ASYNC_CANCELLATION_DELIVERY_PHASE` constant is therefore
`DeliveryPhase::PossiblySent`.

Every implementation must acquire `ResponseWriter::begin_attempt` before
writing. Dropping an uncommitted attempt clears the complete admitted body and
header storage. Prepared and retry execution own a `ResponseBuffer`; pagination
execution borrows its caller's cleanup-enforcing `ResponseWriter`. Dropping any
of those futures therefore clears uncommitted partial response state.

Never retry a mutation merely because its local future was cancelled. Apply
the operation's retry and idempotency policy, or reconcile provider state.

## Local Transport

```rust
use cloud_sdk::transport::{
    LocalAsyncTransport, ResponseMetadata, ResponseWriter, StatusCode,
    TransportRequest,
};

struct BrowserTransport;

impl LocalAsyncTransport for BrowserTransport {
    type Error = ();

    async fn send_local<'transport, 'request, 'writer>(
        &'transport self,
        _request: TransportRequest<'request>,
        response: &'writer mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
    {
        let mut attempt = response.begin_attempt().map_err(|_| ())?;
        attempt
            .commit(StatusCode::NO_CONTENT, 0, ResponseMetadata::EMPTY)
            .map_err(|_| ())
    }
}
```

This example is intentionally transport-mechanism neutral. A browser adapter
could use JavaScript fetch bindings, while an embedded adapter could poll a
device-specific network stack. Neither belongs in the core crate.

## Concurrency

Local futures may be outstanding together and cooperatively polled on one
thread when the implementation supports that pattern. The shared `&self`
receiver does not itself guarantee concurrent safety. Callers own task count,
fairness, cancellation, and concurrency limits. Implementations may use local
interior mutability and remain `!Sync`.

`cloud-sdk-testkit::LocalMockTransport` is deliberately `!Sync` and implements
the local basic and authenticated contracts. It supports deterministic
prepared-request tests without introducing a runtime.

## Reqwest Boundary

`cloud-sdk-reqwest` continues to expose `Send` futures backed by Tokio. Its
cross-thread implementations receive the local traits through the blanket
compatibility implementation, but this does not make reqwest browser-WASM or
embedded compatible. Those targets require another local transport adapter.

## Security Boundaries

- Cancellation is always possibly sent unless stronger transport evidence is
  available.
- A successful response exists only after explicit writer commitment and
  checked response policy.
- Endpoint verification and authenticated dispatch use the same bound
  transport object.
- Local retry permits remain one-use and retain the controller's exclusive
  monotonic-state borrow until completion or cancellation.
- Local execution owns no retry, delay, entropy, clock, queue, or scheduler.
