use cloud_sdk::Method;
use cloud_sdk::transport::{
    ContentType, HeaderSensitivity, RawResponsePolicy, ResponseHeaders, ResponseMediaPolicy,
    StatusCode, TrailerPolicy, TransportFailure,
};
use core::ops::Range;
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE, HeaderMap, TRAILER};

/// Maximum response-header fields parsed by pinned Hyper HTTP/1.
pub const MAX_UPSTREAM_HTTP1_HEADERS: usize = 100;
/// Maximum pinned Hyper HTTP/1 read buffer before a head-too-large failure.
pub const MAX_UPSTREAM_HTTP1_HEAD_BYTES: usize = 64 * 1024;
/// Maximum request body copied into raw adapter-owned staging.
pub const MAX_RAW_REQUEST_BODY_BYTES: usize = cloud_sdk::operation::LARGE_BODY_BYTES;

/// Payload-free raw HTTP execution failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RawHttpError {
    /// The supplied response writer was already committed.
    ResponseAlreadyCommitted,
    /// Endpoint and request-target composition failed.
    TargetRejected,
    /// The validated method could not be represented by reqwest.
    MethodRejected,
    /// A non-empty request body omitted `Content-Type`.
    MissingContentType,
    /// A request header could not be represented by reqwest.
    HeaderRejected,
    /// Adapter-owned request-header staging allocation failed.
    RequestHeaderAllocationFailed,
    /// Adapter-owned request-body staging allocation failed.
    RequestBodyAllocationFailed,
    /// Request-body length cannot be represented by the HTTP implementation.
    RequestBodyTooLarge,
    /// Construction of the exact raw request failed.
    RequestBuildFailed,
    /// Caller streaming output is empty or the response source is no longer usable.
    InvalidStreamState,
    /// The source, progress policy or streaming request-body transfer failed.
    UploadFailed,
    /// A final response arrived before source framing was verified complete.
    UploadIncomplete,
    /// The blocking adapter could not construct its private executor.
    RuntimeInitializationFailed,
    /// The blocking adapter was called from an active Tokio runtime.
    BlockingRuntimeContext,
    /// Connection establishment failed.
    ConnectFailed,
    /// A configured request or read deadline expired.
    TimedOut,
    /// Sending failed without a more precise payload-free classification.
    RequestFailed,
    /// The final response origin differed from the configured endpoint.
    ResponseOriginChanged,
    /// The final status was outside the core HTTP status domain.
    InvalidStatus,
    /// `101 Switching Protocols` is forbidden.
    SwitchingProtocols,
    /// More informational responses were observed than admitted.
    TooManyInformationalResponses,
    /// The response head exceeded pinned upstream count or byte bounds.
    ResponseHeadTooLarge,
    /// A response header name occurred more than once.
    DuplicateResponseHeader,
    /// A response declared trailers while the raw policy rejects them.
    ResponseTrailersRejected,
    /// A no-body response used forbidden framing.
    InvalidNoBodyFraming,
    /// A response content type was required but absent.
    MissingResponseContentType,
    /// A response content type was malformed.
    InvalidResponseContentType,
    /// A response content type did not match the selected status policy.
    UnexpectedResponseContentType,
    /// A response content type was present when forbidden.
    ForbiddenResponseContentType,
    /// A retained response header violated core bounds.
    InvalidResponseHeader,
    /// The declared or observed response body exceeded its status-class limit.
    ResponseTooLarge,
    /// A response exceeded the fixed chunk-observation budget.
    ResponseChunkLimitExceeded,
    /// Reading the response body failed.
    ResponseReadFailed,
    /// The core response writer rejected final commitment.
    ResponseCommitFailed,
}

