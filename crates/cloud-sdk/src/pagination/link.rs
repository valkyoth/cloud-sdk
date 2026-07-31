use core::fmt;

use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes, sanitize_value};

use crate::Method;
use crate::operation::OperationId;
use crate::transport::{
    EndpointIdentity, EndpointScheme, RequestPath, RequestTarget, TransportRequest,
};

use super::{PaginationError, PaginationLimits};

/// Immutable operation boundary used to validate provider pagination links.
#[derive(Clone, Copy)]
pub struct ProviderLinkBinding<'a> {
    endpoint: EndpointIdentity<'a>,
    method: Method,
    operation: OperationId,
    path: RequestPath<'a>,
}

impl<'a> ProviderLinkBinding<'a> {
    /// Binds links to one endpoint, method, operation, and exact path pattern.
    #[must_use]
    pub const fn new(
        endpoint: EndpointIdentity<'a>,
        method: Method,
        operation: OperationId,
        path: RequestPath<'a>,
    ) -> Self {
        Self {
            endpoint,
            method,
            operation,
            path,
        }
    }
}

impl fmt::Debug for ProviderLinkBinding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProviderLinkBinding")
            .field("endpoint", &self.endpoint)
            .field("method", &self.method)
            .field("operation", &self.operation)
            .field("path", &"[redacted]")
            .finish()
    }
}

/// Cleanup-owning operation-bound provider pagination link.
///
/// Exact validated origin-form path/query bytes are copied without decoding,
/// re-encoding, reordering, or conversion into a structured query. This type
/// is intentionally neither `Copy` nor `Clone`.
///
/// ```compile_fail
/// use cloud_sdk::pagination::ValidatedProviderLink;
/// fn require_copy<T: Copy>() {}
/// require_copy::<ValidatedProviderLink<'static>>();
/// ```
///
/// ```compile_fail
/// use cloud_sdk::pagination::ValidatedProviderLink;
/// use cloud_sdk::transport::RequestQuery;
///
/// fn into_structured<'a>(link: ValidatedProviderLink<'a>) -> RequestQuery<'a> {
///     link.into()
/// }
/// ```
pub struct ValidatedProviderLink<'storage> {
    target: SecretBuffer<'storage>,
    target_len: usize,
    method: Method,
    operation: OperationId,
}

impl<'storage> ValidatedProviderLink<'storage> {
    /// Atomically validates and transfers an absolute or origin-form link.
    ///
    /// Source and complete destination storage are cleared on every failure;
    /// source is also cleared after successful transfer. Absolute links must
    /// match the exact bound scheme and authority. The resulting target must
    /// match the exact operation path.
    pub fn transfer_from(
        source: &mut [u8],
        destination: &'storage mut [u8],
        binding: ProviderLinkBinding<'_>,
        limits: PaginationLimits,
    ) -> Result<Self, PaginationError> {
        sanitize_bytes(destination);
        let source = SecretBuffer::new(source);
        let mut destination = SecretBuffer::new(destination);
        if source.as_slice().is_empty() {
            return Err(PaginationError::MissingState);
        }
        if source.as_slice().len() > limits.max_state_bytes() {
            return Err(PaginationError::StateTooLong);
        }
        let link = core::str::from_utf8(source.as_slice())
            .map_err(|_| PaginationError::InvalidProviderLink)?;
        let raw_target = extract_target(link, binding.endpoint)?;
        let target = RequestTarget::from_provider_link(raw_target)
            .map_err(|_| PaginationError::InvalidProviderLink)?;
        if target.path() != binding.path {
            return Err(PaginationError::ProviderLinkPathChanged);
        }
        let output = destination
            .as_mut_slice()
            .get_mut(..raw_target.len())
            .ok_or(PaginationError::OutputTooSmall)?;
        output.copy_from_slice(raw_target.as_bytes());
        Ok(Self {
            target: destination,
            target_len: raw_target.len(),
            method: binding.method,
            operation: binding.operation,
        })
    }

