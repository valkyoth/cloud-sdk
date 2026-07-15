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
mod endpoint;
mod error;
mod rate_limit;

pub use auth::{BearerToken, BearerTokenError, MAX_BEARER_TOKEN_BYTES};
pub use config::{MAX_TIMEOUT_SECONDS, RequestTimeouts, TimeoutError, UserAgent, UserAgentError};
pub use endpoint::{EndpointError, HttpsEndpoint};
pub use error::{BuildError, TransportError};
pub(crate) use rate_limit::parse_rate_limit;

#[cfg(test)]
mod error_tests {
    use super::{
        BearerTokenError, BuildError, EndpointError, TimeoutError, TransportError, UserAgentError,
    };

    #[test]
    fn public_errors_implement_payload_free_core_error() {
        fn assert_error<E: core::error::Error>() {}

        assert_error::<BearerTokenError>();
        assert_error::<BuildError>();
        assert_error::<EndpointError>();
        assert_error::<TimeoutError>();
        assert_error::<TransportError>();
        assert_error::<UserAgentError>();
    }
}
