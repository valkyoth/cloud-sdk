//! Sealed response-buffer admission, commitment, and cleanup.

use core::fmt;

use crate::operation::RequestIdPolicy;
use crate::rate_limit::RateLimit;

use super::cleanup::sanitize_response_storage;
use super::retained::{ProtectedRequestId, RetainedMetadataError, RetainedResponseMetadata};
use super::{ResponseContentType, ResponseHeaders, ResponseStorageSanitizer, StatusCode};

/// Bounded response metadata captured by a transport.
///
/// This type is intentionally neither `Copy` nor `Clone`. Header values and
/// protected request identifiers remain under one cleanup owner.
pub struct ResponseMetadata {
    content_type: Option<ResponseContentType>,
    rate_limit: Option<RateLimit>,
    headers: ResponseHeaders,
    request_id: Option<ProtectedRequestId>,
}

impl ResponseMetadata {
    /// Empty response metadata.
    pub const EMPTY: Self = Self {
        content_type: None,
        rate_limit: None,
        headers: ResponseHeaders::new(),
        request_id: None,
    };

    /// Adds a validated response content type.
    #[must_use]
    pub fn with_content_type(mut self, content_type: ResponseContentType) -> Self {
        self.content_type = Some(content_type);
        self
    }

    /// Adds validated rate-limit metadata.
    #[must_use]
    pub fn with_rate_limit(mut self, rate_limit: RateLimit) -> Self {
        self.rate_limit = Some(rate_limit);
        self
    }

    /// Adds complete bounded response headers.
    #[must_use]
    pub fn with_headers(mut self, headers: ResponseHeaders) -> Self {
        self.headers = headers;
        self
    }

    pub(crate) fn apply_request_id_policy(
        &mut self,
        policy: RequestIdPolicy,
    ) -> Result<(), RetainedMetadataError> {
        let request_id = self.headers.take_request_id()?;
        match policy {
            RequestIdPolicy::Discard => drop(request_id),
            RequestIdPolicy::Protected | RequestIdPolicy::Retain => {
                self.request_id = request_id;
            }
        }
        Ok(())
    }

    pub(crate) fn content_type(&self) -> Option<&ResponseContentType> {
        self.content_type.as_ref()
    }

    pub(crate) const fn rate_limit(&self) -> Option<RateLimit> {
        self.rate_limit
    }

    pub(crate) const fn headers(&self) -> &ResponseHeaders {
        &self.headers
    }

    pub(crate) fn with_request_id<R>(&self, inspect: impl FnOnce(Option<&[u8]>) -> R) -> R {
        inspect(self.request_id.as_ref().map(ProtectedRequestId::as_bytes))
    }

    pub(crate) fn request_id(&self) -> Option<&[u8]> {
        self.request_id.as_ref().map(ProtectedRequestId::as_bytes)
    }

    pub(crate) fn retain_request_id(
        &mut self,
        retention_limit: usize,
    ) -> Result<RetainedResponseMetadata, RetainedMetadataError> {
        let result = match self.request_id.as_mut() {
            Some(request_id) => request_id.transfer(retention_limit),
            None => Ok(RetainedResponseMetadata::empty_for_core()),
        };
        self.request_id = None;
        result
    }
}

impl fmt::Debug for ResponseMetadata {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResponseMetadata")
            .field("content_type", &self.content_type)
            .field("rate_limit", &self.rate_limit)
            .field("headers", &self.headers)
            .field("request_id", &"[redacted]")
            .finish()
    }
}

/// Response-writer state violation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseWriterError {
    /// The transport has already committed this response.
    AlreadyCommitted,
    /// The transport has not committed this response.
    NotCommitted,
    /// The committed initialized length exceeds the admitted body capacity.
    InitializedLengthTooLarge,
}

impl_static_error!(ResponseWriterError,
    Self::AlreadyCommitted => "response writer is already committed",
    Self::NotCommitted => "response writer is not committed",
    Self::InitializedLengthTooLarge => "response length exceeds admitted storage",
);

struct ResponseCommit {
    status: StatusCode,
    initialized_len: usize,
    metadata: ResponseMetadata,
}

