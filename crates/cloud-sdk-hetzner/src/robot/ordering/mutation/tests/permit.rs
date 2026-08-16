use alloc::vec::Vec;

use cloud_sdk::operation::{
    AttemptBudget, ExecutionPermitError, PermitContext, PermitDisposition, PermitIdempotencyKey,
    PermitState, PermitTimestamp, PermitValidity, PlanFingerprintBuildError, ReplayPolicy,
};
use cloud_sdk::transport::DeliveryPhase;

use super::*;
use crate::association::Sha256PlanHasher;
use crate::endpoint::official_robot_endpoint_identity;
use crate::robot::ordering::RobotStandardTransactionList;

const IDEMPOTENCY: &[u8] = b"v0.93-order-0001";

#[test]
fn sensitive_order_requires_digest_and_exact_budgeted_authority() {
    with_standard_plan(|plan| {
        let request = RobotStandardOrderCreateRequest::new(plan, 1_100_000)
            .unwrap_or_else(|_| unreachable!("order cost failed"));
        let mut target = [0_u8; 128];
        let mut body = [0_u8; 256];
        let mut guard = cloud_sdk::operation::PreparationStorageGuard::new(&mut target, &mut body);
        let prepared = prepared_standard(&request, &mut guard);
        let exact_confirmation = confirmation(prepared, ReplayPolicy::SingleAttempt, 1);
        let mut canonical = [0xa5_u8; 4_096];
        assert!(matches!(
            build_robot_order_canonical_plan(exact_confirmation, &mut canonical),
            Err(PlanFingerprintBuildError::SensitiveBodyRequiresDigest)
        ));
        assert_eq!(canonical, [0_u8; 4_096]);

        let mut target = [0_u8; 128];
        let mut body = [0_u8; 256];
        let mut guard = cloud_sdk::operation::PreparationStorageGuard::new(&mut target, &mut body);
        let prepared = prepared_standard(&request, &mut guard);
        let mut scratch = [0xa5_u8; 4_096];
        let mut digest = [0x5a_u8; 32];
        let fingerprint = build_robot_order_plan_digest(
            confirmation(prepared, ReplayPolicy::SingleAttempt, 1),
            &mut scratch,
            &mut digest,
            &Sha256PlanHasher,
        )
        .unwrap_or_else(|_| unreachable!("order digest failed"));
        assert_eq!(scratch, [0_u8; 4_096]);
        let mut permit =
            RobotOrderCostPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(101))
                .unwrap_or_else(|_| unreachable!("cost permit failed"));
        assert_eq!(permit.state(), PermitState::Ready);
        let attempt = permit
            .begin_for(fingerprint.subject(), PermitTimestamp::from_seconds(102))
            .unwrap_or_else(|_| unreachable!("order attempt failed"));
        assert_eq!(
            attempt.complete(DeliveryPhase::NotSent),
            PermitDisposition::Spent
        );
        assert_eq!(
            permit.begin(PermitTimestamp::from_seconds(103)).err(),
            Some(ExecutionPermitError::Spent)
        );
    });
}

#[test]
fn not_sent_recovery_and_uncertain_reconciliation_are_distinct() {
    with_standard_plan(|plan| {
        let request = RobotStandardOrderCreateRequest::new(plan, 1_100_000)
            .unwrap_or_else(|_| unreachable!());
        let mut target = [0_u8; 128];
        let mut body = [0_u8; 256];
        let mut guard = cloud_sdk::operation::PreparationStorageGuard::new(&mut target, &mut body);
        let prepared = prepared_standard(&request, &mut guard);
        let mut scratch = [0_u8; 4_096];
        let mut digest = [0_u8; 32];
        let fingerprint = build_robot_order_plan_digest(
            confirmation(prepared, ReplayPolicy::ReconcileThenRetry, 3),
            &mut scratch,
            &mut digest,
            &Sha256PlanHasher,
        )
        .unwrap_or_else(|_| unreachable!());
        let mut permit =
            RobotOrderCostPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(101))
                .unwrap_or_else(|_| unreachable!());

        let first = permit
            .begin(PermitTimestamp::from_seconds(102))
            .unwrap_or_else(|_| unreachable!());
        let PermitDisposition::Recoverable(recovery) = first.complete(DeliveryPhase::NotSent)
        else {
            unreachable!("not-sent order did not become recoverable");
        };
        permit
            .recover_not_sent(recovery, PermitTimestamp::from_seconds(103))
            .unwrap_or_else(|_| unreachable!("not-sent recovery failed"));

        let second = permit
            .begin(PermitTimestamp::from_seconds(104))
            .unwrap_or_else(|_| unreachable!());
        let PermitDisposition::PendingReconciliation(token) =
            second.complete(DeliveryPhase::PossiblySent)
        else {
            unreachable!("uncertain order did not require reconciliation");
        };
        assert_eq!(
            permit.begin(PermitTimestamp::from_seconds(105)).err(),
            Some(ExecutionPermitError::ReconciliationRequired)
        );

        let absent = RobotStandardTransactionList(Vec::new());
        let proof = request
            .reconcile_not_applied(&absent)
            .unwrap_or_else(|_| unreachable!("empty complete history should reconcile"));
        permit
            .reconcile_not_applied(
                token,
                fingerprint.subject(),
                proof,
                idempotency(),
                PermitTimestamp::from_seconds(106),
            )
            .unwrap_or_else(|_| unreachable!("order reconciliation failed"));
        assert_eq!(permit.state(), PermitState::Ready);

        let third = permit
            .begin(PermitTimestamp::from_seconds(107))
            .unwrap_or_else(|_| unreachable!());
        assert!(matches!(
            third.complete(DeliveryPhase::ResponseStarted),
            PermitDisposition::PendingReconciliation(_)
        ));
    });
}

