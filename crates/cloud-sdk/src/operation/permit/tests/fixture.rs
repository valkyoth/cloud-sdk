use core::sync::atomic::{AtomicUsize, Ordering};

use crate::authentication::{
    AsyncAuthenticatedTransport, AuthenticatedRequest, AuthenticationScopePolicy,
    BlockingAuthenticatedTransport, ScopeRequirement,
};
use crate::operation::{
    ContentTypePolicy, CostIntent, OperationId, OperationImpact, OperationMetadata,
    PreparedRequest, ProviderService, RequestIdPolicy, RequestSemantics, ResponseBodyPolicy,
    ResponsePolicy, RetryEligibility,
};
use crate::transport::{
    AsyncResponseStaging, BoundTransport, DeliveryPhase, EndpointIdentity, EndpointIdentityError,
    EndpointPolicy, EndpointScheme, HeaderSensitivity, MediaType, RawResponsePolicy, RequestTarget,
    ResponseCompletion, ResponseMediaPolicy, ResponseMetadata, ResponseWriter, StatusCode,
    TransportFailure, TransportRequest,
};
use crate::{Method, ProviderId, ServiceId};

static OK: [StatusCode; 1] = [StatusCode::OK];
static JSON: [MediaType<'static>; 1] = [MediaType::JSON];

pub fn endpoint() -> Option<EndpointIdentity<'static>> {
    EndpointIdentity::new(EndpointScheme::Https, "api.example.invalid", 443, "/v1").ok()
}

pub fn prepared(
    target: &'static str,
    impact: OperationImpact,
    cost: CostIntent,
) -> Option<PreparedRequest<'static>> {
    let endpoint = endpoint()?;
    prepared_with_policy(target, impact, cost, EndpointPolicy::fixed(endpoint))
}

pub fn prepared_with_policy<'a>(
    target: &'a str,
    impact: OperationImpact,
    cost: CostIntent,
    endpoint_policy: EndpointPolicy<'a>,
) -> Option<PreparedRequest<'a>> {
    let request = TransportRequest::new(Method::Post, RequestTarget::new(target).ok()?)
        .with_body(br#"{"name":"example"}"#);
    let metadata = OperationMetadata::new(
        impact,
        RequestSemantics::NonIdempotent,
        RetryEligibility::Never,
        cost,
        RequestIdPolicy::Discard,
    )
    .ok()?;
    let response = ResponsePolicy::new(
        &OK,
        ContentTypePolicy::Required(&JSON),
        ResponseBodyPolicy::Required,
        64,
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
        64,
        128,
        ResponseMediaPolicy::Required(&JSON),
        ResponseMediaPolicy::Required(&JSON),
        &[],
        0,
    )
    .ok()?;
    let prepared = PreparedRequest::new(
        request,
        ProviderService::new(
            ProviderId::new("example").ok()?,
            ServiceId::new("compute").ok()?,
            endpoint_policy,
        ),
        metadata,
        response,
        authentication,
        raw,
        crate::operation::RequestBodySensitivity::Public,
    )
    .ok()?;
    Some(prepared.with_operation_id(OperationId::new("create_resource").ok()?))
}

pub fn read_only(target: &'static str) -> Option<PreparedRequest<'static>> {
    let endpoint = endpoint()?;
    let request = TransportRequest::new(Method::Get, RequestTarget::new(target).ok()?);
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
        64,
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
        64,
        128,
        ResponseMediaPolicy::Required(&JSON),
        ResponseMediaPolicy::Required(&JSON),
        &[],
        0,
    )
    .ok()?;
    let prepared = PreparedRequest::new(
        request,
        ProviderService::new(
            ProviderId::new("example").ok()?,
            ServiceId::new("compute").ok()?,
            EndpointPolicy::fixed(endpoint),
        ),
        metadata,
        response,
        authentication,
        raw,
        crate::operation::RequestBodySensitivity::Public,
    )
    .ok()?;
    Some(prepared.with_operation_id(OperationId::new("get_resource").ok()?))
}

pub struct ClassifiedTransport {
    endpoint: EndpointIdentity<'static>,
    failure: Option<DeliveryPhase>,
    calls: AtomicUsize,
}

impl ClassifiedTransport {
    pub fn new(endpoint: EndpointIdentity<'static>, failure: Option<DeliveryPhase>) -> Self {
        Self {
            endpoint,
            failure,
            calls: AtomicUsize::new(0),
        }
    }

    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::Acquire)
    }

    fn send(&self, response: &mut ResponseWriter<'_>) -> Result<(), TransportFailure<()>> {
        self.calls.fetch_add(1, Ordering::AcqRel);
        if let Some(phase) = self.failure {
            return Err(match phase {
                DeliveryPhase::NotSent => TransportFailure::not_sent(()),
                DeliveryPhase::PossiblySent => TransportFailure::possibly_sent(()),
                DeliveryPhase::ResponseStarted => TransportFailure::response_started(()),
            });
        }
        let mut attempt = response
            .begin_attempt()
            .map_err(|_| TransportFailure::possibly_sent(()))?;
        attempt
            .body_mut()
            .map_err(|_| TransportFailure::possibly_sent(()))?
            .get_mut(..2)
            .ok_or_else(|| TransportFailure::possibly_sent(()))?
            .copy_from_slice(b"{}");
        attempt
            .headers_mut()
            .map_err(|_| TransportFailure::response_started(()))?
            .try_push(
                "content-type",
                b"application/json",
                HeaderSensitivity::Public,
            )
            .map_err(|_| TransportFailure::response_started(()))?;
        attempt
            .commit(StatusCode::OK, 2, ResponseMetadata::EMPTY)
            .map_err(|_| TransportFailure::response_started(()))
    }
}

impl BoundTransport for ClassifiedTransport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        Ok(self.endpoint)
    }
}

impl BlockingAuthenticatedTransport for ClassifiedTransport {
    type Error = TransportFailure<()>;

    fn send_authenticated(
        &self,
        _request: AuthenticatedRequest<'_, '_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        self.send(response)
    }
}

impl AsyncAuthenticatedTransport for ClassifiedTransport {
    type Error = TransportFailure<()>;

    async fn send_authenticated<'transport, 'request, 'policy, 'writer, 'buffer>(
        &'transport self,
        _request: AuthenticatedRequest<'request, 'policy>,
        mut response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
        'policy: 'writer,
        'buffer: 'writer,
    {
        self.calls.fetch_add(1, Ordering::AcqRel);
        if let Some(phase) = self.failure {
            return Err(match phase {
                DeliveryPhase::NotSent => TransportFailure::not_sent(()),
                DeliveryPhase::PossiblySent => TransportFailure::possibly_sent(()),
                DeliveryPhase::ResponseStarted => TransportFailure::response_started(()),
            });
        }
        response
            .body_mut()
            .map_err(|_| TransportFailure::possibly_sent(()))?
            .get_mut(..2)
            .ok_or_else(|| TransportFailure::possibly_sent(()))?
            .copy_from_slice(b"{}");
        response
            .headers_mut()
            .map_err(|_| TransportFailure::response_started(()))?
            .try_push(
                "content-type",
                b"application/json",
                HeaderSensitivity::Public,
            )
            .map_err(|_| TransportFailure::response_started(()))?;
        Ok(ResponseCompletion::new(
            StatusCode::OK,
            2,
            ResponseMetadata::EMPTY,
        ))
    }
}
