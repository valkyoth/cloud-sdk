//! Source-bound OVHcloud cursor and schema-header conformance fixtures.

use core::future::Future;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::{Context, Poll, Waker};

use cloud_sdk::Method;
use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, AuthenticatedRequest, AuthenticationScopePolicy,
    BlockingAuthenticatedTransport, ScopeRequirement, ScopeValue,
};
use cloud_sdk::operation::{
    ContentTypePolicy, CostIntent, OperationId, OperationImpact, OperationMetadata,
    PreparedRequest, ProviderService, RequestIdPolicy, RequestSemantics, ResponseBodyPolicy,
    ResponsePolicy, RetryEligibility,
};
use cloud_sdk::pagination::{
    CursorDigest, CursorHistory, HeaderCursorNext, HeaderCursorPolicy, PaginationError,
    PaginationLimits,
};
use cloud_sdk::transport::{
    AsyncResponseStaging, BoundTransport, EndpointIdentity, EndpointIdentityError, EndpointPolicy,
    EndpointScheme, HeaderName, HeaderSensitivity, MediaType, RawResponsePolicy, RequestHeader,
    RequestHeaders, RequestTarget, ResponseCompletion, ResponseMediaPolicy, ResponseMetadata,
    ResponseWriter, StatusCode, TransportRequest,
};
use cloud_sdk::{ProviderId, ServiceId};

fn policy() -> HeaderCursorPolicy<'static> {
    HeaderCursorPolicy::new(
        OperationId::new("ovhcloud_iam_policy_list").unwrap_or_else(|_| unreachable!()),
        "X-Pagination-Cursor",
        "X-Pagination-Size",
        "X-Pagination-Cursor-Next",
        5,
    )
    .unwrap_or_else(|_| unreachable!())
}

fn limits() -> PaginationLimits {
    PaginationLimits::new(8, 1_000, 64).unwrap_or_else(|_| unreachable!())
}

fn endpoint() -> EndpointIdentity<'static> {
    EndpointIdentity::new(EndpointScheme::Https, "eu.api.ovh.com", 443, "/v2")
        .unwrap_or_else(|_| unreachable!())
}

fn alternate_endpoint() -> EndpointIdentity<'static> {
    EndpointIdentity::new(EndpointScheme::Https, "ca.api.ovh.com", 443, "/v2")
        .unwrap_or_else(|_| unreachable!())
}

fn authentication() -> AuthenticationScopePolicy<'static> {
    AuthenticationScopePolicy::new(
        ScopeRequirement::Required(ProviderId::new("ovhcloud").unwrap_or_else(|_| unreachable!())),
        ScopeRequirement::Required(ServiceId::new("iam").unwrap_or_else(|_| unreachable!())),
        ScopeRequirement::Required(endpoint()),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Required(ScopeValue::new("tenant-a").unwrap_or_else(|_| unreachable!())),
    )
}

fn prepared(operation: &'static str, target: &'static str) -> PreparedRequest<'static> {
    prepared_with_configuration(operation, target, RequestHeaders::EMPTY, true)
}

