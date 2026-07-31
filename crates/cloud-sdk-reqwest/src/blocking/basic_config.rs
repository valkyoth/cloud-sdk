use core::fmt;

use crate::shared::{BasicCredential, BuildError, HttpsEndpoint, RequestTimeouts, UserAgent};

use super::BlockingBasicClient;
#[cfg(feature = "blocking-rustls-fips")]
use super::config::FipsTlsPolicy;
use super::config::{ClientSettings, configured_raw_client};

/// Builder requiring a scoped Basic credential and complete transport limits.
pub struct BlockingBasicClientBuilder {
    endpoint: HttpsEndpoint,
    credential: BasicCredential,
    user_agent: UserAgent,
    timeouts: RequestTimeouts,
    #[cfg(feature = "blocking-rustls-fips")]
    fips_tls_policy: Option<FipsTlsPolicy>,
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
            #[cfg(feature = "blocking-rustls-fips")]
            fips_tls_policy: None,
        }
    }

    /// Supplies mandatory deployment-managed roots and CRLs for FIPS TLS.
    #[cfg(feature = "blocking-rustls-fips")]
    #[must_use]
    pub fn with_fips_tls_policy(mut self, policy: FipsTlsPolicy) -> Self {
        self.fips_tls_policy = Some(policy);
        self
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
            #[cfg(feature = "blocking-rustls-fips")]
            fips_tls_policy: self.fips_tls_policy.as_ref(),
            #[cfg(not(feature = "blocking-rustls-fips"))]
            _lifetime: core::marker::PhantomData,
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
        #[cfg(feature = "blocking-rustls-fips")]
        debug.field("fips_tls_policy", &self.fips_tls_policy);
        debug.finish()
    }
}
