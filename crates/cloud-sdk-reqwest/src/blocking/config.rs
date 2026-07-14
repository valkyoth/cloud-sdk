use core::fmt;

use reqwest::blocking::Client;
#[cfg(feature = "blocking-rustls-fips")]
use reqwest::blocking::ClientBuilder;
use reqwest::redirect::Policy;
use reqwest::tls::Version;
#[cfg(feature = "blocking-rustls-fips")]
use rustls::ClientConfig;
#[cfg(feature = "blocking-rustls-fips")]
use rustls::crypto::CryptoProvider;
#[cfg(feature = "blocking-rustls-fips")]
use rustls_platform_verifier::BuilderVerifierExt;
#[cfg(feature = "blocking-rustls-fips")]
use std::sync::Arc;

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
    configured_tls_builder()?
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

#[cfg(not(feature = "blocking-rustls-fips"))]
fn configured_tls_builder() -> Result<reqwest::blocking::ClientBuilder, BuildError> {
    Ok(Client::builder().tls_backend_rustls())
}

#[cfg(feature = "blocking-rustls-fips")]
fn configured_tls_builder() -> Result<ClientBuilder, BuildError> {
    let config = fips_client_config()?;
    Ok(Client::builder().tls_backend_preconfigured(config))
}

#[cfg(feature = "blocking-rustls-fips")]
fn fips_client_config() -> Result<ClientConfig, BuildError> {
    let provider = Arc::new(rustls::crypto::default_fips_provider());
    validate_fips_provider(provider.as_ref())?;
    let config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|_| BuildError::FipsProtocolConfigurationFailed)?
        .with_platform_verifier()
        .map_err(|_| BuildError::FipsPlatformVerifierFailed)?
        .with_no_client_auth();
    validate_fips_config(&config)?;
    Ok(config)
}

#[cfg(feature = "blocking-rustls-fips")]
fn validate_fips_provider(provider: &CryptoProvider) -> Result<(), BuildError> {
    if provider.fips() {
        Ok(())
    } else {
        Err(BuildError::FipsProviderRejected)
    }
}

#[cfg(feature = "blocking-rustls-fips")]
fn validate_fips_config(config: &ClientConfig) -> Result<(), BuildError> {
    if config.fips() {
        Ok(())
    } else {
        Err(BuildError::FipsClientConfigurationRejected)
    }
}

#[cfg(all(test, feature = "blocking-rustls-fips"))]
pub(super) fn test_fips_configuration() -> Result<bool, BuildError> {
    fips_client_config().map(|config| config.fips())
}
