use core::fmt;

use reqwest::blocking::Client;
#[cfg(feature = "blocking-rustls-fips")]
use reqwest::blocking::ClientBuilder;
use reqwest::redirect::Policy;
use reqwest::tls::Version;
#[cfg(feature = "blocking-rustls-fips")]
use rustls::client::WebPkiServerVerifier;
#[cfg(feature = "blocking-rustls-fips")]
use rustls::crypto::CryptoProvider;
#[cfg(feature = "blocking-rustls-fips")]
use rustls::pki_types::CertificateRevocationListDer;
#[cfg(feature = "blocking-rustls-fips")]
use rustls::{ClientConfig, RootCertStore};
#[cfg(feature = "blocking-rustls-fips")]
use std::sync::Arc;
#[cfg(feature = "blocking-rustls-fips")]
use std::vec::Vec;

use crate::shared::{BearerToken, BuildError, HttpsEndpoint, RequestTimeouts, UserAgent};

use super::BlockingClient;

/// Deployment-managed trust anchors and complete CRLs for FIPS TLS.
#[cfg(feature = "blocking-rustls-fips")]
pub struct FipsTlsPolicy {
    roots: Arc<RootCertStore>,
    crls: Vec<CertificateRevocationListDer<'static>>,
}

#[cfg(feature = "blocking-rustls-fips")]
impl FipsTlsPolicy {
    /// Creates a policy that checks the complete certificate chain, rejects
    /// unknown revocation status, and rejects expired CRLs.
    pub fn new(
        roots: RootCertStore,
        crls: Vec<CertificateRevocationListDer<'static>>,
    ) -> Result<Self, BuildError> {
        if roots.is_empty() {
            return Err(BuildError::FipsTrustRootsRequired);
        }
        if crls.is_empty() {
            return Err(BuildError::FipsCertificateRevocationListsRequired);
        }
        Ok(Self {
            roots: Arc::new(roots),
            crls,
        })
    }
}

#[cfg(feature = "blocking-rustls-fips")]
impl fmt::Debug for FipsTlsPolicy {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FipsTlsPolicy")
            .field("trust_anchors", &self.roots.len())
            .field("crls", &self.crls.len())
            .finish_non_exhaustive()
    }
}

/// Builder requiring endpoint, bearer token, user agent, and all timeout
/// dimensions before a client can be constructed.
pub struct BlockingClientBuilder {
    endpoint: HttpsEndpoint,
    token: BearerToken,
    user_agent: UserAgent,
    timeouts: RequestTimeouts,
    #[cfg(feature = "blocking-rustls-fips")]
    fips_tls_policy: Option<FipsTlsPolicy>,
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

    /// Builds a hardened HTTPS-only client.
    pub fn build(self) -> Result<BlockingClient, BuildError> {
        self.build_inner(true)
    }

    fn build_inner(self, https_only: bool) -> Result<BlockingClient, BuildError> {
        let client = configured_client(&self, https_only)?;
        Ok(BlockingClient::new(client, self.endpoint, self.token))
    }

    #[cfg(test)]
    pub(super) fn build_for_loopback(self) -> Result<BlockingClient, BuildError> {
        self.build_inner(false)
    }
}

impl fmt::Debug for BlockingClientBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = formatter.debug_struct("BlockingClientBuilder");
        debug
            .field("endpoint", &"[redacted]")
            .field("token", &"[redacted]")
            .field("user_agent", &self.user_agent)
            .field("timeouts", &self.timeouts);
        #[cfg(feature = "blocking-rustls-fips")]
        debug.field("fips_tls_policy", &self.fips_tls_policy);
        debug.finish()
    }
}

fn configured_client(
    configuration: &BlockingClientBuilder,
    https_only: bool,
) -> Result<Client, BuildError> {
    configured_tls_builder(configuration)?
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
        .timeout(configuration.timeouts.total())
        .connect_timeout(configuration.timeouts.connect())
        .connection_verbose(false)
        .user_agent(configuration.user_agent.value.clone())
        .build()
        .map_err(|_| BuildError::ClientBuildFailed)
}

