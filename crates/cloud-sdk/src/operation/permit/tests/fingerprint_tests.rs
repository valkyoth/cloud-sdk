use super::fixture::{endpoint, prepared, read_only};
use crate::operation::{
    AttemptBudget, CostIntent, CurrencyCode, OperationImpact, PermitContext, PermitIdempotencyKey,
    PermitTimestamp, PermitValidity, PlanChange, PlanConfirmation, PlanCost,
    PlanFingerprintBuildError, PlanFingerprintScope, ReplayPolicy, build_canonical_plan,
    build_plan_digest,
};
use crate::retry::{DigestAlgorithm, FingerprintHasher};

#[cfg(feature = "std")]
use crate::std as test_std;

const ACCOUNT: &[u8] = b"account-a";
const TENANT: &[u8] = b"tenant-a";
const CONTEXT: &[u8] = b"review-ticket-42";
const IDENTITY: &[u8] = b"0123456789abcdef0123456789abcdef";

#[cfg(feature = "std")]
#[test]
fn canonical_plan_is_domain_separated_bounded_and_cleared() {
    let Some(request) = prepared(
        "/resources?label=one",
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
        CONTEXT,
        200,
        ReplayPolicy::ReconcileThenRetry,
    ) else {
        unreachable!("permit security fixture construction failed");
    };
    let mut output = [0xa5_u8; 4096];
    {
        let Ok(fingerprint) = build_canonical_plan(plan, &mut output) else {
            unreachable!("permit security fixture construction failed");
        };
        assert!(fingerprint.len() > request.transport_request().body().len());
        assert_eq!(
            fingerprint.subject().scope(),
            crate::operation::PermitScope::Mutation
        );
        assert!(!format_debug(&fingerprint).contains("review-ticket"));
    }
    assert_eq!(output, [0_u8; 4096]);
}

#[test]
fn exact_query_bytes_change_plan_identity() {
    let Some(endpoint) = endpoint() else {
        unreachable!("permit security fixture construction failed")
    };
    let Some(first_request) = prepared(
        "/resources?label=one",
        OperationImpact::Mutation,
        CostIntent::NoKnownCost,
    ) else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(second_request) = prepared(
        "/resources?label=two",
        OperationImpact::Mutation,
        CostIntent::NoKnownCost,
    ) else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(first_plan) = plan(
        first_request,
        endpoint,
        CONTEXT,
        200,
        ReplayPolicy::ReconcileThenRetry,
    ) else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(second_plan) = plan(
        second_request,
        endpoint,
        CONTEXT,
        200,
        ReplayPolicy::ReconcileThenRetry,
    ) else {
        unreachable!("permit security fixture construction failed");
    };
    let mut first_output = [0_u8; 4096];
    let mut second_output = [0_u8; 4096];
    let Ok(first) = build_canonical_plan(first_plan, &mut first_output) else {
        unreachable!("permit security fixture construction failed");
    };
    let Ok(second) = build_canonical_plan(second_plan, &mut second_output) else {
        unreachable!("permit security fixture construction failed");
    };
    assert!(!first.as_ref().matches(second.as_ref()));
}

