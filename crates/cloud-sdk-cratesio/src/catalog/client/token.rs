use super::*;
use crate::{
    credentials::{ApiToken, CredentialContext, CredentialOrigin, ScopedCredentialMaterial},
    query::MAX_TARGET_BYTES,
};
use cloud_sdk::{
    Method,
    transport::{
        RawResponsePolicy, RequestHeaders, ResponseBuffer, ResponseWriter, TransportRequest,
    },
};

impl<T: BoundTransport + BoundUserAgent + ?Sized> CatalogClient<'_, T> {
    /// Optional API-token list execution through an explicitly trusted blocking
    /// adapter callback. Only List admits tokens, including the following filter.
    ///
    /// The callback must send exactly the supplied request to the bound executor,
    /// apply the raw Authorization value once as sensitive, enforce the response
    /// policy, and disable cookies/redirects/retries. This is an adapter boundary,
    /// not an authorization-header escape hatch on the anonymous raw executor.
    /// Secret scratch is cleared on every exit. Built-in token transports and
    /// async credential workflows are separate from anonymous catalog execution.
    pub fn execute_with_token<E>(
        &self,
        request: CatalogRequest<'_>,
        token: &ApiToken,
        secret_storage: &mut [u8],
        storage: &mut [u8],
        headers: &mut [u8],
        send: impl for<'r, 'p, 'w, 'b> FnOnce(
            &T,
            ScopedCredentialMaterial<'r>,
            TransportRequest<'r>,
            RawResponsePolicy<'p>,
            &'w mut ResponseWriter<'b>,
        ) -> Result<(), E>,
    ) -> Result<CatalogResponse, CatalogExecutionError<E>> {
        let mut secret = cloud_sdk_sanitization::SecretBuffer::new(secret_storage);
        cloud_sdk_sanitization::sanitize_bytes(secret.as_mut_slice());
        let mut response = ResponseBuffer::new(storage, self.0.maximum, headers);
        if request.operation() != super::super::CatalogOperation::List {
            return Err(CatalogExecutionError::Model(CatalogError::Binding));
        }
        self.0.verify().map_err(CatalogExecutionError::Model)?;
        let mut bytes = [0; MAX_TARGET_BYTES];
        let target = request
            .write_target(&mut bytes)
            .map_err(|_| CatalogExecutionError::Model(CatalogError::Binding))?;
        let origin =
            if self.0.endpoint == crate::endpoint::OfficialCratesIoEndpoint::production_api() {
                CredentialOrigin::Production
            } else {
                CredentialOrigin::Staging
            };
        let context = CredentialContext::api(origin, Method::Get, target)
            .map_err(|_| CatalogExecutionError::Model(CatalogError::Binding))?;
        let headers = self.0.headers().map_err(CatalogExecutionError::Model)?;
        let wire = TransportRequest::new(Method::Get, target.as_request_target()).with_headers(
            RequestHeaders::new(&headers)
                .map_err(|_| CatalogExecutionError::Model(CatalogError::Value))?,
        );
        let policy = self.0.policy().map_err(CatalogExecutionError::Model)?;
        // Admission is shared with every anonymous operation. The material
        // closure cannot return borrowed secret storage or a borrowing future.
        let mut attempt = self
            .0
            .gate
            .begin()
            .map_err(CatalogExecutionError::Schedule)?;
        token
            .with_material_for_adapter(
                &context,
                self.0.executor,
                secret.as_mut_slice(),
                |executor, material| send(executor, material, wire, policy, response.writer()),
            )
            .map_err(|_| CatalogExecutionError::Model(CatalogError::Binding))?
            .map_err(CatalogExecutionError::Transport)?;
        self.0.decode(request, response, &mut attempt)
    }
}