fn prepared_with_configuration<'headers>(
    operation: &'static str,
    target: &'static str,
    headers: RequestHeaders<'headers>,
    retain_cursor_header: bool,
) -> PreparedRequest<'headers> {
    static OK: [StatusCode; 1] = [StatusCode::OK];
    static JSON: [MediaType<'static>; 1] = [MediaType::JSON];
    let retained = [HeaderName::new("X-Pagination-Cursor-Next").unwrap_or_else(|_| unreachable!())];
    let retained = if retain_cursor_header {
        retained.as_slice()
    } else {
        &[]
    };
    let metadata = OperationMetadata::new(
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        CostIntent::NoKnownCost,
        RequestIdPolicy::Discard,
    )
    .unwrap_or_else(|_| unreachable!());
    let response = ResponsePolicy::new(
        &OK,
        ContentTypePolicy::Required(&JSON),
        ResponseBodyPolicy::Required,
        64,
    )
    .unwrap_or_else(|_| unreachable!());
    let raw = RawResponsePolicy::new(
        64,
        64,
        ResponseMediaPolicy::Required(&JSON),
        ResponseMediaPolicy::Required(&JSON),
        retained,
        0,
    )
    .unwrap_or_else(|_| unreachable!());
    PreparedRequest::new(
        TransportRequest::new(
            Method::Get,
            RequestTarget::new(target).unwrap_or_else(|_| unreachable!()),
        )
        .with_headers(headers),
        ProviderService::new(
            ProviderId::new("ovhcloud").unwrap_or_else(|_| unreachable!()),
            ServiceId::new("iam").unwrap_or_else(|_| unreachable!()),
            EndpointPolicy::fixed(endpoint()),
        ),
        metadata,
        response,
        authentication(),
        raw,
        cloud_sdk::operation::RequestBodySensitivity::Public,
    )
    .unwrap_or_else(|_| unreachable!())
    .with_operation_id(OperationId::new(operation).unwrap_or_else(|_| unreachable!()))
}

struct CursorTransport {
    calls: AtomicUsize,
    endpoint: EndpointIdentity<'static>,
}

impl CursorTransport {
    fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
            endpoint: endpoint(),
        }
    }

    fn at(endpoint: EndpointIdentity<'static>) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            endpoint,
        }
    }

    fn inspect_request(&self, request: AuthenticatedRequest<'_, '_>) -> Result<usize, ()> {
        let call = self
            .calls
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |value| {
                value.checked_add(1)
            })
            .map_err(|_| ())?;
        assert_eq!(request.policy(), authentication());
        let request = request.transport_request();
        assert_eq!(request.target().as_str(), "/iam/policy");
        assert_eq!(
            request
                .headers()
                .get("X-Pagination-Size")
                .map(|header| header.value().as_str()),
            Some("5")
        );
        let cursor = request.headers().get("X-Pagination-Cursor");
        if call == 0 {
            assert!(cursor.is_none());
        } else {
            assert_eq!(
                cursor.map(|header| (header.value().as_str(), header.sensitivity())),
                Some(("source-locked-cursor", HeaderSensitivity::Sensitive))
            );
        }
        Ok(call)
    }

    fn stage(
        &self,
        request: AuthenticatedRequest<'_, '_>,
        mut response: AsyncResponseStaging<'_, '_>,
    ) -> Result<ResponseCompletion, ()> {
        let call = self.inspect_request(request)?;
        response
            .body_mut()
            .map_err(|_| ())?
            .get_mut(..2)
            .ok_or(())?
            .copy_from_slice(b"{}");
        response
            .headers_mut()
            .map_err(|_| ())?
            .try_push(
                "content-type",
                b"application/json",
                HeaderSensitivity::Public,
            )
            .map_err(|_| ())?;
        if call == 0 {
            response
                .headers_mut()
                .map_err(|_| ())?
                .try_push(
                    "X-Pagination-Cursor-Next",
                    b"source-locked-cursor",
                    HeaderSensitivity::Sensitive,
                )
                .map_err(|_| ())?;
        }
        Ok(ResponseCompletion::new(
            StatusCode::OK,
            2,
            ResponseMetadata::EMPTY,
        ))
    }
}

impl BoundTransport for CursorTransport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        Ok(self.endpoint)
    }
}

impl BlockingAuthenticatedTransport for CursorTransport {
    type Error = ();

    fn send_authenticated(
        &self,
        request: AuthenticatedRequest<'_, '_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        let call = self.inspect_request(request)?;
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
                HeaderSensitivity::Public,
            )
            .map_err(|_| ())?;
        if call == 0 {
            attempt
                .headers_mut()
                .map_err(|_| ())?
                .try_push(
                    "X-Pagination-Cursor-Next",
                    b"source-locked-cursor",
                    HeaderSensitivity::Sensitive,
                )
                .map_err(|_| ())?;
        }
        attempt
            .commit(StatusCode::OK, 2, ResponseMetadata::EMPTY)
            .map_err(|_| ())
    }
}

