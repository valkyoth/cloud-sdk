use core::future::Future;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::{Context, Poll, Waker};

use super::fixture::{ClassifiedTransport, endpoint, prepared};
use crate::operation::{
    AttemptBudget, CostIntent, CostPermit, ExecutionPermitError, MutationPermit, OperationImpact,
    PermitContext, PermitDisposition, PermitIdempotencyKey, PermitState, PermitTimestamp,
    PermitValidity, PlanChange, PlanConfirmation, PlanFingerprintScope, RecoveryToken,
    ReplayPolicy, SharedMutationPermit, SharedPermitState, build_canonical_plan,
};
use crate::transport::DeliveryPhase;

#[cfg(feature = "std")]
use crate::std as test_std;

const IDENTITY: &[u8] = b"0123456789abcdef0123456789abcdef";

struct TestClock(AtomicU32);

impl TestClock {
    const fn new(now: u32) -> Self {
        Self(AtomicU32::new(now))
    }
}

impl crate::operation::PermitClock for TestClock {
    fn now(&self) -> PermitTimestamp {
        time(u64::from(self.0.load(Ordering::Acquire)))
    }
}

#[test]
fn direct_recovery_reconciliation_and_budget_are_generation_bound() {
    let Some((mut storage, plan)) =
        mutation_plan("/resources", ReplayPolicy::ReconcileThenRetry, 4, 200)
    else {
        return;
    };
    let Ok(fingerprint) = build_canonical_plan(plan, &mut storage) else {
        return;
    };
    let subject = fingerprint.subject();
    let Ok(mut permit) = MutationPermit::new(subject, time(100)) else {
        return;
    };

    let Ok(first) = permit.begin(time(101)) else {
        return;
    };
    let PermitDisposition::Recoverable(first_token) = first.complete(DeliveryPhase::NotSent) else {
        return;
    };
    assert_eq!(permit.state(), PermitState::Recoverable);
    assert!(permit.recover_not_sent(first_token, time(102)).is_ok());

    let Ok(second) = permit.begin(time(103)) else {
        return;
    };
    let PermitDisposition::Recoverable(second_token) = second.complete(DeliveryPhase::NotSent)
    else {
        return;
    };
    assert_eq!(
        permit.recover_not_sent(first_token, time(104)),
        Err(ExecutionPermitError::StaleGeneration)
    );
    assert!(permit.recover_not_sent(second_token, time(104)).is_ok());

    let Ok(third) = permit.begin(time(105)) else {
        return;
    };
    let PermitDisposition::PendingReconciliation(pending) =
        third.complete(DeliveryPhase::PossiblySent)
    else {
        return;
    };
    let Ok(wrong) = PermitIdempotencyKey::new(b"fedcba9876543210fedcba9876543210") else {
        return;
    };
    assert_eq!(
        permit.reconcile_not_applied(pending, subject, wrong, time(106)),
        Err(ExecutionPermitError::IdempotencyMismatch)
    );
    let Ok(identity) = PermitIdempotencyKey::new(IDENTITY) else {
        return;
    };
    assert!(
        permit
            .reconcile_not_applied(pending, subject, identity, time(106))
            .is_ok()
    );
    let Ok(last) = permit.begin(time(107)) else {
        return;
    };
    assert_eq!(last.complete_applied(), PermitDisposition::Spent);
    assert_eq!(permit.state(), PermitState::Spent);
    assert!(matches!(
        permit.begin(time(108)),
        Err(ExecutionPermitError::Spent)
    ));
}

