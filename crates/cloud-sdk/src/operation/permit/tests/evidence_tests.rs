#[cfg(feature = "std")]
use crate::std as test_std;

use super::fingerprint_tests::{TestEvidence, TestHasher, plan, time};
use super::fixture::{endpoint, prepared};
use crate::operation::{
    CostIntent, ExecutionPermitError, MutationPermit, OperationImpact, PlanAuthorizationEvidence,
    PlanFingerprintBuildError, ReplayPolicy, build_plan_digest_with_authorization_evidence,
};
#[cfg(feature = "std")]
use crate::retry::{DigestAlgorithm, FingerprintHasher};

#[test]
fn authorization_evidence_changes_digest_and_clears_scratch() {
    let Some(request) = prepared(
        "/resources",
        OperationImpact::Mutation,
        CostIntent::NoKnownCost,
    ) else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(endpoint) = endpoint() else {
        unreachable!("permit security fixture construction failed")
    };
    let Some(plan) = plan(
        request,
        endpoint,
        b"evidence-test",
        200,
        ReplayPolicy::SingleAttempt,
    ) else {
        unreachable!("permit security fixture construction failed");
    };
    let mut first_scratch = [0xa5_u8; 4_096];
    let mut first_digest = [0_u8; 32];
    let mut second_scratch = [0xa5_u8; 4_096];
    let mut second_digest = [0_u8; 32];
    {
        let first_fingerprint = build_plan_digest_with_authorization_evidence(
            plan,
            &TestEvidence(b"evidence-a"),
            &mut first_scratch,
            &mut first_digest,
            &TestHasher,
        )
        .unwrap_or_else(|_| unreachable!("evidence digest failed"));
        let second_fingerprint = build_plan_digest_with_authorization_evidence(
            plan,
            &TestEvidence(b"evidence-b"),
            &mut second_scratch,
            &mut second_digest,
            &TestHasher,
        )
        .unwrap_or_else(|_| unreachable!("evidence digest failed"));
        let mut permit = MutationPermit::new(first_fingerprint.subject(), time(100))
            .unwrap_or_else(|_| unreachable!("evidence permit failed"));
        assert!(matches!(
            permit.begin_for(second_fingerprint.subject(), time(101)),
            Err(ExecutionPermitError::FingerprintMismatch)
        ));
    }
    assert_eq!(first_scratch, [0_u8; 4_096]);
    assert_eq!(second_scratch, [0_u8; 4_096]);
}

#[test]
fn evidence_validity_must_cover_the_complete_permit_lifetime() {
    let Some(request) = prepared(
        "/resources",
        OperationImpact::Mutation,
        CostIntent::NoKnownCost,
    ) else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(endpoint) = endpoint() else {
        unreachable!("permit security fixture construction failed")
    };
    let Some(plan) = plan(
        request,
        endpoint,
        b"evidence-validity-test",
        200,
        ReplayPolicy::SingleAttempt,
    ) else {
        unreachable!("permit security fixture construction failed");
    };
    let evidence = ExpiringEvidence(time(150));
    let mut scratch = [0xa5_u8; 4_096];
    let mut digest = [0x5a_u8; 32];
    assert!(matches!(
        build_plan_digest_with_authorization_evidence(
            plan,
            &evidence,
            &mut scratch,
            &mut digest,
            &TestHasher,
        ),
        Err(PlanFingerprintBuildError::AuthorizationEvidenceValidityMismatch)
    ));
    assert_eq!(scratch, [0_u8; 4_096]);
    assert_eq!(digest, [0_u8; 32]);
}

