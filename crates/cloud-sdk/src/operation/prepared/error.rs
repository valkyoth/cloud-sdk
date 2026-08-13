//! Redacted prepared-request construction and execution failures.

use core::fmt;

use crate::operation::{ExecutionPermitError, ResponsePolicyError};
use crate::transport::{EndpointIdentityError, ResponseWriterError};

/// Incoherent policy supplied while constructing a prepared request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreparedRequestPolicyError {
    /// Protected or retainable request IDs were not admitted by raw transport.
    MissingRequestIdHeader,
    /// Read-only metadata was paired with a method that can change state.
    ReadOnlyMethodMismatch,
    /// The explicit read-only POST constructor received incompatible policy.
    ReadOnlyPostQueryMismatch,
}

impl fmt::Display for PreparedRequestPolicyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MissingRequestIdHeader => {
                "prepared request ID policy requires raw x-request-id admission"
            }
            Self::ReadOnlyMethodMismatch => "read-only operation metadata requires GET or HEAD",
            Self::ReadOnlyPostQueryMismatch => {
                "read-only POST query requires POST and read-only safe metadata"
            }
        })
    }
}

impl core::error::Error for PreparedRequestPolicyError {}

/// Prepared execution failure with transport details redacted from diagnostics.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PreparedExecutionError<E> {
    /// A state-changing request was executed without plan-confirm authority.
    AuthorizationRequired,
    /// Plan-confirm authority became invalid before transport dispatch.
    AuthorizationInvalid(ExecutionPermitError),
    /// The bound transport returned invalid endpoint identity.
    EndpointIdentity(EndpointIdentityError),
    /// The bound endpoint differs from the prepared provider service.
    EndpointMismatch,
    /// The concrete transport failed.
    Transport(E),
    /// The SDK-owned response transaction failed.
    ResponseWriter(ResponseWriterError),
    /// The response failed provider-neutral policy.
    ResponsePolicy(ResponsePolicyError),
}

impl<E> fmt::Debug for PreparedExecutionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthorizationRequired => formatter.write_str("AuthorizationRequired"),
            Self::AuthorizationInvalid(error) => formatter
                .debug_tuple("AuthorizationInvalid")
                .field(error)
                .finish(),
            Self::EndpointIdentity(error) => formatter
                .debug_tuple("EndpointIdentity")
                .field(error)
                .finish(),
            Self::EndpointMismatch => formatter.write_str("EndpointMismatch"),
            Self::Transport(_) => formatter.write_str("Transport([redacted])"),
            Self::ResponseWriter(error) => formatter
                .debug_tuple("ResponseWriter")
                .field(error)
                .finish(),
            Self::ResponsePolicy(error) => formatter
                .debug_tuple("ResponsePolicy")
                .field(error)
                .finish(),
        }
    }
}

impl<E> fmt::Display for PreparedExecutionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::AuthorizationRequired => "state-changing request requires execution authority",
            Self::AuthorizationInvalid(_) => "execution authority is no longer valid",
            Self::EndpointIdentity(_) => "transport endpoint identity is invalid",
            Self::EndpointMismatch => "transport endpoint differs from prepared service",
            Self::Transport(_) => "prepared request transport failed",
            Self::ResponseWriter(_) => "prepared response transaction failed",
            Self::ResponsePolicy(_) => "prepared response policy failed",
        })
    }
}

impl<E> core::error::Error for PreparedExecutionError<E> {}

impl<E: crate::transport::DeliveryClassified> crate::transport::DeliveryClassified
    for PreparedExecutionError<E>
{
    fn delivery_phase(&self) -> crate::transport::DeliveryPhase {
        use crate::transport::DeliveryPhase;
        match self {
            Self::AuthorizationRequired
            | Self::AuthorizationInvalid(_)
            | Self::EndpointIdentity(_)
            | Self::EndpointMismatch => DeliveryPhase::NotSent,
            Self::Transport(error) => error.delivery_phase(),
            Self::ResponseWriter(_) => DeliveryPhase::PossiblySent,
            Self::ResponsePolicy(_) => DeliveryPhase::ResponseStarted,
        }
    }
}

pub(super) enum EndpointCheckError {
    Invalid(EndpointIdentityError),
    Mismatch,
}

pub(super) fn map_endpoint_error<E>(error: EndpointCheckError) -> PreparedExecutionError<E> {
    match error {
        EndpointCheckError::Invalid(error) => PreparedExecutionError::EndpointIdentity(error),
        EndpointCheckError::Mismatch => PreparedExecutionError::EndpointMismatch,
    }
}
