macro_rules! impl_static_error {
    ($error:ty, $($pattern:pat => $message:literal),+ $(,)?) => {
        impl core::fmt::Display for $error {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str(match self {
                    $($pattern => $message,)+
                })
            }
        }

        impl core::error::Error for $error {}
    };
}

mod auth;
mod authentication;
mod basic;
mod config;
#[cfg(test)]
mod content_type;
mod credentials;
mod endpoint;
mod error;
#[cfg(test)]
mod headers;
#[cfg(test)]
mod rate_limit;
mod raw;
#[cfg(feature = "fuzzing")]
mod raw_fuzz;
mod raw_hyper;
#[cfg(feature = "fuzzing")]
mod raw_wire_fuzz;
mod scope;
mod secret_header;

pub use auth::{BearerToken, BearerTokenError, MAX_BEARER_TOKEN_BYTES};
pub(crate) use authentication::{
    map_authentication_error, validate_basic_authentication, validate_bearer_authentication,
};
pub use basic::{
    BasicCredential, BasicCredentialError, BasicPassword, BasicPasswordError, BasicUsername,
    BasicUsernameError, MAX_BASIC_AUTHORIZATION_BYTES, MAX_BASIC_PASSWORD_BYTES,
    MAX_BASIC_USERNAME_BYTES,
};
pub use cloud_sdk::transport::CustomEndpointAcknowledgement;
pub use config::{MAX_TIMEOUT_SECONDS, RequestTimeouts, TimeoutError, UserAgent, UserAgentError};
pub(crate) use credentials::CredentialStore;
pub use credentials::{
    BearerCredentialSnapshot, BearerRefreshHandoff, CredentialStateError, CredentialUpdateError,
    TokenRefreshError, TokenRotationError,
};
pub use endpoint::{EndpointError, HttpsEndpoint, MAX_CONFIGURED_ENDPOINT_BYTES};
pub use error::{BuildError, TransportError};
pub(crate) use raw::inspect_response_head;
pub use raw::{
    AuthenticatedTransportFailure, MAX_RAW_REQUEST_BODY_BYTES, MAX_UPSTREAM_HTTP1_HEAD_BYTES,
    MAX_UPSTREAM_HTTP1_HEADERS, RawHttpError, RawTransportFailure,
};
#[cfg(feature = "fuzzing")]
pub use raw_fuzz::fuzz_raw_response_parser;
pub(crate) use raw_hyper::RawHyperClient;
#[cfg(any(
    feature = "async-rustls",
    all(
        feature = "blocking-rustls",
        not(feature = "blocking-rustls-fips"),
        not(feature = "blocking-rustls-webpki-roots")
    )
))]
pub(crate) use raw_hyper::platform_client_config;
#[cfg(feature = "fuzzing")]
pub use raw_wire_fuzz::fuzz_raw_http1_wire;
pub(crate) use scope::CredentialScopeView;
pub use scope::{
    BasicCredentialScope, BasicCredentialScopeError, BearerCredential, BearerCredentialScope,
    BearerCredentialScopeError,
};
pub(crate) use secret_header::sensitive_header_value;
#[cfg(test)]
pub(crate) use secret_header::sensitive_header_value_with_probe;

#[cfg(test)]
mod error_tests {
    use super::{
        BasicCredentialError, BasicCredentialScopeError, BasicPasswordError, BasicUsernameError,
        BearerCredentialScopeError, BearerTokenError, BuildError, CredentialStateError,
        CredentialUpdateError, EndpointError, RawHttpError, RawTransportFailure, TimeoutError,
        TokenRefreshError, TokenRotationError, TransportError, UserAgentError,
    };
    use std::string::ToString;

    #[test]
    fn public_errors_implement_payload_free_core_error() {
        fn assert_error<E: core::error::Error>() {}

        assert_error::<BearerTokenError>();
        assert_error::<BasicUsernameError>();
        assert_error::<BasicPasswordError>();
        assert_error::<BasicCredentialError>();
        assert_error::<BasicCredentialScopeError>();
        assert_error::<BearerCredentialScopeError>();
        assert_error::<BuildError>();
        assert_error::<CredentialStateError>();
        assert_error::<CredentialUpdateError>();
        assert_error::<EndpointError>();
        assert_error::<TimeoutError>();
        assert_error::<TokenRotationError>();
        assert_error::<TokenRefreshError>();
        assert_error::<TransportError>();
        assert_error::<UserAgentError>();
        assert_error::<RawHttpError>();
        assert_error::<RawTransportFailure>();

        assert_eq!(BearerTokenError::Empty.to_string(), "bearer token is empty");
        assert_eq!(
            BuildError::ClientBuildFailed.to_string(),
            "HTTP client construction failed"
        );
        assert_eq!(
            CredentialStateError::Unavailable.to_string(),
            "credential state is unavailable"
        );
        assert_eq!(
            RawHttpError::ResponseTooLarge.to_string(),
            "response body exceeds its status-class limit"
        );
        assert_eq!(
            RawTransportFailure::unknown(RawHttpError::RequestFailed).to_string(),
            "transport failed with uncertain delivery"
        );
        assert_eq!(
            EndpointError::InvalidUrl.to_string(),
            "endpoint URL is invalid"
        );
        assert_eq!(
            EndpointError::InputTooLong.to_string(),
            "endpoint input exceeds the length limit"
        );
        assert_eq!(TimeoutError::Zero.to_string(), "timeout must be nonzero");
        assert_eq!(TransportError::RequestFailed.to_string(), "request failed");
        assert_eq!(UserAgentError::Empty.to_string(), "user agent is empty");
    }
}