/// Exclusive transport access to one admitted caller-owned response prefix.
///
/// This handle is only obtainable from [`ResponseBuffer::writer`]. It cannot
/// substitute an external or static body slice.
pub struct ResponseWriter<'buffer> {
    storage: &'buffer mut [u8],
    admitted_len: usize,
    commit: Option<ResponseCommit>,
}

impl ResponseWriter<'_> {
    /// Returns the admitted response-body capacity.
    #[must_use]
    pub const fn body_capacity(&self) -> usize {
        self.admitted_len
    }

    /// Returns exclusive access to the admitted prefix before commitment.
    pub fn body_mut(&mut self) -> Result<&mut [u8], ResponseWriterError> {
        if self.commit.is_some() {
            return Err(ResponseWriterError::AlreadyCommitted);
        }
        self.storage
            .get_mut(..self.admitted_len)
            .ok_or(ResponseWriterError::InitializedLengthTooLarge)
    }

    /// Commits status, initialized length, and bounded metadata exactly once.
    pub fn commit(
        &mut self,
        status: StatusCode,
        initialized_len: usize,
        metadata: ResponseMetadata,
    ) -> Result<(), ResponseWriterError> {
        if self.commit.is_some() {
            return Err(ResponseWriterError::AlreadyCommitted);
        }
        if initialized_len > self.admitted_len {
            return Err(ResponseWriterError::InitializedLengthTooLarge);
        }
        self.commit = Some(ResponseCommit {
            status,
            initialized_len,
            metadata,
        });
        Ok(())
    }

    /// Reports whether the transport committed the response.
    #[must_use]
    pub const fn is_committed(&self) -> bool {
        self.commit.is_some()
    }

    fn response(&self) -> Result<TransportResponse<'_>, ResponseWriterError> {
        let commit = self
            .commit
            .as_ref()
            .ok_or(ResponseWriterError::NotCommitted)?;
        let body = self
            .storage
            .get(..commit.initialized_len)
            .ok_or(ResponseWriterError::InitializedLengthTooLarge)?;
        Ok(TransportResponse::from_commit(commit, body))
    }

    fn metadata_mut(&mut self) -> Result<&mut ResponseMetadata, ResponseWriterError> {
        self.commit
            .as_mut()
            .map(|commit| &mut commit.metadata)
            .ok_or(ResponseWriterError::NotCommitted)
    }

    fn metadata(&self) -> Result<&ResponseMetadata, ResponseWriterError> {
        self.commit
            .as_ref()
            .map(|commit| &commit.metadata)
            .ok_or(ResponseWriterError::NotCommitted)
    }

    fn initialized_body(&self, initialized_len: usize) -> &[u8] {
        self.storage.get(..initialized_len).unwrap_or_default()
    }
}

impl fmt::Debug for ResponseWriter<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResponseWriter")
            .field("storage_capacity", &self.storage.len())
            .field("admitted_len", &self.admitted_len)
            .field("committed", &self.commit.is_some())
            .field("body", &"[redacted]")
            .finish()
    }
}

/// Cleanup-owning admission guard around one sealed [`ResponseWriter`].
///
/// Core volatile-clears the complete original storage before admission and on
/// drop. [`Self::with_additive_sanitizer`] can add a platform operation without
/// replacing either mandatory clear.
pub struct ResponseBuffer<'buffer> {
    writer: ResponseWriter<'buffer>,
    additive: Option<&'buffer dyn ResponseStorageSanitizer>,
}

impl<'buffer> ResponseBuffer<'buffer> {
    /// Admits at most `max_body_bytes` and clears all supplied storage.
    #[must_use]
    pub fn new(storage: &'buffer mut [u8], max_body_bytes: usize) -> Self {
        Self::construct(storage, max_body_bytes, None)
    }

    /// Adds a platform cleanup hook between mandatory core clears.
    #[must_use]
    pub fn with_additive_sanitizer(
        storage: &'buffer mut [u8],
        max_body_bytes: usize,
        additive: &'buffer dyn ResponseStorageSanitizer,
    ) -> Self {
        Self::construct(storage, max_body_bytes, Some(additive))
    }

