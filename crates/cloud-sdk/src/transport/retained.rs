//! Cleanup-owning response identifiers retained beyond a response guard.

use core::fmt;

use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes, sanitize_value};

/// Maximum opaque provider request-identifier length.
pub const MAX_REQUEST_ID_BYTES: usize = 1024;

/// Request-identifier admission or transfer failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetainedMetadataError {
    /// A present request identifier was empty.
    EmptyRequestId,
    /// The request identifier exceeds the SDK hard limit.
    RequestIdTooLong,
    /// The caller's retention limit is smaller than the identifier.
    RetentionLimitExceeded,
    /// Operation policy does not permit retention beyond the response guard.
    RetentionForbidden,
}

impl_static_error!(RetainedMetadataError,
    Self::EmptyRequestId => "response request identifier is empty",
    Self::RequestIdTooLong => "response request identifier exceeds the hard limit",
    Self::RetentionLimitExceeded => "response request identifier exceeds the retention limit",
    Self::RetentionForbidden => "operation policy forbids retaining the request identifier",
);

#[derive(Clone, Copy)]
pub(crate) struct ProtectedRequestId {
    start: u16,
    len: u16,
}

impl ProtectedRequestId {
    pub(crate) fn new(start: u16, len: u16) -> Result<Self, RetainedMetadataError> {
        let len_usize = usize::from(len);
        if len_usize == 0 {
            return Err(RetainedMetadataError::EmptyRequestId);
        }
        if len_usize > MAX_REQUEST_ID_BYTES {
            return Err(RetainedMetadataError::RequestIdTooLong);
        }
        Ok(Self { start, len })
    }

    pub(crate) const fn start(self) -> u16 {
        self.start
    }

    pub(crate) const fn len(self) -> u16 {
        self.len
    }
}

impl fmt::Debug for ProtectedRequestId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ProtectedRequestId([redacted])")
    }
}

/// Request identifier deliberately retained beyond a checked response guard.
///
/// This type is intentionally neither `Copy` nor `Clone`. Access is
/// closure-scoped and the complete caller-owned destination is
/// volatile-cleared on drop.
///
/// ```compile_fail
/// use cloud_sdk::transport::RetainedResponseMetadata;
///
/// fn require_copy<T: Copy>() {}
/// require_copy::<RetainedResponseMetadata<'static>>();
/// ```
pub struct RetainedResponseMetadata<'storage> {
    request_id: SecretBuffer<'storage>,
    request_id_len: usize,
}

impl<'storage> RetainedResponseMetadata<'storage> {
    pub(crate) fn empty_for_core(storage: &'storage mut [u8]) -> Self {
        sanitize_bytes(storage);
        Self {
            request_id: SecretBuffer::new(storage),
            request_id_len: 0,
        }
    }

    pub(crate) fn write_request_id(&mut self, source: &[u8]) -> Result<(), RetainedMetadataError> {
        let target = self
            .request_id
            .as_mut_slice()
            .get_mut(..source.len())
            .ok_or(RetainedMetadataError::RetentionLimitExceeded)?;
        target.copy_from_slice(source);
        self.request_id_len = source.len();
        Ok(())
    }

    /// Reports whether a request identifier was retained.
    #[must_use]
    pub const fn has_request_id(&self) -> bool {
        self.request_id_len != 0
    }

    /// Runs a closure with the retained opaque request-identifier bytes.
    pub fn with_request_id<R>(&self, inspect: impl FnOnce(Option<&[u8]>) -> R) -> R {
        let value = self.has_request_id().then(|| {
            self.request_id
                .as_slice()
                .get(..self.request_id_len)
                .unwrap_or_default()
        });
        inspect(value)
    }
}

impl Drop for RetainedResponseMetadata<'_> {
    fn drop(&mut self) {
        sanitize_value(&mut self.request_id_len);
    }
}

impl fmt::Debug for RetainedResponseMetadata<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RetainedResponseMetadata")
            .field("request_id", &"[redacted]")
            .finish()
    }
}
