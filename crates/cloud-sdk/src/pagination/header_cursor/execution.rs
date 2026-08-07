use core::fmt;

use cloud_sdk_sanitization::sanitize_bytes;

use crate::authentication::BlockingAuthenticatedTransport;
use crate::operation::{CheckedResponseGuard, PreparedExecutionError, PreparedRequest};
use crate::transport::{
    BoundTransport, EndpointIdentity, MAX_REQUEST_HEADERS, RequestHeader, RequestHeaders,
};

use super::{DecodedHeaderCursor, HeaderCursorPolicy};
use crate::pagination::{
    CursorDigest, CursorHistory, PaginationCursor, PaginationError, PaginationLimits,
};

mod asynchronous;
mod local_async;

/// Cursor validation or prepared-request execution failure.
pub enum HeaderCursorExecutionError<E> {
    /// Cursor policy, provenance, storage, or header validation failed.
    Pagination(PaginationError),
    /// The exact retained prepared request failed to execute.
    Prepared(PreparedExecutionError<E>),
}

impl<E> fmt::Debug for HeaderCursorExecutionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pagination(error) => formatter.debug_tuple("Pagination").field(error).finish(),
            Self::Prepared(_) => formatter.write_str("Prepared([redacted])"),
        }
    }
}

impl<E> fmt::Display for HeaderCursorExecutionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Pagination(_) => "header cursor validation failed",
            Self::Prepared(_) => "header cursor prepared request failed",
        })
    }
}

impl<E> core::error::Error for HeaderCursorExecutionError<E> {}

/// Exact prepared-request context retained for one header-cursor traversal.
#[derive(Clone, Copy)]
pub struct HeaderCursorSession<'request, 'policy> {
    pub(super) policy: HeaderCursorPolicy<'policy>,
    pub(super) prepared: PreparedRequest<'request>,
}

impl<'policy> HeaderCursorPolicy<'policy> {
    /// Binds pagination to one complete prepared request before any dispatch.
    pub fn bind<'request>(
        self,
        prepared: PreparedRequest<'request>,
    ) -> Result<HeaderCursorSession<'request, 'policy>, PaginationError> {
        if prepared.operation_id() != Some(self.operation_id()) {
            return Err(PaginationError::OperationMismatch);
        }
        Ok(HeaderCursorSession {
            policy: self,
            prepared,
        })
    }
}

impl HeaderCursorSession<'_, '_> {
    /// Returns the operation retained by this exact request context.
    #[must_use]
    pub const fn operation_id(&self) -> crate::operation::OperationId {
        self.policy.operation_id()
    }

    /// Executes the initial blocking request and decodes only its own response.
    #[allow(clippy::too_many_arguments)]
    pub fn execute_blocking<'response, 'cursor, 'endpoint, T>(
        &self,
        transport: &'endpoint T,
        response_storage: &'response mut [u8],
        response_header_storage: &'response mut [u8],
        decimal_scratch: &mut [u8],
        transfer_scratch: &mut [u8],
        cursor_destination: &'cursor mut [u8],
        limits: PaginationLimits,
    ) -> Result<
        HeaderCursorPage<'response, 'cursor, 'endpoint, '_, '_, '_>,
        HeaderCursorExecutionError<T::Error>,
    >
    where
        T: BlockingAuthenticatedTransport + BoundTransport,
    {
        sanitize_bytes(decimal_scratch);
        sanitize_bytes(transfer_scratch);
        sanitize_bytes(cursor_destination);
        let endpoint = transport.endpoint_identity().map_err(|error| {
            HeaderCursorExecutionError::Prepared(PreparedExecutionError::EndpointIdentity(error))
        })?;
        execute_blocking(
            self,
            None,
            endpoint,
            transport,
            response_storage,
            response_header_storage,
            decimal_scratch,
            transfer_scratch,
            cursor_destination,
            limits,
        )
    }
}

impl fmt::Debug for HeaderCursorSession<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HeaderCursorSession")
            .field("policy", &self.policy)
            .field("prepared", &"[bound]")
            .finish()
    }
}

/// Checked response and provenance-bound next-page state.
pub struct HeaderCursorPage<'response, 'cursor, 'endpoint, 'session, 'request, 'policy> {
    response: CheckedResponseGuard<'response>,
    next: HeaderCursorNext<'cursor, 'endpoint, 'session, 'request, 'policy>,
}

