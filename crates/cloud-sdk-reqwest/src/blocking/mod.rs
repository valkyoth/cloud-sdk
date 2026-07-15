//! Hardened provider-neutral blocking transport implementation.

mod body;
mod client;
mod config;

pub use crate::shared::{
    BearerToken, BearerTokenError, BuildError, CredentialStateError, EndpointError, HttpsEndpoint,
    MAX_BEARER_TOKEN_BYTES, MAX_TIMEOUT_SECONDS, RequestTimeouts, TimeoutError, TokenRotationError,
    TransportError, UserAgent, UserAgentError,
};
pub use client::BlockingClient;
pub use config::BlockingClientBuilder;
#[cfg(feature = "blocking-rustls-fips")]
pub use config::FipsTlsPolicy;

#[cfg(test)]
mod tests;
#[cfg(all(
    test,
    feature = "blocking-rustls-webpki-roots",
    not(feature = "blocking-rustls-fips")
))]
mod webpki_roots_tests;
