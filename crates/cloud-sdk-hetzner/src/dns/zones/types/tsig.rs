//! Hardened TSIG request values.

use core::fmt;

use cloud_sdk::buffer;

use super::ZoneRequestError;

/// Maximum TSIG key bytes admitted by this SDK boundary.
pub const MAX_TSIG_KEY_BYTES: usize = 4096;
/// Minimum decoded TSIG secret size for HMAC-SHA256.
pub const MIN_TSIG_SECRET_BYTES: usize = 32;

/// Hardened TSIG algorithm policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TsigAlgorithm {
    /// Hardened HMAC-SHA256 mode.
    HmacSha256,
}

impl TsigAlgorithm {
    /// Returns the API value.
    #[must_use]
    pub const fn as_api_str(self) -> &'static str {
        match self {
            Self::HmacSha256 => "hmac-sha256",
        }
    }
}

/// Base64-encoded TSIG key. Debug output is redacted and ordinary equality is
/// intentionally unavailable.
///
/// ```compile_fail
/// use cloud_sdk_hetzner::dns::zones::TsigKey;
/// fn insecure_compare(left: TsigKey<'_>, right: TsigKey<'_>) -> bool {
///     left == right
/// }
/// ```
#[derive(Clone, Copy)]
pub struct TsigKey<'a>(&'a str);

impl<'a> TsigKey<'a> {
    /// Validates canonical padded standard Base64 containing at least 32 bytes.
    ///
    /// This validates representation and size, not entropy. Generate secrets
    /// with a CSPRNG, keep each secret between two entities, and rotate it.
    pub fn new(value: &'a str) -> Result<Self, ZoneRequestError> {
        let bytes = value.as_bytes();
        let padding = value
            .len()
            .saturating_sub(value.trim_end_matches('=').len());
        let data_len = bytes.len().saturating_sub(padding);
        let invalid_data = match bytes.get(..data_len) {
            Some(data) => data
                .iter()
                .any(|byte| !(byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/'))),
            None => true,
        };
        if value.is_empty()
            || value.len() > MAX_TSIG_KEY_BYTES
            || !value.len().is_multiple_of(4)
            || padding > 2
            || invalid_data
            || !has_canonical_base64_padding(bytes, data_len, padding)
            || decoded_base64_len(value.len(), padding)
                .is_none_or(|length| length < MIN_TSIG_SECRET_BYTES)
        {
            return Err(ZoneRequestError::InvalidTsigKey);
        }
        Ok(Self(value))
    }

    /// Writes the complete JSON string without exposing a raw accessor.
    ///
    /// The caller owns `output` and must securely erase it after transport use.
    pub fn write_json_string(self, output: &mut [u8]) -> Result<usize, ZoneRequestError> {
        let mut len = 0;
        buffer::write_json_string(
            output,
            &mut len,
            self.0,
            ZoneRequestError::BodyBufferTooSmall,
        )?;
        Ok(len)
    }
}

impl fmt::Debug for TsigKey<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TsigKey([redacted])")
    }
}

/// Coherent TSIG credentials without ordinary equality.
///
/// ```compile_fail
/// use cloud_sdk_hetzner::dns::zones::TsigCredentials;
/// fn insecure_compare(left: TsigCredentials<'_>, right: TsigCredentials<'_>) -> bool {
///     left == right
/// }
/// ```
#[derive(Clone, Copy)]
pub struct TsigCredentials<'a> {
    key: TsigKey<'a>,
    algorithm: TsigAlgorithm,
}

impl<'a> TsigCredentials<'a> {
    /// Creates credentials with an explicit key and algorithm.
    #[must_use]
    pub const fn new(key: TsigKey<'a>, algorithm: TsigAlgorithm) -> Self {
        Self { key, algorithm }
    }

    /// Returns the secret key marker.
    #[must_use]
    pub const fn key(self) -> TsigKey<'a> {
        self.key
    }

    /// Returns the algorithm.
    #[must_use]
    pub const fn algorithm(self) -> TsigAlgorithm {
        self.algorithm
    }
}

impl fmt::Debug for TsigCredentials<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TsigCredentials")
            .field("key", &"[redacted]")
            .field("algorithm", &self.algorithm)
            .finish()
    }
}

fn decoded_base64_len(encoded_len: usize, padding: usize) -> Option<usize> {
    encoded_len
        .checked_div(4)?
        .checked_mul(3)?
        .checked_sub(padding)
}

fn has_canonical_base64_padding(bytes: &[u8], data_len: usize, padding: usize) -> bool {
    if padding == 0 {
        return true;
    }
    let last = data_len
        .checked_sub(1)
        .and_then(|index| bytes.get(index))
        .and_then(|byte| base64_sextet(*byte));
    match (padding, last) {
        (1, Some(value)) => value & 0b0000_0011 == 0,
        (2, Some(value)) => value & 0b0000_1111 == 0,
        _ => false,
    }
}

fn base64_sextet(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => byte.checked_sub(b'A'),
        b'a'..=b'z' => byte.checked_sub(b'a')?.checked_add(26),
        b'0'..=b'9' => byte.checked_sub(b'0')?.checked_add(52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}