impl<'response, 'cursor, 'endpoint, 'session, 'request, 'policy>
    HeaderCursorPage<'response, 'cursor, 'endpoint, 'session, 'request, 'policy>
{
    /// Separates the checked response from its next-page state.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        CheckedResponseGuard<'response>,
        HeaderCursorNext<'cursor, 'endpoint, 'session, 'request, 'policy>,
    ) {
        (self.response, self.next)
    }
}

impl fmt::Debug for HeaderCursorPage<'_, '_, '_, '_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HeaderCursorPage")
            .field("response", &self.response)
            .field("next", &self.next)
            .finish()
    }
}

/// Provenance-bound continuation state from one executed prepared request.
pub enum HeaderCursorNext<'cursor, 'endpoint, 'session, 'request, 'policy> {
    /// The next-cursor header was absent, so traversal is complete.
    Complete,
    /// The exact request context and bounded cursor are retained together.
    Continue(HeaderCursorContinuation<'cursor, 'endpoint, 'session, 'request, 'policy>),
}

impl HeaderCursorNext<'_, '_, '_, '_, '_> {
    /// Reports whether the provider declared a terminal page.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        matches!(self, Self::Complete)
    }
}

impl fmt::Debug for HeaderCursorNext<'_, '_, '_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Complete => formatter.write_str("HeaderCursorNext::Complete"),
            Self::Continue(_) => formatter.write_str("HeaderCursorNext::Continue([redacted])"),
        }
    }
}

/// Cleanup-owning cursor inseparable from its original prepared request.
///
/// Continuations expose execution only; another request cannot be attached.
///
/// ```compile_fail
/// use cloud_sdk::operation::PreparedRequest;
/// use cloud_sdk::pagination::HeaderCursorContinuation;
///
/// fn rebind(
///     continuation: HeaderCursorContinuation<'_, '_, '_, '_, '_>,
///     replacement: PreparedRequest<'_>,
/// ) {
///     continuation.bind(replacement);
/// }
/// ```
pub struct HeaderCursorContinuation<'cursor, 'endpoint, 'session, 'request, 'policy> {
    pub(super) session: &'session HeaderCursorSession<'request, 'policy>,
    pub(super) cursor: PaginationCursor<'cursor>,
    pub(super) endpoint: EndpointIdentity<'endpoint>,
}

impl<'cursor, 'endpoint, 'session, 'request, 'policy>
    HeaderCursorContinuation<'cursor, 'endpoint, 'session, 'request, 'policy>
{
    /// Returns the provider operation that produced this continuation.
    #[must_use]
    pub const fn operation_id(&self) -> crate::operation::OperationId {
        self.session.operation_id()
    }

    /// Checks and transactionally records the exact cursor in caller history.
    pub fn observe_history(
        &self,
        history: &mut CursorHistory<'_>,
        digest: CursorDigest,
    ) -> Result<(), PaginationError> {
        history.observe(&self.cursor, digest)
    }

    /// Executes the next blocking request using the retained request context.
    #[allow(clippy::too_many_arguments)]
    pub fn execute_blocking<'response, 'next, T>(
        &self,
        transport: &T,
        response_storage: &'response mut [u8],
        response_header_storage: &'response mut [u8],
        decimal_scratch: &mut [u8],
        transfer_scratch: &mut [u8],
        cursor_destination: &'next mut [u8],
        limits: PaginationLimits,
    ) -> Result<
        HeaderCursorPage<'response, 'next, 'endpoint, 'session, 'request, 'policy>,
        HeaderCursorExecutionError<T::Error>,
    >
    where
        T: BlockingAuthenticatedTransport + BoundTransport,
    {
        sanitize_bytes(decimal_scratch);
        sanitize_bytes(transfer_scratch);
        sanitize_bytes(cursor_destination);
        let endpoint = transport.endpoint_identity().map_err(|error| {
            HeaderCursorExecutionError::Prepared(PreparedExecutionError::EndpointIdentity(error))
        })?;
        if endpoint != self.endpoint {
            return Err(HeaderCursorExecutionError::Pagination(
                PaginationError::EndpointMismatch,
            ));
        }
        execute_blocking(
            self.session,
            Some(&self.cursor),
            self.endpoint,
            transport,
            response_storage,
            response_header_storage,
            decimal_scratch,
            transfer_scratch,
            cursor_destination,
            limits,
        )
    }
}

