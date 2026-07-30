use core::fmt;

use reqwest::blocking::Client;
#[cfg(any(
    feature = "blocking-rustls-fips",
    feature = "blocking-rustls-webpki-roots"
))]
use reqwest::blocking::ClientBuilder;
use reqwest::redirect::Policy;
use reqwest::tls::Version;
use rustls::ClientConfig;
#[cfg(any(
    feature = "blocking-rustls-fips",
    feature = "blocking-rustls-webpki-roots"
))]
use rustls::RootCertStore;
#[cfg(feature = "blocking-rustls-fips")]
use rustls::client::WebPkiServerVerifier;
#[cfg(feature = "blocking-rustls-fips")]
use rustls::crypto::CryptoProvider;
#[cfg(feature = "blocking-rustls-fips")]
use rustls::pki_types::CertificateRevocationListDer;
#[cfg(any(
    feature = "blocking-rustls-fips",
    feature = "blocking-rustls-webpki-roots"
))]
use std::sync::Arc;
#[cfg(feature = "blocking-rustls-fips")]
use std::vec::Vec;

#[cfg(all(
    not(feature = "blocking-rustls-fips"),
    not(feature = "blocking-rustls-webpki-roots")
))]
use crate::shared::platform_client_config;
use crate::shared::{
    BearerCredential, BuildError, HttpsEndpoint, RawHyperClient, RequestTimeouts, UserAgent,
};

use super::{BlockingClient, RawBlockingClient};

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
    credential: BearerCredential,
    user_agent: UserAgent,
    timeouts: RequestTimeouts,
    #[cfg(feature = "blocking-rustls-fips")]
    fips_tls_policy: Option<FipsTlsPolicy>,
}

/// Builder for a raw client with no credential, provider, media, or retry policy.
pub struct RawBlockingClientBuilder {
    endpoint: HttpsEndpoint,
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
        credential: BearerCredential,
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

    /// Builds a hardened HTTPS-only client.
    pub fn build(self) -> Result<BlockingClient, BuildError> {
        self.build_inner(true)
    }

    fn build_inner(self, https_only: bool) -> Result<BlockingClient, BuildError> {
        let settings = ClientSettings {
            user_agent: &self.user_agent,
            timeouts: self.timeouts,
            #[cfg(feature = "blocking-rustls-fips")]
            fips_tls_policy: self.fips_tls_policy.as_ref(),
        };
        let client = configured_client(settings, https_only)?;
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

    /// Builds an HTTPS-only executor with no implicit authorization.
    pub fn build(self) -> Result<RawBlockingClient, BuildError> {
        self.build_inner(true)
    }

    fn build_inner(self, https_only: bool) -> Result<RawBlockingClient, BuildError> {
        let settings = ClientSettings {
            user_agent: &self.user_agent,
            timeouts: self.timeouts,
            #[cfg(feature = "blocking-rustls-fips")]
            fips_tls_policy: self.fips_tls_policy.as_ref(),
        };
        let tls_config = raw_tls_config(settings)?;
        let client = RawHyperClient::new(
            self.endpoint.clone(),
            &self.user_agent,
            self.timeouts,
            tls_config,
            https_only,
        )?;
        Ok(RawBlockingClient::new(client, self.endpoint))
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
        #[cfg(feature = "blocking-rustls-fips")]
        debug.field("fips_tls_policy", &self.fips_tls_policy);
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
        #[cfg(feature = "blocking-rustls-fips")]
        debug.field("fips_tls_policy", &self.fips_tls_policy);
        debug.finish()
    }
}

#[derive(Clone, Copy)]
struct ClientSettings<'a> {
    user_agent: &'a UserAgent,
    timeouts: RequestTimeouts,
    #[cfg(feature = "blocking-rustls-fips")]
    fips_tls_policy: Option<&'a FipsTlsPolicy>,
}

fn configured_client(settings: ClientSettings<'_>, https_only: bool) -> Result<Client, BuildError> {
    configured_tls_builder(settings)?
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
        .timeout(settings.timeouts.total())
        .connect_timeout(settings.timeouts.connect())
        .connection_verbose(false)
        .user_agent(settings.user_agent.value.clone())
        .build()
        .map_err(|_| BuildError::ClientBuildFailed)
}

#[cfg(all(
    not(feature = "blocking-rustls-fips"),
    not(feature = "blocking-rustls-webpki-roots")
))]
fn raw_tls_config(_settings: ClientSettings<'_>) -> Result<ClientConfig, BuildError> {
    platform_client_config()
}

#[cfg(all(
    not(feature = "blocking-rustls-fips"),
    feature = "blocking-rustls-webpki-roots"
))]
fn raw_tls_config(_settings: ClientSettings<'_>) -> Result<ClientConfig, BuildError> {
    webpki_roots_client_config()
}

#[cfg(feature = "blocking-rustls-fips")]
fn raw_tls_config(settings: ClientSettings<'_>) -> Result<ClientConfig, BuildError> {
    let policy = settings
        .fips_tls_policy
        .ok_or(BuildError::FipsTlsPolicyRequired)?;
    fips_client_config(policy)
}

#[cfg(all(
    not(feature = "blocking-rustls-fips"),
    not(feature = "blocking-rustls-webpki-roots")
))]
fn configured_tls_builder(
    _settings: ClientSettings<'_>,
) -> Result<reqwest::blocking::ClientBuilder, BuildError> {
    Ok(Client::builder().tls_backend_rustls())
}

#[cfg(all(
    not(feature = "blocking-rustls-fips"),
    feature = "blocking-rustls-webpki-roots"
))]
fn configured_tls_builder(_settings: ClientSettings<'_>) -> Result<ClientBuilder, BuildError> {
    let config = webpki_roots_client_config()?;
    Ok(Client::builder().tls_backend_preconfigured(config))
}

#[cfg(all(
    not(feature = "blocking-rustls-fips"),
    feature = "blocking-rustls-webpki-roots"
))]
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

#[cfg(all(
    test,
    not(feature = "blocking-rustls-fips"),
    feature = "blocking-rustls-webpki-roots"
))]
pub(super) fn test_webpki_roots_configuration() -> Result<(usize, bool), BuildError> {
    let config = webpki_roots_client_config()?;
    Ok((webpki_roots::TLS_SERVER_ROOTS.len(), config.fips()))
}

#[cfg(feature = "blocking-rustls-fips")]
fn configured_tls_builder(settings: ClientSettings<'_>) -> Result<ClientBuilder, BuildError> {
    let policy = settings
        .fips_tls_policy
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