#[test]
fn rollback_expiry_mismatch_scope_and_drop_fail_closed() {
    let Some((mut storage, plan)) =
        mutation_plan("/resources", ReplayPolicy::ReconcileThenRetry, 3, 110)
    else {
        return;
    };
    let Ok(fingerprint) = build_canonical_plan(plan, &mut storage) else {
        return;
    };
    let subject = fingerprint.subject();
    assert!(matches!(
        CostPermit::new(subject, time(100)),
        Err(ExecutionPermitError::ScopeMismatch)
    ));
    let Ok(mut permit) = MutationPermit::new(subject, time(105)) else {
        return;
    };
    assert!(matches!(
        permit.begin(time(104)),
        Err(ExecutionPermitError::ClockRollback)
    ));
    assert_eq!(permit.state(), PermitState::Spent);
    assert!(matches!(
        permit.begin(time(106)),
        Err(ExecutionPermitError::Spent)
    ));

    let Some((mut other_storage, other_plan)) =
        mutation_plan("/other", ReplayPolicy::ReconcileThenRetry, 3, 110)
    else {
        return;
    };
    let Ok(other) = build_canonical_plan(other_plan, &mut other_storage) else {
        return;
    };
    let Ok(mut permit) = MutationPermit::new(subject, time(105)) else {
        return;
    };
    assert!(matches!(
        permit.begin_for(other.subject(), time(106)),
        Err(ExecutionPermitError::FingerprintMismatch)
    ));

    let Ok(attempt) = permit.begin(time(106)) else {
        return;
    };
    drop(attempt);
    assert_eq!(permit.state(), PermitState::PendingReconciliation);
    assert!(matches!(
        permit.begin(time(111)),
        Err(ExecutionPermitError::Expired)
    ));
    assert_eq!(permit.state(), PermitState::Spent);
    assert!(matches!(
        permit.begin(time(109)),
        Err(ExecutionPermitError::Spent)
    ));
}

#[cfg(feature = "std")]
#[test]
fn shared_clones_cannot_double_spend_or_restore_dropped_authority() {
    let Some((mut storage, plan)) =
        mutation_plan("/resources", ReplayPolicy::ReconcileThenRetry, 2, 200)
    else {
        return;
    };
    let Ok(fingerprint) = build_canonical_plan(plan, &mut storage) else {
        return;
    };
    let mut state = SharedPermitState::new();
    let Ok(first) = SharedMutationPermit::new(&mut state, fingerprint.subject(), time(100)) else {
        return;
    };
    let second = first.clone();
    {
        let unused = first.clone();
        assert_eq!(unused.state(), PermitState::Ready);
    }
    let entered = test_std::sync::Barrier::new(2);
    let release = test_std::sync::Barrier::new(2);
    test_std::thread::scope(|threads| {
        let entered_worker = &entered;
        let release_worker = &release;
        let first_worker = &first;
        threads.spawn(move || {
            let Ok(attempt) = first_worker.begin(time(101)) else {
                return;
            };
            entered_worker.wait();
            release_worker.wait();
            drop(attempt);
        });
        entered.wait();
        assert!(matches!(
            second.begin(time(101)),
            Err(ExecutionPermitError::AttemptInFlight)
        ));
        release.wait();
    });
    assert_eq!(second.state(), PermitState::PendingReconciliation);
}

#[test]
fn erased_mutations_cannot_bypass_authority_and_buffers_are_cleared() {
    let Some(request) = prepared(
        "/resources",
        OperationImpact::Mutation,
        CostIntent::NoKnownCost,
    ) else {
        return;
    };
    let Some(endpoint) = endpoint() else { return };
    let transport = ClassifiedTransport::new(endpoint, None);
    let mut body = [0xa5_u8; 64];
    let mut headers = [0xa5_u8; 128];
    assert!(matches!(
        request.execute_blocking(&transport, &mut body, &mut headers),
        Err(crate::operation::PreparedExecutionError::AuthorizationRequired)
    ));
    assert_eq!(transport.calls(), 0);
    assert_eq!(body, [0_u8; 64]);
    assert_eq!(headers, [0_u8; 128]);
}

