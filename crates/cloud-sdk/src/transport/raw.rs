//! Provider-neutral raw HTTP execution and response-wire policy.

use core::future::Future;

use super::{HeaderName, MediaType, ResponseWriter, StatusCode, TransportRequest};

/// Maximum informational response heads accepted before a final response.
pub const MAX_INFORMATIONAL_RESPONSES: u8 = 8;
/// Maximum chunks admitted by one buffered raw response.
pub const MAX_RESPONSE_CHUNKS: usize = 4_096;
/// Maximum per-class body limit represented by the raw buffered contract.
pub const MAX_RAW_RESPONSE_BODY_BYTES: usize = 64 * 1024 * 1024;

/// Response media-type requirement for one status class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseMediaPolicy<'a> {
    /// A content type is required and must match one admitted essence.
    Required(&'a [MediaType<'a>]),
    /// A content type may be absent, but a present value must match.
    Optional(&'a [MediaType<'a>]),
    /// A content type and response body are forbidden.
    Forbidden,
}

/// Response trailer handling.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TrailerPolicy {
    /// Reject declared or observed response trailers.
    Reject,
}

/// Invalid raw response-wire policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RawResponsePolicyError {
    /// An informational-response limit exceeded the global hard ceiling.
    InformationalLimitTooLarge,
    /// A body limit exceeded the buffered raw-response ceiling.
    BodyLimitTooLarge,
    /// Required or optional media policy omitted accepted media types.
    MissingMediaType,
    /// A media policy contains duplicate media-type essences.
    DuplicateMediaType,
    /// Too many response-header names were admitted.
    TooManyAdmittedHeaders,
    /// Admitted response-header names contain a duplicate.
    DuplicateAdmittedHeader,
    /// Credential, cookie, framing, proxy, or upgrade response metadata was admitted.
    UnsafeAdmittedHeader,
    /// A status class forbids media and body data but has a nonzero body limit.
    ForbiddenMediaHasBodyLimit,
}

impl_static_error!(RawResponsePolicyError,
    Self::InformationalLimitTooLarge => "informational response limit is too large",
    Self::BodyLimitTooLarge => "raw response body limit is too large",
    Self::MissingMediaType => "response media policy has no accepted media type",
    Self::DuplicateMediaType => "response media policy contains duplicate media types",
    Self::TooManyAdmittedHeaders => "too many response headers are admitted",
    Self::DuplicateAdmittedHeader => "an admitted response header is duplicated",
    Self::UnsafeAdmittedHeader => "an unsafe response header was admitted",
    Self::ForbiddenMediaHasBodyLimit => "forbidden response media has a nonzero body limit",
);

/// Complete provider-neutral admission policy for one final HTTP response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RawResponsePolicy<'a> {
    success_body_bytes: usize,
    error_body_bytes: usize,
    success_media: ResponseMediaPolicy<'a>,
    error_media: ResponseMediaPolicy<'a>,
    admitted_headers: [Option<HeaderName<'a>>; super::MAX_RESPONSE_HEADERS],
    admitted_header_count: usize,
    informational_limit: u8,
    trailer_policy: TrailerPolicy,
}

impl<'a> RawResponsePolicy<'a> {
    /// Validates separate success/error bounds, media rules, and retained headers.
    pub fn new(
        success_body_bytes: usize,
        error_body_bytes: usize,
        success_media: ResponseMediaPolicy<'a>,
        error_media: ResponseMediaPolicy<'a>,
        admitted_headers: &[HeaderName<'a>],
        informational_limit: u8,
    ) -> Result<Self, RawResponsePolicyError> {
        if success_body_bytes > MAX_RAW_RESPONSE_BODY_BYTES
            || error_body_bytes > MAX_RAW_RESPONSE_BODY_BYTES
        {
            return Err(RawResponsePolicyError::BodyLimitTooLarge);
        }
        if informational_limit > MAX_INFORMATIONAL_RESPONSES {
            return Err(RawResponsePolicyError::InformationalLimitTooLarge);
        }
        validate_media(success_media)?;
        validate_media(error_media)?;
        if (matches!(success_media, ResponseMediaPolicy::Forbidden) && success_body_bytes != 0)
            || (matches!(error_media, ResponseMediaPolicy::Forbidden) && error_body_bytes != 0)
        {
            return Err(RawResponsePolicyError::ForbiddenMediaHasBodyLimit);
        }
        validate_headers(admitted_headers)?;
        let mut owned_headers = [None; super::MAX_RESPONSE_HEADERS];
        for (index, header) in admitted_headers.iter().copied().enumerate() {
            if let Some(slot) = owned_headers.get_mut(index) {
                *slot = Some(header);
            }
        }
        Ok(Self {
            success_body_bytes,
            error_body_bytes,
            success_media,
            error_media,
            admitted_headers: owned_headers,
            admitted_header_count: admitted_headers.len(),
            informational_limit,
            trailer_policy: TrailerPolicy::Reject,
        })
    }

    /// Returns the body limit selected by the final status class.
    #[must_use]
    pub const fn body_limit(self, status: StatusCode) -> usize {
        if status.is_success() {
            self.success_body_bytes
        } else {
            self.error_body_bytes
        }
    }

    /// Returns the larger status-class bound needed for caller response storage.
    #[must_use]
    pub const fn max_body_bytes(self) -> usize {
        if self.success_body_bytes > self.error_body_bytes {
            self.success_body_bytes
        } else {
            self.error_body_bytes
        }
    }

    /// Returns the media policy selected by the final status class.
    #[must_use]
    pub const fn media_policy(self, status: StatusCode) -> ResponseMediaPolicy<'a> {
        if status.is_success() {
            self.success_media
        } else {
            self.error_media
        }
    }