#[test]
fn validation_rejects_read_only_no_op_cost_mismatch_and_small_output() {
    let Some(endpoint) = endpoint() else {
        unreachable!("permit security fixture construction failed")
    };
    let Some(read_only) = read_only("/resources") else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(context) = PermitContext::new(CONTEXT).ok() else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(validity) = PermitValidity::new(time(100), time(200)).ok() else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(attempts) = AttemptBudget::new(1).ok() else {
        unreachable!("permit security fixture construction failed");
    };
    let read_only_plan = PlanConfirmation::new(
        read_only,
        endpoint,
        PlanFingerprintScope::Absent,
        PlanFingerprintScope::Absent,
        context,
        validity,
        ReplayPolicy::SingleAttempt,
        attempts,
        PlanChange::ChangesState,
        None,
        None,
    );
    assert!(matches!(
        build_canonical_plan(read_only_plan, &mut [0_u8; 1024]),
        Err(PlanFingerprintBuildError::ReadOnlyOperation)
    ));

    let Some(mutation) = prepared(
        "/resources",
        OperationImpact::Mutation,
        CostIntent::NoKnownCost,
    ) else {
        unreachable!("permit security fixture construction failed");
    };
    let no_op = PlanConfirmation::new(
        mutation,
        endpoint,
        PlanFingerprintScope::Absent,
        PlanFingerprintScope::Absent,
        context,
        validity,
        ReplayPolicy::SingleAttempt,
        attempts,
        PlanChange::NoOp,
        None,
        None,
    );
    assert!(matches!(
        build_canonical_plan(no_op, &mut [0_u8; 1024]),
        Err(PlanFingerprintBuildError::NoOp)
    ));

    let Some(cost_request) = prepared(
        "/resources",
        OperationImpact::Mutation,
        CostIntent::MayIncurCost,
    ) else {
        unreachable!("permit security fixture construction failed");
    };
    let missing_cost = PlanConfirmation::new(
        cost_request,
        endpoint,
        PlanFingerprintScope::Absent,
        PlanFingerprintScope::Absent,
        context,
        validity,
        ReplayPolicy::SingleAttempt,
        attempts,
        PlanChange::ChangesState,
        None,
        None,
    );
    assert!(matches!(
        build_canonical_plan(missing_cost, &mut [0_u8; 1024]),
        Err(PlanFingerprintBuildError::MissingCost)
    ));

    let Some(valid_plan) = plan(
        mutation,
        endpoint,
        CONTEXT,
        200,
        ReplayPolicy::SingleAttempt,
    ) else {
        unreachable!("permit security fixture construction failed");
    };
    let mut tiny = [0xa5_u8; 4];
    assert!(matches!(
        build_canonical_plan(valid_plan, &mut tiny),
        Err(PlanFingerprintBuildError::OutputTooSmall)
    ));
    assert_eq!(tiny, [0_u8; 4]);
}

#[test]
fn cost_and_digest_policy_are_exact_and_cleanup_owned() {
    let Some(endpoint) = endpoint() else {
        unreachable!("permit security fixture construction failed")
    };
    let Some(request) = prepared(
        "/resources",
        OperationImpact::Mutation,
        CostIntent::MayIncurCost,
    ) else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(context) = PermitContext::new(CONTEXT).ok() else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(validity) = PermitValidity::new(time(100), time(200)).ok() else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(attempts) = AttemptBudget::new(1).ok() else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(currency) = CurrencyCode::new("EUR").ok() else {
        unreachable!("permit security fixture construction failed");
    };
    assert!(PlanCost::new(currency, 2, 101, 100).is_err());
    let Some(cost) = PlanCost::new(currency, 2, 100, 100).ok() else {
        unreachable!("permit security fixture construction failed");
    };
    let plan = PlanConfirmation::new(
        request,
        endpoint,
        PlanFingerprintScope::Value(ACCOUNT),
        PlanFingerprintScope::Value(TENANT),
        context,
        validity,
        ReplayPolicy::SingleAttempt,
        attempts,
        PlanChange::ChangesState,
        Some(cost),
        None,
    );
    let mut scratch = [0xa5_u8; 4096];
    let mut digest = [0xa5_u8; 64];
    {
        let Ok(fingerprint) = build_plan_digest(plan, &mut scratch, &mut digest, &TestHasher)
        else {
            unreachable!("permit security fixture construction failed");
        };
        assert_eq!(fingerprint.algorithm(), DigestAlgorithm::Sha256);
        assert_eq!(
            fingerprint.subject().scope(),
            crate::operation::PermitScope::Cost
        );
        assert_eq!(scratch, [0_u8; 4096]);
    }
    assert_eq!(digest, [0_u8; 64]);
}

