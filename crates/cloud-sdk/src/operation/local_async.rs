//! Local asynchronous prepared-request execution.

use super::{CheckedResponseGuard, PreparedExecutionError, PreparedRequest};
use crate::authentication::{LocalAsyncAuthenticatedTransport, drive_local_authenticated};
use crate::transport::{AsyncExecutionError, BoundTransport, ResponseBuffer};

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
        let mut response = ResponseBuffer::new(
            response_storage,
            self.raw_response_policy().max_body_bytes(),
            response_header_storage,
        );
        let actual = transport
            .endpoint_identity()
            .map_err(PreparedExecutionError::EndpointIdentity)?;
        self.service()
            .endpoint_policy()
            .verify(actual)
            .map_err(|_| PreparedExecutionError::EndpointMismatch)?;
        drive_local_authenticated(transport, self.authenticated_request(), response.writer())
            .await
            .map_err(map_local_error)?;
        self.validate_response(response)
            .map_err(PreparedExecutionError::ResponsePolicy)
    }
}

fn map_local_error<E>(error: AsyncExecutionError<E>) -> PreparedExecutionError<E> {
    match error {
        AsyncExecutionError::Transport(error) => PreparedExecutionError::Transport(error),
        AsyncExecutionError::Response(error) => PreparedExecutionError::ResponseWriter(error),
    }
}
