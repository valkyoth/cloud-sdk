//! Checked provider-neutral response policy.

use core::fmt;

use super::RequestIdPolicy;
use crate::rate_limit::RateLimit;
use crate::transport::{
    MediaType, ResponseBuffer, ResponseContentType, ResponseDecodeWorkspace, ResponseMetadata,
    ResponseWriterError, RetainedMetadataError, RetainedResponseMetadata, StatusCode,
    TransportResponse,
};

/// Expected response-body shape.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResponseBodyPolicy {
    /// A non-empty response body is required.
    Required,
    /// An empty or non-empty response body is accepted.
    Optional,
    /// Any response body is rejected.
    Forbidden,
}

/// Response content-type requirement and accepted media types.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContentTypePolicy {
    /// A content type is required and must match one accepted media type.
    Required(&'static [MediaType<'static>]),
    /// A content type may be absent, but when present it must match.
    Optional(&'static [MediaType<'static>]),
    /// Any response content type is rejected.
    Forbidden,
}

/// Invalid response-policy construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponsePolicyValidationError {
    /// At least one success status is required.
    MissingSuccessStatus,
    /// Expected success statuses must be in the HTTP `2xx` range.
    NonSuccessStatus,
    /// Expected success statuses must not contain duplicates.
    DuplicateSuccessStatus,
    /// Required or optional content-type policy needs accepted media types.
    MissingAcceptedMediaType,
    /// Accepted media types must not contain duplicates.
    DuplicateAcceptedMediaType,
    /// Required response bodies need a nonzero maximum length.
    RequiredBodyHasZeroLimit,
    /// Forbidden response bodies require a zero maximum length.
    ForbiddenBodyHasNonzeroLimit,
    /// A forbidden body cannot require or optionally accept a content type.
    ForbiddenBodyAllowsContentType,
    /// A required body cannot forbid its content type.
    RequiredBodyForbidsContentType,
}

impl_static_error!(ResponsePolicyValidationError,
    Self::MissingSuccessStatus => "response policy has no success status",
    Self::NonSuccessStatus => "response policy contains a non-success status",
    Self::DuplicateSuccessStatus => "response policy contains duplicate statuses",
    Self::MissingAcceptedMediaType => "response policy has no accepted media type",
    Self::DuplicateAcceptedMediaType => "response policy contains duplicate media types",
    Self::RequiredBodyHasZeroLimit => "required response body has a zero limit",
    Self::ForbiddenBodyHasNonzeroLimit => "forbidden response body has a nonzero limit",
    Self::ForbiddenBodyAllowsContentType => "forbidden response body allows a content type",
    Self::RequiredBodyForbidsContentType => "required response body forbids its content type",
);

/// Response rejected before provider decoding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponsePolicyError {
    /// The response status is not an expected success status.
    UnexpectedStatus,
    /// The initialized body exceeds the operation's admitted limit.
    BodyTooLarge,
    /// A required response body is empty.
    MissingBody,
    /// A response body was supplied when forbidden.
    ForbiddenBody,
    /// A required response content type is absent.
    MissingContentType,
    /// The supplied content type is not accepted.
    UnexpectedContentType,
    /// A content type was supplied when forbidden.
    ForbiddenContentType,
    /// The response writer was not successfully committed.
    UncommittedResponse,
    /// A present provider request identifier violated its bounded policy.
    InvalidRequestId,
}

impl_static_error!(ResponsePolicyError,
    Self::UnexpectedStatus => "response status is not expected",
    Self::BodyTooLarge => "response body exceeds the operation limit",
    Self::MissingBody => "required response body is missing",
    Self::ForbiddenBody => "response body is forbidden",
    Self::MissingContentType => "required response content type is missing",
    Self::UnexpectedContentType => "response content type is not accepted",
    Self::ForbiddenContentType => "response content type is forbidden",
    Self::UncommittedResponse => "response writer is not committed",
    Self::InvalidRequestId => "response request identifier is invalid",
);

/// Complete checked-response policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResponsePolicy {
    success_statuses: &'static [StatusCode],
    content_type: ContentTypePolicy,
    body: ResponseBodyPolicy,
    max_body_bytes: usize,
}