#[test]
fn expiry_and_changed_plan_are_rejected_before_execution() {
    with_standard_plan(|plan| {
        let request = RobotStandardOrderCreateRequest::new(plan, 1_100_000)
            .unwrap_or_else(|_| unreachable!("first order cost failed"));
        let changed_request = RobotStandardOrderCreateRequest::new(plan, 1_200_000)
            .unwrap_or_else(|_| unreachable!("changed order cost failed"));

        let mut target = [0_u8; 128];
        let mut body = [0_u8; 256];
        let mut guard = cloud_sdk::operation::PreparationStorageGuard::new(&mut target, &mut body);
        let prepared = prepared_standard(&request, &mut guard);
        let mut scratch = [0_u8; 4_096];
        let mut digest = [0_u8; 32];
        let fingerprint = build_robot_order_plan_digest(
            confirmation(prepared, ReplayPolicy::SingleAttempt, 1),
            &mut scratch,
            &mut digest,
            &Sha256PlanHasher,
        )
        .unwrap_or_else(|_| unreachable!("first fingerprint failed"));
        assert_eq!(
            RobotOrderCostPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(200))
                .err(),
            Some(ExecutionPermitError::Expired)
        );

        let mut changed_target = [0_u8; 128];
        let mut changed_body = [0_u8; 256];
        let mut changed_guard = cloud_sdk::operation::PreparationStorageGuard::new(
            &mut changed_target,
            &mut changed_body,
        );
        let changed_prepared = prepared_standard(&changed_request, &mut changed_guard);
        let mut changed_scratch = [0_u8; 4_096];
        let mut changed_digest = [0_u8; 32];
        let changed = build_robot_order_plan_digest(
            confirmation(changed_prepared, ReplayPolicy::SingleAttempt, 1),
            &mut changed_scratch,
            &mut changed_digest,
            &Sha256PlanHasher,
        )
        .unwrap_or_else(|_| unreachable!("changed fingerprint failed"));
        let mut permit =
            RobotOrderCostPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(101))
                .unwrap_or_else(|_| unreachable!("cost permit failed"));
        assert_eq!(
            permit
                .begin_for(changed.subject(), PermitTimestamp::from_seconds(102))
                .err(),
            Some(ExecutionPermitError::FingerprintMismatch)
        );
    });
}

