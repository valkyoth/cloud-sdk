//! Hardened provider-neutral asynchronous transport implementation.

mod body;
mod client;
mod config;
mod raw;

pub use crate::shared::{
    BearerCredential, BearerCredentialScope, BearerCredentialScopeError, BearerCredentialSnapshot,
    BearerRefreshHandoff, BearerToken, BearerTokenError, BuildError, CredentialStateError,
    CredentialUpdateError, CustomEndpointAcknowledgement, EndpointError, HttpsEndpoint,
    MAX_BEARER_TOKEN_BYTES, MAX_CONFIGURED_ENDPOINT_BYTES, MAX_RAW_REQUEST_BODY_BYTES,
    MAX_TIMEOUT_SECONDS, MAX_UPSTREAM_HTTP1_HEAD_BYTES, MAX_UPSTREAM_HTTP1_HEADERS, RawHttpError,
    RawTransportFailure, RequestTimeouts, TimeoutError, TokenRefreshError, TokenRotationError,
    TransportError, UserAgent, UserAgentError,
};
pub use client::AsyncClient;
pub use config::{AsyncClientBuilder, RawAsyncClientBuilder};
pub use raw::RawAsyncClient;

#[cfg(test)]
mod tests;
