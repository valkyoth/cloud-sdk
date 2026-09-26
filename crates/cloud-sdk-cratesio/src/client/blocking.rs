use super::{RegistryBuffers, RegistryClient};
use crate::{credentials::ScopedCredentialMaterial, discovery::DiscoveryExecutionError};
use cloud_sdk::transport::{
    BlockingAuthorizedRawHttpExecutor, BoundUserAgent, RawResponsePolicy, ResponseWriter,
    TransportRequest,
};

mod sealed {
    pub trait Operation {}
}

/// Sealed binding from a reviewed provider request/permit to its checked runner.
/// Downstream code cannot register arbitrary paths or bypass mutation consent.
pub trait BlockingRegistryOperation: sealed::Operation + Sized {
    /// Operation-specific, checked success value.
    type Response;
    /// Internal dispatch used by [`RegistryClient::execute`]. All scratch regions
    /// are cleared on every exit, including pre-dispatch failures.
    #[doc(hidden)]
    fn run<T: BlockingAuthorizedRawHttpExecutor + BoundUserAgent + ?Sized>(
        self,
        client: &RegistryClient<'_, T>,
        buffers: RegistryBuffers<'_>,
    ) -> Result<Self::Response, DiscoveryExecutionError<T::Error>>;
}

pub(super) fn dispatch<T: BlockingAuthorizedRawHttpExecutor + ?Sized>(
    executor: &T,
    material: ScopedCredentialMaterial<'_>,
    request: TransportRequest<'_>,
    policy: RawResponsePolicy<'_>,
    response: &mut ResponseWriter<'_>,
) -> Result<(), T::Error> {
    match material.authorization() {
        Some(authorization) => executor.execute_authorized(
            material.endpoint(),
            authorization,
            request,
            policy,
            response,
        ),
        None => executor.execute(request, policy, response),
    }
}

macro_rules! read_operation {
    ($request:ty, $response:ty, $client:path) => {
        impl sealed::Operation for $request {}
        impl BlockingRegistryOperation for $request {
            type Response = $response;
            fn run<T: BlockingAuthorizedRawHttpExecutor + BoundUserAgent + ?Sized>(
                self,
                c: &RegistryClient<'_, T>,
                buffers: RegistryBuffers<'_>,
            ) -> Result<Self::Response, DiscoveryExecutionError<T::Error>> {
                use $client as Client;
                let mut guard = super::buffers::Guard::new(buffers);
                let buffers = guard.parts();
                let client = if c.staging {
                    Client::staging(c.executor, c.identity, c.maximum)
                } else {
                    Client::production(c.executor, c.identity, c.maximum)
                }
                .map_err(DiscoveryExecutionError::Model)?;
                client.execute(self, buffers.response, buffers.headers)
            }
        }
    };
}
read_operation!(
    crate::discovery::DiscoveryRequest<'_>,
    crate::discovery::DiscoveryResponse,
    crate::discovery::DiscoveryClient
);
read_operation!(
    crate::catalog::CatalogRequest<'_>,
    crate::catalog::CatalogResponse,
    crate::catalog::CatalogClient
);
read_operation!(
    crate::versions::VersionRequest<'_>,
    crate::versions::VersionResponse,
    crate::versions::VersionClient
);
read_operation!(
    crate::downloads::DownloadRequest<'_>,
    crate::downloads::DownloadResponse,
    crate::downloads::DownloadClient
);
read_operation!(
    crate::accounts::AccountRequest<'_>,
    crate::accounts::AccountResponse,
    crate::accounts::AccountClient
);

macro_rules! permit_operation {
    ($permit:ty, $response:ty, $client:path, $buffers:path, $($field:ident),+) => {
        impl<'a> sealed::Operation for $permit {}
        impl<'a> BlockingRegistryOperation for $permit {
            type Response = $response;
            fn run<T: BlockingAuthorizedRawHttpExecutor + BoundUserAgent + ?Sized>(
                self, c: &RegistryClient<'_, T>, buffers: RegistryBuffers<'_>,
            ) -> Result<Self::Response, DiscoveryExecutionError<T::Error>> {
                use $client as Client;
                use $buffers as Buffers;
                let mut guard = super::buffers::Guard::new(buffers);
                let buffers = guard.parts();
                let client = if c.staging {
                    Client::staging(c.executor, c.identity, c.maximum)
                } else {
                    Client::production(c.executor, c.identity, c.maximum)
                }.map_err(DiscoveryExecutionError::Model)?;
                client.execute(self, Buffers { $($field: buffers.$field,)+ }, dispatch)
            }
        }
    };
}
impl sealed::Operation for crate::accounts::personal::PersonalPermit<'_> {}
impl BlockingRegistryOperation for crate::accounts::personal::PersonalPermit<'_> {
    type Response = crate::accounts::personal::PersonalResponse;
    fn run<T: BlockingAuthorizedRawHttpExecutor + BoundUserAgent + ?Sized>(
        self,
        c: &RegistryClient<'_, T>,
        buffers: RegistryBuffers<'_>,
    ) -> Result<Self::Response, DiscoveryExecutionError<T::Error>> {
        use crate::accounts::personal::{PersonalBuffers, PersonalClient};
        let mut guard = super::buffers::Guard::new(buffers);
        let buffers = guard.parts();
        let client = if c.staging {
            PersonalClient::staging(c.executor, c.identity, c.maximum)
        } else {
            PersonalClient::production(c.executor, c.identity, c.maximum)
        }
        .map_err(DiscoveryExecutionError::Model)?;
        client.execute(
            self,
            PersonalBuffers {
                credential: buffers.credential,
                body: buffers.body,
                response: buffers.response,
                headers: buffers.headers,
            },
            dispatch,
        )
    }
}
permit_operation!(
    crate::accounts::tokens::TokenPermit<'a>,
    crate::accounts::tokens::TokenResponse,
    crate::accounts::tokens::TokenClient,
    crate::accounts::tokens::TokenBuffers,
    credential,
    response,
    headers
);
permit_operation!(
    crate::settings::SettingsPermit<'a>,
    crate::settings::SettingsResponse,
    crate::settings::SettingsClient,
    crate::settings::SettingsBuffers,
    credential,
    body,
    response,
    headers
);
permit_operation!(
    crate::ownership::OwnerChangePermit<'a>,
    crate::ownership::OwnerChangeResponse,
    crate::ownership::OwnerChangeClient,
    crate::ownership::OwnerChangeBuffers,
    credential,
    body,
    response,
    headers
);
permit_operation!(
    crate::publishing::YankPermit<'a>,
    crate::publishing::YankAcknowledgement<'a>,
    crate::publishing::YankClient,
    crate::publishing::YankBuffers,
    credential,
    response,
    headers
);
permit_operation!(
    crate::trusted_publishing::TrustedPublishingPermit<'a>,
    crate::trusted_publishing::TrustedPublishingResponse,
    crate::trusted_publishing::TrustedPublishingClient,
    crate::trusted_publishing::TrustedPublishingBuffers,
    credential,
    body,
    response,
    headers
);
