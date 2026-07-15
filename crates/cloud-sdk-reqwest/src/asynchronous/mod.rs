//! Hardened provider-neutral asynchronous transport implementation.

mod body;
mod client;
mod config;

pub use crate::shared::{
    BearerToken, BearerTokenError, BuildError, CredentialStateError, EndpointError, HttpsEndpoint,
    MAX_BEARER_TOKEN_BYTES, MAX_TIMEOUT_SECONDS, RequestTimeouts, TimeoutError, TokenRotationError,
    TransportError, UserAgent, UserAgentError,
};
pub use client::AsyncClient;
pub use config::AsyncClientBuilder;

#[cfg(test)]
mod tests;