impl ResponsePolicy {
    /// Creates a complete policy without implicit status, media, or body defaults.
    pub fn new(
        success_statuses: &'static [StatusCode],
        content_type: ContentTypePolicy,
        body: ResponseBodyPolicy,
        max_body_bytes: usize,
    ) -> Result<Self, ResponsePolicyValidationError> {
        validate_statuses(success_statuses)?;
        validate_media_types(content_type)?;
        match (body, content_type, max_body_bytes) {
            (ResponseBodyPolicy::Required, _, 0) => {
                return Err(ResponsePolicyValidationError::RequiredBodyHasZeroLimit);
            }
            (ResponseBodyPolicy::Forbidden, _, limit) if limit != 0 => {
                return Err(ResponsePolicyValidationError::ForbiddenBodyHasNonzeroLimit);
            }
            (
                ResponseBodyPolicy::Forbidden,
                ContentTypePolicy::Required(_) | ContentTypePolicy::Optional(_),
                _,
            ) => {
                return Err(ResponsePolicyValidationError::ForbiddenBodyAllowsContentType);
            }
            (ResponseBodyPolicy::Required, ContentTypePolicy::Forbidden, _) => {
                return Err(ResponsePolicyValidationError::RequiredBodyForbidsContentType);
            }
            _ => {}
        }
        Ok(Self {
            success_statuses,
            content_type,
            body,
            max_body_bytes,
        })
    }

    /// Returns expected success statuses.
    #[must_use]
    pub const fn success_statuses(self) -> &'static [StatusCode] {
        self.success_statuses
    }

    /// Returns response content-type policy.
    #[must_use]
    pub const fn content_type_policy(self) -> ContentTypePolicy {
        self.content_type
    }

    /// Returns response-body policy.
    #[must_use]
    pub const fn body_policy(self) -> ResponseBodyPolicy {
        self.body
    }

    /// Returns maximum admitted initialized response bytes.
    #[must_use]
    pub const fn max_body_bytes(self) -> usize {
        self.max_body_bytes
    }

    /// Checks status, initialized length, body shape, and content type.
    pub fn validate<'buffer>(
        self,
        mut writer: ResponseBuffer<'buffer>,
        request_id_policy: RequestIdPolicy,
    ) -> Result<CheckedResponseGuard<'buffer>, ResponsePolicyError> {
        writer.response().map_err(map_writer_error)?;
        writer
            .apply_request_id_policy(request_id_policy)
            .map_err(|_| ResponsePolicyError::InvalidRequestId)?;
        let snapshot = {
            let response = writer.response().map_err(map_writer_error)?;
            self.validate_view(response, request_id_policy)?
        };
        Ok(CheckedResponseGuard {
            writer,
            snapshot,
            workspace: ResponseDecodeWorkspace::new(),
        })
    }

    fn validate_view(
        self,
        response: TransportResponse<'_>,
        request_id_policy: RequestIdPolicy,
    ) -> Result<CheckedResponseSnapshot, ResponsePolicyError> {
        if !self.success_statuses.contains(&response.status()) {
            return Err(ResponsePolicyError::UnexpectedStatus);
        }
        match self.body {
            ResponseBodyPolicy::Forbidden if !response.body().is_empty() => {
                return Err(ResponsePolicyError::ForbiddenBody);
            }
            _ => {}
        }
        if response.body().len() > self.max_body_bytes {
            return Err(ResponsePolicyError::BodyTooLarge);
        }
        if matches!(self.body, ResponseBodyPolicy::Required) && response.body().is_empty() {
            return Err(ResponsePolicyError::MissingBody);
        }
        validate_content_type(self.content_type, response.content_type())?;
        Ok(CheckedResponseSnapshot {
            status: response.status(),
            body_len: response.body().len(),
            rate_limit: response.rate_limit(),
            request_id_policy,
        })
    }
}

/// Response that passed one operation's complete provider-neutral policy.
#[derive(Clone, Copy)]
pub struct CheckedResponse<'body> {
    status: StatusCode,
    body: &'body [u8],
    content_type: Option<&'body ResponseContentType>,
    rate_limit: Option<RateLimit>,
    request_id: Option<&'body [u8]>,
    request_id_policy: RequestIdPolicy,
}