    fn construct(
        storage: &'buffer mut [u8],
        max_body_bytes: usize,
        additive: Option<&'buffer dyn ResponseStorageSanitizer>,
    ) -> Self {
        sanitize_response_storage(storage, additive);
        Self {
            writer: ResponseWriter {
                admitted_len: core::cmp::min(storage.len(), max_body_bytes),
                storage,
                commit: None,
            },
            additive,
        }
    }

    /// Returns exclusive transport access to the admitted response prefix.
    #[must_use]
    pub const fn writer(&mut self) -> &mut ResponseWriter<'buffer> {
        &mut self.writer
    }

    /// Inspects a committed response without allowing its body to escape.
    pub fn with_response<R>(
        &self,
        inspect: impl for<'response> FnOnce(TransportResponse<'response>) -> R,
    ) -> Result<R, ResponseWriterError> {
        let response = self.writer.response()?;
        Ok(inspect(response))
    }

    pub(crate) fn response(&self) -> Result<TransportResponse<'_>, ResponseWriterError> {
        self.writer.response()
    }

    pub(crate) fn apply_request_id_policy(
        &mut self,
        policy: RequestIdPolicy,
    ) -> Result<(), RetainedMetadataError> {
        self.writer
            .metadata_mut()
            .map_err(|_| RetainedMetadataError::RetentionForbidden)?
            .apply_request_id_policy(policy)
    }

    pub(crate) fn metadata(&self) -> Result<&ResponseMetadata, ResponseWriterError> {
        self.writer.metadata()
    }

    pub(crate) fn metadata_mut(&mut self) -> Result<&mut ResponseMetadata, ResponseWriterError> {
        self.writer.metadata_mut()
    }

    pub(crate) fn initialized_body(&self, initialized_len: usize) -> &[u8] {
        self.writer.initialized_body(initialized_len)
    }
}

impl fmt::Debug for ResponseBuffer<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResponseBuffer")
            .field("writer", &self.writer)
            .field("additive", &self.additive.is_some())
            .finish()
    }
}

impl Drop for ResponseBuffer<'_> {
    fn drop(&mut self) {
        sanitize_response_storage(self.writer.storage, self.additive);
    }
}

/// Borrowed view committed from an admitted writer.
///
/// Safe callers cannot construct a response from unrelated storage.
///
/// ```compile_fail
/// use cloud_sdk::transport::{StatusCode, TransportResponse};
///
/// let external = b"unadmitted";
/// let _ = TransportResponse {
///     status: StatusCode::OK,
///     body: external,
///     metadata: unreachable!(),
/// };
/// ```
#[derive(Clone, Copy)]
pub struct TransportResponse<'response> {
    status: StatusCode,
    body: &'response [u8],
    metadata: &'response ResponseMetadata,
}

impl<'response> TransportResponse<'response> {
    fn from_commit(commit: &'response ResponseCommit, body: &'response [u8]) -> Self {
        Self {
            status: commit.status,
            body,
            metadata: &commit.metadata,
        }
    }

    /// Returns the status code.
    #[must_use]
    pub const fn status(&self) -> StatusCode {
        self.status
    }

    /// Returns initialized response body bytes.
    #[must_use]
    pub const fn body(&self) -> &'response [u8] {
        self.body
    }

    /// Returns the validated response content type when supplied.
    #[must_use]
    pub fn content_type(&self) -> Option<&'response ResponseContentType> {
        self.metadata.content_type()
    }

    /// Returns validated rate-limit metadata when supplied.
    #[must_use]
    pub const fn rate_limit(&self) -> Option<RateLimit> {
        self.metadata.rate_limit()
    }

    /// Returns complete bounded response-header metadata.
    #[must_use]
    pub const fn headers(&self) -> &'response ResponseHeaders {
        self.metadata.headers()
    }

    /// Runs a closure with a protected provider request identifier.
    pub fn with_request_id<R>(&self, inspect: impl FnOnce(Option<&[u8]>) -> R) -> R {
        self.metadata.with_request_id(inspect)
    }
}

impl fmt::Debug for TransportResponse<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TransportResponse")
            .field("status", &self.status)
            .field("body_len", &self.body.len())
            .field("body", &"[redacted]")
            .field("metadata", &self.metadata)
            .finish()
    }
}
