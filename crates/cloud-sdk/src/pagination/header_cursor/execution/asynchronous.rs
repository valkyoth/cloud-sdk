use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};

use crate::authentication::AsyncAuthenticatedTransport;
use crate::buffer::write_u64;
use crate::operation::PreparedExecutionError;
use crate::pagination::{PaginationCursor, PaginationError, PaginationLimits};
use crate::transport::{
    BoundTransport, EndpointIdentity, MAX_REQUEST_HEADERS, RequestHeader, RequestHeaders,
};

use super::{
    HeaderCursorContinuation, HeaderCursorExecutionError, HeaderCursorPage, HeaderCursorSession,
    finish_page,
};

impl HeaderCursorSession<'_, '_> {
    /// Executes the initial executor-neutral async request and decodes its response.
    #[allow(clippy::too_many_arguments)]
    pub async fn execute_async<'response, 'cursor, 'endpoint, T>(
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
        T: AsyncAuthenticatedTransport + BoundTransport,
    {
        sanitize_bytes(decimal_scratch);
        sanitize_bytes(transfer_scratch);
        sanitize_bytes(cursor_destination);
        let endpoint = transport.endpoint_identity().map_err(|error| {
            HeaderCursorExecutionError::Prepared(PreparedExecutionError::EndpointIdentity(error))
        })?;
        execute_async(
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
        .await
    }
}

impl<'cursor, 'endpoint, 'session, 'request, 'policy>
    HeaderCursorContinuation<'cursor, 'endpoint, 'session, 'request, 'policy>
{
    /// Executes the next executor-neutral async request with retained context.
    #[allow(clippy::too_many_arguments)]
    pub async fn execute_async<'response, 'next, T>(
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
        T: AsyncAuthenticatedTransport + BoundTransport,
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
        execute_async(
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
        .await
    }
}

#[allow(clippy::too_many_arguments)]
async fn execute_async<'response, 'cursor, 'endpoint, 'session, 'request, 'policy, T>(
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
    T: AsyncAuthenticatedTransport + BoundTransport,
{
    let response = match cursor {
        Some(cursor) => {
            execute_with_value(
                session,
                Some(cursor.as_bytes()),
                transport,
                response_storage,
                response_header_storage,
                decimal_scratch,
            )
            .await
        }
        None => {
            execute_with_value(
                session,
                None,
                transport,
                response_storage,
                response_header_storage,
                decimal_scratch,
            )
            .await
        }
    }?;
    finish_page(
        session,
        endpoint,
        response,
        transfer_scratch,
        cursor_destination,
        limits,
    )
}

async fn execute_with_value<'response, T>(
    session: &HeaderCursorSession<'_, '_>,
    cursor: Option<&[u8]>,
    transport: &T,
    response_storage: &'response mut [u8],
    response_header_storage: &'response mut [u8],
    decimal_scratch: &mut [u8],
) -> Result<crate::operation::CheckedResponseGuard<'response>, HeaderCursorExecutionError<T::Error>>
where
    T: AsyncAuthenticatedTransport + BoundTransport,
{
    let mut decimal = SecretBuffer::new(decimal_scratch);
    let mut len = 0_usize;
    write_u64(
        decimal.as_mut_slice(),
        &mut len,
        session.policy.page_size(),
        PaginationError::OutputTooSmall,
    )
    .map_err(HeaderCursorExecutionError::Pagination)?;
    let size = core::str::from_utf8(
        decimal
            .as_slice()
            .get(..len)
            .ok_or(PaginationError::OutputTooSmall)
            .map_err(HeaderCursorExecutionError::Pagination)?,
    )
    .map_err(|_| HeaderCursorExecutionError::Pagination(PaginationError::InvalidHeaderState))?;
    let size = RequestHeader::new(session.policy.size_request().as_str(), size)
        .map_err(|_| HeaderCursorExecutionError::Pagination(PaginationError::InvalidHeaderState))?;
    let cursor_header = match cursor {
        Some(value) => {
            let value = core::str::from_utf8(value).map_err(|_| {
                HeaderCursorExecutionError::Pagination(PaginationError::InvalidHeaderState)
            })?;
            Some(
                RequestHeader::sensitive(session.policy.cursor_request().as_str(), value).map_err(
                    |_| HeaderCursorExecutionError::Pagination(PaginationError::InvalidHeaderState),
                )?,
            )
        }
        None => None,
    };
    let pagination_entries = [size, cursor_header.unwrap_or(size)];
    let pagination_len = if cursor_header.is_some() { 2 } else { 1 };
    let pagination = pagination_entries
        .get(..pagination_len)
        .ok_or(PaginationError::InvalidHeaderState)
        .map_err(HeaderCursorExecutionError::Pagination)?;
    let base_headers = session.prepared.transport_request().headers();
    let base = base_headers.as_slice();
    let count = base
        .len()
        .checked_add(pagination.len())
        .ok_or(PaginationError::RequestHeaderConflict)
        .map_err(HeaderCursorExecutionError::Pagination)?;
    if count > MAX_REQUEST_HEADERS {
        return Err(HeaderCursorExecutionError::Pagination(
            PaginationError::RequestHeaderConflict,
        ));
    }
    let mut entries = [size; MAX_REQUEST_HEADERS];
    entries
        .get_mut(..base.len())
        .ok_or(PaginationError::RequestHeaderConflict)
        .map_err(HeaderCursorExecutionError::Pagination)?
        .copy_from_slice(base);
    entries
        .get_mut(base.len()..count)
        .ok_or(PaginationError::RequestHeaderConflict)
        .map_err(HeaderCursorExecutionError::Pagination)?
        .copy_from_slice(pagination);
    let selected = entries
        .get(..count)
        .ok_or(PaginationError::RequestHeaderConflict)
        .map_err(HeaderCursorExecutionError::Pagination)?;
    let headers = RequestHeaders::new(selected).map_err(|_| {
        HeaderCursorExecutionError::Pagination(PaginationError::RequestHeaderConflict)
    })?;
    let prepared = session.prepared.with_request_headers(headers);
    prepared
        .execute_async(transport, response_storage, response_header_storage)
        .await
        .map_err(HeaderCursorExecutionError::Prepared)
}
