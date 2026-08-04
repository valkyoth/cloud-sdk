//! Local asynchronous provider-link execution.

use super::{ProviderLinkExecutionError, ValidatedProviderLink};
use crate::Method;
use crate::authentication::{
    AuthenticatedRequest, AuthenticationScopePolicy, LocalAsyncAuthenticatedTransport,
    drive_local_authenticated,
};
use crate::operation::OperationId;
use crate::transport::{AsyncExecutionError, BoundTransport, RawResponsePolicy, ResponseWriter};

impl ValidatedProviderLink<'_, '_> {
    /// Validates and executes one authenticated local asynchronous
    /// continuation request.
    ///
    /// Endpoint verification and dispatch use the same transport object. The
    /// method owns no executor and permits `!Send` futures.
    pub async fn execute_local_async<'transport, 'request, 'policy, 'writer, T>(
        &'request self,
        transport: &'transport T,
        method: Method,
        operation: OperationId,
        authentication: AuthenticationScopePolicy<'policy>,
        response_policy: RawResponsePolicy<'policy>,
        response: &'writer mut ResponseWriter<'_>,
    ) -> Result<(), ProviderLinkExecutionError<T::Error>>
    where
        T: BoundTransport + LocalAsyncAuthenticatedTransport,
        'transport: 'writer,
        'request: 'writer,
        'policy: 'writer,
    {
        let request = self
            .request_for(transport, method, operation)
            .map_err(ProviderLinkExecutionError::Pagination)?;
        drive_local_authenticated(
            transport,
            AuthenticatedRequest::new(request, authentication, &response_policy),
            response,
        )
        .await
        .map_err(|error| match error {
            AsyncExecutionError::Transport(error) => ProviderLinkExecutionError::Transport(error),
            AsyncExecutionError::Response(error) => {
                ProviderLinkExecutionError::ResponseWriter(error)
            }
        })
    }
}
