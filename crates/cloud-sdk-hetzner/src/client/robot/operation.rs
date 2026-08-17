use core::fmt;

use cloud_sdk::authentication::CredentialBinding;
use cloud_sdk::client::{CheckedDecodeError, ClientResponse, ClientResponseKind};
use cloud_sdk::operation::{CheckedResponseGuard, PreparationStorageGuard, PreparedRequest};
use cloud_sdk::transport::{ResponseDecodeWorkspace, ResponseWriterError, TransportResponse};

use crate::robot::{RobotDecodeError, RobotFailure};

pub(crate) mod private {
    pub trait Sealed {}
    pub trait DirectSealed {}
}

/// Typed result from one admitted Robot response.
#[derive(Debug)]
pub enum RobotClientResponse<T> {
    /// The operation-specific success response was checked and decoded.
    Success(T),
    /// A source-locked Robot provider failure was checked and decoded.
    Failure(RobotFailure),
}

/// Failure while classifying or checked-decoding one Robot response.
pub enum RobotResponseDecodeError<E> {
    /// The committed response status could not be inspected.
    ResponseWriter(ResponseWriterError),
    /// A success response failed policy or operation-specific decoding.
    Success(CheckedDecodeError<E>),
    /// A provider error failed source-locked decoding.
    Failure {
        /// Whether the committed status was exactly `401 Unauthorized`.
        authentication_rejected: bool,
        /// The payload-redacting checked decoder failure.
        error: CheckedDecodeError<RobotDecodeError>,
    },
    /// Informational and redirect statuses are never Robot operation results.
    UnsupportedStatus,
}

impl<E> RobotResponseDecodeError<E> {
    pub(super) const fn closes_credential_generation(&self) -> bool {
        matches!(
            self,
            Self::Failure {
                authentication_rejected: true,
                ..
            }
        )
    }
}

impl<E> fmt::Debug for RobotResponseDecodeError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ResponseWriter(_) => "RobotResponseDecodeError::ResponseWriter",
            Self::Success(_) => "RobotResponseDecodeError::Success([redacted])",
            Self::Failure { .. } => "RobotResponseDecodeError::Failure([redacted])",
            Self::UnsupportedStatus => "RobotResponseDecodeError::UnsupportedStatus",
        })
    }
}

impl<E> fmt::Display for RobotResponseDecodeError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Robot response decoding failed")
    }
}

impl<E> core::error::Error for RobotResponseDecodeError<E> {}

/// Sealed request-bound Robot success and failure decoder.
#[allow(private_bounds)]
pub trait RobotClientOperation: private::Sealed {
    /// Owned or request-borrowing checked result.
    type Output<'request>
    where
        Self: 'request;
    /// Operation-specific success decoder failure.
    type SuccessError;

    /// Decodes success while retaining the transport credential provenance.
    fn decode_success<'request>(
        &'request self,
        response: CheckedResponseGuard<'_>,
        credential: CredentialBinding,
    ) -> Result<Self::Output<'request>, Self::SuccessError>;

    /// Decodes only failures source-locked for this operation.
    fn decode_failure(
        &self,
        response: TransportResponse<'_, '_>,
        workspace: &mut ResponseDecodeWorkspace,
    ) -> Result<RobotFailure, RobotDecodeError>;
}

/// Robot operation admitted for direct client execution without a mutation permit.
#[allow(private_bounds)]
pub trait RobotDirectClientOperation: RobotClientOperation + private::DirectSealed {
    /// Failure while preparing this operation in cleanup-owning storage.
    type PreparationError;

    /// Prepares without weakening mandatory guarded operation paths.
    fn prepare_client<'guard>(
        &self,
        storage: &'guard mut PreparationStorageGuard<'_>,
    ) -> Result<PreparedRequest<'guard>, Self::PreparationError>;
}

pub(super) fn decode_response<'request, O>(
    operation: &'request O,
    response: ClientResponse<'_, '_>,
    credential: CredentialBinding,
) -> Result<RobotClientResponse<O::Output<'request>>, RobotResponseDecodeError<O::SuccessError>>
where
    O: RobotClientOperation,
{
    let status = response
        .status()
        .map_err(RobotResponseDecodeError::ResponseWriter)?;
    match response
        .kind()
        .map_err(RobotResponseDecodeError::ResponseWriter)?
    {
        ClientResponseKind::Success => response
            .decode_success_guarded(|checked| operation.decode_success(checked, credential))
            .map(RobotClientResponse::Success)
            .map_err(RobotResponseDecodeError::Success),
        ClientResponseKind::Error => response
            .decode_error_owned(|response, workspace| operation.decode_failure(response, workspace))
            .map(RobotClientResponse::Failure)
            .map_err(|error| RobotResponseDecodeError::Failure {
                authentication_rejected: status.get() == 401,
                error,
            }),
        ClientResponseKind::Other => Err(RobotResponseDecodeError::UnsupportedStatus),
    }
}
