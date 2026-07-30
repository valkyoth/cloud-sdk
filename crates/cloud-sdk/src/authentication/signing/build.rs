use core::fmt;

use cloud_sdk_sanitization::sanitize_bytes;

use super::{MAX_SIGNING_BODY_DIGEST_BYTES, SigningInputError, SigningValueError};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct SigningBodyDigest<'a>(&'a [u8]);

impl<'a> SigningBodyDigest<'a> {
    pub(super) fn new(value: &'a [u8]) -> Result<Self, SigningValueError> {
        if value.is_empty() {
            return Err(SigningValueError::Empty);
        }
        if value.len() > MAX_SIGNING_BODY_DIGEST_BYTES {
            return Err(SigningValueError::TooLong);
        }
        Ok(Self(value))
    }

    pub(super) const fn as_bytes(self) -> &'a [u8] {
        self.0
    }
}

impl fmt::Debug for SigningBodyDigest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SigningBodyDigest([redacted])")
    }
}

/// Body hashing and canonical signing-input construction failure.
pub enum SigningBuildError<E> {
    /// The hasher and signed context declare different digest algorithms.
    DigestAlgorithmMismatch,
    /// The caller-provided body hasher rejected the exact request body.
    Hasher(E),
    /// The hasher returned an empty or out-of-bounds digest.
    Digest(SigningValueError),
    /// Canonical input validation or encoding failed.
    Input(SigningInputError),
}

impl<E> fmt::Debug for SigningBuildError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::DigestAlgorithmMismatch => "SigningBuildError::DigestAlgorithmMismatch",
            Self::Hasher(_) => "SigningBuildError::Hasher([redacted])",
            Self::Digest(_) => "SigningBuildError::Digest",
            Self::Input(_) => "SigningBuildError::Input",
        })
    }
}

impl<E> fmt::Display for SigningBuildError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::DigestAlgorithmMismatch => {
                "request body hasher does not match the signed digest algorithm"
            }
            Self::Hasher(_) => "request body hashing failed",
            Self::Digest(_) => "request body hasher returned an invalid digest length",
            Self::Input(_) => "canonical signing input construction failed",
        })
    }
}

impl<E> core::error::Error for SigningBuildError<E>
where
    E: core::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Hasher(error) => Some(error),
            Self::Digest(error) => Some(error),
            Self::Input(error) => Some(error),
            Self::DigestAlgorithmMismatch => None,
        }
    }
}

pub(super) struct DigestScratch<'a>(&'a mut [u8]);

impl<'a> DigestScratch<'a> {
    pub(super) fn new(storage: &'a mut [u8]) -> Self {
        sanitize_bytes(storage);
        Self(storage)
    }

    pub(super) fn as_mut(&mut self) -> &mut [u8] {
        self.0
    }

    pub(super) fn as_slice(&self) -> &[u8] {
        self.0
    }
}

impl Drop for DigestScratch<'_> {
    fn drop(&mut self) {
        sanitize_bytes(self.0);
    }
}
