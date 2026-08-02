#![no_main]

use cloud_sdk::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use cloud_sdk::operation::{
    ContentTypePolicy, CostIntent, OperationId, OperationImpact, OperationMetadata,
    PreparedRequest, ProviderService, RequestIdPolicy, RequestSemantics, ResponseBodyPolicy,
    ResponsePolicy, RetryEligibility,
};
use cloud_sdk::retry::{
    FingerprintScope, IdempotencyIntent, MaxAttempts, MonotonicDuration, MonotonicInstant,
    RetryController, RetryEvent, RetryPolicy, build_canonical_fingerprint,
};
use cloud_sdk::transport::{
    DeliveryPhase, EndpointIdentity, EndpointPolicy, EndpointScheme, MediaType, RawResponsePolicy,
    RequestTarget, ResponseMediaPolicy, StatusCode, TransportRequest,
};
use cloud_sdk::{Method, provider_id, service_id};
use libfuzzer_sys::fuzz_target;

static OK: [StatusCode; 1] = [StatusCode::OK];
static JSON: [MediaType<'static>; 1] = [MediaType::JSON];

fuzz_target!(|data: &[u8]| {
    let mut intent_source = data.to_vec();
    let _ = IdempotencyIntent::new(&mut intent_source);
    let Some(prepared) = prepared(data) else {
        return;
    };
    let Some(endpoint) = endpoint() else { return };
    let mut scratch = [0_u8; 2048];
    let scope = if data.first().is_some_and(|byte| byte & 1 == 1) {
        FingerprintScope::Value(data)
    } else {
        FingerprintScope::Absent
    };
    let Ok(fingerprint) = build_canonical_fingerprint(prepared, endpoint, scope, &mut scratch)
    else {
        return;
    };
    let attempts = data.get(1).copied().map_or(1, u16::from);
    let attempts = attempts.max(1);
    let Ok(max_attempts) = MaxAttempts::new(attempts) else {
        return;
    };
    let delay = u64::from(data.get(2).copied().unwrap_or_default());
    let elapsed = u64::from(data.get(3).copied().unwrap_or_default());
    let policy = RetryPolicy::new(
        max_attempts,
        MonotonicDuration::new(delay),
        MonotonicDuration::new(elapsed),
    );
    let Ok(mut owner) = RetryController::new(
        fingerprint.subject(),
        None,
        policy,
        MonotonicInstant::new(0),
    ) else {
        return;
    };
    let event = match data.get(4).copied().unwrap_or_default() % 4 {
        0 => RetryEvent::Transport(DeliveryPhase::NotSent),
        1 => RetryEvent::Transport(DeliveryPhase::PossiblySent),
        2 => RetryEvent::Transport(DeliveryPhase::ResponseStarted),
        _ => RetryEvent::Response(StatusCode::TOO_MANY_REQUESTS),
    };
    let decision = owner.decide_retry(
        event,
        fingerprint.subject(),
        MonotonicDuration::new(delay),
        MonotonicInstant::new(elapsed),
    );
    if let Ok(cloud_sdk::retry::RetryDecision::Retry(permit)) = decision {
        let _ = permit.authorize_execution(MonotonicInstant::new(elapsed));
    }
});

fn prepared(body: &[u8]) -> Option<PreparedRequest<'_>> {
    let endpoint = endpoint()?;
    let request =
        TransportRequest::new(Method::Get, RequestTarget::new("/resources").ok()?).with_body(body);
    let metadata = OperationMetadata::new(
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        CostIntent::NoKnownCost,
        RequestIdPolicy::Discard,
    )
    .ok()?;
    let response = ResponsePolicy::new(
        &OK,
        ContentTypePolicy::Required(&JSON),
        ResponseBodyPolicy::Required,
        1024,
    )
    .ok()?;
    let authentication = AuthenticationScopePolicy::new(
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let raw = RawResponsePolicy::new(
        1024,
        1024,
        ResponseMediaPolicy::Required(&JSON),
        ResponseMediaPolicy::Required(&JSON),
        &[],
        0,
    )
    .ok()?;
    let prepared = PreparedRequest::new(
        request,
        ProviderService::new(
            provider_id!("fuzz"),
            service_id!("retry"),
            EndpointPolicy::fixed(endpoint),
        ),
        metadata,
        response,
        authentication,
        raw,
    )
    .ok()?;
    Some(
        prepared
            .with_operation_id(OperationId::new("fuzz_retry").ok()?)
            .with_replayable_body(),
    )
}

fn endpoint() -> Option<EndpointIdentity<'static>> {
    EndpointIdentity::new(EndpointScheme::Https, "api.example.invalid", 443, "/v1").ok()
}