#[test]
fn reconciliation_rejects_wrong_identity_request_and_exhausted_budget() {
    with_standard_plan(|plan| {
        let request = RobotStandardOrderCreateRequest::new(plan, 1_100_000)
            .unwrap_or_else(|_| unreachable!("order cost failed"));
        let other_request = RobotStandardOrderCreateRequest::new(plan, 1_100_000)
            .unwrap_or_else(|_| unreachable!("other order cost failed"));
        let mut target = [0_u8; 128];
        let mut body = [0_u8; 256];
        let mut guard = cloud_sdk::operation::PreparationStorageGuard::new(&mut target, &mut body);
        let prepared = prepared_standard(&request, &mut guard);
        let mut scratch = [0_u8; 4_096];
        let mut digest = [0_u8; 32];
        let fingerprint = build_robot_order_plan_digest(
            confirmation(prepared, ReplayPolicy::ReconcileThenRetry, 2),
            &mut scratch,
            &mut digest,
            &Sha256PlanHasher,
        )
        .unwrap_or_else(|_| unreachable!("order fingerprint failed"));
        let mut permit =
            RobotOrderCostPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(101))
                .unwrap_or_else(|_| unreachable!("cost permit failed"));
        let attempt = permit
            .begin(PermitTimestamp::from_seconds(102))
            .unwrap_or_else(|_| unreachable!("first attempt failed"));
        let PermitDisposition::PendingReconciliation(token) =
            attempt.complete(DeliveryPhase::PossiblySent)
        else {
            unreachable!("uncertain order did not require reconciliation")
        };
        let absent = RobotStandardTransactionList(Vec::new());
        let proof = request
            .reconcile_not_applied(&absent)
            .unwrap_or_else(|_| unreachable!("empty history rejected"));
        let wrong = PermitIdempotencyKey::new(b"v0.93-order-wrong")
            .unwrap_or_else(|_| unreachable!("wrong key fixture failed"));
        assert_eq!(
            permit.reconcile_not_applied(
                token,
                fingerprint.subject(),
                proof,
                wrong,
                PermitTimestamp::from_seconds(103),
            ),
            Err(ExecutionPermitError::IdempotencyMismatch)
        );

        let other_proof = other_request
            .reconcile_not_applied(&absent)
            .unwrap_or_else(|_| unreachable!("other empty history rejected"));
        assert_eq!(
            permit.reconcile_not_applied(
                token,
                fingerprint.subject(),
                other_proof,
                idempotency(),
                PermitTimestamp::from_seconds(104),
            ),
            Err(ExecutionPermitError::FingerprintMismatch)
        );

        let proof = request
            .reconcile_not_applied(&absent)
            .unwrap_or_else(|_| unreachable!("fresh empty history rejected"));
        permit
            .reconcile_not_applied(
                token,
                fingerprint.subject(),
                proof,
                idempotency(),
                PermitTimestamp::from_seconds(105),
            )
            .unwrap_or_else(|_| unreachable!("valid reconciliation failed"));
        let second = permit
            .begin(PermitTimestamp::from_seconds(106))
            .unwrap_or_else(|_| unreachable!("second attempt failed"));
        let PermitDisposition::PendingReconciliation(second_token) =
            second.complete(DeliveryPhase::ResponseStarted)
        else {
            unreachable!("response-started order did not require reconciliation")
        };
        let proof = request
            .reconcile_not_applied(&absent)
            .unwrap_or_else(|_| unreachable!("second empty history rejected"));
        assert_eq!(
            permit.reconcile_not_applied(
                second_token,
                fingerprint.subject(),
                proof,
                idempotency(),
                PermitTimestamp::from_seconds(107),
            ),
            Err(ExecutionPermitError::Spent)
        );
    });
}

#[test]
fn abandoned_attempt_requires_reconciliation() {
    with_standard_plan(|plan| {
        let request = RobotStandardOrderCreateRequest::new(plan, 1_100_000)
            .unwrap_or_else(|_| unreachable!("order cost failed"));
        let mut target = [0_u8; 128];
        let mut body = [0_u8; 256];
        let mut guard = cloud_sdk::operation::PreparationStorageGuard::new(&mut target, &mut body);
        let prepared = prepared_standard(&request, &mut guard);
        let mut scratch = [0_u8; 4_096];
        let mut digest = [0_u8; 32];
        let fingerprint = build_robot_order_plan_digest(
            confirmation(prepared, ReplayPolicy::ReconcileThenRetry, 2),
            &mut scratch,
            &mut digest,
            &Sha256PlanHasher,
        )
        .unwrap_or_else(|_| unreachable!("order fingerprint failed"));
        let mut permit =
            RobotOrderCostPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(101))
                .unwrap_or_else(|_| unreachable!("cost permit failed"));
        let attempt = permit
            .begin(PermitTimestamp::from_seconds(102))
            .unwrap_or_else(|_| unreachable!("order attempt failed"));
        drop(attempt);
        assert_eq!(permit.state(), PermitState::PendingReconciliation);
        assert_eq!(
            permit.begin(PermitTimestamp::from_seconds(103)).err(),
            Some(ExecutionPermitError::ReconciliationRequired)
        );
    });
}

fn confirmation<'storage, 'request>(
    prepared: PreparedRobotOrderMutation<
        'storage,
        'request,
        RobotStandardOrderCreateRequest<'request>,
    >,
    replay: ReplayPolicy,
    attempts: u16,
) -> RobotOrderPlanConfirmation<
    'static,
    'storage,
    'request,
    RobotStandardOrderCreateRequest<'request>,
> {
    RobotOrderPlanConfirmation::new(
        prepared,
        official_robot_endpoint_identity().unwrap_or_else(|_| unreachable!()),
        RobotOrderAccount::new(b"robot-account").unwrap_or_else(|_| unreachable!()),
        PermitContext::new(b"v0.93 Robot order fixture").unwrap_or_else(|_| unreachable!()),
        PermitValidity::new(
            PermitTimestamp::from_seconds(100),
            PermitTimestamp::from_seconds(200),
        )
        .unwrap_or_else(|_| unreachable!()),
        replay,
        AttemptBudget::new(attempts).unwrap_or_else(|_| unreachable!()),
        (replay == ReplayPolicy::ReconcileThenRetry).then(idempotency),
    )
}

fn idempotency() -> PermitIdempotencyKey<'static> {
    PermitIdempotencyKey::new(IDEMPOTENCY).unwrap_or_else(|_| unreachable!())
}