#[cfg(feature = "std")]
#[test]
fn every_evidence_digest_callback_panic_clears_all_storage() {
    let Some(request) = prepared(
        "/resources",
        OperationImpact::Mutation,
        CostIntent::NoKnownCost,
    ) else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(endpoint) = endpoint() else {
        unreachable!("permit security fixture construction failed")
    };
    let Some(plan) = plan(
        request,
        endpoint,
        b"evidence-panic-test",
        200,
        ReplayPolicy::SingleAttempt,
    ) else {
        unreachable!("permit security fixture construction failed");
    };

    for phase in [
        PanicPhase::Encode,
        PanicPhase::Algorithm,
        PanicPhase::Digest,
    ] {
        let mut scratch = [0xa5_u8; 4_096];
        let mut digest = [0x5a_u8; 64];
        let panic =
            test_std::panic::catch_unwind(test_std::panic::AssertUnwindSafe(|| match phase {
                PanicPhase::Encode => {
                    let _ = build_plan_digest_with_authorization_evidence(
                        plan,
                        &PanickingEvidence,
                        &mut scratch,
                        &mut digest,
                        &TestHasher,
                    );
                }
                PanicPhase::Algorithm => {
                    let _ = build_plan_digest_with_authorization_evidence(
                        plan,
                        &TestEvidence(b"sensitive-evidence"),
                        &mut scratch,
                        &mut digest,
                        &AlgorithmPanickingHasher,
                    );
                }
                PanicPhase::Digest => {
                    let _ = build_plan_digest_with_authorization_evidence(
                        plan,
                        &TestEvidence(b"sensitive-evidence"),
                        &mut scratch,
                        &mut digest,
                        &DigestPanickingHasher,
                    );
                }
            }));
        assert!(panic.is_err());
        assert_eq!(scratch, [0_u8; 4_096]);
        assert_eq!(digest, [0_u8; 64]);
    }
}

struct ExpiringEvidence(crate::operation::PermitTimestamp);

impl PlanAuthorizationEvidence for ExpiringEvidence {
    fn valid_until(&self) -> Option<crate::operation::PermitTimestamp> {
        Some(self.0)
    }

    fn encode<E: Copy>(
        &self,
        writer: &mut crate::buffer::SnapshotEncoder<'_, PlanFingerprintBuildError<E>>,
    ) -> Result<(), PlanFingerprintBuildError<E>> {
        writer.bytes(b"expiring-evidence")
    }
}

#[cfg(feature = "std")]
#[derive(Clone, Copy)]
enum PanicPhase {
    Encode,
    Algorithm,
    Digest,
}

#[cfg(feature = "std")]
struct PanickingEvidence;

#[cfg(feature = "std")]
impl PlanAuthorizationEvidence for PanickingEvidence {
    fn encode<E: Copy>(
        &self,
        writer: &mut crate::buffer::SnapshotEncoder<'_, PlanFingerprintBuildError<E>>,
    ) -> Result<(), PlanFingerprintBuildError<E>> {
        writer.bytes(b"sensitive-evidence")?;
        test_std::panic::resume_unwind(test_std::boxed::Box::new("test evidence panic"))
    }
}

#[cfg(feature = "std")]
struct AlgorithmPanickingHasher;

#[cfg(feature = "std")]
impl FingerprintHasher for AlgorithmPanickingHasher {
    type Error = core::convert::Infallible;

    fn algorithm(&self) -> DigestAlgorithm {
        test_std::panic::resume_unwind(test_std::boxed::Box::new("test algorithm panic"))
    }

    fn digest(&self, _input: &[u8], _output: &mut [u8]) -> Result<usize, Self::Error> {
        unreachable!("algorithm panic must stop digest")
    }
}

#[cfg(feature = "std")]
struct DigestPanickingHasher;

#[cfg(feature = "std")]
impl FingerprintHasher for DigestPanickingHasher {
    type Error = core::convert::Infallible;

    fn algorithm(&self) -> DigestAlgorithm {
        DigestAlgorithm::Sha256
    }

    fn digest(&self, _input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        output.fill(0x5a);
        test_std::panic::resume_unwind(test_std::boxed::Box::new("test digest panic"))
    }
}
