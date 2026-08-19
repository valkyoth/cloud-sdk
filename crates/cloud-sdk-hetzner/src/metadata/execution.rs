use cloud_sdk::transport::{
    AsyncExecutionError, AsyncRawHttpExecutor, BlockingRawHttpExecutor, BoundTransport,
    LocalAsyncRawHttpExecutor, ResponseWriter, ResponseWriterError, drive_async_raw,
    drive_local_raw,
};

use super::{MetadataEndpointError, MetadataRequest, MetadataWireError, verify_metadata_endpoint};

/// Metadata request preparation, endpoint, transport, or response-stage failure.
#[derive(Eq, PartialEq)]
pub enum MetadataExecutionError<E> {
    /// The transport does not expose the exact link-local metadata identity.
    Endpoint(MetadataEndpointError),
    /// An SDK-owned static request or policy failed validation.
    Wire(MetadataWireError),
    /// The executor failed after endpoint and request validation.
    Transport(E),
    /// Async response staging or commitment failed.
    Response(ResponseWriterError),
}

impl<E> core::fmt::Debug for MetadataExecutionError<E> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::Endpoint(_) => "MetadataExecutionError::Endpoint([redacted])",
            Self::Wire(_) => "MetadataExecutionError::Wire([redacted])",
            Self::Transport(_) => "MetadataExecutionError::Transport([redacted])",
            Self::Response(_) => "MetadataExecutionError::Response([redacted])",
        })
    }
}

impl<E> core::fmt::Display for MetadataExecutionError<E> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::Endpoint(_) => "metadata transport endpoint validation failed",
            Self::Wire(_) => "metadata request construction failed",
            Self::Transport(_) => "metadata transport execution failed",
            Self::Response(_) => "metadata response staging failed",
        })
    }
}

impl<E> core::error::Error for MetadataExecutionError<E> {}

/// Executes exactly one blocking metadata read without authentication or retry.
pub fn execute_metadata_blocking<T>(
    executor: &T,
    request: MetadataRequest,
    response: &mut ResponseWriter<'_>,
) -> Result<(), MetadataExecutionError<T::Error>>
where
    T: BlockingRawHttpExecutor + BoundTransport,
{
    verify_metadata_endpoint(executor).map_err(MetadataExecutionError::Endpoint)?;
    let wire = request
        .transport_request()
        .map_err(MetadataExecutionError::Wire)?;
    let policy = request
        .response_policy()
        .map_err(MetadataExecutionError::Wire)?;
    executor
        .execute(wire, policy, response)
        .map_err(MetadataExecutionError::Transport)
}

/// Executes exactly one Send asynchronous metadata read.
pub async fn execute_metadata_async<T>(
    executor: &T,
    request: MetadataRequest,
    response: &mut ResponseWriter<'_>,
) -> Result<(), MetadataExecutionError<T::Error>>
where
    T: AsyncRawHttpExecutor + BoundTransport,
{
    verify_metadata_endpoint(executor).map_err(MetadataExecutionError::Endpoint)?;
    let wire = request
        .transport_request()
        .map_err(MetadataExecutionError::Wire)?;
    let policy = request
        .response_policy()
        .map_err(MetadataExecutionError::Wire)?;
    drive_async_raw(executor, wire, policy, response)
        .await
        .map_err(map_async_error)
}

/// Executes exactly one local asynchronous metadata read.
pub async fn execute_metadata_local_async<T>(
    executor: &T,
    request: MetadataRequest,
    response: &mut ResponseWriter<'_>,
) -> Result<(), MetadataExecutionError<T::Error>>
where
    T: LocalAsyncRawHttpExecutor + BoundTransport,
{
    verify_metadata_endpoint(executor).map_err(MetadataExecutionError::Endpoint)?;
    let wire = request
        .transport_request()
        .map_err(MetadataExecutionError::Wire)?;
    let policy = request
        .response_policy()
        .map_err(MetadataExecutionError::Wire)?;
    drive_local_raw(executor, wire, policy, response)
        .await
        .map_err(map_async_error)
}

fn map_async_error<E>(error: AsyncExecutionError<E>) -> MetadataExecutionError<E> {
    match error {
        AsyncExecutionError::Transport(error) => MetadataExecutionError::Transport(error),
        AsyncExecutionError::Response(error) => MetadataExecutionError::Response(error),
    }
}
