#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

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

pub mod action_polling;
pub mod authentication;
pub mod buffer;
pub mod client;
pub mod diagnostics;
mod identity;
mod method;
pub mod operation;
pub mod pagination;
pub mod rate_limit;
pub mod retry;
pub mod transport;

pub use identity::{
    IdentityError, MAX_PROVIDER_ID_BYTES, MAX_SERVICE_ID_BYTES, ProviderId, ProviderMarker,
    ServiceId, ServiceMarker,
};
pub use method::{MAX_METHOD_BYTES, Method, MethodError};

#[cfg(test)]
mod tests {
    use super::{Method, MethodError, ProviderId, ServiceId};
    use crate::action_polling::{ActionObserveError, ActionPollError};
    use crate::authentication::{
        AuthenticationScopeError, CredentialGenerationError, CredentialLifetimeError,
        ScopeValueError, SigningBuildError, SigningContextValueError, SigningInputError,
        SigningOutputError, SigningValueError,
    };
    use crate::client::{
        CheckedDecodeError, ClientExecutionError, WorkspaceAcquireError, WorkspacePoolError,
    };
    use crate::operation::{
        AttemptBudgetError, CurrencyCodeError, ExecutionPermitError, OperationMetadataError,
        PermitContextError, PermitExecutionError, PermitIdempotencyKeyError, PermitValidityError,
        PlanCostError, PlanFingerprintBuildError, PreparedExecutionError, ResponsePolicyError,
        ResponsePolicyValidationError,
    };
    use crate::pagination::PaginationError;
    use crate::rate_limit::{
        DelayDecisionError, QuotaError, QuotaExtensionError, RateLimitError, RetryAfterError,
    };
    use crate::retry::{
        FingerprintBuildError, IdempotencyIntentError, MaxAttemptsError, RetryExecutionError,
        RetryPermitError, RetryPolicyError,
    };
    use crate::transport::{
        AsyncExecutionError, ContentTypeError, EndpointIdentityError, EndpointPairPolicyError,
        HeaderError, InformationalResponseError, RawResponsePolicyError, RequestPathError,
        RequestTargetError, ResponseWriterError, StreamExecutionError, StreamLimitsError,
        StreamPolicyError, StreamProgressError, StreamReplayError, StreamSourceIdError,
        TransportFailure,
    };
    use core::fmt::{self, Write};

    #[test]
    fn exposes_provider_neutral_domains() {
        assert_eq!(
            ProviderId::new("example").map(ProviderId::as_str),
            Ok("example")
        );
        assert_eq!(
            ServiceId::new("compute").map(ServiceId::as_str),
            Ok("compute")
        );
        assert_eq!(Method::Get, Method::Get);
        assert_eq!(Method::Post.as_str(), "POST");
    }

