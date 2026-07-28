//! Sealed response-buffer admission, commitment, and cleanup.

use core::fmt;

use crate::operation::RequestIdPolicy;
use crate::rate_limit::RateLimit;

use super::cleanup::sanitize_response_storage;
use super::retained::{ProtectedRequestId, RetainedMetadataError, RetainedResponseMetadata};
use super::{ResponseContentType, ResponseHeaders, ResponseStorageSanitizer, StatusCode};

/// Non-sensitive interpreted response metadata captured by a transport.
#[derive(Debug)]
pub struct ResponseMetadata {
    rate_limit: Option<RateLimit>,
}

impl ResponseMetadata {
    /// Empty response metadata.
    pub const EMPTY: Self = Self { rate_limit: None };

    /// Adds validated rate-limit metadata.
    #[must_use]
    pub fn with_rate_limit(mut self, rate_limit: RateLimit) -> Self {
        self.rate_limit = Some(rate_limit);
        self
    }

    pub(crate) const fn rate_limit(&self) -> Option<RateLimit> {
        self.rate_limit
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
    headers: ResponseHeaders<'buffer>,
    request_id: Option<ProtectedRequestId>,
    commit: Option<ResponseCommit>,
}

impl<'buffer> ResponseWriter<'buffer> {
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

    /// Returns stable caller-owned response-header storage before commitment.
    pub fn headers_mut(&mut self) -> Result<&mut ResponseHeaders<'buffer>, ResponseWriterError> {
        if self.commit.is_some() {
            return Err(ResponseWriterError::AlreadyCommitted);
        }
        Ok(&mut self.headers)
    }

    /// Returns the response headers captured so far.
    pub fn headers(&self) -> &ResponseHeaders<'buffer> {
        &self.headers
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

    fn response(&self) -> Result<TransportResponse<'_, 'buffer>, ResponseWriterError> {
        let commit = self
            .commit
            .as_ref()
            .ok_or(ResponseWriterError::NotCommitted)?;
        let body = self
            .storage
            .get(..commit.initialized_len)
            .ok_or(ResponseWriterError::InitializedLengthTooLarge)?;
        Ok(TransportResponse::from_commit(
            commit,
            body,
            &self.headers,
            self.request_id,
        ))
    }

    fn apply_request_id_policy(
        &mut self,
        policy: RequestIdPolicy,
    ) -> Result<(), RetainedMetadataError> {
        let request_id = self.headers.hide_request_id()?;
        match policy {
            RequestIdPolicy::Discard => {
                if let Some(request_id) = request_id {
                    self.headers.clear_protected(request_id);
                }
            }
            RequestIdPolicy::Protected | RequestIdPolicy::Retain => {
                self.request_id = request_id;
            }
        }
        Ok(())
    }

    fn request_id(&self) -> Option<&[u8]> {
        self.request_id
            .and_then(|request_id| self.headers.protected_value(request_id))
    }

    fn retain_request_id<'destination>(
        &mut self,
        destination: &'destination mut [u8],
        retention_limit: usize,
    ) -> Result<RetainedResponseMetadata<'destination>, RetainedMetadataError> {
        let mut retained = RetainedResponseMetadata::empty_for_core(destination);
        let Some(request_id) = self.request_id.take() else {
            return Ok(retained);
        };
        let result = {
            let source = self
                .headers
                .protected_value(request_id)
                .ok_or(RetainedMetadataError::RequestIdTooLong)?;
            if source.len() > retention_limit {
                Err(RetainedMetadataError::RetentionLimitExceeded)
            } else {
                retained.write_request_id(source)
            }
        };
        self.headers.clear_protected(request_id);
        result.map(|()| retained)
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
    pub fn new(
        storage: &'buffer mut [u8],
        max_body_bytes: usize,
        header_storage: &'buffer mut [u8],
    ) -> Self {
        Self::construct(storage, max_body_bytes, header_storage, None)
    }

    /// Adds a platform cleanup hook between mandatory core clears.
    #[must_use]
    pub fn with_additive_sanitizer(
        storage: &'buffer mut [u8],
        max_body_bytes: usize,
        header_storage: &'buffer mut [u8],
        additive: &'buffer dyn ResponseStorageSanitizer,
    ) -> Self {
        Self::construct(storage, max_body_bytes, header_storage, Some(additive))
    }

    fn construct(
        storage: &'buffer mut [u8],
        max_body_bytes: usize,
        header_storage: &'buffer mut [u8],
        additive: Option<&'buffer dyn ResponseStorageSanitizer>,
    ) -> Self {
        let headers = ResponseHeaders::new(header_storage);
        sanitize_response_storage(storage, additive);
        Self {
            writer: ResponseWriter {
                admitted_len: core::cmp::min(storage.len(), max_body_bytes),
                storage,
                headers,
                request_id: None,
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
        inspect: impl for<'response> FnOnce(TransportResponse<'response, 'buffer>) -> R,
    ) -> Result<R, ResponseWriterError> {
        let response = self.writer.response()?;
        Ok(inspect(response))
    }

    pub(crate) fn response(&self) -> Result<TransportResponse<'_, 'buffer>, ResponseWriterError> {
        self.writer.response()
    }

    pub(crate) fn apply_request_id_policy(
        &mut self,
        policy: RequestIdPolicy,
    ) -> Result<(), RetainedMetadataError> {
        self.writer.apply_request_id_policy(policy)
    }

    pub(crate) fn request_id(&self) -> Option<&[u8]> {
        self.writer.request_id()
    }

    pub(crate) fn retain_request_id<'destination>(
        &mut self,
        destination: &'destination mut [u8],
        retention_limit: usize,
    ) -> Result<RetainedResponseMetadata<'destination>, RetainedMetadataError> {
        self.writer.retain_request_id(destination, retention_limit)
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
pub struct TransportResponse<'response, 'storage> {
    status: StatusCode,
    body: &'response [u8],
    metadata: &'response ResponseMetadata,
    headers: &'response ResponseHeaders<'storage>,
    request_id: Option<ProtectedRequestId>,
}

impl<'response, 'storage> TransportResponse<'response, 'storage> {
    fn from_commit(
        commit: &'response ResponseCommit,
        body: &'response [u8],
        headers: &'response ResponseHeaders<'storage>,
        request_id: Option<ProtectedRequestId>,
    ) -> Self {
        Self {
            status: commit.status,
            body,
            metadata: &commit.metadata,
            headers,
            request_id,
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
    pub fn content_type(&self) -> Option<ResponseContentType<'response>> {
        let header = self.headers.get("content-type")?;
        let value = core::str::from_utf8(header.value()).ok()?;
        ResponseContentType::new(value).ok()
    }

    /// Returns validated rate-limit metadata when supplied.
    #[must_use]
    pub const fn rate_limit(&self) -> Option<RateLimit> {
        self.metadata.rate_limit()
    }

    /// Returns complete bounded response-header metadata.
    #[must_use]
    pub const fn headers(&self) -> &'response ResponseHeaders<'storage> {
        self.headers
    }

    /// Runs a closure with a protected provider request identifier.
    pub fn with_request_id<R>(&self, inspect: impl FnOnce(Option<&[u8]>) -> R) -> R {
        inspect(
            self.request_id
                .and_then(|request_id| self.headers.protected_value(request_id)),
        )
    }
}

impl fmt::Debug for TransportResponse<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TransportResponse")
            .field("status", &self.status)
            .field("body_len", &self.body.len())
            .field("body", &"[redacted]")
            .field("metadata", &self.metadata)
            .field("headers", &self.headers)
            .field("request_id", &"[redacted]")
            .finish()
    }
}
