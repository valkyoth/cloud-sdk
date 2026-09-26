//! Official-origin convenience execution over reviewed provider operations.
//!
//! Mutations still require consumed permits. There are no implicit retries,
//! sleeps, credentials, or custom destinations. Raw adapters remain trusted.

mod blocking;
mod buffers;
#[cfg(test)]
mod tests;
pub use blocking::BlockingRegistryOperation;
pub use buffers::RegistryBuffers;

use crate::{
    discovery::{DiscoveryClient, DiscoveryError, DiscoveryExecutionError},
    wire::IdentifyingUserAgent,
};
use cloud_sdk::transport::{BlockingAuthorizedRawHttpExecutor, BoundTransport, BoundUserAgent};

/// One fixed-origin registry client. Cloning the underlying transport never
/// bypasses the process-wide crates.io admission gate.
pub struct RegistryClient<'a, T: ?Sized> {
    executor: &'a T,
    identity: IdentifyingUserAgent<'a>,
    maximum: usize,
    staging: bool,
}

impl<T: ?Sized> core::fmt::Debug for RegistryClient<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("RegistryClient([redacted])")
    }
}

impl<'a, T: BoundTransport + BoundUserAgent + ?Sized> RegistryClient<'a, T> {
    /// Requires exactly the official production API and the supplied identity.
    pub fn production(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum_response_bytes: usize,
    ) -> Result<Self, DiscoveryError> {
        DiscoveryClient::production(executor, identity, maximum_response_bytes)?;
        Ok(Self {
            executor,
            identity,
            maximum: maximum_response_bytes,
            staging: false,
        })
    }

    /// Requires exactly staging; production credentials cannot be reused here.
    pub fn staging(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum_response_bytes: usize,
    ) -> Result<Self, DiscoveryError> {
        DiscoveryClient::staging(executor, identity, maximum_response_bytes)?;
        Ok(Self {
            executor,
            identity,
            maximum: maximum_response_bytes,
            staging: true,
        })
    }
}

impl<T: BlockingAuthorizedRawHttpExecutor + BoundUserAgent + ?Sized> RegistryClient<'_, T> {
    /// Executes a typed read or consumes a mutation permit without caller HTTP
    /// assembly. All scratch is cleared, including constructor/admission failures.
    /// No retry occurs; mutation errors can follow a committed upstream change.
    /// Email confirmation and token-based invitation acceptance currently return
    /// [`DiscoveryError::Binding`] before dispatch: URI-secret cleanup is not yet
    /// qualified for this facade. Their existing trusted callback API is unchanged.
    pub fn execute<R: BlockingRegistryOperation>(
        &self,
        operation: R,
        buffers: RegistryBuffers<'_>,
    ) -> Result<R::Response, DiscoveryExecutionError<T::Error>> {
        operation.run(self, buffers)
    }

    /// Explicit API-token catalog search, including the following filter. Only
    /// the source-locked List operation accepts this credential mode.
    pub fn catalog_with_token(
        &self,
        request: crate::catalog::CatalogRequest<'_>,
        token: &crate::credentials::ApiToken,
        buffers: RegistryBuffers<'_>,
    ) -> Result<crate::catalog::CatalogResponse, DiscoveryExecutionError<T::Error>> {
        let mut buffers = buffers::Guard::new(buffers);
        let buffers = buffers.parts();
        let client = if self.staging {
            crate::catalog::CatalogClient::staging(self.executor, self.identity, self.maximum)
        } else {
            crate::catalog::CatalogClient::production(self.executor, self.identity, self.maximum)
        }
        .map_err(DiscoveryExecutionError::Model)?;
        client.execute_with_token(
            request,
            token,
            buffers.credential,
            buffers.response,
            buffers.headers,
            blocking::dispatch,
        )
    }
}