impl_static_error!(RawHttpError,
    Self::ResponseAlreadyCommitted => "response writer is already committed",
    Self::TargetRejected => "request target was rejected",
    Self::MethodRejected => "request method was rejected",
    Self::MissingContentType => "request body content type is missing",
    Self::HeaderRejected => "request header was rejected",
    Self::RequestHeaderAllocationFailed => "request-header allocation failed",
    Self::RequestBodyAllocationFailed => "request-body allocation failed",
    Self::RequestBodyTooLarge => "request body is too large",
    Self::RequestBuildFailed => "raw request construction failed",
    Self::InvalidStreamState => "stream source or output state is invalid",
    Self::UploadFailed => "streaming upload failed",
    Self::UploadIncomplete => "response arrived before upload completion",
    Self::RuntimeInitializationFailed => "blocking executor initialization failed",
    Self::BlockingRuntimeContext => "blocking executor called from an async runtime",
    Self::ConnectFailed => "connection failed",
    Self::TimedOut => "request timed out",
    Self::RequestFailed => "request failed",
    Self::ResponseOriginChanged => "response origin changed",
    Self::InvalidStatus => "response status is invalid",
    Self::SwitchingProtocols => "switching protocols is forbidden",
    Self::TooManyInformationalResponses => "too many informational responses",
    Self::ResponseHeadTooLarge => "response head exceeds wire limits",
    Self::DuplicateResponseHeader => "response header is duplicated",
    Self::ResponseTrailersRejected => "response trailers are rejected",
    Self::InvalidNoBodyFraming => "no-body response framing is invalid",
    Self::MissingResponseContentType => "response content type is missing",
    Self::InvalidResponseContentType => "response content type is invalid",
    Self::UnexpectedResponseContentType => "response content type is not admitted",
    Self::ForbiddenResponseContentType => "response content type is forbidden",
    Self::InvalidResponseHeader => "retained response header is invalid",
    Self::ResponseTooLarge => "response body exceeds its status-class limit",
    Self::ResponseChunkLimitExceeded => "response chunk limit is exceeded",
    Self::ResponseReadFailed => "response body read failed",
    Self::ResponseCommitFailed => "response commitment failed",
);

/// Delivery-phased raw reqwest failure.
pub type RawTransportFailure = TransportFailure<RawHttpError>;

/// Delivery-phased authenticated reqwest failure.
pub type AuthenticatedTransportFailure = TransportFailure<super::TransportError>;

pub(crate) struct ResponseBodyBudget {
    limit: usize,
    len: usize,
    chunks: usize,
}

impl ResponseBodyBudget {
    pub(crate) const fn new(limit: usize) -> Self {
        Self {
            limit,
            len: 0,
            chunks: 0,
        }
    }

    pub(crate) fn observe(&mut self, bytes: usize) -> Result<Range<usize>, RawHttpError> {
        self.chunks = self
            .chunks
            .checked_add(1)
            .ok_or(RawHttpError::ResponseChunkLimitExceeded)?;
        if self.chunks > cloud_sdk::transport::MAX_RESPONSE_CHUNKS {
            return Err(RawHttpError::ResponseChunkLimitExceeded);
        }
        let end = self
            .len
            .checked_add(bytes)
            .ok_or(RawHttpError::ResponseTooLarge)?;
        if end > self.limit {
            return Err(RawHttpError::ResponseTooLarge);
        }
        let range = self.len..end;
        self.len = end;
        Ok(range)
    }

    pub(crate) const fn len(&self) -> usize {
        self.len
    }
}