impl AsyncAuthenticatedTransport for CursorTransport {
    type Error = ();

    async fn send_authenticated<'transport, 'request, 'policy, 'writer, 'buffer>(
        &'transport self,
        request: AuthenticatedRequest<'request, 'policy>,
        response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
        'policy: 'writer,
        'buffer: 'writer,
    {
        self.stage(request, response)
    }
}

fn run_ready<F: Future>(future: F) -> F::Output {
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(output) => output,
        Poll::Pending => unreachable!("fixture future unexpectedly returned pending"),
    }
}

#[test]
fn source_locked_header_cursor_round_trip_and_terminal_signal_are_exact() {
    let session = policy()
        .bind(prepared("ovhcloud_iam_policy_list", "/iam/policy"))
        .unwrap_or_else(|_| unreachable!());
    let transport = CursorTransport::new();
    let mut body = [0_u8; 64];
    let mut response_headers = [0_u8; 256];
    let mut decimal = [0xa5_u8; 20];
    let mut transfer = [0xa5_u8; 64];
    let mut destination = [0xa5_u8; 64];
    let page = session
        .execute_blocking(
            &transport,
            &mut body,
            &mut response_headers,
            &mut decimal,
            &mut transfer,
            &mut destination,
            limits(),
        )
        .unwrap_or_else(|_| unreachable!());
    let (response, next) = page.into_parts();
    drop(response);
    let HeaderCursorNext::Continue(continuation) = next else {
        unreachable!("source-locked continuation became terminal");
    };
    {
        let mut history_storage = [0_u8; 256];
        let mut history =
            CursorHistory::new(&mut history_storage, 4).unwrap_or_else(|_| unreachable!());
        let digest = CursorDigest::new([0x42; 32]);
        assert_eq!(continuation.observe_history(&mut history, digest), Ok(()));
        assert_eq!(
            continuation.observe_history(&mut history, digest),
            Err(PaginationError::CursorCycle)
        );
    }
    let mut next_body = [0_u8; 64];
    let mut next_headers = [0_u8; 256];
    let mut next_decimal = [0xa5_u8; 20];
    let mut next_transfer = [0xa5_u8; 64];
    let mut next_destination = [0xa5_u8; 64];
    let replacement = CursorTransport::at(alternate_endpoint());
    {
        let result = continuation.execute_blocking(
            &replacement,
            &mut next_body,
            &mut next_headers,
            &mut next_decimal,
            &mut next_transfer,
            &mut next_destination,
            limits(),
        );
        assert!(matches!(
            result,
            Err(
                cloud_sdk::pagination::HeaderCursorExecutionError::Pagination(
                    PaginationError::EndpointMismatch
                )
            )
        ));
    }
    assert_eq!(replacement.calls.load(Ordering::Acquire), 0);
    assert_eq!(next_body, [0; 64]);
    assert_eq!(next_headers, [0; 256]);
    let terminal = continuation
        .execute_blocking(
            &transport,
            &mut next_body,
            &mut next_headers,
            &mut next_decimal,
            &mut next_transfer,
            &mut next_destination,
            limits(),
        )
        .unwrap_or_else(|_| unreachable!());
    let (response, next) = terminal.into_parts();
    assert!(next.is_complete());
    drop(response);
    assert_eq!(transport.calls.load(Ordering::Acquire), 2);
    assert_eq!(decimal, [0; 20]);
    assert_eq!(transfer, [0; 64]);
    assert_eq!(next_decimal, [0; 20]);
    assert_eq!(next_transfer, [0; 64]);
}

