use super::{
    ClientExecutionError, ClientKernel, ClientOperation, ClientResponse, ClientResult,
    ClientWorkspaceLease,
};
use crate::authentication::{
    AsyncAuthenticatedTransport, BlockingAuthenticatedTransport, LocalAsyncAuthenticatedTransport,
};
use crate::operation::PreparationStorage;
use crate::transport::BoundTransport;

impl<T> ClientKernel<T>
where
    T: BlockingAuthenticatedTransport + BoundTransport,
{
    /// Prepares, authenticates, sends once, and checked-decodes synchronously.
    pub fn execute_blocking<O, const N: usize>(
        &self,
        operation: &O,
        mut lease: ClientWorkspaceLease<'_, '_, N>,
    ) -> ClientResult<O::Output, O::Error, T::Error, O::DecodeError>
    where
        O: ClientOperation,
    {
        let mut parts = lease.parts_mut();
        parts.clear();
        let prepared = operation
            .prepare(PreparationStorage::new(parts.target, parts.request_body))
            .map_err(ClientExecutionError::Preparation)?;
        let response = prepared
            .send_blocking(&self.transport, parts.response_body, parts.response_headers)
            .map_err(ClientExecutionError::Execution)?;
        operation
            .decode_response(ClientResponse::new(prepared, response))
            .map_err(ClientExecutionError::Decode)
    }
}

impl<T> ClientKernel<T>
where
    T: AsyncAuthenticatedTransport + BoundTransport + Sync,
{
    /// Prepares, authenticates, sends once, and checked-decodes with a Send transport.
    // The explicit opaque return is required for the compiler to prove the
    // public Send guarantee across the authenticated transport RPITIT.
    #[allow(clippy::manual_async_fn)]
    pub fn execute_async<O, const N: usize>(
        &self,
        operation: &O,
        mut lease: ClientWorkspaceLease<'_, '_, N>,
    ) -> impl core::future::Future<
        Output = ClientResult<O::Output, O::Error, T::Error, O::DecodeError>,
    > + Send
    where
        O: ClientOperation + Sync,
        O::Output: Send,
        O::Error: Send,
        O::DecodeError: Send,
        T::Error: Send,
    {
        async move {
            let mut parts = lease.parts_mut();
            parts.clear();
            let prepared = operation
                .prepare(PreparationStorage::new(parts.target, parts.request_body))
                .map_err(ClientExecutionError::Preparation)?;
            let response = prepared
                .send_async(&self.transport, parts.response_body, parts.response_headers)
                .await
                .map_err(ClientExecutionError::Execution)?;
            operation
                .decode_response(ClientResponse::new(prepared, response))
                .map_err(ClientExecutionError::Decode)
        }
    }
}

impl<T> ClientKernel<T>
where
    T: LocalAsyncAuthenticatedTransport + BoundTransport,
{
    /// Prepares, authenticates, sends once, and checked-decodes locally.
    pub async fn execute_local_async<O, const N: usize>(
        &self,
        operation: &O,
        mut lease: ClientWorkspaceLease<'_, '_, N>,
    ) -> ClientResult<O::Output, O::Error, T::Error, O::DecodeError>
    where
        O: ClientOperation,
    {
        let mut parts = lease.parts_mut();
        parts.clear();
        let prepared = operation
            .prepare(PreparationStorage::new(parts.target, parts.request_body))
            .map_err(ClientExecutionError::Preparation)?;
        let response = prepared
            .send_local_async(&self.transport, parts.response_body, parts.response_headers)
            .await
            .map_err(ClientExecutionError::Execution)?;
        operation
            .decode_response(ClientResponse::new(prepared, response))
            .map_err(ClientExecutionError::Decode)
    }
}