pub(crate) fn inspect_response_head(
    method: Method,
    status: StatusCode,
    source: &HeaderMap,
    policy: RawResponsePolicy<'_>,
    captured: &mut ResponseHeaders<'_>,
    writer_capacity: usize,
) -> Result<usize, RawHttpError> {
    validate_wire_head(source)?;
    if status.get() == 101 {
        return Err(RawHttpError::SwitchingProtocols);
    }
    if status.get() < 200 {
        return Err(RawHttpError::InvalidStatus);
    }
    if matches!(policy.trailer_policy(), TrailerPolicy::Reject) && source.contains_key(TRAILER) {
        return Err(RawHttpError::ResponseTrailersRejected);
    }

    let policy_limit = policy.body_limit(status);
    let body_forbidden = method == Method::Head || matches!(status.get(), 204 | 304);
    if status.get() == 204 && source.contains_key(CONTENT_LENGTH) {
        return Err(RawHttpError::InvalidNoBodyFraming);
    }
    validate_media(source, policy.media_policy(status))?;
    let selected_limit = if body_forbidden {
        0
    } else {
        core::cmp::min(policy_limit, writer_capacity)
    };
    if let Some(declared) = declared_content_length(source)? {
        let declared = usize::try_from(declared).map_err(|_| RawHttpError::ResponseTooLarge)?;
        if !body_forbidden && declared > selected_limit {
            return Err(RawHttpError::ResponseTooLarge);
        }
    }

    for name in source.keys() {
        if !policy.admits_header(name.as_str()) {
            continue;
        }
        let Some(value) = source.get(name) else {
            return Err(RawHttpError::InvalidResponseHeader);
        };
        let sensitivity = if is_reviewed_public(name.as_str()) {
            HeaderSensitivity::Public
        } else {
            HeaderSensitivity::Sensitive
        };
        captured
            .try_push(name.as_str(), value.as_bytes(), sensitivity)
            .map_err(|_| RawHttpError::InvalidResponseHeader)?;
    }
    Ok(selected_limit)
}

pub(super) fn validate_wire_head(headers: &HeaderMap) -> Result<(), RawHttpError> {
    if headers.len() > MAX_UPSTREAM_HTTP1_HEADERS {
        return Err(RawHttpError::ResponseHeadTooLarge);
    }
    let mut encoded_len = 0_usize;
    for name in headers.keys() {
        let values = headers.get_all(name);
        if values.iter().count() != 1 {
            return Err(RawHttpError::DuplicateResponseHeader);
        }
        let Some(value) = values.iter().next() else {
            return Err(RawHttpError::InvalidResponseHeader);
        };
        encoded_len = encoded_len
            .checked_add(name.as_str().len())
            .and_then(|length| length.checked_add(value.as_bytes().len()))
            .and_then(|length| length.checked_add(4))
            .ok_or(RawHttpError::ResponseHeadTooLarge)?;
        if encoded_len > MAX_UPSTREAM_HTTP1_HEAD_BYTES {
            return Err(RawHttpError::ResponseHeadTooLarge);
        }
    }
    Ok(())
}

pub(super) fn declared_content_length(headers: &HeaderMap) -> Result<Option<u64>, RawHttpError> {
    let Some(value) = headers.get(CONTENT_LENGTH) else {
        return Ok(None);
    };
    let text = value
        .to_str()
        .map_err(|_| RawHttpError::InvalidNoBodyFraming)?;
    text.parse::<u64>()
        .map(Some)
        .map_err(|_| RawHttpError::InvalidNoBodyFraming)
}

fn validate_media(
    headers: &HeaderMap,
    policy: ResponseMediaPolicy<'_>,
) -> Result<(), RawHttpError> {
    let content_type = headers.get(CONTENT_TYPE);
    match (policy, content_type) {
        (ResponseMediaPolicy::Required(_), None) => Err(RawHttpError::MissingResponseContentType),
        (ResponseMediaPolicy::Optional(_), None) | (ResponseMediaPolicy::Forbidden, None) => Ok(()),
        (ResponseMediaPolicy::Forbidden, Some(_)) => {
            Err(RawHttpError::ForbiddenResponseContentType)
        }
        (
            ResponseMediaPolicy::Required(admitted) | ResponseMediaPolicy::Optional(admitted),
            Some(value),
        ) => {
            let text = value
                .to_str()
                .map_err(|_| RawHttpError::InvalidResponseContentType)?;
            let parsed =
                ContentType::new(text).map_err(|_| RawHttpError::InvalidResponseContentType)?;
            if admitted.iter().any(|media| parsed.matches(*media)) {
                Ok(())
            } else {
                Err(RawHttpError::UnexpectedResponseContentType)
            }
        }
    }
}

fn is_reviewed_public(name: &str) -> bool {
    ["content-length", "content-type", "date"]
        .iter()
        .any(|candidate| name.eq_ignore_ascii_case(candidate))
}

#[cfg(test)]
mod tests;
