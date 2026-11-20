use super::{RegistryBuffers, RegistryClient};
use crate::{credentials::ScopedCredentialMaterial, discovery::DiscoveryExecutionError};
use cloud_sdk::transport::{
    BlockingAuthorizedRawHttpExecutor, BoundUserAgent, RawResponsePolicy, ResponseWriter,
    TransportRequest,
};

mod sealed {
    pub trait Operation {}
}

use crate::accounts::personal::invitations::InvitationPermit;
impl sealed::Operation for InvitationPermit<'_> {}
impl BlockingRegistryOperation for InvitationPermit<'_> {
    type Response = crate::discovery::DiscoveryValue;
    fn run<T: BlockingAuthorizedRawHttpExecutor + BoundUserAgent + ?Sized>(
        self,
        c: &RegistryClient<'_, T>,
        buffers: RegistryBuffers<'_>,
    ) -> Result<Self::Response, DiscoveryExecutionError<T::Error>> {
        use super::prepared::Permit;
        use crate::discovery::{
            DiscoveryClient, DiscoveryError as Error, DiscoveryExecutionError as Failure,
        };
        use cloud_sdk::transport::{MediaType, RequestHeader, RequestHeaders, ResponseBuffer};
        let mut guard = super::buffers::Guard::new(buffers);
        let buffers = guard.parts();
        let client = if c.staging {
            DiscoveryClient::staging(c.executor, c.identity, c.maximum)
        } else {
            DiscoveryClient::production(c.executor, c.identity, c.maximum)
        }
        .map_err(Failure::Model)?;
        client.verify().map_err(Failure::Model)?;
        if Permit::origin(&self).endpoint() != client.endpoint {
            return Err(Failure::Model(Error::Binding));
        }
        let mut target = [0; crate::query::MAX_TARGET_BYTES];
        let mut secret = cloud_sdk_sanitization::SecretBuffer::new(buffers.credential);
        let p = self
            .prepare(c.executor, &mut target, &mut secret, buffers.body, 0)
            .map_err(Failure::Model)?;
        let mut response = ResponseBuffer::new(buffers.response, c.maximum, buffers.headers);
        let headers = [
            RequestHeader::accept(MediaType::JSON),
            RequestHeader::new("accept-encoding", "identity")
                .map_err(|_| Failure::Model(Error::Value))?,
            RequestHeader::new("content-type", "application/json")
                .map_err(|_| Failure::Model(Error::Value))?,
        ];
        let n = if p.body.is_empty() { 2 } else { 3 };
        let headers = RequestHeaders::new(headers.get(..n).ok_or(Failure::Model(Error::Limit))?)
            .map_err(|_| Failure::Model(Error::Value))?;
        let wire = TransportRequest::new(p.material.method(), p.material.target())
            .with_headers(headers)
            .with_body(p.body);
        let policy = client.policy().map_err(Failure::Model)?;
        let mut attempt = client.gate.begin().map_err(Failure::Schedule)?;
        dispatch(c.executor, p.material, wire, policy, response.writer())
            .map_err(Failure::Transport)?;
        self.finish(&client, response, &mut attempt)
    }
}

impl sealed::Operation for crate::accounts::cargo::CargoOwnersRequest<'_> {}
impl BlockingRegistryOperation for crate::accounts::cargo::CargoOwnersRequest<'_> {
    type Response = crate::accounts::cargo::CargoOwners;
    fn run<T: BlockingAuthorizedRawHttpExecutor + BoundUserAgent + ?Sized>(
        self,
        c: &RegistryClient<'_, T>,
        buffers: RegistryBuffers<'_>,
    ) -> Result<Self::Response, DiscoveryExecutionError<T::Error>> {
        use crate::discovery::DiscoveryClient;
        let mut guard = super::buffers::Guard::new(buffers);
        let client = if c.staging {
            DiscoveryClient::staging(c.executor, c.identity, c.maximum)
        } else {
            DiscoveryClient::production(c.executor, c.identity, c.maximum)
        }
        .map_err(DiscoveryExecutionError::Model)?;
        self.execute(&client, guard.parts())
    }
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