#[test]
fn permit_execution_maps_delivery_and_spends_success_across_modes() {
    let Some(endpoint) = endpoint() else { return };
    let Some((mut first_storage, first_plan)) =
        mutation_plan("/resources", ReplayPolicy::RecoverNotSent, 2, 200)
    else {
        return;
    };
    let Ok(first_fingerprint) = build_canonical_plan(first_plan, &mut first_storage) else {
        return;
    };
    let Ok(mut first_permit) = MutationPermit::new(first_fingerprint.subject(), time(100)) else {
        return;
    };
    let failing = ClassifiedTransport::new(endpoint, Some(DeliveryPhase::NotSent));
    let Ok(attempt) = first_permit.begin(time(101)) else {
        return;
    };
    let mut body = [0_u8; 64];
    let mut headers = [0_u8; 128];
    let clock = TestClock::new(102);
    let error = attempt.execute_blocking(&clock, &failing, &mut body, &mut headers);
    assert!(matches!(
        error.as_ref().map_err(|error| error.disposition()),
        Err(PermitDisposition::Recoverable(_))
    ));
    drop(error);

    let Some((mut second_storage, second_plan)) =
        mutation_plan("/resources", ReplayPolicy::SingleAttempt, 1, 200)
    else {
        return;
    };
    let Ok(second_fingerprint) = build_canonical_plan(second_plan, &mut second_storage) else {
        return;
    };
    let Ok(mut second_permit) = MutationPermit::new(second_fingerprint.subject(), time(100)) else {
        return;
    };
    let successful = ClassifiedTransport::new(endpoint, None);
    let Ok(attempt) = second_permit.begin(time(101)) else {
        return;
    };
    {
        let result = attempt.execute_blocking(&clock, &successful, &mut body, &mut headers);
        assert!(result.is_ok());
    }
    assert_eq!(second_permit.state(), PermitState::Spent);

    let Some((mut async_storage, async_plan)) =
        mutation_plan("/resources", ReplayPolicy::SingleAttempt, 1, 200)
    else {
        return;
    };
    let Ok(async_fingerprint) = build_canonical_plan(async_plan, &mut async_storage) else {
        return;
    };
    let Ok(mut async_permit) = MutationPermit::new(async_fingerprint.subject(), time(100)) else {
        return;
    };
    let Ok(attempt) = async_permit.begin(time(101)) else {
        return;
    };
    {
        let future = attempt.execute_async(&clock, &successful, &mut body, &mut headers);
        let mut future = core::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Ready(Ok(_))
        ));
    }
    assert_eq!(async_permit.state(), PermitState::Spent);

    let Some((mut local_storage, local_plan)) =
        mutation_plan("/resources", ReplayPolicy::SingleAttempt, 1, 200)
    else {
        return;
    };
    let Ok(local_fingerprint) = build_canonical_plan(local_plan, &mut local_storage) else {
        return;
    };
    let Ok(mut local_permit) = MutationPermit::new(local_fingerprint.subject(), time(100)) else {
        return;
    };
    let Ok(attempt) = local_permit.begin(time(101)) else {
        return;
    };
    {
        let future = attempt.execute_local_async(&clock, &successful, &mut body, &mut headers);
        let mut future = core::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Ready(Ok(_))
        ));
    }
    assert_eq!(local_permit.state(), PermitState::Spent);
}

fn mutation_plan(
    target: &'static str,
    replay: ReplayPolicy,
    attempts: u16,
    expires: u64,
) -> Option<([u8; 4096], PlanConfirmation<'static, 'static>)> {
    let request = prepared(target, OperationImpact::Mutation, CostIntent::NoKnownCost)?;
    let endpoint = endpoint()?;
    let idempotency = if replay == ReplayPolicy::ReconcileThenRetry {
        Some(PermitIdempotencyKey::new(IDENTITY).ok()?)
    } else {
        None
    };
    Some((
        [0_u8; 4096],
        PlanConfirmation::new(
            request,
            endpoint,
            PlanFingerprintScope::Value(b"account-a"),
            PlanFingerprintScope::Value(b"tenant-a"),
            PermitContext::new(b"review-ticket-42").ok()?,
            PermitValidity::new(time(100), time(expires)).ok()?,
            replay,
            AttemptBudget::new(attempts).ok()?,
            PlanChange::ChangesState,
            None,
            idempotency,
        ),
    ))
}

const fn time(value: u64) -> PermitTimestamp {
    PermitTimestamp::from_seconds(value)
}

