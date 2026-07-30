use core::fmt;

use reqwest::Client;
use reqwest::redirect::Policy;
use reqwest::tls::Version;

use crate::shared::{
    BearerCredential, BuildError, HttpsEndpoint, RawHyperClient, RequestTimeouts, UserAgent,
    platform_client_config,
};

use super::{AsyncClient, RawAsyncClient};

/// Builder requiring endpoint, bearer token, user agent, and all timeout
/// dimensions before an asynchronous client can be constructed.
pub struct AsyncClientBuilder {
    endpoint: HttpsEndpoint,
    credential: BearerCredential,
    user_agent: UserAgent,
    timeouts: RequestTimeouts,
}

/// Builder for a raw async client with no credentials or provider policy.
pub struct RawAsyncClientBuilder {
    endpoint: HttpsEndpoint,
    user_agent: UserAgent,
    timeouts: RequestTimeouts,
}

impl AsyncClientBuilder {
    /// Creates a complete asynchronous-client configuration.
    #[must_use]
    pub const fn new(
        endpoint: HttpsEndpoint,
        credential: BearerCredential,
        user_agent: UserAgent,
        timeouts: RequestTimeouts,
    ) -> Self {
        Self {
            endpoint,
            credential,
            user_agent,
            timeouts,
        }
    }

    /// Builds a hardened HTTPS-only client.
    ///
    /// Sending requests requires an active Tokio executor because reqwest uses
    /// Tokio internally. The core [`cloud_sdk::transport::AsyncTransport`]
    /// contract remains executor-neutral.
    pub fn build(self) -> Result<AsyncClient, BuildError> {
        self.build_inner(true)
    }

    fn build_inner(self, https_only: bool) -> Result<AsyncClient, BuildError> {
        let client = configured_client(&self.user_agent, self.timeouts, https_only)?;
        Ok(AsyncClient::new(
            client,
            self.endpoint,
            self.credential,
            !https_only,
        ))
    }

    #[cfg(test)]
    pub(super) fn build_for_loopback(self) -> Result<AsyncClient, BuildError> {
        self.build_inner(false)
    }
}

impl RawAsyncClientBuilder {
    /// Creates a complete asynchronous raw executor configuration.
    #[must_use]
    pub const fn new(
        endpoint: HttpsEndpoint,
        user_agent: UserAgent,
        timeouts: RequestTimeouts,
    ) -> Self {
        Self {
            endpoint,
            user_agent,
            timeouts,
        }
    }

    /// Builds an HTTPS-only executor with no implicit authorization.
    pub fn build(self) -> Result<RawAsyncClient, BuildError> {
        self.build_inner(true)
    }

    fn build_inner(self, https_only: bool) -> Result<RawAsyncClient, BuildError> {
        let tls_config = platform_client_config()?;
        let client = RawHyperClient::new(
            self.endpoint.clone(),
            &self.user_agent,
            self.timeouts,
            tls_config,
            https_only,
        )?;
        Ok(RawAsyncClient::new(client, self.endpoint))
    }

    #[cfg(test)]
    pub(super) fn build_for_loopback(self) -> Result<RawAsyncClient, BuildError> {
        self.build_inner(false)
    }
}

impl fmt::Debug for AsyncClientBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AsyncClientBuilder")
            .field("endpoint", &"[redacted]")
            .field("credential", &"[redacted]")
            .field("user_agent", &self.user_agent)
            .field("timeouts", &self.timeouts)
            .finish()
    }
}

impl fmt::Debug for RawAsyncClientBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RawAsyncClientBuilder")
            .field("endpoint", &"[redacted]")
            .field("user_agent", &self.user_agent)
            .field("timeouts", &self.timeouts)
            .finish()
    }
}

fn configured_client(
    user_agent: &UserAgent,
    timeouts: RequestTimeouts,
    https_only: bool,
) -> Result<Client, BuildError> {
    Client::builder()
        .tls_backend_rustls()
        .https_only(https_only)
        .http1_only()
        .no_hickory_dns()
        .min_tls_version(Version::TLS_1_2)
        .redirect(Policy::none())
        .retry(reqwest::retry::never())
        .referer(false)
        .no_proxy()
        .no_gzip()
        .no_brotli()
        .no_zstd()
        .no_deflate()
        .timeout(timeouts.total())
        .connect_timeout(timeouts.connect())
        .connection_verbose(false)
        .user_agent(user_agent.value.clone())
        .build()
        .map_err(|_| BuildError::ClientBuildFailed)
}
