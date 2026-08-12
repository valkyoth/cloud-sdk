use core::cell::Cell;

use super::fingerprint_tests::{TestEvidence, plan};
use super::fixture::{endpoint, prepared};
use crate::operation::{
    CostIntent, OperationImpact, ReplayPolicy, build_plan_digest,
    build_plan_digest_with_authorization_evidence,
};
use crate::retry::{DigestAlgorithm, FingerprintHasher};

#[test]
fn ordinary_plan_preserves_the_v1_golden_vector() {
    let Some(plan) = fixture(false) else {
        unreachable!("permit version fixture construction failed");
    };
    let hasher = GoldenHasher::new();
    let mut scratch = [0xa5_u8; 4_096];
    let mut output = [0xa5_u8; 32];
    let fingerprint = build_plan_digest(plan, &mut scratch, &mut output, &hasher);
    assert!(fingerprint.is_ok());
    drop(fingerprint);
    assert_eq!(hasher.input_len(), 475);
    assert_eq!(
        hasher.digest(),
        [
            16, 161, 30, 70, 27, 144, 27, 173, 120, 106, 8, 132, 119, 183, 88, 85, 228, 239, 218,
            14, 75, 55, 143, 64, 106, 33, 95, 62, 164, 229, 210, 12,
        ]
    );
    assert_eq!(output, [0_u8; 32]);
}

#[test]
fn evidence_required_plan_uses_the_v2_golden_vector() {
    let Some(plan) = fixture(true) else {
        unreachable!("permit version fixture construction failed");
    };
    let hasher = GoldenHasher::new();
    let mut scratch = [0xa5_u8; 4_096];
    let mut output = [0xa5_u8; 32];
    let fingerprint = build_plan_digest_with_authorization_evidence(
        plan,
        &TestEvidence(b"golden-authorization-evidence"),
        &mut scratch,
        &mut output,
        &hasher,
    );
    assert!(fingerprint.is_ok());
    drop(fingerprint);
    assert_eq!(hasher.input_len(), 550);
    assert_eq!(
        hasher.digest(),
        [
            132, 234, 83, 43, 19, 216, 255, 104, 101, 229, 205, 170, 38, 4, 12, 94, 20, 49, 225,
            145, 184, 116, 37, 80, 94, 238, 103, 147, 232, 185, 70, 134,
        ]
    );
    assert_eq!(output, [0_u8; 32]);
}

fn fixture(
    evidence_required: bool,
) -> Option<crate::operation::PlanConfirmation<'static, 'static>> {
    let request = prepared(
        "/resources?label=one",
        OperationImpact::Mutation,
        CostIntent::NoKnownCost,
    )?;
    let request = if evidence_required {
        request.with_required_authorization_evidence()
    } else {
        request
    };
    plan(
        request,
        endpoint()?,
        b"canonical-version-golden",
        200,
        ReplayPolicy::SingleAttempt,
    )
}

struct GoldenHasher {
    input_len: Cell<usize>,
    digest: Cell<[u8; 32]>,
}

impl GoldenHasher {
    const fn new() -> Self {
        Self {
            input_len: Cell::new(0),
            digest: Cell::new([0_u8; 32]),
        }
    }

    fn input_len(&self) -> usize {
        self.input_len.get()
    }

    fn digest(&self) -> [u8; 32] {
        self.digest.get()
    }
}

impl FingerprintHasher for GoldenHasher {
    type Error = core::convert::Infallible;

    fn algorithm(&self) -> DigestAlgorithm {
        DigestAlgorithm::Sha256
    }

    fn digest(&self, input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        self.input_len.set(input.len());
        let Some(digest) = crate::test_sha256::sha256(input) else {
            return Ok(0);
        };
        self.digest.set(digest);
        output.copy_from_slice(&digest);
        Ok(digest.len())
    }
}
