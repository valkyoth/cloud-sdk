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
mod config;
mod content_type;
mod credentials;
mod endpoint;
mod error;
mod headers;
mod rate_limit;
mod raw;
mod raw_hyper;

pub use auth::{BearerToken, BearerTokenError, MAX_BEARER_TOKEN_BYTES};
pub use cloud_sdk::transport::CustomEndpointAcknowledgement;
pub use config::{MAX_TIMEOUT_SECONDS, RequestTimeouts, TimeoutError, UserAgent, UserAgentError};
pub(crate) use content_type::parse_response_content_type;
pub(crate) use credentials::CredentialStore;
pub use credentials::{CredentialStateError, TokenRotationError};
pub use endpoint::{EndpointError, HttpsEndpoint, MAX_CONFIGURED_ENDPOINT_BYTES};
pub use error::{BuildError, TransportError};
pub(crate) use headers::capture_response_headers;
pub(crate) use rate_limit::parse_rate_limit;
pub(crate) use raw::inspect_response_head;
pub use raw::{
    MAX_UPSTREAM_HTTP1_HEAD_BYTES, MAX_UPSTREAM_HTTP1_HEADERS, RawHttpError, RawTransportFailure,
};
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

#[cfg(test)]
mod error_tests {
    use super::{
        BearerTokenError, BuildError, CredentialStateError, EndpointError, RawHttpError,
        RawTransportFailure, TimeoutError, TokenRotationError, TransportError, UserAgentError,
    };
    use std::string::ToString;

    #[test]
    fn public_errors_implement_payload_free_core_error() {
        fn assert_error<E: core::error::Error>() {}

        assert_error::<BearerTokenError>();
        assert_error::<BuildError>();
        assert_error::<CredentialStateError>();
        assert_error::<EndpointError>();
        assert_error::<TimeoutError>();
        assert_error::<TokenRotationError>();
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
