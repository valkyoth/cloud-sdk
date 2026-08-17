//! Local asynchronous prepared-request execution.

use super::{CheckedResponseGuard, PreparedExecutionError, PreparedRequest};
use crate::authentication::{LocalAsyncAuthenticatedTransport, drive_local_authenticated};
use crate::transport::{AsyncExecutionError, BoundTransport, EndpointIdentity, ResponseBuffer};
use cloud_sdk_sanitization::sanitize_bytes;

impl<'request> PreparedRequest<'request> {
    /// Verifies endpoint identity, executes once on a local async transport,
    /// and validates the response.
    ///
    /// This method owns no executor and does not require the returned future
    /// to be `Send`. Dropping it clears the response buffer while request
    /// delivery remains conservatively possibly sent.
    pub async fn execute_local_async<'transport, 'buffer, T>(
        &'transport self,
        transport: &'transport T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<CheckedResponseGuard<'buffer>, PreparedExecutionError<T::Error>>
    where
        T: LocalAsyncAuthenticatedTransport + BoundTransport,
        'request: 'transport,
    {
        let response = self
            .send_local_async(transport, response_storage, response_header_storage)
            .await?;
        self.validate_executed_response(response)
    }

    pub(crate) async fn send_local_async<'transport, 'buffer, T>(
        &'transport self,
        transport: &'transport T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<ResponseBuffer<'buffer>, PreparedExecutionError<T::Error>>
    where
        T: LocalAsyncAuthenticatedTransport + BoundTransport,
        'request: 'transport,
    {
        if self.requires_execution_permit() {
            sanitize_bytes(response_storage);
            sanitize_bytes(response_header_storage);
            return Err(PreparedExecutionError::AuthorizationRequired);
        }
        self.send_local_async_authorized(transport, None, response_storage, response_header_storage)
            .await
    }

    pub(crate) async fn execute_local_async_authorized<'transport, 'buffer, T>(
        &'transport self,
        transport: &'transport T,
        confirmed_endpoint: Option<EndpointIdentity<'_>>,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<CheckedResponseGuard<'buffer>, PreparedExecutionError<T::Error>>
    where
        T: LocalAsyncAuthenticatedTransport + BoundTransport,
        'request: 'transport,
    {
        let response = self
            .send_local_async_authorized(
                transport,
                confirmed_endpoint,
                response_storage,
                response_header_storage,
            )
            .await?;
        self.validate_executed_response(response)
    }

    pub(crate) async fn send_local_async_authorized<'transport, 'buffer, T>(
        &'transport self,
        transport: &'transport T,
        confirmed_endpoint: Option<EndpointIdentity<'_>>,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<ResponseBuffer<'buffer>, PreparedExecutionError<T::Error>>
    where
        T: LocalAsyncAuthenticatedTransport + BoundTransport,
        'request: 'transport,
    {
        let mut response = ResponseBuffer::new(
            response_storage,
            self.raw_response_policy().max_body_bytes(),
            response_header_storage,
        );
        let actual = transport
            .endpoint_identity()
            .map_err(PreparedExecutionError::EndpointIdentity)?;
        match confirmed_endpoint {
            Some(expected) if actual == expected => {}
            Some(_) => return Err(PreparedExecutionError::EndpointMismatch),
            None => self
                .service()
                .endpoint_policy()
                .verify(actual)
                .map_err(|_| PreparedExecutionError::EndpointMismatch)?,
        }
        drive_local_authenticated(transport, self.authenticated_request(), response.writer())
            .await
            .map_err(map_local_error)?;
        Ok(response)
    }
}

fn map_local_error<E>(error: AsyncExecutionError<E>) -> PreparedExecutionError<E> {
    match error {
        AsyncExecutionError::Transport(error) => PreparedExecutionError::Transport(error),
        AsyncExecutionError::Response(error) => PreparedExecutionError::ResponseWriter(error),
    }
}
