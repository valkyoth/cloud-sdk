//! Hardened provider-neutral asynchronous transport implementation.

mod basic_client;
mod basic_config;
mod client;
mod config;
mod raw;

pub use crate::shared::{
    AuthenticatedTransportFailure, BasicCredential, BasicCredentialError, BasicCredentialScope,
    BasicCredentialScopeError, BasicPassword, BasicPasswordError, BasicUsername,
    BasicUsernameError, BearerCredential, BearerCredentialScope, BearerCredentialScopeError,
    BearerCredentialSnapshot, BearerRefreshHandoff, BearerToken, BearerTokenError, BuildError,
    CredentialStateError, CredentialUpdateError, CustomEndpointAcknowledgement, EndpointError,
    HttpsEndpoint, LinkLocalHttpEndpoint, MAX_BASIC_AUTHORIZATION_BYTES, MAX_BASIC_PASSWORD_BYTES,
    MAX_BASIC_USERNAME_BYTES, MAX_BEARER_TOKEN_BYTES, MAX_CONFIGURED_ENDPOINT_BYTES,
    MAX_RAW_REQUEST_BODY_BYTES, MAX_TIMEOUT_SECONDS, MAX_UPSTREAM_HTTP1_HEAD_BYTES,
    MAX_UPSTREAM_HTTP1_HEADERS, RawHttpError, RawTransportFailure, RefreshHandoffError,
    RequestTimeouts, TimeoutError, TokenRefreshError, TokenRotationError, TransportError,
    UserAgent, UserAgentError,
};
pub use basic_client::AsyncBasicClient;
pub use basic_config::AsyncBasicClientBuilder;
pub use client::AsyncClient;
pub use config::{AsyncClientBuilder, RawAsyncClientBuilder};
pub use raw::RawAsyncClient;

#[cfg(test)]
mod tests;
