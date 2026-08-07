use core::fmt;

use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};

use super::{CursorDigest, CursorHistory, PaginationCursor, PaginationError, PaginationLimits};
use crate::buffer::write_u64;
use crate::operation::OperationId;
use crate::transport::{
    HeaderName, HeaderSensitivity, RequestHeader, RequestHeaders, ResponseHeaders,
};

/// Source-bound header names and page size for opaque cursor pagination.
#[derive(Clone, Copy)]
pub struct HeaderCursorPolicy<'a> {
    operation: OperationId,
    cursor_request: HeaderName<'a>,
    size_request: HeaderName<'a>,
    next_response: HeaderName<'a>,
    page_size: u64,
}

impl<'a> HeaderCursorPolicy<'a> {
    /// Creates a policy with three distinct HTTP header names and a nonzero size.
    pub fn new(
        operation: OperationId,
        cursor_request: &'a str,
        size_request: &'a str,
        next_response: &'a str,
        page_size: u64,
    ) -> Result<Self, PaginationError> {
        let cursor_request =
            HeaderName::new(cursor_request).map_err(|_| PaginationError::InvalidHeaderPolicy)?;
        let size_request =
            HeaderName::new(size_request).map_err(|_| PaginationError::InvalidHeaderPolicy)?;
        let next_response =
            HeaderName::new(next_response).map_err(|_| PaginationError::InvalidHeaderPolicy)?;
        RequestHeader::sensitive(cursor_request.as_str(), "cursor")
            .map_err(|_| PaginationError::InvalidHeaderPolicy)?;
        RequestHeader::new(size_request.as_str(), "1")
            .map_err(|_| PaginationError::InvalidHeaderPolicy)?;
        if page_size == 0 {
            return Err(PaginationError::PageSizeZero);
        }
        if cursor_request == size_request
            || cursor_request == next_response
            || size_request == next_response
        {
            return Err(PaginationError::InvalidHeaderPolicy);
        }
        Ok(Self {
            operation,
            cursor_request,
            size_request,
            next_response,
            page_size,
        })
    }

    /// Returns the provider operation that owns every decoded cursor.
    #[must_use]
    pub const fn operation_id(self) -> OperationId {
        self.operation
    }

    /// Returns the request cursor header name.
    #[must_use]
    pub const fn cursor_request(self) -> HeaderName<'a> {
        self.cursor_request
    }

    /// Returns the request page-size header name.
    #[must_use]
    pub const fn size_request(self) -> HeaderName<'a> {
        self.size_request
    }

    /// Returns the response next-cursor header name.
    #[must_use]
    pub const fn next_response(self) -> HeaderName<'a> {
        self.next_response
    }

    /// Returns the fixed page size for this traversal.
    #[must_use]
    pub const fn page_size(self) -> u64 {
        self.page_size
    }

    /// Builds the exact request headers only for the duration of `inspect`.
    ///
    /// A continuation cursor is always marked sensitive. The decimal scratch
    /// buffer is cleared on success, failure, unwind, and return.
    pub fn with_initial_request_headers<R>(
        self,
        decimal_scratch: &mut [u8],
        inspect: impl FnOnce(RequestHeaders<'_>) -> R,
    ) -> Result<R, PaginationError> {
        self.with_request_headers(None, decimal_scratch, inspect)
    }

    fn with_request_headers<R>(
        self,
        cursor: Option<&PaginationCursor<'_>>,
        decimal_scratch: &mut [u8],
        inspect: impl FnOnce(RequestHeaders<'_>) -> R,
    ) -> Result<R, PaginationError> {
        let mut scratch = SecretBuffer::new(decimal_scratch);
        let mut len = 0_usize;
        write_u64(
            scratch.as_mut_slice(),
            &mut len,
            self.page_size,
            PaginationError::OutputTooSmall,
        )?;
        let size = core::str::from_utf8(
            scratch
                .as_slice()
                .get(..len)
                .ok_or(PaginationError::OutputTooSmall)?,
        )
        .map_err(|_| PaginationError::InvalidHeaderState)?;
        match cursor {
            None => self.inspect_request_headers(size, None, inspect),
            Some(cursor) => cursor.with_cursor(|value| {
                let value =
                    core::str::from_utf8(value).map_err(|_| PaginationError::InvalidHeaderState)?;
                self.inspect_request_headers(size, Some(value), inspect)
            }),
        }
    }

    /// Decodes an optional next cursor from retained raw response metadata.
    ///
    /// Absence is the terminal-page signal. A present value must have been
    /// retained as sensitive metadata and must be valid for an HTTP request
    /// header. Scratch and destination are cleared on every path.
    pub fn decode_next<'storage>(
        self,
        headers: &ResponseHeaders<'_>,
        transfer_scratch: &mut [u8],
        destination: &'storage mut [u8],
        limits: PaginationLimits,
    ) -> Result<HeaderCursorNext<'storage, 'a>, PaginationError> {
        sanitize_bytes(transfer_scratch);
        sanitize_bytes(destination);
        let Some(header) = headers.get(self.next_response.as_str()) else {
            return Ok(HeaderCursorNext::Complete);
        };
        if header.sensitivity() != HeaderSensitivity::Sensitive {
            return Err(PaginationError::InsecureHeaderState);
        }
        let value = header.value();
        validate_cursor_value(value, limits)?;
        let source = transfer_scratch
            .get_mut(..value.len())
            .ok_or(PaginationError::OutputTooSmall)?;
        source.copy_from_slice(value);
        PaginationCursor::transfer_from(source, destination, limits).map(|cursor| {
            HeaderCursorNext::Continue(HeaderCursorContinuation {
                policy: self,
                cursor,
            })
        })
    }

    fn inspect_request_headers<R>(
        self,
        size: &str,
        cursor: Option<&str>,
        inspect: impl FnOnce(RequestHeaders<'_>) -> R,
    ) -> Result<R, PaginationError> {
        let size = RequestHeader::new(self.size_request.as_str(), size)
            .map_err(|_| PaginationError::InvalidHeaderState)?;
        if let Some(cursor) = cursor {
            let cursor = RequestHeader::sensitive(self.cursor_request.as_str(), cursor)
                .map_err(|_| PaginationError::InvalidHeaderState)?;
            let entries = [size, cursor];
            let headers =
                RequestHeaders::new(&entries).map_err(|_| PaginationError::InvalidHeaderPolicy)?;
            return Ok(inspect(headers));
        }
        let entries = [size];
        let headers =
            RequestHeaders::new(&entries).map_err(|_| PaginationError::InvalidHeaderPolicy)?;
        Ok(inspect(headers))
    }
}

impl fmt::Debug for HeaderCursorPolicy<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HeaderCursorPolicy")
            .field("operation", &self.operation)
            .field("cursor_request", &self.cursor_request)
            .field("size_request", &self.size_request)
            .field("next_response", &self.next_response)
            .field("page_size", &self.page_size)
            .finish()
    }
}

