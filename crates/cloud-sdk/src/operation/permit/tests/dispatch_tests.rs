use core::future::Future;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::{Context, Poll, Waker};

use super::fixture::{ClassifiedTransport, endpoint, prepared, prepared_with_policy};
use crate::operation::{
    AttemptBudget, CostIntent, ExecutionPermitError, MutationPermit, OperationImpact,
    PermitContext, PermitState, PermitTimestamp, PermitValidity, PlanChange, PlanConfirmation,
    PlanFingerprintScope, ReplayPolicy, SharedMutationPermit, SharedPermitState,
    build_canonical_plan,
};
use crate::transport::{EndpointIdentity, EndpointPolicy, EndpointScheme};

#[cfg(feature = "std")]
use crate::std as test_std;

struct TestClock(AtomicU32);

impl TestClock {
    const fn new(now: u32) -> Self {
        Self(AtomicU32::new(now))
    }

    fn set(&self, now: u32) {
        self.0.store(now, Ordering::Release);
    }
}

impl crate::operation::PermitClock for TestClock {
    fn now(&self) -> PermitTimestamp {
        time(u64::from(self.0.load(Ordering::Acquire)))
    }
}

#[cfg(feature = "std")]
struct PanickingClock;

#[cfg(feature = "std")]
impl crate::operation::PermitClock for PanickingClock {
    fn now(&self) -> PermitTimestamp {
        test_std::panic::resume_unwind(test_std::boxed::Box::new("clock failure"))
    }
}

#[test]
fn confirmed_endpoint_is_exact_within_an_admitted_official_set() {
    let Some(first) = endpoint() else {
        unreachable!("permit security fixture construction failed")
    };
    let Some(second) =
        EndpointIdentity::new(EndpointScheme::Https, "api-alt.example.invalid", 443, "/v1").ok()
    else {
        unreachable!("permit security fixture construction failed");
    };
    let official = [first, second];
    let Some(policy) = EndpointPolicy::official_set(&official).ok() else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(request) = prepared_with_policy(
        "/resources",
        OperationImpact::Mutation,
        CostIntent::NoKnownCost,
        policy,
    ) else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(plan) = plan_for_request(request, first, 200) else {
        unreachable!("permit security fixture construction failed");
    };
    let mut storage = [0_u8; 4096];
    let Ok(fingerprint) = build_canonical_plan(plan, &mut storage) else {
        unreachable!("permit security fixture construction failed");
    };
    let Ok(mut permit) = MutationPermit::new(fingerprint.subject(), time(100)) else {
        unreachable!("permit security fixture construction failed");
    };
    let Ok(attempt) = permit.begin(time(101)) else {
        unreachable!("permit security fixture construction failed");
    };
    let transport = ClassifiedTransport::new(second, None);
    let clock = TestClock::new(102);
    let mut body = [0xa5_u8; 64];
    let mut headers = [0xa5_u8; 128];
    let result = attempt.execute_blocking(&clock, &transport, &mut body, &mut headers);
    assert!(matches!(
        result.as_ref().map_err(|error| error.execution()),
        Err(crate::operation::PreparedExecutionError::EndpointMismatch)
    ));
    drop(result);
    assert_eq!(transport.calls(), 0);
    assert_eq!(permit.state(), PermitState::Spent);
}

#[test]
fn expiry_is_exclusive_and_rechecked_at_blocking_dispatch() {
    let Some((mut storage, plan)) = mutation_plan(110) else {
        unreachable!("permit security fixture construction failed");
    };
    let Ok(fingerprint) = build_canonical_plan(plan, &mut storage) else {
        unreachable!("permit security fixture construction failed");
    };
    let Ok(mut permit) = MutationPermit::new(fingerprint.subject(), time(100)) else {
        unreachable!("permit security fixture construction failed");
    };
    let Ok(attempt) = permit.begin(time(109)) else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(endpoint) = endpoint() else {
        unreachable!("permit security fixture construction failed")
    };
    let transport = ClassifiedTransport::new(endpoint, None);
    let clock = TestClock::new(110);
    let mut body = [0xa5_u8; 64];
    let mut headers = [0xa5_u8; 128];
    let result = attempt.execute_blocking(&clock, &transport, &mut body, &mut headers);
    assert!(matches!(
        result.as_ref().map_err(|error| error.execution()),
        Err(
            crate::operation::PreparedExecutionError::AuthorizationInvalid(
                ExecutionPermitError::Expired
            )
        )
    ));
    drop(result);
    assert_eq!(transport.calls(), 0);
    assert_eq!(body, [0_u8; 64]);
    assert_eq!(headers, [0_u8; 128]);
    assert_eq!(permit.state(), PermitState::Spent);
}

