/// Client construction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildError {
    /// Rustls could not enable its safe protocol-version set.
    ProtocolConfigurationFailed,
    /// The operating-system trust verifier could not be configured.
    PlatformVerifierConfigurationFailed,
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
    /// The bounded raw HTTP executor rejected or failed the request.
    RawHttp(RawHttpError),
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
    Self::InvalidResponseContentType => "response content type is invalid",
    Self::InvalidResponseHeaders => "response headers are invalid",
    Self::ResponseTooLarge => "response body exceeds the caller limit",
    Self::ResponseReadFailed => "response body read failed",
    Self::ResponseCommitFailed => "response commitment failed",
    Self::ResponseOriginChanged => "response origin changed",
    Self::RawHttp(_) => "bounded raw HTTP execution failed",
);
use super::RawHttpError;