    /// Runs a closure with a request only when method and operation match.
    ///
    /// The request target borrows cleanup-owning link storage and cannot
    /// outlive this call.
    pub fn with_request<R>(
        &self,
        method: Method,
        operation: OperationId,
        inspect: impl FnOnce(TransportRequest<'_>) -> R,
    ) -> Result<R, PaginationError> {
        if method != self.method {
            return Err(PaginationError::ProviderLinkMethodChanged);
        }
        if operation != self.operation {
            return Err(PaginationError::ProviderLinkOperationChanged);
        }
        let bytes = self
            .target
            .as_slice()
            .get(..self.target_len)
            .ok_or(PaginationError::InvalidProviderLink)?;
        let value =
            core::str::from_utf8(bytes).map_err(|_| PaginationError::InvalidProviderLink)?;
        let target = RequestTarget::from_provider_link(value)
            .map_err(|_| PaginationError::InvalidProviderLink)?;
        Ok(inspect(TransportRequest::new(method, target)))
    }
}

impl Drop for ValidatedProviderLink<'_> {
    fn drop(&mut self) {
        sanitize_value(&mut self.target_len);
    }
}

impl fmt::Debug for ValidatedProviderLink<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ValidatedProviderLink")
            .field("target", &"[redacted]")
            .field("method", &self.method)
            .field("operation", &self.operation)
            .finish()
    }
}

fn extract_target<'a>(
    link: &'a str,
    endpoint: EndpointIdentity<'_>,
) -> Result<&'a str, PaginationError> {
    if link.contains('#') {
        return Err(PaginationError::ProviderLinkFragment);
    }
    if link.starts_with('/') {
        return Ok(link);
    }
    let (scheme, remainder) = if let Some(value) = link.strip_prefix("https://") {
        (EndpointScheme::Https, value)
    } else if let Some(value) = link.strip_prefix("http://") {
        (EndpointScheme::Http, value)
    } else {
        return Err(PaginationError::InvalidProviderLink);
    };
    if scheme != endpoint.scheme() {
        return Err(PaginationError::ProviderLinkSchemeChanged);
    }
    let path_start = remainder
        .find('/')
        .ok_or(PaginationError::InvalidProviderLink)?;
    let authority = remainder
        .get(..path_start)
        .ok_or(PaginationError::InvalidProviderLink)?;
    if authority.contains('@') {
        return Err(PaginationError::ProviderLinkUserinfo);
    }
    validate_authority(authority, endpoint)?;
    remainder
        .get(path_start..)
        .ok_or(PaginationError::InvalidProviderLink)
}

fn validate_authority(
    authority: &str,
    endpoint: EndpointIdentity<'_>,
) -> Result<(), PaginationError> {
    let (host, port) = split_authority(authority, endpoint.scheme())?;
    let candidate = EndpointIdentity::new(endpoint.scheme(), host, port, endpoint.base_path())
        .map_err(|_| PaginationError::ProviderLinkAuthorityChanged)?;
    if candidate != endpoint {
        return Err(PaginationError::ProviderLinkAuthorityChanged);
    }
    Ok(())
}

fn split_authority(
    authority: &str,
    scheme: EndpointScheme,
) -> Result<(&str, u16), PaginationError> {
    let default_port = match scheme {
        EndpointScheme::Http => 80,
        EndpointScheme::Https => 443,
    };
    if authority.starts_with('[') {
        let close = authority
            .find(']')
            .ok_or(PaginationError::ProviderLinkAuthorityChanged)?;
        let host_end = close
            .checked_add(1)
            .ok_or(PaginationError::ProviderLinkAuthorityChanged)?;
        let host = authority
            .get(..host_end)
            .ok_or(PaginationError::ProviderLinkAuthorityChanged)?;
        let suffix = authority
            .get(host_end..)
            .ok_or(PaginationError::ProviderLinkAuthorityChanged)?;
        let port = if suffix.is_empty() {
            default_port
        } else {
            parse_port(
                suffix
                    .strip_prefix(':')
                    .ok_or(PaginationError::ProviderLinkAuthorityChanged)?,
            )?
        };
        return Ok((host, port));
    }
    if let Some((host, port)) = authority.rsplit_once(':') {
        if host.contains(':') {
            return Err(PaginationError::ProviderLinkAuthorityChanged);
        }
        Ok((host, parse_port(port)?))
    } else {
        Ok((authority, default_port))
    }
}

fn parse_port(value: &str) -> Result<u16, PaginationError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(PaginationError::ProviderLinkAuthorityChanged);
    }
    value
        .parse::<u16>()
        .ok()
        .filter(|port| *port != 0)
        .ok_or(PaginationError::ProviderLinkAuthorityChanged)
}
