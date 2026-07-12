//! Uploaded certificate private-key request value.

use cloud_sdk::buffer;

use crate::security::shared::{PemValue, SecurityRequestError};

/// Uploaded certificate private key PEM value.
///
/// Raw access and ordinary equality are intentionally unavailable. Use
/// [`Self::write_json_string`] for serialization.
///
/// ```compile_fail
/// use cloud_sdk_hetzner::security::certificates::private_key_pem;
///
/// let left = private_key_pem("-----BEGIN PRIVATE KEY-----\nAA==\n-----END PRIVATE KEY-----")?;
/// let right = left;
/// let _ = left == right;
/// # Ok::<(), cloud_sdk_hetzner::security::SecurityRequestError>(())
/// ```
#[derive(Clone, Copy)]
pub struct PrivateKeyPem<'a> {
    value: PemValue<'a>,
}

impl PrivateKeyPem<'_> {
    /// Writes the private key as one complete escaped JSON string.
    ///
    /// # Security
    ///
    /// The destination contains private-key material after success. Guard the
    /// full buffer with `cloud_sdk_sanitization::SecretBuffer` so it is
    /// volatile-cleared after transport use. An undersized buffer is unchanged.
    pub fn write_json_string(self, output: &mut [u8]) -> Result<usize, SecurityRequestError> {
        let mut len = 0;
        buffer::write_json_string(
            output,
            &mut len,
            self.value.as_str(),
            SecurityRequestError::BodyBufferTooSmall,
        )?;
        Ok(len)
    }
}

impl core::fmt::Debug for PrivateKeyPem<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("PrivateKeyPem([redacted])")
    }
}

/// Creates a validated private key PEM value.
pub fn private_key_pem(value: &str) -> Result<PrivateKeyPem<'_>, SecurityRequestError> {
    if value.contains("-----BEGIN PRIVATE KEY-----") {
        return PemValue::new(
            value,
            "-----BEGIN PRIVATE KEY-----",
            "-----END PRIVATE KEY-----",
        )
        .map(|value| PrivateKeyPem { value });
    }
    PemValue::new(
        value,
        "-----BEGIN RSA PRIVATE KEY-----",
        "-----END RSA PRIVATE KEY-----",
    )
    .map(|value| PrivateKeyPem { value })
}
