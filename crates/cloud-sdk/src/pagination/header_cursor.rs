use core::fmt;

use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};

use super::{PaginationCursor, PaginationError, PaginationLimits};
use crate::buffer::write_u64;
use crate::operation::OperationId;
use crate::transport::{
    HeaderName, HeaderSensitivity, RequestHeader, RequestHeaders, ResponseHeaders,
};

mod execution;
pub use execution::{
    HeaderCursorContinuation, HeaderCursorExecutionError, HeaderCursorNext, HeaderCursorPage,
    HeaderCursorSession,
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

    pub(super) fn with_request_headers<R>(
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

    pub(super) fn decode_next<'storage>(
        self,
        headers: &ResponseHeaders<'_>,
        transfer_scratch: &mut [u8],
        destination: &'storage mut [u8],
        limits: PaginationLimits,
    ) -> Result<DecodedHeaderCursor<'storage>, PaginationError> {
        sanitize_bytes(transfer_scratch);
        sanitize_bytes(destination);
        let Some(header) = headers.get(self.next_response.as_str()) else {
            return Ok(DecodedHeaderCursor::Complete);
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
        PaginationCursor::transfer_from(source, destination, limits)
            .map(DecodedHeaderCursor::Continue)
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

pub(super) enum DecodedHeaderCursor<'storage> {
    Complete,
    Continue(PaginationCursor<'storage>),
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
