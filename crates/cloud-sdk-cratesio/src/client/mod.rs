//! Official-origin convenience execution over reviewed provider operations.
//!
//! Mutations still require consumed permits. There are no implicit retries,
//! sleeps, credentials, or custom destinations. Raw adapters remain trusted.
//! Custom adapters must treat targets as potentially secret: never log them,
//! never retain unprotected URI copies, and clear owned staging on completion
//! or cancellation. This also applies to requests without Authorization headers.
//! Bundled raw adapters protect their URI staging; external HTTP/TLS wire buffers
//! and server/proxy access logs remain deployment boundaries.

#[cfg(feature = "async")]
mod asynchronous;
#[cfg(feature = "blocking")]
mod blocking;
mod buffers;
#[cfg(feature = "async")]
pub(crate) mod prepared;
#[cfg(feature = "blocking-rustls")]
mod publish;
#[cfg(feature = "async-rustls")]
mod publish_async;
#[cfg(all(test, feature = "blocking"))]
mod tests;
#[cfg(all(test, feature = "blocking", feature = "async"))]
pub(crate) use asynchronous::tests as async_test_support;
#[cfg(feature = "async")]
pub use asynchronous::{AsyncRegistryOperation, LocalRegistryOperation};
#[cfg(feature = "blocking")]
pub use blocking::BlockingRegistryOperation;
pub use buffers::RegistryBuffers;

#[cfg(feature = "blocking")]
use crate::discovery::DiscoveryExecutionError;
use crate::{
    discovery::{DiscoveryClient, DiscoveryError},
    wire::IdentifyingUserAgent,
};
#[cfg(feature = "blocking")]
use cloud_sdk::transport::BlockingAuthorizedRawHttpExecutor;
use cloud_sdk::transport::{BoundTransport, BoundUserAgent};

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

#[cfg(feature = "blocking")]
impl<T: BlockingAuthorizedRawHttpExecutor + BoundUserAgent + ?Sized> RegistryClient<'_, T> {
    /// Executes a typed read or consumes a mutation permit without caller HTTP
    /// assembly. All scratch is cleared, including constructor/admission failures.
    /// No retry occurs; mutation errors can follow a committed upstream change.
    /// Email confirmation and invitation-token acceptance consume their path
    /// credential and send no Authorization header. Custom executors must meet
    /// this module's secret-target storage and logging obligations.
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
