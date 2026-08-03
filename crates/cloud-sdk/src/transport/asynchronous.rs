//! Runtime-neutral asynchronous transport contracts.

use core::{fmt, future::Future};

use super::{
    AsyncResponseStaging, DeliveryPhase, ResponseCompletion, ResponseWriter, ResponseWriterError,
    TransportRequest,
};

/// Conservative delivery classification after an asynchronous future is
/// cancelled by being dropped.
pub const ASYNC_CANCELLATION_DELIVERY_PHASE: DeliveryPhase = DeliveryPhase::PossiblySent;

/// Failure while SDK-owned asynchronous response staging is driven to completion.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum AsyncExecutionError<E> {
    /// The transport failed before successful response completion.
    Transport(E),
    /// Response staging or final commitment failed.
    Response(ResponseWriterError),
}

impl<E> fmt::Debug for AsyncExecutionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(_) => formatter.write_str("Transport([redacted])"),
            Self::Response(error) => formatter.debug_tuple("Response").field(error).finish(),
        }
    }
}

impl<E> fmt::Display for AsyncExecutionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Transport(_) => "asynchronous transport failed",
            Self::Response(_) => "asynchronous response staging failed",
        })
    }
}

impl<E> core::error::Error for AsyncExecutionError<E> {}

/// Local asynchronous transport for `!Send` futures and executors.
///
/// Implementations receive a non-committing staging view and return completion
/// metadata. [`drive_local`] owns the cleanup attempt across the await and
/// commits only after `Ready(Ok)`. Cancellation clears partial response state
/// and remains conservatively [`DeliveryPhase::PossiblySent`].
///
/// A local implementation may deliberately return a future that is not
/// `Send`:
///
/// ```compile_fail
/// use core::cell::Cell;
/// use cloud_sdk::transport::{
///     AsyncResponseStaging, LocalAsyncTransport, ResponseCompletion,
///     ResponseMetadata, StatusCode, TransportRequest,
/// };
///
/// struct Local(Cell<()>);
///
/// impl LocalAsyncTransport for Local {
///     type Error = ();
///
///     fn send_local<'transport, 'request, 'writer, 'buffer>(
///         &'transport self,
///         _request: TransportRequest<'request>,
///         _response: AsyncResponseStaging<'writer, 'buffer>,
///     ) -> impl core::future::Future<Output = Result<ResponseCompletion, Self::Error>> + 'writer
///     where
///         'transport: 'writer,
///         'request: 'writer,
///         'buffer: 'writer,
///     {
///         async move {
///             self.0.get();
///             Ok(ResponseCompletion::new(
///                 StatusCode::NO_CONTENT,
///                 0,
///                 ResponseMetadata::EMPTY,
///             ))
///         }
///     }
/// }
///
/// fn require_send<T: Send>(_: T) {}
/// fn reject_send<'a, 'buffer: 'a>(
///     transport: &'a Local,
///     request: TransportRequest<'a>,
///     response: AsyncResponseStaging<'a, 'buffer>,
/// ) {
///     require_send(transport.send_local(request, response));
/// }
/// ```
pub trait LocalAsyncTransport {
    /// Transport-specific failure.
    type Error;

    /// Stages one response without requiring the returned future to be `Send`.
    fn send_local<'transport, 'request, 'writer, 'buffer>(
        &'transport self,
        request: TransportRequest<'request>,
        response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> impl Future<Output = Result<ResponseCompletion, Self::Error>> + 'writer
    where
        'transport: 'writer,
        'request: 'writer,
        'buffer: 'writer;
}

/// Drives one local transport attempt and commits only after `Ready(Ok)`.
pub async fn drive_local<'transport, 'request, 'writer, 'buffer, T>(
    transport: &'transport T,
    request: TransportRequest<'request>,
    response: &'writer mut ResponseWriter<'buffer>,
) -> Result<(), AsyncExecutionError<T::Error>>
where
    T: LocalAsyncTransport + ?Sized,
    'transport: 'writer,
    'request: 'writer,
    'buffer: 'writer,
{
    let mut attempt = response
        .begin_attempt()
        .map_err(AsyncExecutionError::Response)?;
    let completion = transport
        .send_local(request, attempt.staging())
        .await
        .map_err(AsyncExecutionError::Transport)?;
    attempt
        .commit_completion(completion)
        .map_err(AsyncExecutionError::Response)
}

/// Cross-thread asynchronous transport over caller-owned buffers.
///
/// Implementations can stage body and headers but cannot commit a response.
/// Callers use [`drive_async`], which owns cleanup across the await and commits
/// returned completion metadata only after `Ready(Ok)`.
pub trait AsyncTransport {
    /// Transport-specific failure.
    type Error;

    /// Stages one complete response without committing it.
    fn send<'transport, 'request, 'writer, 'buffer>(
        &'transport self,
        request: TransportRequest<'request>,
        response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> impl Future<Output = Result<ResponseCompletion, Self::Error>> + Send + 'writer
    where
        'transport: 'writer,
        'request: 'writer,
        'buffer: 'writer;
}

/// Drives one cross-thread async attempt and commits only after `Ready(Ok)`.
pub async fn drive_async<'transport, 'request, 'writer, 'buffer, T>(
    transport: &'transport T,
    request: TransportRequest<'request>,
    response: &'writer mut ResponseWriter<'buffer>,
) -> Result<(), AsyncExecutionError<T::Error>>
where
    T: AsyncTransport + ?Sized,
    'transport: 'writer,
    'request: 'writer,
    'buffer: 'writer,
{
    let mut attempt = response
        .begin_attempt()
        .map_err(AsyncExecutionError::Response)?;
    let completion = transport
        .send(request, attempt.staging())
        .await
        .map_err(AsyncExecutionError::Transport)?;
    attempt
        .commit_completion(completion)
        .map_err(AsyncExecutionError::Response)
}

impl<T> LocalAsyncTransport for T
where
    T: AsyncTransport + ?Sized,
{
    type Error = T::Error;

    async fn send_local<'transport, 'request, 'writer, 'buffer>(
        &'transport self,
        request: TransportRequest<'request>,
        response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
        'buffer: 'writer,
    {
        AsyncTransport::send(self, request, response).await
    }
}
