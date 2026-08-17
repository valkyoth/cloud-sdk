use super::{ClientExecutionError, ClientKernel, ClientResponse, ClientWorkspaceLease};
use crate::authentication::{
    AsyncAuthenticatedTransport, BlockingAuthenticatedTransport, LocalAsyncAuthenticatedTransport,
};
use crate::operation::{PreparationStorageGuard, PreparedRequest};
use crate::transport::BoundTransport;

impl<T> ClientKernel<T>
where
    T: BlockingAuthenticatedTransport + BoundTransport,
{
    /// Executes through cleanup-owning provider preparation and decoding hooks.
    pub fn execute_blocking_with<'operation, O, R, P, D, F, G, const N: usize>(
        &self,
        operation: &'operation O,
        mut lease: ClientWorkspaceLease<'_, '_, N>,
        prepare: F,
        decode: G,
    ) -> Result<R, ClientExecutionError<P, T::Error, D>>
    where
        F: for<'guard> FnOnce(
            &'operation O,
            &'guard mut PreparationStorageGuard<'_>,
        ) -> Result<PreparedRequest<'guard>, P>,
        G: FnOnce(&'operation O, ClientResponse<'_, '_>) -> Result<R, D>,
    {
        let mut parts = lease.parts_mut();
        parts.clear();
        let mut storage = PreparationStorageGuard::new(parts.target, parts.request_body);
        let prepared =
            prepare(operation, &mut storage).map_err(ClientExecutionError::Preparation)?;
        let response = prepared
            .send_blocking(&self.transport, parts.response_body, parts.response_headers)
            .map_err(ClientExecutionError::Execution)?;
        decode(operation, ClientResponse::new(prepared, response))
            .map_err(ClientExecutionError::Decode)
    }
}

impl<T> ClientKernel<T>
where
    T: AsyncAuthenticatedTransport + BoundTransport + Sync,
{
    /// Send-async equivalent of [`Self::execute_blocking_with`].
    #[allow(clippy::manual_async_fn)]
    pub fn execute_async_with<'operation, O, R, P, D, F, G, const N: usize>(
        &self,
        operation: &'operation O,
        mut lease: ClientWorkspaceLease<'_, '_, N>,
        prepare: F,
        decode: G,
    ) -> impl core::future::Future<Output = Result<R, ClientExecutionError<P, T::Error, D>>> + Send
    where
        O: Sync,
        P: Send,
        R: Send,
        D: Send,
        F: for<'guard> FnOnce(
                &'operation O,
                &'guard mut PreparationStorageGuard<'_>,
            ) -> Result<PreparedRequest<'guard>, P>
            + Send,
        G: FnOnce(&'operation O, ClientResponse<'_, '_>) -> Result<R, D> + Send,
        T::Error: Send,
    {
        async move {
            let mut parts = lease.parts_mut();
            parts.clear();
            let mut storage = PreparationStorageGuard::new(parts.target, parts.request_body);
            let prepared =
                prepare(operation, &mut storage).map_err(ClientExecutionError::Preparation)?;
            let response = prepared
                .send_async(&self.transport, parts.response_body, parts.response_headers)
                .await
                .map_err(ClientExecutionError::Execution)?;
            decode(operation, ClientResponse::new(prepared, response))
                .map_err(ClientExecutionError::Decode)
        }
    }
}

impl<T> ClientKernel<T>
where
    T: LocalAsyncAuthenticatedTransport + BoundTransport,
{
    /// Local-async equivalent of [`Self::execute_blocking_with`].
    pub async fn execute_local_async_with<'operation, O, R, P, D, F, G, const N: usize>(
        &self,
        operation: &'operation O,
        mut lease: ClientWorkspaceLease<'_, '_, N>,
        prepare: F,
        decode: G,
    ) -> Result<R, ClientExecutionError<P, T::Error, D>>
    where
        F: for<'guard> FnOnce(
            &'operation O,
            &'guard mut PreparationStorageGuard<'_>,
        ) -> Result<PreparedRequest<'guard>, P>,
        G: FnOnce(&'operation O, ClientResponse<'_, '_>) -> Result<R, D>,
    {
        let mut parts = lease.parts_mut();
        parts.clear();
        let mut storage = PreparationStorageGuard::new(parts.target, parts.request_body);
        let prepared =
            prepare(operation, &mut storage).map_err(ClientExecutionError::Preparation)?;
        let response = prepared
            .send_local_async(&self.transport, parts.response_body, parts.response_headers)
            .await
            .map_err(ClientExecutionError::Execution)?;
        decode(operation, ClientResponse::new(prepared, response))
            .map_err(ClientExecutionError::Decode)
    }
}
