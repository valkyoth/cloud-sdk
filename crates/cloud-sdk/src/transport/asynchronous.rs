//! Runtime-neutral asynchronous transport contract.

use core::future::Future;

use super::{DeliveryPhase, ResponseWriter, TransportRequest};

/// Conservative delivery classification after an asynchronous future is
/// cancelled by being dropped.
///
/// Cancellation proves only that no successful response was committed. It
/// does not prove that request bytes were not delivered, so mutation policy
/// must treat the operation as possibly sent.
pub const ASYNC_CANCELLATION_DELIVERY_PHASE: DeliveryPhase = DeliveryPhase::PossiblySent;

/// Local asynchronous transport for `!Send` futures and executors.
///
/// This contract is suitable for browser WASM, embedded executors, and
/// single-threaded runtimes. It owns no executor and does not imply that two
/// returned futures may be polled concurrently; that remains a property of
/// the implementation and caller.
///
/// Dropping the returned future cancels caller observation, not necessarily
/// remote execution. Implementations must leave the response uncommitted and
/// clear partial body and header state. Callers must classify the request as
/// [`ASYNC_CANCELLATION_DELIVERY_PHASE`] unless stronger transport evidence is
/// available.
///
/// A local implementation may deliberately return a future that is not
/// `Send`:
///
/// ```compile_fail
/// use core::cell::Cell;
/// use cloud_sdk::transport::{
///     LocalAsyncTransport, ResponseWriter, TransportRequest,
/// };
///
/// struct Local(Cell<()>);
///
/// impl LocalAsyncTransport for Local {
///     type Error = ();
///
///     fn send_local<'transport, 'request, 'writer>(
///         &'transport self,
///         _request: TransportRequest<'request>,
///         _response: &'writer mut ResponseWriter<'_>,
///     ) -> impl core::future::Future<Output = Result<(), Self::Error>> + 'writer
///     where
///         'transport: 'writer,
///         'request: 'writer,
///     {
///         async move { self.0.get(); Ok(()) }
///     }
/// }
///
/// fn require_send<T: Send>(_: T) {}
/// fn reject_send(
///     transport: &Local,
///     request: TransportRequest<'_>,
///     response: &mut ResponseWriter<'_>,
/// ) {
///     require_send(transport.send_local(request, response));
/// }
/// ```
pub trait LocalAsyncTransport {
    /// Transport-specific failure.
    type Error;

    /// Sends one request without requiring the returned future to be `Send`.
    fn send_local<'transport, 'request, 'writer>(
        &'transport self,
        request: TransportRequest<'request>,
        response: &'writer mut ResponseWriter<'_>,
    ) -> impl Future<Output = Result<(), Self::Error>> + 'writer
    where
        'transport: 'writer,
        'request: 'writer;
}

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
    /// Implementations must use [`ResponseWriter::begin_attempt`]. Response
    /// mutation and commitment are available only through the returned guard,
    /// which clears state when the future is cancelled.
    fn send<'transport, 'request, 'writer>(
        &'transport self,
        request: TransportRequest<'request>,
        response: &'writer mut ResponseWriter<'_>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'writer
    where
        'transport: 'writer,
        'request: 'writer;
}

impl<T> LocalAsyncTransport for T
where
    T: AsyncTransport + ?Sized,
{
    type Error = T::Error;

    fn send_local<'transport, 'request, 'writer>(
        &'transport self,
        request: TransportRequest<'request>,
        response: &'writer mut ResponseWriter<'_>,
    ) -> impl Future<Output = Result<(), Self::Error>> + 'writer
    where
        'transport: 'writer,
        'request: 'writer,
    {
        AsyncTransport::send(self, request, response)
    }
}
