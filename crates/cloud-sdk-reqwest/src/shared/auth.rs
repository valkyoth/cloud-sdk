use core::fmt;

use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};
use reqwest::header::HeaderValue;
#[cfg(test)]
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::vec::Vec;

/// Maximum bearer-token length accepted by the adapter.
pub const MAX_BEARER_TOKEN_BYTES: usize = 4096;

const BEARER_PREFIX: &[u8] = b"Bearer ";

/// Bearer-token validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BearerTokenError {
    /// Tokens must not be empty.
    Empty,
    /// Tokens exceed [`MAX_BEARER_TOKEN_BYTES`].
    TooLong,
    /// Tokens contain bytes outside the RFC bearer-token alphabet or invalid
    /// non-trailing padding.
    InvalidByte,
    /// Allocation failed while taking owned secret storage.
    AllocationFailed,
}

impl_static_error!(BearerTokenError,
    Self::Empty => "bearer token is empty",
    Self::TooLong => "bearer token exceeds the length limit",
    Self::InvalidByte => "bearer token contains an invalid byte",
    Self::AllocationFailed => "bearer-token allocation failed",
);

/// Owned bearer authorization value with redacted diagnostics and volatile
/// cleanup of the adapter-owned bytes.
pub struct BearerToken {
    authorization: Vec<u8>,
    #[cfg(test)]
    drop_probe: Option<Arc<AtomicUsize>>,
}

impl BearerToken {
    /// Validates and copies a bearer token into adapter-owned secret storage.
    pub fn new(token: &str) -> Result<Self, BearerTokenError> {
        Self::from_bytes(token.as_bytes())
    }

    /// Consumes a mutable bearer-token source and clears the complete source
    /// buffer on success or failure.
    pub fn from_mut_bytes(token: &mut [u8]) -> Result<Self, BearerTokenError> {
        let result = Self::from_bytes(token);
        sanitize_bytes(token);
        result
    }

    /// Consumes guarded bearer-token storage, which clears its complete source
    /// buffer when this function returns on success or failure.
    pub fn from_secret_buffer(token: SecretBuffer<'_>) -> Result<Self, BearerTokenError> {
        Self::from_bytes(token.as_slice())
    }

    fn from_bytes(token: &[u8]) -> Result<Self, BearerTokenError> {
        if token.is_empty() {
            return Err(BearerTokenError::Empty);
        }
        if token.len() > MAX_BEARER_TOKEN_BYTES {
            return Err(BearerTokenError::TooLong);
        }
        let mut padding = false;
        for byte in token.iter().copied() {
            if byte == b'=' {
                padding = true;
            } else if padding || !is_bearer_byte(byte) {
                return Err(BearerTokenError::InvalidByte);
            }
        }

        let capacity = BEARER_PREFIX
            .len()
            .checked_add(token.len())
            .ok_or(BearerTokenError::TooLong)?;
        let mut authorization = Vec::new();
        authorization
            .try_reserve_exact(capacity)
            .map_err(|_| BearerTokenError::AllocationFailed)?;
        authorization.extend_from_slice(BEARER_PREFIX);
        authorization.extend_from_slice(token);
        Ok(Self {
            authorization,
            #[cfg(test)]
            drop_probe: None,
        })
    }

    pub(crate) fn header_value(&self) -> Result<HeaderValue, ()> {
        let mut value = HeaderValue::from_bytes(&self.authorization).map_err(|_| ())?;
        value.set_sensitive(true);
        Ok(value)
    }

    #[cfg(test)]
    pub(crate) fn owned_bytes(&self) -> &[u8] {
        &self.authorization
    }

    #[cfg(test)]
    pub(crate) fn with_drop_probe(
        token: &str,
        probe: Arc<AtomicUsize>,
    ) -> Result<Self, BearerTokenError> {
        let mut value = Self::new(token)?;
        value.drop_probe = Some(probe);
        Ok(value)
    }
}

impl Drop for BearerToken {
    fn drop(&mut self) {
        sanitize_bytes(&mut self.authorization);
        #[cfg(test)]
        if let Some(probe) = &self.drop_probe {
            probe.fetch_add(1, Ordering::SeqCst);
        }
    }
}

impl fmt::Debug for BearerToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BearerToken([redacted])")
    }
}

const fn is_bearer_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'+' | b'/')
}
