//! Hardened provider-neutral blocking transport implementation.

mod basic_client;
mod basic_config;
mod body;
mod client;
mod config;
mod raw;

pub use crate::shared::{
    BasicCredential, BasicCredentialError, BasicCredentialScope, BasicCredentialScopeError,
    BasicPassword, BasicPasswordError, BasicUsername, BasicUsernameError, BearerCredential,
    BearerCredentialScope, BearerCredentialScopeError, BearerCredentialSnapshot,
    BearerRefreshHandoff, BearerToken, BearerTokenError, BuildError, CredentialStateError,
    CredentialUpdateError, CustomEndpointAcknowledgement, EndpointError, HttpsEndpoint,
    MAX_BASIC_AUTHORIZATION_BYTES, MAX_BASIC_PASSWORD_BYTES, MAX_BASIC_USERNAME_BYTES,
    MAX_BEARER_TOKEN_BYTES, MAX_CONFIGURED_ENDPOINT_BYTES, MAX_RAW_REQUEST_BODY_BYTES,
    MAX_TIMEOUT_SECONDS, MAX_UPSTREAM_HTTP1_HEAD_BYTES, MAX_UPSTREAM_HTTP1_HEADERS, RawHttpError,
    RawTransportFailure, RequestTimeouts, TimeoutError, TokenRefreshError, TokenRotationError,
    TransportError, UserAgent, UserAgentError,
};
pub use basic_client::BlockingBasicClient;
pub use basic_config::BlockingBasicClientBuilder;
pub use client::BlockingClient;
#[cfg(feature = "blocking-rustls-fips")]
pub use config::FipsTlsPolicy;
pub use config::{BlockingClientBuilder, RawBlockingClientBuilder};
pub use raw::RawBlockingClient;

#[cfg(test)]
mod tests;
#[cfg(all(
    test,
    feature = "blocking-rustls-webpki-roots",
    not(feature = "blocking-rustls-fips")
))]
mod webpki_roots_tests;
