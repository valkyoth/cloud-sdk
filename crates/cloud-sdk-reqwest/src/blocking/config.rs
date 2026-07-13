use core::fmt;

use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use reqwest::tls::Version;

use crate::shared::{BearerToken, BuildError, HttpsEndpoint, RequestTimeouts, UserAgent};

use super::BlockingClient;

/// Builder requiring endpoint, bearer token, user agent, and all timeout
/// dimensions before a client can be constructed.
pub struct BlockingClientBuilder {
    endpoint: HttpsEndpoint,
    token: BearerToken,
    user_agent: UserAgent,
    timeouts: RequestTimeouts,
}

impl BlockingClientBuilder {
    /// Creates a complete blocking-client configuration.
    #[must_use]
    pub const fn new(
        endpoint: HttpsEndpoint,
        token: BearerToken,
        user_agent: UserAgent,
        timeouts: RequestTimeouts,
    ) -> Self {
        Self {
            endpoint,
            token,
            user_agent,
            timeouts,
        }
    }

    /// Builds a hardened HTTPS-only client.
    pub fn build(self) -> Result<BlockingClient, BuildError> {
        self.build_inner(true)
    }

    fn build_inner(self, https_only: bool) -> Result<BlockingClient, BuildError> {
        let client = configured_client(&self.user_agent, self.timeouts, https_only)?;
        Ok(BlockingClient::new(client, self.endpoint, self.token))
    }

    #[cfg(test)]
    pub(super) fn build_for_loopback(self) -> Result<BlockingClient, BuildError> {
        self.build_inner(false)
    }
}

impl fmt::Debug for BlockingClientBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BlockingClientBuilder")
            .field("endpoint", &"[redacted]")
            .field("token", &"[redacted]")
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
