use super::*;
use crate::{
    catalog::{CatalogOperation, CatalogRequest, CatalogResponse},
    client::prepared::{self, Permit, Prepared},
    credentials::{ApiToken, CredentialOrigin},
    discovery::DiscoveryError as Error,
    wire::JsonSuccess,
};
use cloud_sdk::{Method, transport::BoundTransport};
use cloud_sdk_sanitization::SecretBuffer;

struct CatalogToken<'a>(CatalogRequest<'a>, &'a ApiToken);
impl Permit for CatalogToken<'_> {
    type Response = CatalogResponse;
    fn origin(&self) -> CredentialOrigin {
        self.1.origin()
    }
    fn prepare<'a, T: BoundTransport + ?Sized>(
        &self,
        executor: &T,
        target: &'a mut [u8],
        credential: &'a mut SecretBuffer<'_>,
        _body: &'a mut [u8],
        _now: u64,
    ) -> Result<Prepared<'a>, Error> {
        if self.0.operation() != CatalogOperation::List {
            return Err(Error::Binding);
        }
        let target = self.0.write_target(target).map_err(|_| Error::Binding)?;
        prepared::api(self.1, Method::Get, target, executor, credential, &[])
    }
    fn decode(self, response: JsonSuccess<'_>, _now: u64) -> Result<Self::Response, Error> {
        self.0.decode(self.origin().endpoint(), response)
    }
}
macro_rules! mode {
    ($method:ident, $run:ident, $transport:ident $(, $bound:ident, $sync:ident)?) => {
        impl<T: $transport + BoundUserAgent $(+ $sync)? + ?Sized> RegistryClient<'_, T> {
            /// Explicit API-token list execution, including following filters.
            /// Other catalog operations reject credentials before dispatch.
            /// All scratch is guarded before polling; no retry or sleep occurs.
            pub fn $method<'s>(
                &'s self, request: CatalogRequest<'s>, token: &'s ApiToken,
                buffers: RegistryBuffers<'s>,
            ) -> impl Future<Output = Result<CatalogResponse, DiscoveryExecutionError<T::Error>>> $(+ $bound)? + 's {
                let mut guard = crate::client::buffers::Guard::new(buffers);
                async move {
                    let client = if self.staging {
                        DiscoveryClient::staging(self.executor, self.identity, self.maximum)
                    } else {
                        DiscoveryClient::production(self.executor, self.identity, self.maximum)
                    }.map_err(DiscoveryExecutionError::Model)?;
                    runner::$run(&client, CatalogToken(request, token), guard.parts()).await
                }
            }
        }
    };
}
mode!(
    catalog_with_token_local,
    execute_local,
    LocalAuthorizedRawHttpExecutor
);
mode!(
    catalog_with_token_async,
    execute_async,
    AsyncAuthorizedRawHttpExecutor,
    Send,
    Sync
);
