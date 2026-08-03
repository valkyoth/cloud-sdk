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

Both trait families receive only a non-committing `AsyncResponseStaging` view
and return `ResponseCompletion`. Their SDK drivers commit only after
`Ready(Ok)`. The local traits differ only by removing the future's `Send`
requirement. Neither family adds allocation, networking, TLS, clocks, task
spawning, or an executor.

Every cross-thread transport automatically implements its corresponding local
trait. Existing adapters therefore remain usable through the local API without
duplicate implementations. A transport that returns a genuinely `!Send`
future implements only the local trait.

## Cancellation

Dropping a future cancels caller observation. It does not prove that the
provider did not receive the request. The public
`ASYNC_CANCELLATION_DELIVERY_PHASE` constant is therefore
`DeliveryPhase::PossiblySent`.

Async implementations cannot acquire or commit the driver's `ResponseAttempt`.
Core holds that cleanup-owning attempt across the await and gives the
implementation only staging access. Dropping any async driver before successful
completion rolls back all staged state.

Never retry a mutation merely because its async future was cancelled. Apply
the operation's retry and idempotency policy, or reconcile provider state.

## Local Transport

```rust
use cloud_sdk::transport::{
    AsyncResponseStaging, LocalAsyncTransport, ResponseCompletion,
    ResponseMetadata, StatusCode, TransportRequest,
};

struct BrowserTransport;

impl LocalAsyncTransport for BrowserTransport {
    type Error = ();

    async fn send_local<'transport, 'request, 'writer, 'buffer>(
        &'transport self,
        _request: TransportRequest<'request>,
        mut response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
        'buffer: 'writer,
    {
        response.body_mut().map_err(|_| ())?.fill(0);
        Ok(ResponseCompletion::new(
            StatusCode::NO_CONTENT,
            0,
            ResponseMetadata::EMPTY,
        ))
    }
}
```

Applications invoke implementations through `drive_local`, authenticated
implementations through `drive_local_authenticated`, and raw executors through
`drive_local_raw`. A browser adapter could use JavaScript fetch bindings, while
an embedded adapter could poll a device-specific network stack.

Cross-thread implementations use the equivalent `drive_async`,
`drive_async_authenticated`, and `drive_async_raw` entry points. Direct trait
methods require SDK-created staging and are not response-commit entry points.

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

`cloud-sdk-reqwest` exposes `Send` futures backed by Tokio and uses the same
non-committing staging contract. Its cross-thread implementations receive the
local traits through blanket compatibility, but this does not make reqwest
browser-WASM or embedded compatible. Those targets require another local
transport adapter.

## Security Boundaries

- Cancellation is always possibly sent unless stronger transport evidence is
  available.
- A successful response exists only after the SDK driver receives
  `Ready(Ok(ResponseCompletion))`, commits it, and applies checked policy.
- Endpoint verification and authenticated dispatch use the same bound
  transport object.
- Local retry permits remain one-use and retain the controller's exclusive
  monotonic-state borrow until completion or cancellation.
- Local execution owns no retry, delay, entropy, clock, queue, or scheduler.
