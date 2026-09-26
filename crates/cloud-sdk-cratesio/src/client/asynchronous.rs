use super::{RegistryBuffers, RegistryClient};
use crate::discovery::{DiscoveryClient, DiscoveryExecutionError};
use cloud_sdk::transport::{
    AsyncAuthorizedRawHttpExecutor, BoundUserAgent, LocalAuthorizedRawHttpExecutor,
};
use core::future::Future;
mod catalog;
mod runner;
#[cfg(all(test, feature = "blocking"))]
pub(crate) mod tests;
mod sealed {
    pub trait Operation {}
}

macro_rules! contract {
    ($trait:ident, $run:ident, $execute:ident, $transport:ident $(, $bound:ident, $sync:ident)?) => {
        /// Sealed typed execution over the corresponding async raw transport.
        /// No automatic retries or sleeps; consumed permits are not replayed.
        pub trait $trait: sealed::Operation + Sized $(+ $bound)? {
            /// Checked operation-specific response.
            type Response;
            /// Internal dispatch with cleanup armed before the future is returned.
            #[doc(hidden)]
            fn $run<'s, T: $transport + BoundUserAgent $(+ $sync)? + ?Sized>(
                self, client: &'s RegistryClient<'_, T>, buffers: RegistryBuffers<'s>,
            ) -> impl Future<Output = Result<Self::Response, DiscoveryExecutionError<T::Error>>> $(+ $bound)? + 's
            where Self: 's;
        }
        impl<T: $transport + BoundUserAgent $(+ $sync)? + ?Sized> RegistryClient<'_, T> {
            /// Execute once with guards installed before polling. Dropping an
            /// unpolled or in-flight future clears all scratch. Admission occurs
            /// only on poll, and cancellation does not authorize a retry.
            pub fn $execute<'s, R: $trait + 's>(
                &'s self, request: R, buffers: RegistryBuffers<'s>,
            ) -> impl Future<Output = Result<R::Response, DiscoveryExecutionError<T::Error>>> $(+ $bound)? + 's {
                request.$run(self, buffers)
            }
        }
    };
}
contract!(
    LocalRegistryOperation,
    run_local,
    execute_local,
    LocalAuthorizedRawHttpExecutor
);
contract!(
    AsyncRegistryOperation,
    run_async,
    execute_async,
    AsyncAuthorizedRawHttpExecutor,
    Send,
    Sync
);

macro_rules! implementation {
    ($type:ty, $response:ty, $kind:ident $(, $client:path)?) => {
        impl<'r> sealed::Operation for $type {}
        operation_mode!($type, $response, $kind $(, $client)?; LocalRegistryOperation, run_local,
            execute_local, LocalAuthorizedRawHttpExecutor);
        operation_mode!($type, $response, $kind $(, $client)?; AsyncRegistryOperation, run_async,
            execute_async, AsyncAuthorizedRawHttpExecutor, Send, Sync);
    };
}
macro_rules! operation_mode {
    ($type:ty, $response:ty, $kind:ident $(, $client:path)?; $trait:ident, $run:ident,
        $execute:ident, $transport:ident $(, $bound:ident, $sync:ident)?) => {
        impl<'r> $trait for $type {
            type Response = $response;
            fn $run<'s, T: $transport + BoundUserAgent $(+ $sync)? + ?Sized>(
                self, client: &'s RegistryClient<'_, T>, buffers: RegistryBuffers<'s>,
            ) -> impl Future<Output = Result<Self::Response, DiscoveryExecutionError<T::Error>>> $(+ $bound)? + 's
            where Self: 's {
                let mut guard = super::buffers::Guard::new(buffers);
                async move {
                    let buffers = guard.parts();
                    execute_kind!($kind $(, $client)?, self, client, buffers, $execute)
                }
            }
        }
    };
}
macro_rules! execute_kind {
    (read, $client:path, $request:ident, $registry:ident, $buffers:ident, $execute:ident) => {{
        use $client as Client;
        let client = if $registry.staging {
            Client::staging($registry.executor, $registry.identity, $registry.maximum)
        } else {
            Client::production($registry.executor, $registry.identity, $registry.maximum)
        }
        .map_err(DiscoveryExecutionError::Model)?;
        client
            .$execute($request, $buffers.response, $buffers.headers)
            .await
    }};
    (permit, $request:ident, $registry:ident, $buffers:ident, $execute:ident) => {{
        let client = if $registry.staging {
            DiscoveryClient::staging($registry.executor, $registry.identity, $registry.maximum)
        } else {
            DiscoveryClient::production($registry.executor, $registry.identity, $registry.maximum)
        }
        .map_err(DiscoveryExecutionError::Model)?;
        runner::$execute(&client, $request, $buffers).await
    }};
}
implementation!(
    crate::discovery::DiscoveryRequest<'r>,
    crate::discovery::DiscoveryResponse,
    read,
    crate::discovery::DiscoveryClient
);
implementation!(
    crate::catalog::CatalogRequest<'r>,
    crate::catalog::CatalogResponse,
    read,
    crate::catalog::CatalogClient
);
implementation!(
    crate::versions::VersionRequest<'r>,
    crate::versions::VersionResponse,
    read,
    crate::versions::VersionClient
);
implementation!(
    crate::downloads::DownloadRequest<'r>,
    crate::downloads::DownloadResponse,
    read,
    crate::downloads::DownloadClient
);
implementation!(
    crate::accounts::AccountRequest<'r>,
    crate::accounts::AccountResponse,
    read,
    crate::accounts::AccountClient
);
implementation!(
    crate::accounts::cargo::CargoOwnersRequest<'r>,
    crate::accounts::cargo::CargoOwners,
    permit
);
implementation!(
    crate::accounts::personal::PersonalPermit<'r>,
    crate::accounts::personal::PersonalResponse,
    permit
);
implementation!(
    crate::accounts::tokens::TokenPermit<'r>,
    crate::accounts::tokens::TokenResponse,
    permit
);
implementation!(
    crate::settings::SettingsPermit<'r>,
    crate::settings::SettingsResponse,
    permit
);
implementation!(
    crate::ownership::OwnerChangePermit<'r>,
    crate::ownership::OwnerChangeResponse,
    permit
);
implementation!(
    crate::publishing::YankPermit<'r>,
    crate::publishing::YankAcknowledgement<'r>,
    permit
);
implementation!(
    crate::trusted_publishing::TrustedPublishingPermit<'r>,
    crate::trusted_publishing::TrustedPublishingResponse,
    permit
);