impl CheckedResponse<'_> {
    /// Returns the checked status code.
    #[must_use]
    pub const fn status(&self) -> StatusCode {
        self.status
    }

    /// Returns the checked initialized response body.
    #[must_use]
    pub const fn body(&self) -> &[u8] {
        self.body
    }

    /// Returns the checked response content type when supplied.
    #[must_use]
    pub const fn content_type(&self) -> Option<&ResponseContentType> {
        self.content_type
    }

    /// Returns validated rate-limit metadata when supplied.
    #[must_use]
    pub const fn rate_limit(&self) -> Option<RateLimit> {
        self.rate_limit
    }

    /// Returns the request-identifier lifecycle policy.
    #[must_use]
    pub const fn request_id_policy(&self) -> RequestIdPolicy {
        self.request_id_policy
    }

    /// Runs a closure with the protected request identifier when retained.
    pub fn with_request_id<R>(&self, inspect: impl FnOnce(Option<&[u8]>) -> R) -> R {
        inspect(self.request_id)
    }
}

#[derive(Clone, Copy)]
struct CheckedResponseSnapshot {
    status: StatusCode,
    body_len: usize,
    rate_limit: Option<RateLimit>,
    request_id_policy: RequestIdPolicy,
}

/// Policy-checked response that owns cleanup of its caller storage.
///
/// Borrowed access is closure-scoped. [`Self::decode_owned`] clears the
/// complete response storage before returning the owned decoded result.
pub struct CheckedResponseGuard<'buffer> {
    writer: ResponseBuffer<'buffer>,
    snapshot: CheckedResponseSnapshot,
    workspace: ResponseDecodeWorkspace,
}

impl CheckedResponseGuard<'_> {
    /// Returns the checked status code.
    #[must_use]
    pub const fn status(&self) -> StatusCode {
        self.snapshot.status
    }

    /// Returns the checked response content type when supplied.
    #[must_use]
    pub fn content_type(&self) -> Option<&ResponseContentType> {
        self.writer
            .metadata()
            .ok()
            .and_then(ResponseMetadata::content_type)
    }

    /// Returns validated rate-limit metadata when supplied.
    #[must_use]
    pub const fn rate_limit(&self) -> Option<RateLimit> {
        self.snapshot.rate_limit
    }

    /// Runs a closure with a checked response borrow that cannot escape.
    ///
    /// ```compile_fail
    /// use cloud_sdk::operation::CheckedResponseGuard;
    /// fn escape<'guard>(
    ///     guard: &'guard CheckedResponseGuard<'_>,
    /// ) -> &'guard [u8] {
    ///     guard.with_borrowed(|response| response.body())
    /// }
    /// ```
    pub fn with_borrowed<R>(
        &self,
        inspect: impl for<'response> FnOnce(CheckedResponse<'response>) -> R,
    ) -> R {
        inspect(self.checked_response())
    }

    /// Decodes an owned value, clears all response storage, and then returns.
    pub fn decode_owned<R, E>(
        self,
        decode: impl for<'response> FnOnce(CheckedResponse<'response>) -> Result<R, E>,
    ) -> Result<R, E> {
        self.decode_owned_with_workspace(|response, _workspace| decode(response))
    }

    /// Decodes with guard-owned scratch, then clears the complete workspace.
    pub fn decode_owned_with_workspace<R, E>(
        mut self,
        decode: impl for<'response> FnOnce(
            CheckedResponse<'response>,
            &mut ResponseDecodeWorkspace,
        ) -> Result<R, E>,
    ) -> Result<R, E> {
        let result = {
            let Self {
                writer,
                snapshot,
                workspace,
            } = &mut self;
            let response = checked_response(writer, *snapshot);
            decode(response, workspace)
        };
        drop(self);
        result
    }

    /// Atomically moves a retainable request ID into another cleanup owner.
    pub fn retain_metadata(
        &mut self,
        request_id_limit: usize,
    ) -> Result<RetainedResponseMetadata, RetainedMetadataError> {
        if self.snapshot.request_id_policy != RequestIdPolicy::Retain {
            return Err(RetainedMetadataError::RetentionForbidden);
        }
        self.writer
            .metadata_mut()
            .map_err(|_| RetainedMetadataError::RetentionForbidden)?
            .retain_request_id(request_id_limit)
    }

    fn checked_response(&self) -> CheckedResponse<'_> {
        checked_response(&self.writer, self.snapshot)
    }
}

