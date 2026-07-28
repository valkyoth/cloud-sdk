//! Sealed response-buffer admission, commitment, and cleanup.

use core::fmt;

use crate::rate_limit::RateLimit;

use super::{ResponseContentType, ResponseHeaders, ResponseStorageSanitizer, StatusCode};

/// Bounded response metadata captured by a transport.
#[derive(Clone, Copy, Debug)]
pub struct ResponseMetadata {
    content_type: Option<ResponseContentType>,
    rate_limit: Option<RateLimit>,
    headers: ResponseHeaders,
}

impl ResponseMetadata {
    /// Empty response metadata.
    pub const EMPTY: Self = Self {
        content_type: None,
        rate_limit: None,
        headers: ResponseHeaders::new(),
    };

    /// Adds a validated response content type.
    #[must_use]
    pub const fn with_content_type(mut self, content_type: ResponseContentType) -> Self {
        self.content_type = Some(content_type);
        self
    }

    /// Adds validated rate-limit metadata.
    #[must_use]
    pub const fn with_rate_limit(mut self, rate_limit: RateLimit) -> Self {
        self.rate_limit = Some(rate_limit);
        self
    }

    /// Adds complete bounded response headers.
    #[must_use]
    pub const fn with_headers(mut self, headers: ResponseHeaders) -> Self {
        self.headers = headers;
        self
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

#[derive(Clone, Copy, Debug)]
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
        let commit = self.commit.ok_or(ResponseWriterError::NotCommitted)?;
        let body = self
            .storage
            .get(..commit.initialized_len)
            .ok_or(ResponseWriterError::InitializedLengthTooLarge)?;
        Ok(TransportResponse::from_commit(commit, body))
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
/// The complete original storage is sanitized before admission and again when
/// this guard is dropped, including bytes outside the admitted prefix.
pub struct ResponseBuffer<'buffer, 'sanitizer, S: ResponseStorageSanitizer + ?Sized> {
    writer: ResponseWriter<'buffer>,
    sanitizer: &'sanitizer S,
}

impl<'buffer, 'sanitizer, S> ResponseBuffer<'buffer, 'sanitizer, S>
where
    S: ResponseStorageSanitizer + ?Sized,
{
    /// Admits at most `max_body_bytes` from caller storage and clears all
    /// supplied storage before transport use.
    #[must_use]
    pub fn new(
        storage: &'buffer mut [u8],
        max_body_bytes: usize,
        sanitizer: &'sanitizer S,
    ) -> Self {
        sanitizer.sanitize_response_storage(storage);
        Self {
            writer: ResponseWriter {
                admitted_len: core::cmp::min(storage.len(), max_body_bytes),
                storage,
                commit: None,
            },
            sanitizer,
        }
    }

    /// Returns exclusive transport access to the admitted response prefix.
    #[must_use]
    pub const fn writer(&mut self) -> &mut ResponseWriter<'buffer> {
        &mut self.writer
    }

    /// Inspects a committed response without allowing its body borrow to
    /// escape the closure.
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

    pub(crate) fn initialized_body(&self, initialized_len: usize) -> &[u8] {
        self.writer.initialized_body(initialized_len)
    }
}

impl<S> fmt::Debug for ResponseBuffer<'_, '_, S>
where
    S: ResponseStorageSanitizer + ?Sized,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResponseBuffer")
            .field("writer", &self.writer)
            .finish()
    }
}

impl<S> Drop for ResponseBuffer<'_, '_, S>
where
    S: ResponseStorageSanitizer + ?Sized,
{
    fn drop(&mut self) {
        self.sanitizer
            .sanitize_response_storage(self.writer.storage);
    }
}

/// Borrowed view of response bytes committed from an admitted writer.
///
/// Safe callers cannot construct a response from unrelated storage.
///
/// ```compile_fail
/// use cloud_sdk::rate_limit::RateLimit;
/// use cloud_sdk::transport::{
///     ResponseContentType, ResponseHeaders, StatusCode, TransportResponse,
/// };
///
/// let external = b"unadmitted";
/// let _ = TransportResponse {
///     status: StatusCode::OK,
///     body: external,
///     content_type: None::<ResponseContentType>,
///     rate_limit: None::<RateLimit>,
///     headers: ResponseHeaders::new(),
/// };
/// ```
#[derive(Clone, Copy)]
pub struct TransportResponse<'buffer> {
    status: StatusCode,
    body: &'buffer [u8],
    content_type: Option<ResponseContentType>,
    rate_limit: Option<RateLimit>,
    headers: ResponseHeaders,
}

impl<'buffer> TransportResponse<'buffer> {
    const fn from_commit(commit: ResponseCommit, body: &'buffer [u8]) -> Self {
        Self {
            status: commit.status,
            body,
            content_type: commit.metadata.content_type,
            rate_limit: commit.metadata.rate_limit,
            headers: commit.metadata.headers,
        }
    }

    /// Returns the status code.
    #[must_use]
    pub const fn status(&self) -> StatusCode {
        self.status
    }

    /// Returns the initialized response body bytes.
    #[must_use]
    pub const fn body(&self) -> &'buffer [u8] {
        self.body
    }

    /// Returns the validated response content type when supplied.
    #[must_use]
    pub const fn content_type(&self) -> Option<ResponseContentType> {
        self.content_type
    }

    /// Returns validated rate-limit metadata when supplied.
    #[must_use]
    pub const fn rate_limit(&self) -> Option<RateLimit> {
        self.rate_limit
    }

    /// Returns complete bounded response-header metadata.
    #[must_use]
    pub const fn headers(&self) -> &ResponseHeaders {
        &self.headers
    }
}

impl fmt::Debug for TransportResponse<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TransportResponse")
            .field("status", &self.status)
            .field("body_len", &self.body.len())
            .field("body", &"[redacted]")
            .field("content_type", &self.content_type)
            .field("rate_limit", &self.rate_limit)
            .field("headers", &self.headers)
            .finish()
    }
}
