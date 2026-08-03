//! Local asynchronous prepared-request execution.

use super::{CheckedResponseGuard, PreparedExecutionError, PreparedRequest};
use crate::authentication::LocalAsyncAuthenticatedTransport;
use crate::transport::{BoundTransport, ResponseBuffer};

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
        LocalAsyncAuthenticatedTransport::send_authenticated_local(
            transport,
            self.authenticated_request(),
            response.writer(),
        )
        .await
        .map_err(PreparedExecutionError::Transport)?;
        self.validate_response(response)
            .map_err(PreparedExecutionError::ResponsePolicy)
    }
}