/// Decoded continuation state from a cursor response header.
pub enum HeaderCursorNext<'storage, 'policy> {
    /// The next-cursor header was absent, so traversal is complete.
    Complete,
    /// A bounded cleanup-owning cursor is available for the next request.
    Continue(HeaderCursorContinuation<'storage, 'policy>),
}

impl HeaderCursorNext<'_, '_> {
    /// Reports whether the provider declared a terminal page.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        matches!(self, Self::Complete)
    }
}

impl fmt::Debug for HeaderCursorNext<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Complete => formatter.write_str("HeaderCursorNext::Complete"),
            Self::Continue(_) => formatter.write_str("HeaderCursorNext::Continue([redacted])"),
        }
    }
}

/// Operation-bound cleanup-owning header cursor for one next request.
pub struct HeaderCursorContinuation<'storage, 'policy> {
    policy: HeaderCursorPolicy<'policy>,
    cursor: PaginationCursor<'storage>,
}

impl HeaderCursorContinuation<'_, '_> {
    /// Returns the provider operation that produced this continuation.
    #[must_use]
    pub const fn operation_id(&self) -> OperationId {
        self.policy.operation
    }

    /// Checks and transactionally records the exact cursor in caller history.
    pub fn observe_history(
        &self,
        history: &mut CursorHistory<'_>,
        digest: CursorDigest,
    ) -> Result<(), PaginationError> {
        history.observe(&self.cursor, digest)
    }

    /// Emits the fixed page size and this operation-bound sensitive cursor.
    pub fn with_request_headers<R>(
        &self,
        decimal_scratch: &mut [u8],
        inspect: impl FnOnce(RequestHeaders<'_>) -> R,
    ) -> Result<R, PaginationError> {
        self.policy
            .with_request_headers(Some(&self.cursor), decimal_scratch, inspect)
    }
}

impl fmt::Debug for HeaderCursorContinuation<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HeaderCursorContinuation")
            .field("operation", &self.policy.operation)
            .field("cursor", &"[redacted]")
            .finish()
    }
}

fn validate_cursor_value(value: &[u8], limits: PaginationLimits) -> Result<(), PaginationError> {
    if value.is_empty() {
        return Err(PaginationError::MissingState);
    }
    if value.len() > limits.max_state_bytes() {
        return Err(PaginationError::StateTooLong);
    }
    if !value.iter().all(|byte| (b' '..=b'~').contains(byte))
        || value.first() == Some(&b' ')
        || value.last() == Some(&b' ')
    {
        return Err(PaginationError::InvalidHeaderState);
    }
    Ok(())
}