impl fmt::Debug for CheckedResponseGuard<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CheckedResponseGuard")
            .field("status", &self.status())
            .field("body_len", &self.snapshot.body_len)
            .field("body", &"[redacted]")
            .field("content_type", &self.content_type())
            .field("rate_limit", &self.rate_limit())
            .field("request_id", &"[redacted]")
            .finish()
    }
}

impl fmt::Debug for CheckedResponse<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CheckedResponse")
            .field("status", &self.status())
            .field("body_len", &self.body().len())
            .field("body", &"[redacted]")
            .field("content_type", &self.content_type())
            .field("rate_limit", &self.rate_limit())
            .field("request_id", &"[redacted]")
            .finish()
    }
}

fn checked_response<'response>(
    writer: &'response ResponseBuffer<'_>,
    snapshot: CheckedResponseSnapshot,
) -> CheckedResponse<'response> {
    let metadata = writer.metadata().ok();
    CheckedResponse {
        status: snapshot.status,
        body: writer.initialized_body(snapshot.body_len),
        content_type: metadata.and_then(ResponseMetadata::content_type),
        rate_limit: snapshot.rate_limit,
        request_id: metadata.and_then(ResponseMetadata::request_id),
        request_id_policy: snapshot.request_id_policy,
    }
}

fn validate_statuses(statuses: &[StatusCode]) -> Result<(), ResponsePolicyValidationError> {
    if statuses.is_empty() {
        return Err(ResponsePolicyValidationError::MissingSuccessStatus);
    }
    for (index, status) in statuses.iter().enumerate() {
        if !status.is_success() {
            return Err(ResponsePolicyValidationError::NonSuccessStatus);
        }
        if statuses
            .get(..index)
            .is_some_and(|seen| seen.contains(status))
        {
            return Err(ResponsePolicyValidationError::DuplicateSuccessStatus);
        }
    }
    Ok(())
}

fn validate_media_types(policy: ContentTypePolicy) -> Result<(), ResponsePolicyValidationError> {
    let media_types = match policy {
        ContentTypePolicy::Required(values) | ContentTypePolicy::Optional(values) => values,
        ContentTypePolicy::Forbidden => return Ok(()),
    };
    if media_types.is_empty() {
        return Err(ResponsePolicyValidationError::MissingAcceptedMediaType);
    }
    for (index, media_type) in media_types.iter().enumerate() {
        if media_types.get(..index).is_some_and(|seen| {
            seen.iter()
                .any(|candidate| candidate.as_str().eq_ignore_ascii_case(media_type.as_str()))
        }) {
            return Err(ResponsePolicyValidationError::DuplicateAcceptedMediaType);
        }
    }
    Ok(())
}

fn validate_content_type(
    policy: ContentTypePolicy,
    actual: Option<&ResponseContentType>,
) -> Result<(), ResponsePolicyError> {
    match (policy, actual) {
        (ContentTypePolicy::Required(_), None) => Err(ResponsePolicyError::MissingContentType),
        (ContentTypePolicy::Forbidden, Some(_)) => Err(ResponsePolicyError::ForbiddenContentType),
        (ContentTypePolicy::Forbidden | ContentTypePolicy::Optional(_), None) => Ok(()),
        (
            ContentTypePolicy::Required(accepted) | ContentTypePolicy::Optional(accepted),
            Some(actual),
        ) => {
            if accepted
                .iter()
                .any(|media_type| actual.matches(*media_type))
            {
                Ok(())
            } else {
                Err(ResponsePolicyError::UnexpectedContentType)
            }
        }
    }
}

const fn map_writer_error(error: ResponseWriterError) -> ResponsePolicyError {
    match error {
        ResponseWriterError::NotCommitted
        | ResponseWriterError::AlreadyCommitted
        | ResponseWriterError::InitializedLengthTooLarge => {
            ResponsePolicyError::UncommittedResponse
        }
    }
}