    /// Reports whether an operation admitted retention of this response header.
    #[must_use]
    pub fn admits_header(self, name: &str) -> bool {
        self.admitted_headers
            .iter()
            .take(self.admitted_header_count)
            .flatten()
            .any(|candidate| candidate.eq_ignore_ascii_case(name))
    }

    /// Returns the informational-response limit.
    #[must_use]
    pub const fn informational_limit(self) -> u8 {
        self.informational_limit
    }

    /// Returns the explicit trailer policy.
    #[must_use]
    pub const fn trailer_policy(self) -> TrailerPolicy {
        self.trailer_policy
    }
}

/// Informational or final-response selection failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InformationalResponseError {
    /// `101 Switching Protocols` is never admitted by buffered cloud requests.
    SwitchingProtocols,
    /// Too many informational responses preceded the final response.
    TooManyInformationalResponses,
    /// A final response was still informational.
    MissingFinalResponse,
}

impl_static_error!(InformationalResponseError,
    Self::SwitchingProtocols => "switching protocols is forbidden",
    Self::TooManyInformationalResponses => "too many informational responses",
    Self::MissingFinalResponse => "final HTTP response is missing",
);

/// Small state machine for bounded informational-response handling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InformationalResponseTracker {
    limit: u8,
    observed: u8,
}

impl InformationalResponseTracker {
    /// Creates a tracker from a validated response policy.
    #[must_use]
    pub const fn new(policy: RawResponsePolicy<'_>) -> Self {
        Self {
            limit: policy.informational_limit,
            observed: 0,
        }
    }

    /// Observes one response head and reports whether it is the final head.
    pub fn observe(&mut self, status: StatusCode) -> Result<bool, InformationalResponseError> {
        if status.get() == 101 {
            return Err(InformationalResponseError::SwitchingProtocols);
        }
        if status.get() < 200 {
            self.observed = self
                .observed
                .checked_add(1)
                .ok_or(InformationalResponseError::TooManyInformationalResponses)?;
            if self.observed > self.limit {
                return Err(InformationalResponseError::TooManyInformationalResponses);
            }
            return Ok(false);
        }
        Ok(true)
    }

    /// Returns the number of admitted informational heads.
    #[must_use]
    pub const fn observed(self) -> u8 {
        self.observed
    }
}

/// Blocking execution of one already validated raw HTTP request.
pub trait BlockingRawHttpExecutor {
    /// Executor-specific phased failure.
    type Error;

    /// Executes exactly once without implicit authentication or retries.
    ///
    /// Implementations must use [`ResponseWriter::begin_attempt`]. Response
    /// mutation and commitment are available only through the returned guard.
    fn execute(
        &self,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error>;
}

/// Runtime-neutral asynchronous raw HTTP execution.
pub trait AsyncRawHttpExecutor {
    /// Executor-specific phased failure.
    type Error;