#[test]
fn cursor_policy_rejects_another_operation_before_dispatch() {
    let mismatched = prepared("ovhcloud_iam_identity_list", "/iam/identity");
    assert!(matches!(
        policy().bind(mismatched),
        Err(PaginationError::OperationMismatch)
    ));

    let case_identical_headers = HeaderCursorPolicy::new(
        OperationId::new("ovhcloud_iam_policy_list").unwrap_or_else(|_| unreachable!()),
        "x-pagination-cursor",
        "x-pagination-size",
        "x-pagination-cursor-next",
        5,
    )
    .unwrap_or_else(|_| unreachable!());
    assert!(matches!(
        case_identical_headers.bind(prepared("ovhcloud_iam_identity_list", "/iam/identity")),
        Err(PaginationError::OperationMismatch)
    ));
    let missing_retention = prepared_with_configuration(
        "ovhcloud_iam_policy_list",
        "/iam/policy",
        RequestHeaders::EMPTY,
        false,
    );
    assert!(matches!(
        policy().bind(missing_retention),
        Err(PaginationError::ResponseHeaderNotAdmitted)
    ));
}

#[test]
fn async_execution_modes_use_the_same_bound_initial_request() {
    let session = policy()
        .bind(prepared("ovhcloud_iam_policy_list", "/iam/policy"))
        .unwrap_or_else(|_| unreachable!());
    let async_transport = CursorTransport::new();
    let mut body = [0_u8; 64];
    let mut headers = [0_u8; 256];
    let mut decimal = [0xa5_u8; 20];
    let mut transfer = [0xa5_u8; 64];
    let mut cursor = [0xa5_u8; 64];
    let page = run_ready(session.execute_async(
        &async_transport,
        &mut body,
        &mut headers,
        &mut decimal,
        &mut transfer,
        &mut cursor,
        limits(),
    ))
    .unwrap_or_else(|_| unreachable!());
    assert!(matches!(page.into_parts().1, HeaderCursorNext::Continue(_)));

    let local_transport = CursorTransport::new();
    let mut local_body = [0_u8; 64];
    let mut local_headers = [0_u8; 256];
    let mut local_decimal = [0xa5_u8; 20];
    let mut local_transfer = [0xa5_u8; 64];
    let mut local_cursor = [0xa5_u8; 64];
    let page = run_ready(session.execute_local_async(
        &local_transport,
        &mut local_body,
        &mut local_headers,
        &mut local_decimal,
        &mut local_transfer,
        &mut local_cursor,
        limits(),
    ))
    .unwrap_or_else(|_| unreachable!());
    assert!(matches!(page.into_parts().1, HeaderCursorNext::Continue(_)));
}

#[test]
fn conflicting_prepared_headers_fail_before_dispatch_and_clear_all_buffers() {
    let entries = [RequestHeader::new("X-Pagination-Size", "9").unwrap_or_else(|_| unreachable!())];
    let headers = RequestHeaders::new(&entries).unwrap_or_else(|_| unreachable!());
    let session = policy()
        .bind(prepared_with_configuration(
            "ovhcloud_iam_policy_list",
            "/iam/policy",
            headers,
            true,
        ))
        .unwrap_or_else(|_| unreachable!());
    let transport = CursorTransport::new();
    let mut body = [0xa5_u8; 64];
    let mut response_headers = [0xa5_u8; 256];
    let mut decimal = [0xa5_u8; 20];
    let mut transfer = [0xa5_u8; 64];
    let mut cursor = [0xa5_u8; 64];
    {
        let result = session.execute_blocking(
            &transport,
            &mut body,
            &mut response_headers,
            &mut decimal,
            &mut transfer,
            &mut cursor,
            limits(),
        );
        assert!(matches!(
            result,
            Err(
                cloud_sdk::pagination::HeaderCursorExecutionError::Pagination(
                    PaginationError::RequestHeaderConflict
                )
            )
        ));
    }
    assert_eq!(transport.calls.load(Ordering::Acquire), 0);
    assert_eq!(body, [0; 64]);
    assert_eq!(response_headers, [0; 256]);
    assert_eq!(decimal, [0; 20]);
    assert_eq!(transfer, [0; 64]);
    assert_eq!(cursor, [0; 64]);
}
