//! Explicit retry ownership, replay identity, and bounded decision policy.

mod fingerprint;
mod idempotency;
mod policy;
mod time;

pub use fingerprint::{
    CanonicalFingerprint, DigestAlgorithm, FingerprintBuildError, FingerprintDigest,
    FingerprintHasher, FingerprintRef, FingerprintScope, MAX_FINGERPRINT_DIGEST_BYTES,
    MAX_FINGERPRINT_SCOPE_BYTES, RetrySubject, build_canonical_fingerprint,
    build_fingerprint_digest,
};
pub use idempotency::{
    IdempotencyBinding, IdempotencyIntent, IdempotencyIntentError, MAX_IDEMPOTENCY_INTENT_BYTES,
    MIN_IDEMPOTENCY_INTENT_BYTES,
};
pub use policy::{
    MaxAttempts, MaxAttemptsError, RetryController, RetryDecision, RetryEvent, RetryPermit,
    RetryPermitError, RetryPolicy, RetryPolicyError, RetryStopReason,
};
pub use time::{MonotonicDuration, MonotonicInstant};

#[cfg(test)]
mod tests;
