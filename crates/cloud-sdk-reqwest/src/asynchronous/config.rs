use core::fmt;

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
    /// Tokio internally. The core
    /// [`cloud_sdk::authentication::AsyncAuthenticatedTransport`] contract
    /// remains executor-neutral.
    pub fn build(self) -> Result<AsyncClient, BuildError> {
        self.build_inner(true)
    }

    fn build_inner(self, https_only: bool) -> Result<AsyncClient, BuildError> {
        if !self.credential.scope.matches_endpoint(&self.endpoint) {
            return Err(BuildError::CredentialEndpointMismatch);
        }
        let client = configured_raw_client(
            self.endpoint.clone(),
            &self.user_agent,
            self.timeouts,
            https_only,
        )?;
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
        configured_raw_client(self.endpoint, &self.user_agent, self.timeouts, https_only)
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

pub(super) fn configured_raw_client(
    endpoint: HttpsEndpoint,
    user_agent: &UserAgent,
    timeouts: RequestTimeouts,
    https_only: bool,
) -> Result<RawAsyncClient, BuildError> {
    let tls_config = platform_client_config()?;
    let client = RawHyperClient::new(
        endpoint.clone(),
        user_agent,
        timeouts,
        tls_config,
        https_only,
    )?;
    Ok(RawAsyncClient::new(client, endpoint))
}
