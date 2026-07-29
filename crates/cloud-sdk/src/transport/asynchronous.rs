//! Runtime-neutral asynchronous transport contract.

use core::future::Future;

use super::{ResponseWriter, TransportRequest};

/// Asynchronous transport over caller-owned request and response buffers.
///
/// The contract does not select an executor, allocator, HTTP client, TLS
/// implementation, clock, or retry policy. Adapter crates document any runtime
/// requirements they add.
///
/// The shared receiver does not create concurrency. Callers may overlap or
/// spawn returned futures only when the concrete implementation and future
/// satisfy their executor's `Sync`, `Send`, and lifetime requirements.
///
/// Implementations must treat cancellation as an error path: dropping the
/// returned future must not expose a partially initialized response as a
/// successful response. Implementations handling secret response
/// data should also clear temporary owned storage when the future is dropped.
pub trait AsyncTransport {
    /// Transport-specific failure.
    type Error;

    /// Sends one request and initializes the complete response body in the
    /// caller buffer.
    ///
    /// Implementations should use [`ResponseWriter::begin_attempt`] so failed
    /// and cancelled operations cannot leave reusable response state dirty.
    fn send<'transport, 'request, 'writer>(
        &'transport self,
        request: TransportRequest<'request>,
        response: &'writer mut ResponseWriter<'_>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'writer
    where
        'transport: 'writer,
        'request: 'writer;
}
