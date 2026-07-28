//! Cleanup-owning response identifiers retained beyond a response guard.

use core::fmt;

use cloud_sdk_sanitization::{sanitize_bytes, sanitize_value};

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

pub(crate) struct ProtectedRequestId {
    bytes: [u8; MAX_REQUEST_ID_BYTES],
    len: usize,
}

impl ProtectedRequestId {
    pub(crate) fn take_from(source: &mut [u8]) -> Result<Self, RetainedMetadataError> {
        let mut request_id = Self::empty();
        let result = if source.is_empty() {
            Err(RetainedMetadataError::EmptyRequestId)
        } else if source.len() > MAX_REQUEST_ID_BYTES {
            Err(RetainedMetadataError::RequestIdTooLong)
        } else {
            match request_id.bytes.get_mut(..source.len()) {
                Some(target) => {
                    target.copy_from_slice(source);
                    request_id.len = source.len();
                    Ok(request_id)
                }
                None => Err(RetainedMetadataError::RequestIdTooLong),
            }
        };
        sanitize_bytes(source);
        result
    }

    const fn empty() -> Self {
        Self {
            bytes: [0; MAX_REQUEST_ID_BYTES],
            len: 0,
        }
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.bytes.get(..self.len).unwrap_or_default()
    }

    pub(crate) fn clear(&mut self) {
        sanitize_bytes(&mut self.bytes);
        sanitize_value(&mut self.len);
    }

    pub(crate) fn transfer(
        &mut self,
        retention_limit: usize,
    ) -> Result<RetainedResponseMetadata, RetainedMetadataError> {
        let mut retained = RetainedResponseMetadata::empty_for_core();
        if self.len > retention_limit {
            self.clear();
            return Err(RetainedMetadataError::RetentionLimitExceeded);
        }
        let Some(target) = retained.request_id.get_mut(..self.len) else {
            self.clear();
            return Err(RetainedMetadataError::RequestIdTooLong);
        };
        target.copy_from_slice(self.as_bytes());
        retained.request_id_len = self.len;
        self.clear();
        Ok(retained)
    }
}

impl Drop for ProtectedRequestId {
    fn drop(&mut self) {
        self.clear();
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
/// closure-scoped and the complete fixed-capacity storage is volatile-cleared
/// on drop.
///
/// ```compile_fail
/// use cloud_sdk::transport::RetainedResponseMetadata;
///
/// fn require_copy<T: Copy>() {}
/// require_copy::<RetainedResponseMetadata>();
/// ```
pub struct RetainedResponseMetadata {
    request_id: [u8; MAX_REQUEST_ID_BYTES],
    request_id_len: usize,
}

impl RetainedResponseMetadata {
    pub(crate) const fn empty_for_core() -> Self {
        Self {
            request_id: [0; MAX_REQUEST_ID_BYTES],
            request_id_len: 0,
        }
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
                .get(..self.request_id_len)
                .unwrap_or_default()
        });
        inspect(value)
    }
}

impl Drop for RetainedResponseMetadata {
    fn drop(&mut self) {
        sanitize_bytes(&mut self.request_id);
        sanitize_value(&mut self.request_id_len);
    }
}

impl fmt::Debug for RetainedResponseMetadata {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RetainedResponseMetadata")
            .field("request_id", &"[redacted]")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::{ProtectedRequestId, RetainedMetadataError};

    #[test]
    fn source_clears_after_successful_and_rejected_admission() {
        let mut accepted = *b"request-id";
        let protected = ProtectedRequestId::take_from(&mut accepted);
        assert!(protected.is_ok());
        assert_eq!(accepted, [0; 10]);

        let mut rejected = [0_u8; 0];
        assert!(matches!(
            ProtectedRequestId::take_from(&mut rejected),
            Err(RetainedMetadataError::EmptyRequestId)
        ));
    }

    #[test]
    fn retained_transfer_clears_source_on_success_and_failure() {
        let mut success_source = *b"request-id";
        let mut success =
            ProtectedRequestId::take_from(&mut success_source).unwrap_or_else(|_| unreachable!());
        let retained = success.transfer(10);
        assert!(retained.is_ok());
        assert!(success.bytes.iter().all(|byte| *byte == 0));
        if let Ok(retained) = retained {
            assert!(retained.with_request_id(|value| value == Some(b"request-id".as_slice())));
        }

        let mut failed_source = *b"request-id";
        let mut failed =
            ProtectedRequestId::take_from(&mut failed_source).unwrap_or_else(|_| unreachable!());
        assert!(matches!(
            failed.transfer(3),
            Err(RetainedMetadataError::RetentionLimitExceeded)
        ));
        assert!(failed.bytes.iter().all(|byte| *byte == 0));
    }
}
