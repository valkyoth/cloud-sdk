//! Hardened provider-neutral blocking transport implementation.

mod auth;
mod body;
mod client;
mod config;
mod endpoint;
mod error;

pub use auth::{BearerToken, BearerTokenError, MAX_BEARER_TOKEN_BYTES};
pub use client::BlockingClient;
pub use config::{BlockingClientBuilder, RequestTimeouts, TimeoutError, UserAgent, UserAgentError};
pub use endpoint::{EndpointError, HttpsEndpoint};
pub use error::{BuildError, TransportError};

#[cfg(test)]
mod test_server;
#[cfg(test)]
mod tests;
