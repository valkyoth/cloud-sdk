use super::fixture::{endpoint, prepared};
use crate::operation::{
    AttemptBudget, CostIntent, CurrencyCode, OperationImpact, PermitContext, PermitIdempotencyKey,
    PermitTimestamp, PermitValidity, PlanChange, PlanConfirmation, PlanCost, PlanFingerprintScope,
    ReplayPolicy, build_canonical_plan,
};

const FIRST_ID: &[u8] = b"0123456789abcdef0123456789abcdef";
const SECOND_ID: &[u8] = b"fedcba9876543210fedcba9876543210";

#[test]
fn every_confirmation_policy_field_is_independently_domain_separated() {
    let Some(request) = prepared(
        "/resources?label=one",
        OperationImpact::Mutation,
        CostIntent::NoKnownCost,
    ) else {
        return;
    };
    let Some(endpoint) = endpoint() else { return };
    let Some(base) = confirmation(
        request,
        endpoint,
        b"account-a",
        b"tenant-a",
        b"review-42",
        100,
        200,
        ReplayPolicy::ReconcileThenRetry,
        3,
        Some(FIRST_ID),
    ) else {
        return;
    };
    let variants = [
        confirmation(
            request,
            endpoint,
            b"account-b",
            b"tenant-a",
            b"review-42",
            100,
            200,
            ReplayPolicy::ReconcileThenRetry,
            3,
            Some(FIRST_ID),
        ),
        confirmation(
            request,
            endpoint,
            b"account-a",
            b"tenant-b",
            b"review-42",
            100,
            200,
            ReplayPolicy::ReconcileThenRetry,
            3,
            Some(FIRST_ID),
        ),
        confirmation(
            request,
            endpoint,
            b"account-a",
            b"tenant-a",
            b"review-43",
            100,
            200,
            ReplayPolicy::ReconcileThenRetry,
            3,
            Some(FIRST_ID),
        ),
        confirmation(
            request,
            endpoint,
            b"account-a",
            b"tenant-a",
            b"review-42",
            99,
            200,
            ReplayPolicy::ReconcileThenRetry,
            3,
            Some(FIRST_ID),
        ),
        confirmation(
            request,
            endpoint,
            b"account-a",
            b"tenant-a",
            b"review-42",
            100,
            201,
            ReplayPolicy::ReconcileThenRetry,
            3,
            Some(FIRST_ID),
        ),
        confirmation(
            request,
            endpoint,
            b"account-a",
            b"tenant-a",
            b"review-42",
            100,
            200,
            ReplayPolicy::RecoverNotSent,
            3,
            None,
        ),
        confirmation(
            request,
            endpoint,
            b"account-a",
            b"tenant-a",
            b"review-42",
            100,
            200,
            ReplayPolicy::ReconcileThenRetry,
            4,
            Some(FIRST_ID),
        ),
        confirmation(
            request,
            endpoint,
            b"account-a",
            b"tenant-a",
            b"review-42",
            100,
            200,
            ReplayPolicy::ReconcileThenRetry,
            3,
            Some(SECOND_ID),
        ),
    ];
    for variant in variants {
        let Some(variant) = variant else { return };
        assert!(different(base, variant));
    }
}

#[test]
fn every_cost_field_is_independently_domain_separated() {
    let Some(request) = prepared(
        "/resources",
        OperationImpact::Mutation,
        CostIntent::MayIncurCost,
    ) else {
        return;
    };
    let Some(endpoint) = endpoint() else { return };
    let Some(eur) = CurrencyCode::new("EUR").ok() else {
        return;
    };
    let Some(usd) = CurrencyCode::new("USD").ok() else {
        return;
    };
    let Some(base_cost) = PlanCost::new(eur, 2, 100, 200).ok() else {
        return;
    };
    let Some(base) = cost_confirmation(request, endpoint, base_cost) else {
        return;
    };
    for cost in [
        PlanCost::new(usd, 2, 100, 200),
        PlanCost::new(eur, 3, 100, 200),
        PlanCost::new(eur, 2, 101, 200),
        PlanCost::new(eur, 2, 100, 201),
    ] {
        let Some(cost) = cost.ok() else { return };
        let Some(variant) = cost_confirmation(request, endpoint, cost) else {
            return;
        };
        assert!(different(base, variant));
    }
}

#[allow(clippy::too_many_arguments)]
fn confirmation(
    request: crate::operation::PreparedRequest<'static>,
    endpoint: crate::transport::EndpointIdentity<'static>,
    account: &'static [u8],
    tenant: &'static [u8],
    context: &'static [u8],
    issued: u64,
    expires: u64,
    replay: ReplayPolicy,
    attempts: u16,
    idempotency: Option<&'static [u8]>,
) -> Option<PlanConfirmation<'static, 'static>> {
    Some(PlanConfirmation::new(
        request,
        endpoint,
        PlanFingerprintScope::Value(account),
        PlanFingerprintScope::Value(tenant),
        PermitContext::new(context).ok()?,
        PermitValidity::new(time(issued), time(expires)).ok()?,
        replay,
        AttemptBudget::new(attempts).ok()?,
        PlanChange::ChangesState,
        None,
        idempotency
            .map(PermitIdempotencyKey::new)
            .transpose()
            .ok()?,
    ))
}

fn cost_confirmation(
    request: crate::operation::PreparedRequest<'static>,
    endpoint: crate::transport::EndpointIdentity<'static>,
    cost: PlanCost,
) -> Option<PlanConfirmation<'static, 'static>> {
    Some(PlanConfirmation::new(
        request,
        endpoint,
        PlanFingerprintScope::Value(b"account-a"),
        PlanFingerprintScope::Value(b"tenant-a"),
        PermitContext::new(b"review-42").ok()?,
        PermitValidity::new(time(100), time(200)).ok()?,
        ReplayPolicy::SingleAttempt,
        AttemptBudget::new(1).ok()?,
        PlanChange::ChangesState,
        Some(cost),
        None,
    ))
}

fn different(
    first: PlanConfirmation<'static, 'static>,
    second: PlanConfirmation<'static, 'static>,
) -> bool {
    let mut first_storage = [0_u8; 4096];
    let mut second_storage = [0_u8; 4096];
    let Ok(first) = build_canonical_plan(first, &mut first_storage) else {
        return false;
    };
    let Ok(second) = build_canonical_plan(second, &mut second_storage) else {
        return false;
    };
    !first.as_ref().matches(second.as_ref())
}

const fn time(value: u64) -> PermitTimestamp {
    PermitTimestamp::from_seconds(value)
}