#[test]
fn send_async_samples_time_when_first_polled() {
    let Some((mut storage, plan)) = mutation_plan(110) else {
        unreachable!("permit security fixture construction failed");
    };
    let Ok(fingerprint) = build_canonical_plan(plan, &mut storage) else {
        unreachable!("permit security fixture construction failed");
    };
    let Ok(mut permit) = MutationPermit::new(fingerprint.subject(), time(100)) else {
        unreachable!("permit security fixture construction failed");
    };
    let Ok(attempt) = permit.begin(time(109)) else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(endpoint) = endpoint() else {
        unreachable!("permit security fixture construction failed")
    };
    let transport = ClassifiedTransport::new(endpoint, None);
    let clock = TestClock::new(109);
    let mut body = [0xa5_u8; 64];
    let mut headers = [0xa5_u8; 128];
    {
        let future = attempt.execute_async(&clock, &transport, &mut body, &mut headers);
        clock.set(110);
        let mut future = core::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Ready(Err(ref error))
                if matches!(
                    error.execution(),
                    crate::operation::PreparedExecutionError::AuthorizationInvalid(
                        ExecutionPermitError::Expired
                    )
                )
        ));
    }
    assert_eq!(transport.calls(), 0);
    assert_eq!(body, [0_u8; 64]);
    assert_eq!(headers, [0_u8; 128]);
    assert_eq!(permit.state(), PermitState::Spent);
}

#[test]
fn local_async_samples_time_when_first_polled() {
    let Some((mut storage, plan)) = mutation_plan(110) else {
        unreachable!("permit security fixture construction failed");
    };
    let Ok(fingerprint) = build_canonical_plan(plan, &mut storage) else {
        unreachable!("permit security fixture construction failed");
    };
    let Ok(mut permit) = MutationPermit::new(fingerprint.subject(), time(100)) else {
        unreachable!("permit security fixture construction failed");
    };
    let Ok(attempt) = permit.begin(time(109)) else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(endpoint) = endpoint() else {
        unreachable!("permit security fixture construction failed")
    };
    let transport = ClassifiedTransport::new(endpoint, None);
    let clock = TestClock::new(109);
    let mut body = [0xa5_u8; 64];
    let mut headers = [0xa5_u8; 128];
    {
        let future = attempt.execute_local_async(&clock, &transport, &mut body, &mut headers);
        clock.set(110);
        let mut future = core::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Ready(Err(ref error))
                if matches!(
                    error.execution(),
                    crate::operation::PreparedExecutionError::AuthorizationInvalid(
                        ExecutionPermitError::Expired
                    )
                )
        ));
    }
    assert_eq!(transport.calls(), 0);
    assert_eq!(body, [0_u8; 64]);
    assert_eq!(headers, [0_u8; 128]);
    assert_eq!(permit.state(), PermitState::Spent);
}

#[test]
fn shared_attempt_rechecks_expiry_at_dispatch() {
    let Some((mut storage, plan)) = mutation_plan(110) else {
        unreachable!("permit security fixture construction failed");
    };
    let Ok(fingerprint) = build_canonical_plan(plan, &mut storage) else {
        unreachable!("permit security fixture construction failed");
    };
    let mut state = SharedPermitState::new();
    let Ok(permit) = SharedMutationPermit::new(&mut state, fingerprint.subject(), time(100)) else {
        unreachable!("permit security fixture construction failed");
    };
    let Ok(attempt) = permit.begin(time(109)) else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(endpoint) = endpoint() else {
        unreachable!("permit security fixture construction failed")
    };
    let transport = ClassifiedTransport::new(endpoint, None);
    let clock = TestClock::new(110);
    let mut body = [0xa5_u8; 64];
    let mut headers = [0xa5_u8; 128];
    let result = attempt.execute_blocking(&clock, &transport, &mut body, &mut headers);
    assert!(matches!(
        result.as_ref().map_err(|error| error.execution()),
        Err(
            crate::operation::PreparedExecutionError::AuthorizationInvalid(
                ExecutionPermitError::Expired
            )
        )
    ));
    drop(result);
    assert_eq!(transport.calls(), 0);
    assert_eq!(body, [0_u8; 64]);
    assert_eq!(headers, [0_u8; 128]);
    assert_eq!(permit.state(), PermitState::Spent);
}

