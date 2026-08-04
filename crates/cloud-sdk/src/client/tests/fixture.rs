use core::sync::atomic::{AtomicUsize, Ordering};

use crate::authentication::{
    AsyncAuthenticatedTransport, AuthenticatedRequest, AuthenticationScopePolicy,
    BlockingAuthenticatedTransport, ScopeRequirement,
};
use crate::client::{CheckedDecodeError, ClientOperation, ClientResponse, ClientResponseKind};
use crate::operation::{
    ContentTypePolicy, CostIntent, OperationImpact, OperationMetadata, PreparationStorage,
    PrepareOperation, PreparedRequest, ProviderService, RequestIdPolicy, RequestSemantics,
    ResponseBodyPolicy, ResponsePolicy, RetryEligibility,
};
use crate::transport::{
    AsyncResponseStaging, BoundTransport, EndpointIdentity, EndpointIdentityError, EndpointPolicy,
    EndpointScheme, HeaderName, HeaderSensitivity, MediaType, RawResponsePolicy, RequestTarget,
    ResponseCompletion, ResponseMediaPolicy, ResponseMetadata, ResponseWriter, StatusCode,
    TransportRequest,
};
use crate::{
    Method, ProviderId, ProviderMarker, ServiceId, ServiceMarker, provider_id, service_id,
};

static OK_STATUS: [StatusCode; 1] = [StatusCode::OK];
static JSON_MEDIA: [MediaType<'static>; 1] = [MediaType::JSON];

enum ExampleProvider {}

impl ProviderMarker for ExampleProvider {
    const ID: ProviderId = provider_id!("example");
}

enum ExampleService {}

impl ServiceMarker for ExampleService {
    type Provider = ExampleProvider;
    const ID: ServiceId = service_id!("compute");
}

#[derive(Clone, Copy)]
pub(super) struct ExampleOperation {
    impact: OperationImpact,
}

impl ExampleOperation {
    pub(super) const fn read_only() -> Self {
        Self {
            impact: OperationImpact::ReadOnly,
        }
    }

    pub(super) const fn mutation() -> Self {
        Self {
            impact: OperationImpact::Mutation,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FixtureError {
    Invalid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DecodeError {
    Admission,
    UnexpectedStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Decoded {
    pub(super) status: u16,
    pub(super) body_len: usize,
    pub(super) provider_error: bool,
}

impl PrepareOperation for ExampleOperation {
    type Error = FixtureError;

    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        let (target_storage, _body_storage) = storage.into_parts();
        let target = target_storage.get_mut(..8).ok_or(FixtureError::Invalid)?;
        target.copy_from_slice(b"/servers");
        let target = core::str::from_utf8(target).map_err(|_| FixtureError::Invalid)?;
        let request = TransportRequest::new(
            Method::Get,
            RequestTarget::new(target).map_err(|_| FixtureError::Invalid)?,
        );
        let endpoint = endpoint().ok_or(FixtureError::Invalid)?;
        let metadata = OperationMetadata::new(
            self.impact,
            if self.impact == OperationImpact::ReadOnly {
                RequestSemantics::Safe
            } else {
                RequestSemantics::Idempotent
            },
            RetryEligibility::ExplicitPolicy,
            CostIntent::NoKnownCost,
            RequestIdPolicy::Discard,
        )
        .map_err(|_| FixtureError::Invalid)?;
        PreparedRequest::new(
            request,
            ProviderService::from_marker::<ExampleService>(EndpointPolicy::fixed(endpoint)),
            metadata,
            response_policy()?,
            authentication_policy(endpoint),
            raw_policy()?,
        )
        .map_err(|_| FixtureError::Invalid)
    }
}

impl ClientOperation for ExampleOperation {
    type Output = Decoded;
    type DecodeError = DecodeError;

    fn decode_response(
        &self,
        response: ClientResponse<'_, '_>,
    ) -> Result<Self::Output, Self::DecodeError> {
        match response.kind().map_err(|_| DecodeError::Admission)? {
            ClientResponseKind::Success => response
                .decode_success_owned(|checked, _| {
                    Ok::<_, ()>(Decoded {
                        status: checked.status().get(),
                        body_len: checked.body().len(),
                        provider_error: false,
                    })
                })
                .map_err(map_decode_error),
            ClientResponseKind::Error => response
                .decode_error_owned(|raw, _| {
                    Ok::<_, ()>(Decoded {
                        status: raw.status().get(),
                        body_len: raw.body().len(),
                        provider_error: true,
                    })
                })
                .map_err(map_decode_error),
            ClientResponseKind::Other => Err(DecodeError::UnexpectedStatus),
        }
    }
}

fn map_decode_error(_: CheckedDecodeError<()>) -> DecodeError {
    DecodeError::Admission
}

fn response_policy() -> Result<ResponsePolicy, FixtureError> {
    ResponsePolicy::new(
        &OK_STATUS,
        ContentTypePolicy::Required(&JSON_MEDIA),
        ResponseBodyPolicy::Required,
        64,
    )
    .map_err(|_| FixtureError::Invalid)
}

fn raw_policy() -> Result<RawResponsePolicy<'static>, FixtureError> {
    let content_type = HeaderName::new("content-type").map_err(|_| FixtureError::Invalid)?;
    RawResponsePolicy::new(
        64,
        64,
        ResponseMediaPolicy::Required(&JSON_MEDIA),
        ResponseMediaPolicy::Required(&JSON_MEDIA),
        &[content_type],
        4,
    )
    .map_err(|_| FixtureError::Invalid)
}

const fn authentication_policy(
    endpoint: EndpointIdentity<'static>,
) -> AuthenticationScopePolicy<'static> {
    AuthenticationScopePolicy::new(
        ScopeRequirement::Required(ExampleProvider::ID),
        ScopeRequirement::Required(ExampleService::ID),
        ScopeRequirement::Required(endpoint),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    )
}

pub(super) fn endpoint() -> Option<EndpointIdentity<'static>> {
    EndpointIdentity::new(EndpointScheme::Https, "api.example.invalid", 443, "/v1").ok()
}

pub(super) fn other_endpoint() -> Option<EndpointIdentity<'static>> {
    EndpointIdentity::new(EndpointScheme::Https, "other.example.invalid", 443, "/v1").ok()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TransportError {
    Authentication,
    Response,
}

pub(super) struct FakeTransport {
    endpoint: EndpointIdentity<'static>,
    status: StatusCode,
    accept_auth: bool,
    calls: AtomicUsize,
}

impl FakeTransport {
    pub(super) const fn success(endpoint: EndpointIdentity<'static>) -> Self {
        Self::new(endpoint, StatusCode::OK, true)
    }

    pub(super) const fn provider_error(endpoint: EndpointIdentity<'static>) -> Self {
        Self::new(endpoint, StatusCode::TOO_MANY_REQUESTS, true)
    }

    pub(super) const fn auth_mismatch(endpoint: EndpointIdentity<'static>) -> Self {
        Self::new(endpoint, StatusCode::OK, false)
    }

    const fn new(
        endpoint: EndpointIdentity<'static>,
        status: StatusCode,
        accept_auth: bool,
    ) -> Self {
        Self {
            endpoint,
            status,
            accept_auth,
            calls: AtomicUsize::new(0),
        }
    }

    pub(super) fn calls(&self) -> usize {
        self.calls.load(Ordering::Acquire)
    }

    fn validate(&self, request: AuthenticatedRequest<'_, '_>) -> Result<(), TransportError> {
        self.calls.fetch_add(1, Ordering::AcqRel);
        let Some(endpoint) = endpoint() else {
            return Err(TransportError::Authentication);
        };
        if !self.accept_auth || request.policy() != authentication_policy(endpoint) {
            return Err(TransportError::Authentication);
        }
        Ok(())
    }
}

impl BoundTransport for FakeTransport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        Ok(self.endpoint)
    }
}

