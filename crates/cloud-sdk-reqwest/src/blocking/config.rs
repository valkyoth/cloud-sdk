use core::fmt;

#[cfg(any(feature = "blocking-rustls", feature = "blocking-rustls-webpki-roots"))]
use rustls::ClientConfig;
#[cfg(feature = "blocking-rustls-webpki-roots")]
use rustls::RootCertStore;
#[cfg(feature = "blocking-rustls-webpki-roots")]
use std::sync::Arc;

#[cfg(not(feature = "blocking-rustls-webpki-roots"))]
use crate::shared::platform_client_config;
use crate::shared::{
    BearerCredential, BuildError, HttpsEndpoint, LinkLocalHttpEndpoint, RawHyperClient,
    RequestTimeouts, UserAgent,
};

use super::{BlockingClient, RawBlockingClient};

/// Builder requiring endpoint, bearer token, user agent, and all timeout
/// dimensions before a client can be constructed.
pub struct BlockingClientBuilder {
    endpoint: HttpsEndpoint,
    credential: BearerCredential,
    user_agent: UserAgent,
    timeouts: RequestTimeouts,
}

/// Builder for a raw client with no credential, provider, media, or retry policy.
pub struct RawBlockingClientBuilder {
    endpoint: HttpsEndpoint,
    user_agent: UserAgent,
    timeouts: RequestTimeouts,
    https_only: bool,
}

impl BlockingClientBuilder {
    /// Creates a complete blocking-client configuration.
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
    pub fn build(self) -> Result<BlockingClient, BuildError> {
        self.build_inner(true)
    }

    fn build_inner(self, https_only: bool) -> Result<BlockingClient, BuildError> {
        if !self.credential.scope.matches_endpoint(&self.endpoint) {
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
        Ok(BlockingClient::new(
            client,
            self.endpoint,
            self.credential,
            !https_only,
        ))
    }

    #[cfg(test)]
    pub(super) fn build_for_loopback(self) -> Result<BlockingClient, BuildError> {
        self.build_inner(false)
    }
}

impl RawBlockingClientBuilder {
    /// Creates a complete raw blocking configuration without credentials.
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
            https_only: true,
        }
    }

    /// Creates a raw direct-link-local HTTP builder with no credential path.
    #[must_use]
    pub fn new_link_local(
        endpoint: LinkLocalHttpEndpoint,
        user_agent: UserAgent,
        timeouts: RequestTimeouts,
    ) -> Self {
        Self {
            endpoint: endpoint.into_inner(),
            user_agent,
            timeouts,
            https_only: false,
        }
    }

    /// Builds an HTTPS-only executor with no implicit authorization.
    pub fn build(self) -> Result<RawBlockingClient, BuildError> {
        let https_only = self.https_only;
        self.build_inner(https_only)
    }

    fn build_inner(self, https_only: bool) -> Result<RawBlockingClient, BuildError> {
        let settings = ClientSettings {
            timeouts: self.timeouts,
        };
        configured_raw_client(self.endpoint, &self.user_agent, settings, https_only)
    }

    #[cfg(test)]
    pub(super) fn build_for_loopback(self) -> Result<RawBlockingClient, BuildError> {
        self.build_inner(false)
    }
}

impl fmt::Debug for BlockingClientBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = formatter.debug_struct("BlockingClientBuilder");
        debug
            .field("endpoint", &"[redacted]")
            .field("credential", &"[redacted]")
            .field("user_agent", &self.user_agent)
            .field("timeouts", &self.timeouts);
        debug.finish()
    }
}

impl fmt::Debug for RawBlockingClientBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = formatter.debug_struct("RawBlockingClientBuilder");
        debug
            .field("endpoint", &"[redacted]")
            .field("user_agent", &self.user_agent)
            .field("timeouts", &self.timeouts);
        debug.field("link_local_http", &!self.https_only);
        debug.finish()
    }
}

#[derive(Clone, Copy)]
pub(super) struct ClientSettings {
    pub(super) timeouts: RequestTimeouts,
}

pub(super) fn configured_raw_client(
    endpoint: HttpsEndpoint,
    user_agent: &UserAgent,
    settings: ClientSettings,
    https_only: bool,
) -> Result<RawBlockingClient, BuildError> {
    let tls_config = raw_tls_config(settings)?;
    let client = RawHyperClient::new(
        endpoint.clone(),
        user_agent,
        settings.timeouts,
        tls_config,
        https_only,
    )?;
    Ok(RawBlockingClient::new(client, endpoint))
}

#[cfg(not(feature = "blocking-rustls-webpki-roots"))]
fn raw_tls_config(_settings: ClientSettings) -> Result<ClientConfig, BuildError> {
    platform_client_config()
}

#[cfg(feature = "blocking-rustls-webpki-roots")]
fn raw_tls_config(_settings: ClientSettings) -> Result<ClientConfig, BuildError> {
    webpki_roots_client_config()
}

#[cfg(feature = "blocking-rustls-webpki-roots")]
fn webpki_roots_client_config() -> Result<ClientConfig, BuildError> {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    Ok(ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|_| BuildError::WebPkiRootsProtocolConfigurationFailed)?
        .with_root_certificates(roots)
        .with_no_client_auth())
}

#[cfg(all(test, feature = "blocking-rustls-webpki-roots"))]
pub(super) fn test_webpki_roots_configuration() -> Result<(usize, bool), BuildError> {
    let config = webpki_roots_client_config()?;
    Ok((webpki_roots::TLS_SERVER_ROOTS.len(), config.fips()))
}
