//! Hardened provider-neutral asynchronous transport implementation.

mod body;
mod client;
mod config;

pub use crate::shared::{
    BearerToken, BearerTokenError, BuildError, EndpointError, HttpsEndpoint,
    MAX_BEARER_TOKEN_BYTES, MAX_TIMEOUT_SECONDS, RequestTimeouts, TimeoutError, TransportError,
    UserAgent, UserAgentError,
};
pub use client::AsyncClient;
pub use config::AsyncClientBuilder;

#[cfg(test)]
mod tests;
