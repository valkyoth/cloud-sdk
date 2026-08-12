use cloud_sdk_sanitization::sanitize_bytes;

use super::encoding::encode_with_authorization_evidence;
use super::error::map_infallible;
use super::{
    DigestRollback, MAX_CANONICAL_PLAN_BYTES, PlanConfirmation, PlanFingerprintBuildError,
    PlanFingerprintDigest, validate,
};
use crate::buffer::{SnapshotEncoder, encode_snapshot_bounded};
use crate::operation::PermitTimestamp;
use crate::retry::FingerprintHasher;

const DOMAIN: &[u8] = b"cloud-sdk/authorization-evidence/v1\0";

/// Immutable provider-owned authorization evidence appended only to a digest preimage.
///
/// Implementations must emit the same bounded bytes on every call and must not
/// read clocks, random sources, atomics, or other mutable state. Sensitive
/// values should be exposed only for the duration of [`Self::encode`].
pub trait PlanAuthorizationEvidence {
    /// Returns the exclusive upper bound for authority derived from this evidence.
    ///
    /// `None` means the evidence has no independent time limit. A returned
    /// timestamp must cover the complete permit validity interval.
    fn valid_until(&self) -> Option<PermitTimestamp> {
        None
    }

    /// Encodes a provider-versioned, unambiguous evidence snapshot.
    fn encode<E: Copy>(
        &self,
        writer: &mut SnapshotEncoder<'_, PlanFingerprintBuildError<E>>,
    ) -> Result<(), PlanFingerprintBuildError<E>>;
}

/// Builds a digest over the canonical plan and sensitive authorization evidence.
///
/// Evidence is written only to caller-owned scratch storage, included under a
/// separate domain, hashed with the complete plan, and cleared before return.
pub fn build_plan_digest_with_authorization_evidence<
    'output,
    'plan,
    'request,
    H: FingerprintHasher,
    A: PlanAuthorizationEvidence + ?Sized,
>(
    plan: PlanConfirmation<'plan, 'request>,
    evidence: &A,
    scratch: &mut [u8],
    output: &'output mut [u8],
    hasher: &H,
) -> Result<PlanFingerprintDigest<'output, 'plan, 'request>, PlanFingerprintBuildError<H::Error>> {
    sanitize_bytes(output);
    sanitize_bytes(scratch);
    let mut scratch = SensitiveScratch::new(scratch);
    let mut rollback = DigestRollback::new(output);
    let scope = validate(&plan, true)?;
    if evidence
        .valid_until()
        .is_some_and(|expires_at| plan.validity.expires_at() > expires_at)
    {
        return Err(PlanFingerprintBuildError::AuthorizationEvidenceValidityMismatch);
    }
    let len = encode_snapshot_bounded(
        (plan, evidence),
        scratch.as_mut(),
        MAX_CANONICAL_PLAN_BYTES,
        PlanFingerprintBuildError::InputTooLarge,
        encode_with_evidence::<core::convert::Infallible, A>,
    )
    .map_err(map_infallible)?;
    let algorithm = hasher.algorithm();
    let expected = algorithm.output_len();
    if rollback.len() < expected {
        return Err(PlanFingerprintBuildError::OutputTooSmall);
    }
    let digest_len = hasher
        .digest(scratch.bytes(len), rollback.target(expected))
        .map_err(PlanFingerprintBuildError::Hasher)?;
    if digest_len != expected {
        return Err(PlanFingerprintBuildError::InvalidDigestLength);
    }
    let output = rollback.disarm();
    Ok(PlanFingerprintDigest {
        algorithm,
        storage: output,
        len: digest_len,
        plan,
        scope,
    })
}

fn encode_with_evidence<E: Copy, A: PlanAuthorizationEvidence + ?Sized>(
    (plan, evidence): (PlanConfirmation<'_, '_>, &A),
    writer: &mut SnapshotEncoder<'_, PlanFingerprintBuildError<E>>,
) -> Result<(), PlanFingerprintBuildError<E>> {
    encode_with_authorization_evidence(&plan, writer)?;
    writer.bytes(DOMAIN)?;
    evidence.encode(writer)
}

struct SensitiveScratch<'a>(&'a mut [u8]);

impl<'a> SensitiveScratch<'a> {
    fn new(scratch: &'a mut [u8]) -> Self {
        Self(scratch)
    }

    fn bytes(&self, len: usize) -> &[u8] {
        self.0.get(..len).unwrap_or_default()
    }

    fn as_mut(&mut self) -> &mut [u8] {
        self.0
    }
}

impl Drop for SensitiveScratch<'_> {
    fn drop(&mut self) {
        sanitize_bytes(self.0);
    }
}