#[test]
fn shared_attempt_cannot_dispatch_after_another_handle_spends_its_generation() {
    let Some((mut storage, plan)) = mutation_plan(110) else {
        unreachable!("permit security fixture construction failed");
    };
    let Ok(fingerprint) = build_canonical_plan(plan, &mut storage) else {
        unreachable!("permit security fixture construction failed");
    };
    let mut state = SharedPermitState::new();
    let Ok(permit) = SharedMutationPermit::new(&mut state, fingerprint.subject(), time(100)) else {
        unreachable!("permit security fixture construction failed");
    };
    let invalidator = permit.clone();
    let Ok(attempt) = permit.begin(time(109)) else {
        unreachable!("permit security fixture construction failed");
    };
    assert!(matches!(
        invalidator.begin(time(110)),
        Err(ExecutionPermitError::Expired)
    ));
    assert_eq!(permit.state(), PermitState::Spent);

    let Some(endpoint) = endpoint() else {
        unreachable!("permit security fixture construction failed")
    };
    let transport = ClassifiedTransport::new(endpoint, None);
    let clock = TestClock::new(109);
    let mut body = [0xa5_u8; 64];
    let mut headers = [0xa5_u8; 128];
    let result = attempt.execute_blocking(&clock, &transport, &mut body, &mut headers);
    assert!(matches!(
        result.as_ref().map_err(|error| error.execution()),
        Err(
            crate::operation::PreparedExecutionError::AuthorizationInvalid(
                ExecutionPermitError::Spent
            )
        )
    ));
    drop(result);
    assert_eq!(transport.calls(), 0);
    assert_eq!(body, [0_u8; 64]);
    assert_eq!(headers, [0_u8; 128]);
    assert_eq!(permit.state(), PermitState::Spent);
}

#[cfg(feature = "std")]
#[test]
fn panicking_clock_clears_blocking_response_storage_before_unwind() {
    let fixture = mutation_plan(200);
    assert!(
        fixture.is_some(),
        "mutation permit fixture must remain valid"
    );
    let Some((mut storage, plan)) = fixture else {
        unreachable!("permit security fixture construction failed");
    };
    let fingerprint = build_canonical_plan(plan, &mut storage);
    assert!(
        fingerprint.is_ok(),
        "canonical security fixture must build successfully"
    );
    let Ok(fingerprint) = fingerprint else {
        unreachable!("permit security fixture construction failed");
    };
    let permit = MutationPermit::new(fingerprint.subject(), time(100));
    assert!(permit.is_ok(), "mutation permit fixture must construct");
    let Ok(mut permit) = permit else {
        unreachable!("permit security fixture construction failed");
    };
    let attempt = permit.begin(time(101));
    assert!(attempt.is_ok(), "mutation permit attempt must begin");
    let Ok(attempt) = attempt else {
        unreachable!("permit security fixture construction failed");
    };
    let endpoint = endpoint();
    assert!(endpoint.is_some(), "endpoint fixture must remain valid");
    let Some(endpoint) = endpoint else {
        unreachable!("permit security fixture construction failed")
    };
    let transport = ClassifiedTransport::new(endpoint, None);
    let mut body = [0xa5_u8; 64];
    let mut headers = [0xa5_u8; 128];

    let panic = test_std::panic::catch_unwind(test_std::panic::AssertUnwindSafe(|| {
        let _ = attempt.execute_blocking(&PanickingClock, &transport, &mut body, &mut headers);
    }));

    assert!(panic.is_err());
    assert_eq!(transport.calls(), 0);
    assert_eq!(body, [0_u8; 64]);
    assert_eq!(headers, [0_u8; 128]);
}

