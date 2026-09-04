use core::fmt;

use cloud_sdk::Method;
use cloud_sdk::buffer::sanitize_bytes;
use cloud_sdk::transport::{
    AsyncExecutionError, AsyncRawHttpExecutor, BlockingRawHttpExecutor, BoundTransport,
    HeaderError, HeaderName, LocalAsyncRawHttpExecutor, RawResponsePolicy, RawResponsePolicyError,
    ResponseBuffer, ResponseMediaPolicy, ResponseWriterError, TransportRequest, drive_async_raw,
    drive_local_raw,
};

use super::{
    ApiRequestTarget, CratesIoEndpointError, DownloadRedirectError, OfficialCratesIoEndpoint,
    ProductionDownloadResponse,
};

impl<'storage> ProductionDownloadResponse<'storage> {
    /// Executes and validates one production download redirect synchronously.
    ///
    /// Endpoint verification and dispatch use the same immutable bound
    /// transport. The SDK owns the empty request headers and exact response
    /// policy; safe callers cannot submit an unrelated response for proof
    /// creation.
    pub fn execute_blocking<T>(
        transport: &T,
        source: ApiRequestTarget<'_>,
        response: &mut ResponseBuffer<'_>,
        target_storage: &'storage mut [u8],
    ) -> Result<Self, DownloadProvenanceError<T::Error>>
    where
        T: BlockingRawHttpExecutor + BoundTransport,
    {
        sanitize_bytes(target_storage);
        verify_source_transport(transport)?;
        ensure_uncommitted(response)?;
        let policy = source_policy()?;
        transport
            .execute(
                TransportRequest::new(Method::Get, source.as_request_target()),
                policy,
                response.writer(),
            )
            .map_err(DownloadProvenanceError::Transport)?;
        checked_response(source, response, target_storage)
    }

    /// Executes and validates one production download redirect asynchronously.
    pub async fn execute_async<'transport, 'source, 'writer, 'buffer, T>(
        transport: &'transport T,
        source: ApiRequestTarget<'source>,
        response: &'writer mut ResponseBuffer<'buffer>,
        target_storage: &'storage mut [u8],
    ) -> Result<Self, DownloadProvenanceError<T::Error>>
    where
        T: AsyncRawHttpExecutor + BoundTransport,
        'transport: 'writer,
        'source: 'writer,
        'buffer: 'writer,
    {
        sanitize_bytes(target_storage);
        verify_source_transport(transport)?;
        ensure_uncommitted(response)?;
        let policy = source_policy()?;
        drive_async_raw(
            transport,
            TransportRequest::new(Method::Get, source.as_request_target()),
            policy,
            response.writer(),
        )
        .await
        .map_err(map_async_error)?;
        checked_response(source, response, target_storage)
    }

    /// Executes and validates one production download redirect locally.
    pub async fn execute_local_async<'transport, 'source, 'writer, 'buffer, T>(
        transport: &'transport T,
        source: ApiRequestTarget<'source>,
        response: &'writer mut ResponseBuffer<'buffer>,
        target_storage: &'storage mut [u8],
    ) -> Result<Self, DownloadProvenanceError<T::Error>>
    where
        T: LocalAsyncRawHttpExecutor + BoundTransport,
        'transport: 'writer,
        'source: 'writer,
        'buffer: 'writer,
    {
        sanitize_bytes(target_storage);
        verify_source_transport(transport)?;
        ensure_uncommitted(response)?;
        let policy = source_policy()?;
        drive_local_raw(
            transport,
            TransportRequest::new(Method::Get, source.as_request_target()),
            policy,
            response.writer(),
        )
        .await
        .map_err(map_async_error)?;
        checked_response(source, response, target_storage)
    }
}

