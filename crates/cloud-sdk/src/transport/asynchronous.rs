//! Runtime-neutral asynchronous transport contract.

use core::future::Future;

use super::{TransportRequest, TransportResponse};

/// Asynchronous transport over caller-owned request and response buffers.
///
/// The contract does not select an executor, allocator, HTTP client, TLS
/// implementation, clock, or retry policy. Adapter crates document any runtime
/// requirements they add.
///
/// Implementations must treat cancellation as an error path: dropping the
/// returned future must not expose a partially initialized response as a
/// successful [`TransportResponse`]. Implementations handling secret response
/// data should also clear temporary owned storage when the future is dropped.
pub trait AsyncTransport {
    /// Transport-specific failure.
    type Error;

    /// Sends one request and initializes the complete response body in the
    /// caller buffer.
    fn send<'transport, 'request, 'buffer>(
        &'transport mut self,
        request: TransportRequest<'request>,
        response_body: &'buffer mut [u8],
    ) -> impl Future<Output = Result<TransportResponse<'buffer>, Self::Error>> + Send + 'transport
    where
        'request: 'transport,
        'buffer: 'transport;
}
