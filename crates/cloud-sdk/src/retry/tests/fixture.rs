use core::cell::Cell;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::authentication::{
    AsyncAuthenticatedTransport, AuthenticatedRequest, AuthenticationScopePolicy,
    BlockingAuthenticatedTransport, LocalAsyncAuthenticatedTransport, ScopeRequirement,
};
use crate::operation::{
    BodyReplayability, ContentTypePolicy, CostIntent, OperationId, OperationImpact,
    OperationMetadata, PreparedRequest, ProviderService, RequestIdPolicy, RequestSemantics,
    ResponseBodyPolicy, ResponsePolicy, RetryEligibility,
};
use crate::transport::{
    EndpointIdentity, EndpointPolicy, EndpointScheme, MediaType, RawResponsePolicy, RequestTarget,
    ResponseMediaPolicy, ResponseMetadata, ResponseWriter, StatusCode, TransportRequest,
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
    semantics: RequestSemantics,
    eligibility: RetryEligibility,
    replayability: BodyReplayability,
) -> Option<PreparedRequest<'static>> {
    let endpoint = endpoint()?;
    let request =
        TransportRequest::new(Method::Post, RequestTarget::new(target).ok()?).with_body(b"{}");
    let metadata = OperationMetadata::new(
        impact,
        semantics,
        eligibility,
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
        64,
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
    )
    .ok()?
    .with_operation_id(OperationId::new("create_server").ok()?);
    Some(match replayability {
        BodyReplayability::NotReplayable => prepared,
        BodyReplayability::Replayable => prepared.with_replayable_body(),
    })
}

pub struct RecordingTransport {
    endpoint: EndpointIdentity<'static>,
    calls: AtomicUsize,
}

impl RecordingTransport {
    pub fn new(endpoint: EndpointIdentity<'static>) -> Self {
        Self {
            endpoint,
            calls: AtomicUsize::new(0),
        }
    }

    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::Acquire)
    }

    fn send_inner(&self, response: &mut ResponseWriter<'_>) -> Result<(), ()> {
        self.calls.fetch_add(1, Ordering::AcqRel);
        let mut attempt = response.begin_attempt().map_err(|_| ())?;
        let body = attempt.body_mut().map_err(|_| ())?;
        body.get_mut(..2).ok_or(())?.copy_from_slice(b"{}");
        attempt
            .headers_mut()
            .map_err(|_| ())?
            .try_push(
                "content-type",
                b"application/json",
                crate::transport::HeaderSensitivity::Public,
            )
            .map_err(|_| ())?;
        attempt
            .commit(StatusCode::OK, 2, ResponseMetadata::EMPTY)
            .map_err(|_| ())
    }
}

impl crate::transport::BoundTransport for RecordingTransport {
    fn endpoint_identity(
        &self,
    ) -> Result<EndpointIdentity<'_>, crate::transport::EndpointIdentityError> {
        Ok(self.endpoint)
    }
}

impl BlockingAuthenticatedTransport for RecordingTransport {
    type Error = ();

    fn send_authenticated(
        &self,
        _request: AuthenticatedRequest<'_, '_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        self.send_inner(response)
    }
}

impl AsyncAuthenticatedTransport for RecordingTransport {
    type Error = ();

    async fn send_authenticated<'transport, 'request, 'policy, 'writer>(
        &'transport self,
        _request: AuthenticatedRequest<'request, 'policy>,
        response: &'writer mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
        'policy: 'writer,
    {
        self.send_inner(response)
    }
}

pub struct LocalRecordingTransport {
    endpoint: EndpointIdentity<'static>,
    calls: Cell<usize>,
}

impl LocalRecordingTransport {
    pub fn new(endpoint: EndpointIdentity<'static>) -> Self {
        Self {
            endpoint,
            calls: Cell::new(0),
        }
    }

    pub fn calls(&self) -> usize {
        self.calls.get()
    }
}

impl crate::transport::BoundTransport for LocalRecordingTransport {
    fn endpoint_identity(
        &self,
    ) -> Result<EndpointIdentity<'_>, crate::transport::EndpointIdentityError> {
        Ok(self.endpoint)
    }
}

impl LocalAsyncAuthenticatedTransport for LocalRecordingTransport {
    type Error = ();

    async fn send_authenticated_local<'transport, 'request, 'policy, 'writer>(
        &'transport self,
        _request: AuthenticatedRequest<'request, 'policy>,
        response: &'writer mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
        'policy: 'writer,
    {
        self.calls.set(self.calls.get().saturating_add(1));
        let mut attempt = response.begin_attempt().map_err(|_| ())?;
        attempt
            .body_mut()
            .map_err(|_| ())?
            .get_mut(..2)
            .ok_or(())?
            .copy_from_slice(b"{}");
        attempt
            .headers_mut()
            .map_err(|_| ())?
            .try_push(
                "content-type",
                b"application/json",
                crate::transport::HeaderSensitivity::Public,
            )
            .map_err(|_| ())?;
        attempt
            .commit(StatusCode::OK, 2, ResponseMetadata::EMPTY)
            .map_err(|_| ())
    }
}