impl fmt::Debug for HeaderCursorContinuation<'_, '_, '_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HeaderCursorContinuation")
            .field("operation", &self.operation_id())
            .field("request", &"[bound]")
            .field("cursor", &"[redacted]")
            .finish()
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_blocking<'response, 'cursor, 'endpoint, 'session, 'request, 'policy, T>(
    session: &'session HeaderCursorSession<'request, 'policy>,
    cursor: Option<&PaginationCursor<'_>>,
    endpoint: EndpointIdentity<'endpoint>,
    transport: &T,
    response_storage: &'response mut [u8],
    response_header_storage: &'response mut [u8],
    decimal_scratch: &mut [u8],
    transfer_scratch: &mut [u8],
    cursor_destination: &'cursor mut [u8],
    limits: PaginationLimits,
) -> Result<
    HeaderCursorPage<'response, 'cursor, 'endpoint, 'session, 'request, 'policy>,
    HeaderCursorExecutionError<T::Error>,
>
where
    T: BlockingAuthenticatedTransport + BoundTransport,
{
    let response = session
        .policy
        .with_request_headers(cursor, decimal_scratch, |pagination| {
            with_merged_request(&session.prepared, pagination, |prepared| {
                prepared.execute_blocking(transport, response_storage, response_header_storage)
            })
        })
        .map_err(HeaderCursorExecutionError::Pagination)?
        .map_err(HeaderCursorExecutionError::Pagination)?
        .map_err(HeaderCursorExecutionError::Prepared)?;
    finish_page(
        session,
        endpoint,
        response,
        transfer_scratch,
        cursor_destination,
        limits,
    )
}

pub(super) fn finish_page<'response, 'cursor, 'endpoint, 'session, 'request, 'policy, E>(
    session: &'session HeaderCursorSession<'request, 'policy>,
    endpoint: EndpointIdentity<'endpoint>,
    response: CheckedResponseGuard<'response>,
    transfer_scratch: &mut [u8],
    cursor_destination: &'cursor mut [u8],
    limits: PaginationLimits,
) -> Result<
    HeaderCursorPage<'response, 'cursor, 'endpoint, 'session, 'request, 'policy>,
    HeaderCursorExecutionError<E>,
> {
    let next = session
        .policy
        .decode_next(
            response.response_headers(),
            transfer_scratch,
            cursor_destination,
            limits,
        )
        .map_err(HeaderCursorExecutionError::Pagination)?;
    let next = match next {
        DecodedHeaderCursor::Complete => HeaderCursorNext::Complete,
        DecodedHeaderCursor::Continue(cursor) => {
            HeaderCursorNext::Continue(HeaderCursorContinuation {
                session,
                cursor,
                endpoint,
            })
        }
    };
    Ok(HeaderCursorPage { response, next })
}

pub(super) fn with_merged_request<'request, R>(
    prepared: &PreparedRequest<'request>,
    pagination: RequestHeaders<'_>,
    inspect: impl FnOnce(PreparedRequest<'_>) -> R,
) -> Result<R, PaginationError> {
    let base = prepared.transport_request().headers().as_slice();
    let extra = pagination.as_slice();
    let count = base
        .len()
        .checked_add(extra.len())
        .ok_or(PaginationError::RequestHeaderConflict)?;
    if count > MAX_REQUEST_HEADERS {
        return Err(PaginationError::RequestHeaderConflict);
    }
    let first = extra
        .first()
        .copied()
        .ok_or(PaginationError::InvalidHeaderState)?;
    let mut entries: [RequestHeader<'_>; MAX_REQUEST_HEADERS] = [first; MAX_REQUEST_HEADERS];
    entries
        .get_mut(..base.len())
        .ok_or(PaginationError::RequestHeaderConflict)?
        .copy_from_slice(base);
    entries
        .get_mut(base.len()..count)
        .ok_or(PaginationError::RequestHeaderConflict)?
        .copy_from_slice(extra);
    let selected = entries
        .get(..count)
        .ok_or(PaginationError::RequestHeaderConflict)?;
    let headers =
        RequestHeaders::new(selected).map_err(|_| PaginationError::RequestHeaderConflict)?;
    Ok(inspect((*prepared).with_request_headers(headers)))
}
