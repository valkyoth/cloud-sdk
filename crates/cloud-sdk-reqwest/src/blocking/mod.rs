//! Hardened provider-neutral blocking transport implementation.

mod body;
mod client;
mod config;

pub use crate::shared::{
    BearerToken, BearerTokenError, BuildError, EndpointError, HttpsEndpoint,
    MAX_BEARER_TOKEN_BYTES, MAX_TIMEOUT_SECONDS, RequestTimeouts, TimeoutError, TransportError,
    UserAgent, UserAgentError,
};
pub use client::BlockingClient;
pub use config::BlockingClientBuilder;

#[cfg(test)]
mod test_server;
#[cfg(test)]
mod tests;
