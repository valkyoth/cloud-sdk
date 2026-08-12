use super::fingerprint_tests::{TestEvidence, TestHasher, plan, time};
use super::fixture::{endpoint, prepared};
use crate::operation::{
    CostIntent, ExecutionPermitError, MutationPermit, OperationImpact, ReplayPolicy,
    build_plan_digest_with_authorization_evidence,
};

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
