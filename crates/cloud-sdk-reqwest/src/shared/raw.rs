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

fn validate_wire_head(headers: &HeaderMap) -> Result<(), RawHttpError> {
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

fn declared_content_length(headers: &HeaderMap) -> Result<Option<u64>, RawHttpError> {
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
mod tests {
    use std::format;

    use cloud_sdk::transport::{
        HeaderName, MediaType, RawResponsePolicy, ResponseHeaders, ResponseMediaPolicy, StatusCode,
    };
    use reqwest::header::{HeaderMap, HeaderValue};

    use super::{RawHttpError, inspect_response_head};

    fn policy<'a>(headers: &'a [HeaderName<'a>]) -> Option<RawResponsePolicy<'a>> {
        RawResponsePolicy::new(
            8,
            4,
            ResponseMediaPolicy::Required(&[MediaType::JSON]),
            ResponseMediaPolicy::Optional(&[MediaType::JSON]),
            headers,
            2,
        )
        .ok()
    }

    #[test]
    fn selects_status_limit_and_drops_unadmitted_headers() {
        let admitted = HeaderName::new("content-type");
        assert!(admitted.is_ok());
        let Ok(admitted) = admitted else { return };
        let admitted_headers = [admitted];
        let Some(policy) = policy(&admitted_headers) else {
            return;
        };
        let mut source = HeaderMap::new();
        source.insert("content-type", HeaderValue::from_static("application/json"));
        source.insert("set-cookie", HeaderValue::from_static("secret=1"));
        source.insert("x-unknown", HeaderValue::from_static("secret"));
        let mut storage = [0_u8; 128];
        let mut captured = ResponseHeaders::new(&mut storage);
        let result = inspect_response_head(
            cloud_sdk::Method::Get,
            StatusCode::OK,
            &source,
            policy,
            &mut captured,
            16,
        );
        assert_eq!(result, Ok(8));
        assert!(captured.get("content-type").is_some());
        assert!(captured.get("set-cookie").is_none());
        assert!(captured.get("x-unknown").is_none());
    }

    #[test]
    fn rejects_duplicates_and_no_content_framing() {
        let Some(policy) = policy(&[]) else { return };
        let mut duplicate = HeaderMap::new();
        duplicate.append("x-test", HeaderValue::from_static("one"));
        duplicate.append("x-test", HeaderValue::from_static("two"));
        let mut storage = [0_u8; 128];
        let mut captured = ResponseHeaders::new(&mut storage);
        assert_eq!(
            inspect_response_head(
                cloud_sdk::Method::Get,
                StatusCode::new(400).unwrap_or(StatusCode::TOO_MANY_REQUESTS),
                &duplicate,
                policy,
                &mut captured,
                16,
            ),
            Err(RawHttpError::DuplicateResponseHeader)
        );

        let mut no_content = HeaderMap::new();
        no_content.insert("content-length", HeaderValue::from_static("0"));
        assert_eq!(
            inspect_response_head(
                cloud_sdk::Method::Get,
                StatusCode::NO_CONTENT,
                &no_content,
                policy,
                &mut captured,
                16,
            ),
            Err(RawHttpError::InvalidNoBodyFraming)
        );
    }

    #[test]
    fn rejects_media_mismatch_oversized_length_and_hostile_header_count() {
        let Some(policy) = policy(&[]) else { return };
        let mut storage = [0_u8; 128];
        let mut captured = ResponseHeaders::new(&mut storage);

        let mut wrong_media = HeaderMap::new();
        wrong_media.insert("content-type", HeaderValue::from_static("text/plain"));
        assert_eq!(
            inspect_response_head(
                cloud_sdk::Method::Get,
                StatusCode::OK,
                &wrong_media,
                policy,
                &mut captured,
                16,
            ),
            Err(RawHttpError::UnexpectedResponseContentType)
        );

        let mut oversized = HeaderMap::new();
        oversized.insert("content-type", HeaderValue::from_static("application/json"));
        oversized.insert("content-length", HeaderValue::from_static("9"));
        assert_eq!(
            inspect_response_head(
                cloud_sdk::Method::Get,
                StatusCode::OK,
                &oversized,
                policy,
                &mut captured,
                16,
            ),
            Err(RawHttpError::ResponseTooLarge)
        );

        let mut hostile = HeaderMap::new();
        for index in 0..=super::MAX_UPSTREAM_HTTP1_HEADERS {
            let name = format!("x-field-{index}");
            let Ok(name) = reqwest::header::HeaderName::from_bytes(name.as_bytes()) else {
                return;
            };
            hostile.insert(name, HeaderValue::from_static("value"));
        }
        assert_eq!(
            inspect_response_head(
                cloud_sdk::Method::Get,
                StatusCode::OK,
                &hostile,
                policy,
                &mut captured,
                16,
            ),
            Err(RawHttpError::ResponseHeadTooLarge)
        );
    }

    #[test]
    fn head_and_not_modified_select_zero_body_capacity() {
        let Some(policy) = policy(&[]) else { return };
        let mut source = HeaderMap::new();
        source.insert("content-type", HeaderValue::from_static("application/json"));
        source.insert("content-length", HeaderValue::from_static("8"));
        let mut storage = [0_u8; 128];
        let mut captured = ResponseHeaders::new(&mut storage);
        assert_eq!(
            inspect_response_head(
                cloud_sdk::Method::Head,
                StatusCode::OK,
                &source,
                policy,
                &mut captured,
                16,
            ),
            Ok(0)
        );
        let not_modified = StatusCode::new(304).unwrap_or(StatusCode::NO_CONTENT);
        assert_eq!(
            inspect_response_head(
                cloud_sdk::Method::Get,
                not_modified,
                &source,
                policy,
                &mut captured,
                16,
            ),
            Ok(0)
        );
    }
}
