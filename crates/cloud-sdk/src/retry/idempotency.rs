//! Fresh caller-entropy intent identifiers for one retry owner.

use core::fmt;

use cloud_sdk_sanitization::sanitize_bytes;
use subtle::{Choice, ConstantTimeEq};

use super::fingerprint::FingerprintRef;

/// Minimum entropy bytes admitted for one fresh operation intent.
pub const MIN_IDEMPOTENCY_INTENT_BYTES: usize = 16;
/// Maximum intent bytes retained by the borrowed retry contract.
pub const MAX_IDEMPOTENCY_INTENT_BYTES: usize = 64;

/// Invalid caller-provided idempotency intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdempotencyIntentError {
    /// The identifier does not meet the minimum entropy-bearing length.
    TooShort,
    /// The identifier exceeds the bounded intent length.
    TooLong,
    /// An all-zero identifier cannot represent caller-provided entropy.
    AllZero,
}

impl_static_error!(IdempotencyIntentError,
    Self::TooShort => "idempotency intent is too short",
    Self::TooLong => "idempotency intent is too long",
    Self::AllZero => "idempotency intent cannot be all zero",
);

/// One-use fresh intent identifier supplied by a caller CSPRNG.
///
/// This type is intentionally neither `Copy` nor `Clone`. Construction checks
/// shape and retains exclusive access to caller storage until drop. Invalid
/// sources are cleared immediately. Entropy quality and global uniqueness
/// remain caller duties.
///
/// ```compile_fail
/// use cloud_sdk::retry::IdempotencyIntent;
///
/// let mut entropy = [7_u8; 32];
/// let intent = IdempotencyIntent::new(&mut entropy).unwrap();
/// let _duplicate = intent.clone();
/// ```
pub struct IdempotencyIntent<'secret> {
    bytes: &'secret mut [u8],
}

impl<'secret> IdempotencyIntent<'secret> {
    /// Borrows fresh bytes exclusively and clears invalid sources immediately.
    pub fn new(source: &'secret mut [u8]) -> Result<Self, IdempotencyIntentError> {
        if let Err(error) = validate_source(source) {
            sanitize_bytes(source);
            return Err(error);
        }
        Ok(Self { bytes: source })
    }

    /// Returns the identifier length without exposing entropy bytes.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Reports whether the identifier is empty. A valid intent is never empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }
}

impl fmt::Debug for IdempotencyIntent<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("IdempotencyIntent([redacted])")
    }
}

impl Drop for IdempotencyIntent<'_> {
    fn drop(&mut self) {
        sanitize_bytes(self.bytes);
    }
}

fn validate_source(source: &[u8]) -> Result<(), IdempotencyIntentError> {
    if source.len() < MIN_IDEMPOTENCY_INTENT_BYTES {
        return Err(IdempotencyIntentError::TooShort);
    }
    if source.len() > MAX_IDEMPOTENCY_INTENT_BYTES {
        return Err(IdempotencyIntentError::TooLong);
    }
    let mut any_nonzero = Choice::from(0);
    for byte in source {
        any_nonzero |= !byte.ct_eq(&0);
    }
    if !bool::from(any_nonzero) {
        return Err(IdempotencyIntentError::AllZero);
    }
    Ok(())
}

/// One-use local idempotency identity bound to one exact request fingerprint.
///
/// The binding does not claim that a provider accepts an idempotency header.
/// It prevents this retry owner from applying one intent to different request
/// bytes. Provider retry eligibility remains source-locked operation policy.
pub struct IdempotencyBinding<'a> {
    intent: IdempotencyIntent<'a>,
    fingerprint: FingerprintRef<'a>,
}

impl<'a> IdempotencyBinding<'a> {
    /// Consumes a fresh intent and binds it to one request fingerprint.
    #[must_use]
    pub const fn bind(intent: IdempotencyIntent<'a>, fingerprint: FingerprintRef<'a>) -> Self {
        Self {
            intent,
            fingerprint,
        }
    }

    /// Returns the bounded intent length without exposing entropy bytes.
    #[must_use]
    pub const fn intent_len(&self) -> usize {
        self.intent.len()
    }

    pub(crate) fn matches(&self, fingerprint: FingerprintRef<'_>) -> bool {
        self.fingerprint.matches(fingerprint)
    }
}

impl fmt::Debug for IdempotencyBinding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("IdempotencyBinding")
            .field("intent_len", &self.intent.len())
            .field("fingerprint", &"[redacted]")
            .finish()
    }
}
