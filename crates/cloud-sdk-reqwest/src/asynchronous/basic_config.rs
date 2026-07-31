use core::fmt;

use crate::shared::{BasicCredential, BuildError, HttpsEndpoint, RequestTimeouts, UserAgent};

use super::AsyncBasicClient;
use super::config::configured_raw_client;

/// Builder requiring a scoped Basic credential and complete transport limits.
pub struct AsyncBasicClientBuilder {
    endpoint: HttpsEndpoint,
    credential: BasicCredential,
    user_agent: UserAgent,
    timeouts: RequestTimeouts,
}

impl AsyncBasicClientBuilder {
    /// Creates a complete asynchronous Basic-client configuration.
    #[must_use]
    pub const fn new(
        endpoint: HttpsEndpoint,
        credential: BasicCredential,
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

    /// Builds a hardened HTTPS-only Basic-auth client.
    pub fn build(self) -> Result<AsyncBasicClient, BuildError> {
        self.build_inner(true)
    }

    fn build_inner(self, https_only: bool) -> Result<AsyncBasicClient, BuildError> {
        if !self.credential.scope().matches_endpoint(&self.endpoint) {
            return Err(BuildError::CredentialEndpointMismatch);
        }
        let client = configured_raw_client(
            self.endpoint.clone(),
            &self.user_agent,
            self.timeouts,
            https_only,
        )?;
        Ok(AsyncBasicClient::new(
            client,
            self.endpoint,
            self.credential,
            !https_only,
        ))
    }

    #[cfg(test)]
    pub(super) fn build_for_loopback(self) -> Result<AsyncBasicClient, BuildError> {
        self.build_inner(false)
    }
}

impl fmt::Debug for AsyncBasicClientBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AsyncBasicClientBuilder")
            .field("endpoint", &"[redacted]")
            .field("credential", &"[redacted]")
            .field("user_agent", &self.user_agent)
            .field("timeouts", &self.timeouts)
            .finish()
    }
}
