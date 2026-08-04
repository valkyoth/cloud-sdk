use core::fmt;

use crate::operation::{CheckedResponse, PreparedRequest, ResponsePolicyError};
use crate::transport::{
    ResponseBuffer, ResponseDecodeWorkspace, ResponseWriterError, StatusCode, TransportResponse,
};

/// HTTP response class observed after bounded authenticated transport.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientResponseKind {
    /// A `2xx` response requiring operation success-policy validation.
    Success,
    /// A `4xx` or `5xx` response admitted by the raw error policy.
    Error,
    /// An informational or redirect status that is neither success nor error.
    Other,
}

/// Failure while entering a checked success or provider-error decoder.
pub enum CheckedDecodeError<E> {
    /// The response writer did not contain one committed response.
    ResponseWriter(ResponseWriterError),
    /// Success status, body, media, or metadata policy rejected the response.
    ResponsePolicy(ResponsePolicyError),
    /// Provider-error decoding was requested for a non-error status.
    ExpectedErrorStatus,
    /// The provider-owned decoder rejected the bounded response.
    Decoder(E),
}

impl<E> fmt::Debug for CheckedDecodeError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ResponseWriter(_) => "CheckedDecodeError::ResponseWriter",
            Self::ResponsePolicy(_) => "CheckedDecodeError::ResponsePolicy",
            Self::ExpectedErrorStatus => "CheckedDecodeError::ExpectedErrorStatus",
            Self::Decoder(_) => "CheckedDecodeError::Decoder([redacted])",
        })
    }
}

impl<E> fmt::Display for CheckedDecodeError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ResponseWriter(_) => "client response is not committed",
            Self::ResponsePolicy(_) => "client response policy rejected the response",
            Self::ExpectedErrorStatus => "client error decoder requires an error status",
            Self::Decoder(_) => "provider response decoder failed",
        })
    }
}

impl<E> core::error::Error for CheckedDecodeError<E> {}

/// Bounded response coupled to the prepared policy that admitted it.
///
/// Raw body access is closure-scoped. Success decoding first applies the full
/// operation success policy. Error decoding requires a `4xx`/`5xx` status and
/// applies the operation request-ID policy before invoking provider code.
pub struct ClientResponse<'request, 'buffer> {
    prepared: PreparedRequest<'request>,
    response: ResponseBuffer<'buffer>,
}

impl<'request, 'buffer> ClientResponse<'request, 'buffer> {
    #[allow(clippy::large_types_passed_by_value)]
    pub(crate) const fn new(
        prepared: PreparedRequest<'request>,
        response: ResponseBuffer<'buffer>,
    ) -> Self {
        Self { prepared, response }
    }

    /// Returns the committed status without exposing response bytes.
    pub fn status(&self) -> Result<StatusCode, ResponseWriterError> {
        self.response.with_response(|response| response.status())
    }

    /// Classifies the committed status without applying a decoder.
    pub fn kind(&self) -> Result<ClientResponseKind, ResponseWriterError> {
        self.status().map(|status| {
            if status.is_success() {
                ClientResponseKind::Success
            } else if status.is_error() {
                ClientResponseKind::Error
            } else {
                ClientResponseKind::Other
            }
        })
    }

    /// Applies success policy, decodes an owned value, and clears all storage.
    pub fn decode_success_owned<R, E>(
        self,
        decode: impl for<'response> FnOnce(
            CheckedResponse<'response>,
            &mut ResponseDecodeWorkspace,
        ) -> Result<R, E>,
    ) -> Result<R, CheckedDecodeError<E>> {
        let checked = self
            .prepared
            .validate_response(self.response)
            .map_err(CheckedDecodeError::ResponsePolicy)?;
        checked
            .decode_owned_with_workspace(decode)
            .map_err(CheckedDecodeError::Decoder)
    }

    /// Applies error metadata policy, decodes an owned value, and clears all storage.
    pub fn decode_error_owned<R, E>(
        mut self,
        decode: impl for<'response> FnOnce(
            TransportResponse<'response, 'buffer>,
            &mut ResponseDecodeWorkspace,
        ) -> Result<R, E>,
    ) -> Result<R, CheckedDecodeError<E>> {
        let status = self.status().map_err(CheckedDecodeError::ResponseWriter)?;
        if !status.is_error() {
            return Err(CheckedDecodeError::ExpectedErrorStatus);
        }
        self.prepared
            .apply_response_metadata_policy(&mut self.response)
            .map_err(CheckedDecodeError::ResponsePolicy)?;
        let mut workspace = ResponseDecodeWorkspace::new_for_provider();
        self.response
            .with_response(|response| decode(response, &mut workspace))
            .map_err(CheckedDecodeError::ResponseWriter)?
            .map_err(CheckedDecodeError::Decoder)
    }
}

impl fmt::Debug for ClientResponse<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientResponse")
            .field("prepared", &"[bound]")
            .field("response", &"[redacted]")
            .finish()
    }
}