#[cfg(not(feature = "blocking-rustls-fips"))]
fn configured_tls_builder(
    _configuration: &BlockingClientBuilder,
) -> Result<reqwest::blocking::ClientBuilder, BuildError> {
    Ok(Client::builder().tls_backend_rustls())
}

#[cfg(feature = "blocking-rustls-fips")]
fn configured_tls_builder(
    configuration: &BlockingClientBuilder,
) -> Result<ClientBuilder, BuildError> {
    let policy = configuration
        .fips_tls_policy
        .as_ref()
        .ok_or(BuildError::FipsTlsPolicyRequired)?;
    let config = fips_client_config(policy)?;
    Ok(Client::builder().tls_backend_preconfigured(config))
}

#[cfg(feature = "blocking-rustls-fips")]
fn fips_client_config(policy: &FipsTlsPolicy) -> Result<ClientConfig, BuildError> {
    let provider = Arc::new(rustls::crypto::default_fips_provider());
    validate_fips_provider(provider.as_ref())?;
    let config = client_config_with_provider(provider, policy)?;
    validate_fips_config(&config)?;
    Ok(config)
}

#[cfg(feature = "blocking-rustls-fips")]
fn client_config_with_provider(
    provider: Arc<CryptoProvider>,
    policy: &FipsTlsPolicy,
) -> Result<ClientConfig, BuildError> {
    let verifier = WebPkiServerVerifier::builder_with_provider(
        Arc::clone(&policy.roots),
        Arc::clone(&provider),
    )
    .with_crls(policy.crls.iter().cloned())
    .enforce_revocation_expiration()
    .build()
    .map_err(|_| BuildError::FipsRevocationVerifierFailed)?;
    Ok(ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|_| BuildError::FipsProtocolConfigurationFailed)?
        .dangerous()
        .with_custom_certificate_verifier(verifier)
        .with_no_client_auth())
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
pub(super) fn test_fips_configuration(policy: &FipsTlsPolicy) -> Result<bool, BuildError> {
    fips_client_config(policy).map(|config| config.fips())
}

#[cfg(all(test, feature = "blocking-rustls-fips"))]
fn non_fips_provider() -> CryptoProvider {
    #[derive(Debug)]
    struct NonFipsRandom;

    impl rustls::crypto::SecureRandom for NonFipsRandom {
        fn fill(&self, _buffer: &mut [u8]) -> Result<(), rustls::crypto::GetRandomFailed> {
            Err(rustls::crypto::GetRandomFailed)
        }
    }

    static NON_FIPS_RANDOM: NonFipsRandom = NonFipsRandom;
    let mut provider = rustls::crypto::default_fips_provider();
    provider.secure_random = &NON_FIPS_RANDOM;
    provider
}

#[cfg(all(test, feature = "blocking-rustls-fips"))]
pub(super) fn test_non_fips_rejection(policy: &FipsTlsPolicy) -> Result<bool, BuildError> {
    let provider = non_fips_provider();
    let provider_rejected =
        validate_fips_provider(&provider) == Err(BuildError::FipsProviderRejected);
    let config = client_config_with_provider(Arc::new(provider), policy)?;
    let config_rejected =
        validate_fips_config(&config) == Err(BuildError::FipsClientConfigurationRejected);
    Ok(provider_rejected && config_rejected)
}

#[cfg(all(test, feature = "blocking-rustls-fips"))]
pub(super) fn test_non_fips_global_independence(policy: &FipsTlsPolicy) -> bool {
    let provider = non_fips_provider();
    if provider.fips() || provider.install_default().is_err() {
        return false;
    }
    let global_is_non_fips = CryptoProvider::get_default().is_some_and(|value| !value.fips());
    global_is_non_fips && test_fips_configuration(policy) == Ok(true)
}
