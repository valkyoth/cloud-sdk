use cloud_sdk::ServiceMarker;
use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, BlockingAuthenticatedTransport, LocalAsyncAuthenticatedTransport,
};
use cloud_sdk::client::{ClientExecutionError, ClientOperation, ClientWorkspaceLease};
use cloud_sdk::operation::PrepareOperation;
use cloud_sdk::transport::BoundTransport;

use super::{HetznerClient, OfficialEndpointTrust};
use crate::association::HetznerClientOperation;
use crate::identity::Hetzner;

type HetznerClientResult<O, E> = Result<
    <O as ClientOperation>::Output,
    ClientExecutionError<<O as PrepareOperation>::Error, E, <O as ClientOperation>::DecodeError>,
>;

impl<T, S> HetznerClient<T, S, OfficialEndpointTrust>
where
    T: BlockingAuthenticatedTransport + BoundTransport,
    S: ServiceMarker<Provider = Hetzner>,
{
    /// Sends one service-matched read-only operation synchronously and decodes it.
    pub fn execute_blocking<O, const N: usize>(
        &self,
        operation: &O,
        lease: ClientWorkspaceLease<'_, '_, N>,
    ) -> HetznerClientResult<O, T::Error>
    where
        O: ClientOperation + HetznerClientOperation<Service = S>,
    {
        self.kernel.execute_blocking(operation, lease)
    }
}

impl<T, S> HetznerClient<T, S, OfficialEndpointTrust>
where
    T: AsyncAuthenticatedTransport + BoundTransport + Sync,
    S: ServiceMarker<Provider = Hetzner>,
{
    /// Sends one service-matched read-only operation through a `Send` transport.
    #[allow(clippy::manual_async_fn)]
    pub fn execute_async<O, const N: usize>(
        &self,
        operation: &O,
        lease: ClientWorkspaceLease<'_, '_, N>,
    ) -> impl core::future::Future<Output = HetznerClientResult<O, T::Error>> + Send
    where
        O: ClientOperation + HetznerClientOperation<Service = S> + Sync,
        O::Output: Send,
        <O as PrepareOperation>::Error: Send,
        O::DecodeError: Send,
        T::Error: Send,
    {
        self.kernel.execute_async(operation, lease)
    }
}

impl<T, S> HetznerClient<T, S, OfficialEndpointTrust>
where
    T: LocalAsyncAuthenticatedTransport + BoundTransport,
    S: ServiceMarker<Provider = Hetzner>,
{
    /// Sends one service-matched read-only operation through a local transport.
    pub async fn execute_local_async<O, const N: usize>(
        &self,
        operation: &O,
        lease: ClientWorkspaceLease<'_, '_, N>,
    ) -> HetznerClientResult<O, T::Error>
    where
        O: ClientOperation + HetznerClientOperation<Service = S>,
    {
        self.kernel.execute_local_async(operation, lease).await
    }
}