#[cfg(feature = "std")]
#[test]
fn panicking_clock_clears_send_async_response_storage_before_unwind() {
    let fixture = mutation_plan(200);
    assert!(
        fixture.is_some(),
        "mutation permit fixture must remain valid"
    );
    let Some((mut storage, plan)) = fixture else {
        unreachable!("permit security fixture construction failed");
    };
    let fingerprint = build_canonical_plan(plan, &mut storage);
    assert!(
        fingerprint.is_ok(),
        "canonical security fixture must build successfully"
    );
    let Ok(fingerprint) = fingerprint else {
        unreachable!("permit security fixture construction failed");
    };
    let permit = MutationPermit::new(fingerprint.subject(), time(100));
    assert!(permit.is_ok(), "mutation permit fixture must construct");
    let Ok(mut permit) = permit else {
        unreachable!("permit security fixture construction failed");
    };
    let attempt = permit.begin(time(101));
    assert!(attempt.is_ok(), "mutation permit attempt must begin");
    let Ok(attempt) = attempt else {
        unreachable!("permit security fixture construction failed");
    };
    let endpoint = endpoint();
    assert!(endpoint.is_some(), "endpoint fixture must remain valid");
    let Some(endpoint) = endpoint else {
        unreachable!("permit security fixture construction failed")
    };
    let transport = ClassifiedTransport::new(endpoint, None);
    let mut body = [0xa5_u8; 64];
    let mut headers = [0xa5_u8; 128];

    let panic = test_std::panic::catch_unwind(test_std::panic::AssertUnwindSafe(|| {
        let future = attempt.execute_async(&PanickingClock, &transport, &mut body, &mut headers);
        let mut future = core::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        let _ = Future::poll(future.as_mut(), &mut context);
    }));

    assert!(panic.is_err());
    assert_eq!(transport.calls(), 0);
    assert_eq!(body, [0_u8; 64]);
    assert_eq!(headers, [0_u8; 128]);
}

#[cfg(feature = "std")]
#[test]
fn panicking_clock_clears_local_async_response_storage_before_unwind() {
    let fixture = mutation_plan(200);
    assert!(
        fixture.is_some(),
        "mutation permit fixture must remain valid"
    );
    let Some((mut storage, plan)) = fixture else {
        unreachable!("permit security fixture construction failed");
    };
    let fingerprint = build_canonical_plan(plan, &mut storage);
    assert!(
        fingerprint.is_ok(),
        "canonical security fixture must build successfully"
    );
    let Ok(fingerprint) = fingerprint else {
        unreachable!("permit security fixture construction failed");
    };
    let permit = MutationPermit::new(fingerprint.subject(), time(100));
    assert!(permit.is_ok(), "mutation permit fixture must construct");
    let Ok(mut permit) = permit else {
        unreachable!("permit security fixture construction failed");
    };
    let attempt = permit.begin(time(101));
    assert!(attempt.is_ok(), "mutation permit attempt must begin");
    let Ok(attempt) = attempt else {
        unreachable!("permit security fixture construction failed");
    };
    let endpoint = endpoint();
    assert!(endpoint.is_some(), "endpoint fixture must remain valid");
    let Some(endpoint) = endpoint else {
        unreachable!("permit security fixture construction failed")
    };
    let transport = ClassifiedTransport::new(endpoint, None);
    let mut body = [0xa5_u8; 64];
    let mut headers = [0xa5_u8; 128];

    let panic = test_std::panic::catch_unwind(test_std::panic::AssertUnwindSafe(|| {
        let future =
            attempt.execute_local_async(&PanickingClock, &transport, &mut body, &mut headers);
        let mut future = core::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        let _ = Future::poll(future.as_mut(), &mut context);
    }));

    assert!(panic.is_err());
    assert_eq!(transport.calls(), 0);
    assert_eq!(body, [0_u8; 64]);
    assert_eq!(headers, [0_u8; 128]);
}

fn mutation_plan(expires: u64) -> Option<([u8; 4096], PlanConfirmation<'static, 'static>)> {
    let endpoint = endpoint()?;
    let request = prepared(
        "/resources",
        OperationImpact::Mutation,
        CostIntent::NoKnownCost,
    )?;
    Some(([0_u8; 4096], plan_for_request(request, endpoint, expires)?))
}

fn plan_for_request<'a>(
    request: crate::operation::PreparedRequest<'a>,
    endpoint: EndpointIdentity<'a>,
    expires: u64,
) -> Option<PlanConfirmation<'a, 'a>> {
    Some(PlanConfirmation::new(
        request,
        endpoint,
        PlanFingerprintScope::Value(b"account-a"),
        PlanFingerprintScope::Value(b"tenant-a"),
        PermitContext::new(b"review-ticket-42").ok()?,
        PermitValidity::new(time(100), time(expires)).ok()?,
        ReplayPolicy::SingleAttempt,
        AttemptBudget::new(1).ok()?,
        PlanChange::ChangesState,
        None,
        None,
    ))
}

const fn time(value: u64) -> PermitTimestamp {
    PermitTimestamp::from_seconds(value)
}
