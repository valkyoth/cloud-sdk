use core::cell::Cell;
use core::future::{Future, pending};
use core::task::{Context, Poll, Waker};

use crate::authentication::{
    AuthenticatedRequest, AuthenticationScopePolicy, LocalAsyncAuthenticatedTransport,
    ScopeRequirement,
};
use crate::transport::{
    AsyncResponseStaging, BoundTransport, EndpointIdentity, EndpointIdentityError, EndpointPolicy,
    EndpointScheme, HeaderSensitivity, MediaType, RawResponsePolicy, RequestTarget,
    ResponseCompletion, ResponseMediaPolicy, ResponseMetadata, StatusCode, TransportRequest,
};
use crate::{Method, ProviderId, ServiceId};

use super::{
    ContentTypePolicy, CostIntent, OperationImpact, OperationMetadata, PreparedRequest,
    ProviderService, RequestBodySensitivity, RequestIdPolicy, RequestSemantics, ResponseBodyPolicy,
    ResponsePolicy, RetryEligibility,
};

static OK: [StatusCode; 1] = [StatusCode::OK];
static JSON: [MediaType<'static>; 1] = [MediaType::JSON];

struct PendingLocalAuthenticatedTransport {
    endpoint: EndpointIdentity<'static>,
    calls: Cell<u8>,
}

impl BoundTransport for PendingLocalAuthenticatedTransport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        Ok(self.endpoint)
    }
}

impl LocalAsyncAuthenticatedTransport for PendingLocalAuthenticatedTransport {
    type Error = ();

    async fn send_authenticated_local<'transport, 'request, 'policy, 'writer, 'buffer>(
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
        self.calls.set(self.calls.get().saturating_add(1));
        response.body_mut().map_err(|_| ())?.fill(0x5a);
        response
            .headers_mut()
            .map_err(|_| ())?
            .try_push("x-secret", b"partial", HeaderSensitivity::Sensitive)
            .map_err(|_| ())?;
        pending::<()>().await;
        Ok(ResponseCompletion::new(
            StatusCode::OK,
            2,
            ResponseMetadata::EMPTY,
        ))
    }
}

#[test]
fn local_async_prepared_cancellation_clears_complete_response_storage() {
    let Some((prepared, endpoint)) = prepared() else {
        unreachable!("security fixture construction failed");
    };
    let transport = PendingLocalAuthenticatedTransport {
        endpoint,
        calls: Cell::new(0),
    };
    let mut body = [0xa5_u8; 32];
    let mut headers = [0xa5_u8; 256];
    {
        let future = prepared.execute_local_async(&transport, &mut body, &mut headers);
        let mut future = core::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Pending
        ));
    }
    assert_eq!(transport.calls.get(), 1);
    assert_eq!(body, [0_u8; 32]);
    assert_eq!(headers, [0_u8; 256]);
}

fn prepared() -> Option<(PreparedRequest<'static>, EndpointIdentity<'static>)> {
    let endpoint =
        EndpointIdentity::new(EndpointScheme::Https, "api.example.invalid", 443, "/v1").ok()?;
    let request = TransportRequest::new(Method::Get, RequestTarget::new("/resources").ok()?);
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
        32,
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
        32,
        256,
        ResponseMediaPolicy::Required(&JSON),
        ResponseMediaPolicy::Required(&JSON),
        &[],
        0,
    )
    .ok()?;
    let service = ProviderService::new(
        ProviderId::new("example").ok()?,
        ServiceId::new("compute").ok()?,
        EndpointPolicy::fixed(endpoint),
    );
    Some((
        PreparedRequest::new(
            request,
            service,
            metadata,
            response,
            authentication,
            raw,
            RequestBodySensitivity::Public,
        )
        .ok()?,
        endpoint,
    ))
}