    /// Executes exactly once without implicit authentication or retries.
    ///
    /// Implementations must use [`ResponseWriter::begin_attempt`]. Response
    /// mutation and commitment are available only through the returned guard,
    /// which clears state across future cancellation.
    fn execute<'executor, 'request, 'policy, 'writer>(
        &'executor self,
        request: TransportRequest<'request>,
        policy: RawResponsePolicy<'policy>,
        response: &'writer mut ResponseWriter<'_>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'writer
    where
        'executor: 'writer,
        'request: 'writer,
        'policy: 'writer;
}

fn validate_media(policy: ResponseMediaPolicy<'_>) -> Result<(), RawResponsePolicyError> {
    let media = match policy {
        ResponseMediaPolicy::Required(media) | ResponseMediaPolicy::Optional(media) => media,
        ResponseMediaPolicy::Forbidden => return Ok(()),
    };
    if media.is_empty() {
        return Err(RawResponsePolicyError::MissingMediaType);
    }
    for (index, value) in media.iter().enumerate() {
        if media.get(..index).is_some_and(|seen| {
            seen.iter()
                .any(|candidate| candidate.as_str().eq_ignore_ascii_case(value.as_str()))
        }) {
            return Err(RawResponsePolicyError::DuplicateMediaType);
        }
    }
    Ok(())
}

fn validate_headers(headers: &[HeaderName<'_>]) -> Result<(), RawResponsePolicyError> {
    if headers.len() > super::MAX_RESPONSE_HEADERS {
        return Err(RawResponsePolicyError::TooManyAdmittedHeaders);
    }
    for (index, header) in headers.iter().enumerate() {
        if is_unsafe_response_header(header.as_str()) {
            return Err(RawResponsePolicyError::UnsafeAdmittedHeader);
        }
        if headers.get(..index).is_some_and(|seen| {
            seen.iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(header.as_str()))
        }) {
            return Err(RawResponsePolicyError::DuplicateAdmittedHeader);
        }
    }
    Ok(())
}

fn is_unsafe_response_header(name: &str) -> bool {
    [
        "authorization",
        "connection",
        "cookie",
        "proxy-authenticate",
        "proxy-authorization",
        "set-cookie",
        "te",
        "trailer",
        "transfer-encoding",
        "upgrade",
    ]
    .iter()
    .any(|candidate| name.eq_ignore_ascii_case(candidate))
}

#[cfg(test)]
mod tests {
    use super::{
        InformationalResponseError, InformationalResponseTracker, RawResponsePolicy,
        RawResponsePolicyError, ResponseMediaPolicy,
    };
    use crate::transport::{HeaderName, MediaType, StatusCode};

    fn policy(limit: u8) -> Result<RawResponsePolicy<'static>, RawResponsePolicyError> {
        RawResponsePolicy::new(
            1024,
            256,
            ResponseMediaPolicy::Required(&[MediaType::JSON]),
            ResponseMediaPolicy::Optional(&[MediaType::JSON]),
            &[],
            limit,
        )
    }

    #[test]
    fn selects_independent_success_and_error_limits() {
        let Ok(policy) = policy(2) else {
            return;
        };
        assert_eq!(policy.body_limit(StatusCode::OK), 1024);
        assert_eq!(
            policy.body_limit(StatusCode::new(400).unwrap_or(StatusCode::TOO_MANY_REQUESTS)),
            256
        );
    }

    #[test]
    fn bounds_informationals_and_rejects_switching_protocols() {
        let Ok(policy) = policy(2) else {
            return;
        };
        let mut tracker = InformationalResponseTracker::new(policy);
        let early = StatusCode::new(103).unwrap_or(StatusCode::OK);
        assert_eq!(tracker.observe(early), Ok(false));
        assert_eq!(tracker.observe(early), Ok(false));
        assert_eq!(
            tracker.observe(early),
            Err(InformationalResponseError::TooManyInformationalResponses)
        );
        let switching = StatusCode::new(101).unwrap_or(StatusCode::OK);
        assert_eq!(
            InformationalResponseTracker::new(policy).observe(switching),
            Err(InformationalResponseError::SwitchingProtocols)
        );
    }

    #[test]
    fn rejects_unsafe_and_duplicate_admitted_headers() {
        let unsafe_header = HeaderName::new("set-cookie");
        assert!(unsafe_header.is_ok());
        if let Ok(unsafe_header) = unsafe_header {
            assert!(matches!(
                RawResponsePolicy::new(
                    1,
                    1,
                    ResponseMediaPolicy::Optional(&[MediaType::JSON]),
                    ResponseMediaPolicy::Optional(&[MediaType::JSON]),
                    &[unsafe_header],
                    0,
                ),
                Err(RawResponsePolicyError::UnsafeAdmittedHeader)
            ));
        }
        let first = HeaderName::new("x-request-id");
        let second = HeaderName::new("X-Request-ID");
        assert!(first.is_ok() && second.is_ok());
        if let (Ok(first), Ok(second)) = (first, second) {
            assert!(matches!(
                RawResponsePolicy::new(
                    1,
                    1,
                    ResponseMediaPolicy::Optional(&[MediaType::JSON]),
                    ResponseMediaPolicy::Optional(&[MediaType::JSON]),
                    &[first, second],
                    0,
                ),
                Err(RawResponsePolicyError::DuplicateAdmittedHeader)
            ));
        }
    }
}
