use core::fmt;

use crate::shared::{BasicCredential, BuildError, HttpsEndpoint, RequestTimeouts, UserAgent};

use super::BlockingBasicClient;
use super::config::{ClientSettings, configured_raw_client};

/// Builder requiring a scoped Basic credential and complete transport limits.
pub struct BlockingBasicClientBuilder {
    endpoint: HttpsEndpoint,
    credential: BasicCredential,
    user_agent: UserAgent,
    timeouts: RequestTimeouts,
}

impl BlockingBasicClientBuilder {
    /// Creates a complete blocking Basic-client configuration.
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
    pub fn build(self) -> Result<BlockingBasicClient, BuildError> {
        self.build_inner(true)
    }

    fn build_inner(self, https_only: bool) -> Result<BlockingBasicClient, BuildError> {
        if !self.credential.scope().matches_endpoint(&self.endpoint) {
            return Err(BuildError::CredentialEndpointMismatch);
        }
        let settings = ClientSettings {
            timeouts: self.timeouts,
        };
        let client = configured_raw_client(
            self.endpoint.clone(),
            &self.user_agent,
            settings,
            https_only,
        )?;
        Ok(BlockingBasicClient::new(
            client,
            self.endpoint,
            self.credential,
            !https_only,
        ))
    }

    #[cfg(test)]
    pub(super) fn build_for_loopback(self) -> Result<BlockingBasicClient, BuildError> {
        self.build_inner(false)
    }
}

impl fmt::Debug for BlockingBasicClientBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = formatter.debug_struct("BlockingBasicClientBuilder");
        debug
            .field("endpoint", &"[redacted]")
            .field("credential", &"[redacted]")
            .field("user_agent", &self.user_agent)
            .field("timeouts", &self.timeouts);
        debug.finish()
    }
}