#[test]
fn stale_manual_recovery_token_never_rearms_spent_state() {
    let Some((mut storage, plan)) =
        mutation_plan("/resources", ReplayPolicy::RecoverNotSent, 1, 200)
    else {
        return;
    };
    let Ok(fingerprint) = build_canonical_plan(plan, &mut storage) else {
        return;
    };
    let Ok(mut permit) = MutationPermit::new(fingerprint.subject(), time(100)) else {
        return;
    };
    let Ok(attempt) = permit.begin(time(101)) else {
        return;
    };
    assert_eq!(
        attempt.complete(DeliveryPhase::NotSent),
        PermitDisposition::Spent
    );
    assert_eq!(
        permit.recover_not_sent(RecoveryToken(0), time(102)),
        Err(ExecutionPermitError::StaleGeneration)
    );
}

#[test]
fn shared_recovery_tokens_are_generation_bound() {
    let Some((mut storage, plan)) =
        mutation_plan("/resources", ReplayPolicy::RecoverNotSent, 3, 200)
    else {
        return;
    };
    let Ok(fingerprint) = build_canonical_plan(plan, &mut storage) else {
        return;
    };
    let mut state = SharedPermitState::new();
    let Ok(permit) = SharedMutationPermit::new(&mut state, fingerprint.subject(), time(100)) else {
        return;
    };
    let Ok(first) = permit.begin(time(101)) else {
        return;
    };
    let PermitDisposition::Recoverable(first_token) = first.complete(DeliveryPhase::NotSent) else {
        return;
    };
    assert!(permit.recover_not_sent(first_token, time(102)).is_ok());
    let Ok(second) = permit.begin(time(103)) else {
        return;
    };
    let PermitDisposition::Recoverable(second_token) = second.complete(DeliveryPhase::NotSent)
    else {
        return;
    };
    assert_eq!(
        permit.recover_not_sent(first_token, time(104)),
        Err(ExecutionPermitError::StaleGeneration)
    );
    assert!(permit.recover_not_sent(second_token, time(104)).is_ok());
}

#[test]
fn expiry_permanently_spends_recoverable_and_reconcilable_permits() {
    let Some((mut direct_storage, direct_plan)) =
        mutation_plan("/direct", ReplayPolicy::RecoverNotSent, 2, 110)
    else {
        return;
    };
    let Ok(direct_fingerprint) = build_canonical_plan(direct_plan, &mut direct_storage) else {
        return;
    };
    let Ok(mut direct) = MutationPermit::new(direct_fingerprint.subject(), time(100)) else {
        return;
    };
    let Ok(attempt) = direct.begin(time(101)) else {
        return;
    };
    let PermitDisposition::Recoverable(recovery) = attempt.complete(DeliveryPhase::NotSent) else {
        return;
    };
    assert_eq!(
        direct.recover_not_sent(recovery, time(110)),
        Err(ExecutionPermitError::Expired)
    );
    assert_eq!(direct.state(), PermitState::Spent);
    assert!(matches!(
        direct.begin(time(109)),
        Err(ExecutionPermitError::Spent)
    ));

    let Some((mut shared_storage, shared_plan)) =
        mutation_plan("/shared", ReplayPolicy::ReconcileThenRetry, 2, 110)
    else {
        return;
    };
    let Ok(shared_fingerprint) = build_canonical_plan(shared_plan, &mut shared_storage) else {
        return;
    };
    let subject = shared_fingerprint.subject();
    let mut state = SharedPermitState::new();
    let Ok(shared) = SharedMutationPermit::new(&mut state, subject, time(100)) else {
        return;
    };
    let Ok(attempt) = shared.begin(time(101)) else {
        return;
    };
    let PermitDisposition::PendingReconciliation(reconciliation) =
        attempt.complete(DeliveryPhase::PossiblySent)
    else {
        return;
    };
    let Ok(identity) = PermitIdempotencyKey::new(IDENTITY) else {
        return;
    };
    assert_eq!(
        shared.reconcile_not_applied(reconciliation, subject, identity, time(110)),
        Err(ExecutionPermitError::Expired)
    );
    assert_eq!(shared.state(), PermitState::Spent);
    assert!(matches!(
        shared.begin(time(109)),
        Err(ExecutionPermitError::Spent)
    ));
}
