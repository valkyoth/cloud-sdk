//! Fresh caller-entropy intent identifiers for one retry owner.

use core::fmt;

use cloud_sdk_sanitization::sanitize_bytes;

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
/// shape, moves bytes into fixed storage, and clears the mutable source on
/// every path. Entropy quality and global uniqueness remain caller duties.
///
/// ```compile_fail
/// use cloud_sdk::retry::IdempotencyIntent;
///
/// let mut entropy = [7_u8; 32];
/// let intent = IdempotencyIntent::new(&mut entropy).unwrap();
/// let _duplicate = intent.clone();
/// ```
pub struct IdempotencyIntent {
    bytes: [u8; MAX_IDEMPOTENCY_INTENT_BYTES],
    len: usize,
}

impl IdempotencyIntent {
    /// Moves fresh bytes into owned storage and clears `source` on every path.
    pub fn new(source: &mut [u8]) -> Result<Self, IdempotencyIntentError> {
        let validation = validate_source(source);
        let mut intent = Self {
            bytes: [0_u8; MAX_IDEMPOTENCY_INTENT_BYTES],
            len: 0,
        };
        let result = validation.and_then(|()| {
            let destination = intent
                .bytes
                .get_mut(..source.len())
                .ok_or(IdempotencyIntentError::TooLong)?;
            destination.copy_from_slice(source);
            intent.len = source.len();
            Ok(intent)
        });
        sanitize_bytes(source);
        result
    }

    /// Returns the identifier length without exposing entropy bytes.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Reports whether the identifier is empty. A valid intent is never empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }
}

impl fmt::Debug for IdempotencyIntent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("IdempotencyIntent([redacted])")
    }
}

impl Drop for IdempotencyIntent {
    fn drop(&mut self) {
        sanitize_bytes(&mut self.bytes);
        self.len = 0;
    }
}

fn validate_source(source: &[u8]) -> Result<(), IdempotencyIntentError> {
    if source.len() < MIN_IDEMPOTENCY_INTENT_BYTES {
        return Err(IdempotencyIntentError::TooShort);
    }
    if source.len() > MAX_IDEMPOTENCY_INTENT_BYTES {
        return Err(IdempotencyIntentError::TooLong);
    }
    if source.iter().all(|byte| *byte == 0) {
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
    intent: IdempotencyIntent,
    fingerprint: FingerprintRef<'a>,
}

impl<'a> IdempotencyBinding<'a> {
    /// Consumes a fresh intent and binds it to one request fingerprint.
    #[must_use]
    pub const fn bind(intent: IdempotencyIntent, fingerprint: FingerprintRef<'a>) -> Self {
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
