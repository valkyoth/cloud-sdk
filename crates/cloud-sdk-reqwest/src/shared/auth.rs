use core::fmt;

use cloud_sdk_sanitization::sanitize_bytes;
use reqwest::header::HeaderValue;
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

/// Owned bearer authorization value with redacted diagnostics and volatile
/// cleanup of the adapter-owned bytes.
pub struct BearerToken {
    authorization: Vec<u8>,
}

impl BearerToken {
    /// Validates and copies a bearer token into adapter-owned secret storage.
    pub fn new(token: &str) -> Result<Self, BearerTokenError> {
        if token.is_empty() {
            return Err(BearerTokenError::Empty);
        }
        if token.len() > MAX_BEARER_TOKEN_BYTES {
            return Err(BearerTokenError::TooLong);
        }
        let mut padding = false;
        for byte in token.bytes() {
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
        authorization.extend_from_slice(token.as_bytes());
        Ok(Self { authorization })
    }

    pub(crate) fn header_value(&self) -> Result<HeaderValue, ()> {
        let mut value = HeaderValue::from_bytes(&self.authorization).map_err(|_| ())?;
        value.set_sensitive(true);
        Ok(value)
    }

    #[cfg(all(
        test,
        any(
            feature = "blocking-rustls",
            feature = "blocking-rustls-webpki-roots",
            feature = "blocking-rustls-fips"
        )
    ))]
    pub(crate) fn owned_bytes(&self) -> &[u8] {
        &self.authorization
    }
}

impl Drop for BearerToken {
    fn drop(&mut self) {
        sanitize_bytes(&mut self.authorization);
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