    #[test]
    fn public_errors_implement_payload_free_core_error() {
        fn assert_error<E: core::error::Error>() {}

        assert_error::<PaginationError>();
        assert_error::<AuthenticationScopeError>();
        assert_error::<CredentialGenerationError>();
        assert_error::<CredentialLifetimeError>();
        assert_error::<ScopeValueError>();
        assert_error::<SigningValueError>();
        assert_error::<SigningContextValueError>();
        assert_error::<SigningInputError>();
        assert_error::<SigningBuildError<core::convert::Infallible>>();
        assert_error::<SigningOutputError<core::convert::Infallible>>();
        assert_error::<RateLimitError>();
        assert_error::<QuotaError>();
        assert_error::<QuotaExtensionError>();
        assert_error::<RetryAfterError>();
        assert_error::<DelayDecisionError>();
        assert_error::<FingerprintBuildError<core::convert::Infallible>>();
        assert_error::<IdempotencyIntentError>();
        assert_error::<MaxAttemptsError>();
        assert_error::<RetryPolicyError>();
        assert_error::<RetryPermitError>();
        assert_error::<RetryExecutionError<()>>();
        assert_error::<ContentTypeError>();
        assert_error::<HeaderError>();
        assert_error::<EndpointIdentityError>();
        assert_error::<EndpointPairPolicyError>();
        assert_error::<RequestTargetError>();
        assert_error::<MethodError>();
        assert_error::<ActionPollError>();
        assert_error::<ActionObserveError<&'static str>>();
        assert_error::<OperationMetadataError>();
        assert_error::<ResponsePolicyError>();
        assert_error::<ResponsePolicyValidationError>();
        assert_error::<PreparedExecutionError<()>>();
        assert_error::<AttemptBudgetError>();
        assert_error::<CurrencyCodeError>();
        assert_error::<ExecutionPermitError>();
        assert_error::<PermitContextError>();
        assert_error::<PermitExecutionError<()>>();
        assert_error::<PermitIdempotencyKeyError>();
        assert_error::<PermitValidityError>();
        assert_error::<PlanCostError>();
        assert_error::<PlanFingerprintBuildError<core::convert::Infallible>>();
        assert_error::<ResponseWriterError>();
        assert_error::<AsyncExecutionError<()>>();
        assert_error::<RawResponsePolicyError>();
        assert_error::<InformationalResponseError>();
        assert_error::<StreamLimitsError>();
        assert_error::<StreamPolicyError>();
        assert_error::<StreamProgressError>();
        assert_error::<StreamSourceIdError>();
        assert_error::<StreamReplayError>();
        assert_error::<StreamExecutionError<(), ()>>();
        assert_error::<TransportFailure<()>>();
        assert_error::<CheckedDecodeError<()>>();
        assert_error::<ClientExecutionError<(), (), ()>>();
        assert_error::<WorkspaceAcquireError>();
        assert_error::<WorkspacePoolError>();

        assert_display(PaginationError::PageZero, "page number must be nonzero");
        assert_display(RateLimitError::LimitZero, "rate limit must be nonzero");
        assert_display(ContentTypeError::Empty, "content type is empty");
        assert_display(HeaderError::DuplicateName, "HTTP header name is duplicated");
        assert_display(
            EndpointIdentityError::UnboundTransport,
            "transport endpoint identity is unbound",
        );
        assert_display(
            RequestTargetError::Path(RequestPathError::Empty),
            "invalid request path: request path is empty",
        );
        assert_display(
            MethodError::DeniedMethod,
            "HTTP method is denied by the transport contract",
        );
        assert_display(
            OperationMetadataError::NonIdempotentRetry,
            "non-idempotent operation cannot be retry eligible",
        );
        assert_display(
            ResponsePolicyError::UnexpectedContentType,
            "response content type is not accepted",
        );
        assert_display(
            ResponsePolicyError::InvalidContentType,
            "response content type is invalid",
        );
        assert_display(
            ResponsePolicyValidationError::MissingSuccessStatus,
            "response policy has no success status",
        );
        assert_display(
            PreparedExecutionError::<()>::Transport(()),
            "prepared request transport failed",
        );
        assert_display(
            PreparedExecutionError::<()>::AuthorizationRequired,
            "state-changing request requires execution authority",
        );
        assert_display(
            ResponseWriterError::AlreadyCommitted,
            "response writer is already committed",
        );
        assert_display(
            RawResponsePolicyError::UnsafeAdmittedHeader,
            "an unsafe response header was admitted",
        );
        assert_display(
            InformationalResponseError::SwitchingProtocols,
            "switching protocols is forbidden",
        );
        assert_display(
            TransportFailure::unknown("sentinel-secret"),
            "transport failed with uncertain delivery",
        );
        assert_display(
            ActionObserveError::Backoff("sentinel-secret"),
            "action poll backoff policy failed",
        );
    }

    fn assert_display(error: impl fmt::Display, expected: &str) {
        let mut output = DisplayBuffer::new();
        assert!(write!(&mut output, "{error}").is_ok());
        assert_eq!(output.as_str(), expected);
    }

    struct DisplayBuffer {
        bytes: [u8; 128],
        len: usize,
    }

    impl DisplayBuffer {
        const fn new() -> Self {
            Self {
                bytes: [0; 128],
                len: 0,
            }
        }

        fn as_str(&self) -> &str {
            core::str::from_utf8(self.bytes.get(..self.len).unwrap_or_default()).unwrap_or_default()
        }
    }

    impl Write for DisplayBuffer {
        fn write_str(&mut self, value: &str) -> fmt::Result {
            let end = self.len.checked_add(value.len()).ok_or(fmt::Error)?;
            let target = self.bytes.get_mut(self.len..end).ok_or(fmt::Error)?;
            target.copy_from_slice(value.as_bytes());
            self.len = end;
            Ok(())
        }
    }
}
