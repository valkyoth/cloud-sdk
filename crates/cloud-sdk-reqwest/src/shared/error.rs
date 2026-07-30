/// Client construction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildError {
    /// Rustls could not enable its safe protocol-version set.
    ProtocolConfigurationFailed,
    /// The operating-system trust verifier could not be configured.
    PlatformVerifierConfigurationFailed,
    /// FIPS transport construction omitted its required trust and revocation policy.
    FipsTlsPolicyRequired,
    /// The FIPS trust policy contained no trust anchors.
    FipsTrustRootsRequired,
    /// The FIPS trust policy contained no certificate revocation lists.
    FipsCertificateRevocationListsRequired,
    /// The selected cryptographic provider did not report FIPS operation.
    FipsProviderRejected,
    /// Rustls could not enable its safe protocol-version set for the FIPS provider.
    FipsProtocolConfigurationFailed,
    /// The chain-wide, fail-closed certificate revocation verifier could not be configured.
    FipsRevocationVerifierFailed,
    /// The complete TLS client configuration did not report FIPS operation.
    FipsClientConfigurationRejected,
    /// Rustls could not enable its safe protocol-version set for deterministic roots.
    WebPkiRootsProtocolConfigurationFailed,
    /// Reqwest rejected the fixed hardened client configuration.
    ClientBuildFailed,
    /// The bearer credential is bound to a different transport endpoint.
    CredentialEndpointMismatch,
}

impl_static_error!(BuildError,
    Self::ProtocolConfigurationFailed => "TLS protocol configuration failed",
    Self::PlatformVerifierConfigurationFailed => "platform verifier configuration failed",
    Self::FipsTlsPolicyRequired => "FIPS TLS policy is required",
    Self::FipsTrustRootsRequired => "FIPS trust roots are required",
    Self::FipsCertificateRevocationListsRequired => "FIPS revocation lists are required",
    Self::FipsProviderRejected => "cryptographic provider did not report FIPS operation",
    Self::FipsProtocolConfigurationFailed => "FIPS protocol configuration failed",
    Self::FipsRevocationVerifierFailed => "FIPS revocation verifier configuration failed",
    Self::FipsClientConfigurationRejected => "TLS client did not report FIPS operation",
    Self::WebPkiRootsProtocolConfigurationFailed => "web PKI protocol configuration failed",
    Self::ClientBuildFailed => "HTTP client construction failed",
    Self::CredentialEndpointMismatch => "credential endpoint differs from transport endpoint",
);

/// Payload-free transport failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportError {
    /// The shared credential state could not be read safely.
    CredentialStateUnavailable,
    /// Authentication was attempted against a non-HTTPS endpoint.
    InsecureAuthenticationEndpoint,
    /// The operation authentication endpoint differs from the configured endpoint.
    AuthenticationEndpointMismatch,
    /// The credential scope failed provider or operation policy.
    AuthenticationScopeRejected,
    /// The target could not be composed without parsing or normalization.
    TargetRejected,
    /// The validated SDK method could not be represented by the HTTP implementation.
    MethodRejected,
    /// A non-empty body omitted its required explicit content type.
    MissingContentType,
    /// A validated header could not be represented by the HTTP implementation.
    HeaderRejected,
    /// Adapter-owned request-body allocation failed.
    RequestBodyAllocationFailed,
    /// The request body length cannot be represented by the HTTP client.
    RequestBodyTooLarge,
    /// Adapter-owned response-body allocation failed.
    ResponseBodyAllocationFailed,
    /// Connection establishment failed.
    ConnectFailed,
    /// The configured request or read deadline expired.
    TimedOut,
    /// Sending failed for another payload-free reason.
    RequestFailed,
    /// The response status is outside the core SDK's admitted HTTP range.
    InvalidStatus,
    /// Rate-limit response headers were incomplete, non-decimal, or incoherent.
    InvalidRateLimitHeaders,
    /// The response content type was duplicated, non-textual, or malformed.
    InvalidResponseContentType,
    /// Response headers exceeded bounds, contained controls, or were duplicated.
    InvalidResponseHeaders,
    /// The declared or observed response body exceeds the caller buffer.
    ResponseTooLarge,
    /// Reading the response body failed.
    ResponseReadFailed,
    /// The admitted core response writer rejected the completed response.
    ResponseCommitFailed,
    /// The final response origin differed from the configured endpoint.
    ResponseOriginChanged,
}

impl_static_error!(TransportError,
    Self::CredentialStateUnavailable => "credential state is unavailable",
    Self::InsecureAuthenticationEndpoint => "authenticated transport endpoint is not HTTPS",
    Self::AuthenticationEndpointMismatch => "authentication endpoint differs from transport endpoint",
    Self::AuthenticationScopeRejected => "bearer credential scope was rejected",
    Self::TargetRejected => "request target was rejected",
    Self::MethodRejected => "request method was rejected",
    Self::MissingContentType => "request body content type is missing",
    Self::HeaderRejected => "request header was rejected",
    Self::RequestBodyAllocationFailed => "request-body allocation failed",
    Self::RequestBodyTooLarge => "request body is too large",
    Self::ResponseBodyAllocationFailed => "response-body allocation failed",
    Self::ConnectFailed => "connection failed",
    Self::TimedOut => "request timed out",
    Self::RequestFailed => "request failed",
    Self::InvalidStatus => "response status is invalid",
    Self::InvalidRateLimitHeaders => "rate-limit headers are invalid",
    Self::InvalidResponseContentType => "response content type is invalid",
    Self::InvalidResponseHeaders => "response headers are invalid",
    Self::ResponseTooLarge => "response body exceeds the caller limit",
    Self::ResponseReadFailed => "response body read failed",
    Self::ResponseCommitFailed => "response commitment failed",
    Self::ResponseOriginChanged => "response origin changed",
);