/// Atomic source-execution or redirect-proof failure.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum DownloadProvenanceError<E> {
    /// The source executor is not bound to the production crates.io API.
    InvalidSourceEndpoint(CratesIoEndpointError),
    /// The SDK-owned redirect response policy could not be constructed.
    InvalidResponsePolicy(RawResponsePolicyError),
    /// The SDK-owned retained response-header name could not be constructed.
    InvalidResponseHeader(HeaderError),
    /// Source request execution failed.
    Transport(E),
    /// Source response commitment or access failed.
    ResponseWriter(ResponseWriterError),
    /// The executed response did not prove an accepted redirect.
    InvalidRedirect(DownloadRedirectError),
}

impl<E> fmt::Debug for DownloadProvenanceError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSourceEndpoint(error) => formatter
                .debug_tuple("InvalidSourceEndpoint")
                .field(error)
                .finish(),
            Self::InvalidResponsePolicy(error) => formatter
                .debug_tuple("InvalidResponsePolicy")
                .field(error)
                .finish(),
            Self::InvalidResponseHeader(error) => formatter
                .debug_tuple("InvalidResponseHeader")
                .field(error)
                .finish(),
            Self::Transport(_) => formatter.write_str("Transport([redacted])"),
            Self::ResponseWriter(error) => formatter
                .debug_tuple("ResponseWriter")
                .field(error)
                .finish(),
            Self::InvalidRedirect(error) => formatter
                .debug_tuple("InvalidRedirect")
                .field(error)
                .finish(),
        }
    }
}

impl<E> fmt::Display for DownloadProvenanceError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidSourceEndpoint(_) => "download source endpoint is invalid",
            Self::InvalidResponsePolicy(_) => "download redirect response policy is invalid",
            Self::InvalidResponseHeader(_) => "download redirect response header is invalid",
            Self::Transport(_) => "download source transport failed",
            Self::ResponseWriter(_) => "download source response transaction failed",
            Self::InvalidRedirect(_) => "download source did not return an accepted redirect",
        })
    }
}

impl<E> core::error::Error for DownloadProvenanceError<E> {}

fn checked_response<'storage, E>(
    source: ApiRequestTarget<'_>,
    response: &ResponseBuffer<'_>,
    target_storage: &'storage mut [u8],
) -> Result<ProductionDownloadResponse<'storage>, DownloadProvenanceError<E>> {
    response
        .with_response(|response| {
            ProductionDownloadResponse::from_executed_response(source, response, target_storage)
        })
        .map_err(DownloadProvenanceError::ResponseWriter)?
        .map_err(DownloadProvenanceError::InvalidRedirect)
}

fn verify_source_transport<E>(
    transport: &(impl BoundTransport + ?Sized),
) -> Result<(), DownloadProvenanceError<E>> {
    OfficialCratesIoEndpoint::production_api()
        .verify_transport(transport)
        .map_err(DownloadProvenanceError::InvalidSourceEndpoint)
}

fn ensure_uncommitted<E>(
    response: &mut ResponseBuffer<'_>,
) -> Result<(), DownloadProvenanceError<E>> {
    if response.writer().is_committed() {
        return Err(DownloadProvenanceError::ResponseWriter(
            ResponseWriterError::AlreadyCommitted,
        ));
    }
    Ok(())
}

fn source_policy<E>() -> Result<RawResponsePolicy<'static>, DownloadProvenanceError<E>> {
    let location =
        HeaderName::new("location").map_err(DownloadProvenanceError::InvalidResponseHeader)?;
    RawResponsePolicy::new(
        0,
        0,
        ResponseMediaPolicy::Forbidden,
        ResponseMediaPolicy::Forbidden,
        &[location],
        0,
    )
    .map_err(DownloadProvenanceError::InvalidResponsePolicy)
}

fn map_async_error<E>(error: AsyncExecutionError<E>) -> DownloadProvenanceError<E> {
    match error {
        AsyncExecutionError::Transport(error) => DownloadProvenanceError::Transport(error),
        AsyncExecutionError::Response(error) => DownloadProvenanceError::ResponseWriter(error),
    }
}