impl BlockingAuthenticatedTransport for FakeTransport {
    type Error = TransportError;

    fn send_authenticated(
        &self,
        request: AuthenticatedRequest<'_, '_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        self.validate(request)?;
        let mut attempt = response
            .begin_attempt()
            .map_err(|_| TransportError::Response)?;
        write_response(self.status, &mut attempt.staging())?;
        attempt
            .commit(self.status, 2, ResponseMetadata::EMPTY)
            .map_err(|_| TransportError::Response)
    }
}

impl AsyncAuthenticatedTransport for FakeTransport {
    type Error = TransportError;

    async fn send_authenticated<'transport, 'request, 'policy, 'writer, 'buffer>(
        &'transport self,
        request: AuthenticatedRequest<'request, 'policy>,
        mut response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
        'policy: 'writer,
        'buffer: 'writer,
    {
        self.validate(request)?;
        write_response(self.status, &mut response)?;
        Ok(ResponseCompletion::new(
            self.status,
            2,
            ResponseMetadata::EMPTY,
        ))
    }
}

fn write_response(
    _status: StatusCode,
    response: &mut AsyncResponseStaging<'_, '_>,
) -> Result<(), TransportError> {
    response
        .body_mut()
        .map_err(|_| TransportError::Response)?
        .get_mut(..2)
        .ok_or(TransportError::Response)?
        .copy_from_slice(b"{}");
    response
        .headers_mut()
        .map_err(|_| TransportError::Response)?
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .map_err(|_| TransportError::Response)
}

pub(super) struct PendingTransport {
    endpoint: EndpointIdentity<'static>,
}

impl PendingTransport {
    pub(super) const fn new(endpoint: EndpointIdentity<'static>) -> Self {
        Self { endpoint }
    }
}

impl BoundTransport for PendingTransport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        Ok(self.endpoint)
    }
}

impl AsyncAuthenticatedTransport for PendingTransport {
    type Error = TransportError;

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
        response
            .body_mut()
            .map_err(|_| TransportError::Response)?
            .fill(0xa5);
        core::future::pending().await
    }
}