#[cfg(feature = "std")]
#[test]
fn digest_failures_and_panics_clear_all_caller_storage() {
    let Some(endpoint) = endpoint() else {
        unreachable!("permit security fixture construction failed")
    };
    let Some(request) = prepared(
        "/resources",
        OperationImpact::Mutation,
        CostIntent::NoKnownCost,
    ) else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(plan) = plan(request, endpoint, CONTEXT, 200, ReplayPolicy::SingleAttempt) else {
        unreachable!("permit security fixture construction failed");
    };
    let mut scratch = [0xa5_u8; 4096];
    let mut digest = [0xa5_u8; 64];
    assert!(matches!(
        build_plan_digest(plan, &mut scratch, &mut digest, &FailingHasher),
        Err(PlanFingerprintBuildError::Hasher(HashFailure))
    ));
    assert_eq!(scratch, [0_u8; 4096]);
    assert_eq!(digest, [0_u8; 64]);

    scratch.fill(0xa5);
    digest.fill(0xa5);
    let panic = test_std::panic::catch_unwind(test_std::panic::AssertUnwindSafe(|| {
        let _ = build_plan_digest(plan, &mut scratch, &mut digest, &PanickingHasher);
    }));
    assert!(panic.is_err());
    assert_eq!(scratch, [0_u8; 4096]);
    assert_eq!(digest, [0_u8; 64]);
}

fn plan<'a>(
    request: crate::operation::PreparedRequest<'static>,
    endpoint: crate::transport::EndpointIdentity<'static>,
    context: &'a [u8],
    expires: u64,
    replay: ReplayPolicy,
) -> Option<PlanConfirmation<'a, 'static>> {
    let attempts = if replay == ReplayPolicy::SingleAttempt {
        1
    } else {
        3
    };
    Some(PlanConfirmation::new(
        request,
        endpoint,
        PlanFingerprintScope::Value(ACCOUNT),
        PlanFingerprintScope::Value(TENANT),
        PermitContext::new(context).ok()?,
        PermitValidity::new(time(100), time(expires)).ok()?,
        replay,
        AttemptBudget::new(attempts).ok()?,
        PlanChange::ChangesState,
        None,
        (replay == ReplayPolicy::ReconcileThenRetry)
            .then(|| PermitIdempotencyKey::new(IDENTITY).ok())
            .flatten(),
    ))
}

const fn time(value: u64) -> PermitTimestamp {
    PermitTimestamp::from_seconds(value)
}

struct TestHasher;

impl FingerprintHasher for TestHasher {
    type Error = core::convert::Infallible;

    fn algorithm(&self) -> DigestAlgorithm {
        DigestAlgorithm::Sha256
    }

    fn digest(&self, input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        let mut accumulator = 0_u8;
        for byte in input {
            accumulator ^= byte;
        }
        if let Some(target) = output.get_mut(..32) {
            target.fill(accumulator);
            Ok(32)
        } else {
            Ok(0)
        }
    }
}

#[cfg(feature = "std")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct HashFailure;

#[cfg(feature = "std")]
struct FailingHasher;

#[cfg(feature = "std")]
impl FingerprintHasher for FailingHasher {
    type Error = HashFailure;

    fn algorithm(&self) -> DigestAlgorithm {
        DigestAlgorithm::Sha256
    }

    fn digest(&self, _input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        output.fill(0x5a);
        Err(HashFailure)
    }
}

#[cfg(feature = "std")]
struct PanickingHasher;

#[cfg(feature = "std")]
impl FingerprintHasher for PanickingHasher {
    type Error = core::convert::Infallible;

    fn algorithm(&self) -> DigestAlgorithm {
        DigestAlgorithm::Sha256
    }

    fn digest(&self, _input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        output.fill(0x5a);
        test_std::panic::resume_unwind(test_std::boxed::Box::new("test hasher panic"))
    }
}

#[cfg(feature = "std")]
fn format_debug(value: &impl core::fmt::Debug) -> test_std::string::String {
    test_std::format!("{value:?}")
}
