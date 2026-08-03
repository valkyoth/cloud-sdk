use super::super::{
    PaginationError, PaginationLimits, ProviderLinkBinding, ProviderLinkExecutionError,
    ValidatedProviderLink,
};
use super::{DebugBuffer, assert_redacted};
use core::fmt::Write;
use core::future::Future;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::{Context, Poll, Waker};

use crate::Method;
use crate::authentication::{
    AsyncAuthenticatedTransport, AuthenticatedRequest, AuthenticationScopePolicy,
    BlockingAuthenticatedTransport, ScopeRequirement,
};
use crate::operation::OperationId;
use crate::transport::{
    AsyncResponseStaging, BoundTransport, EndpointIdentity, EndpointIdentityError, EndpointScheme,
    RawResponsePolicy, RequestPath, RequestTarget, ResponseBuffer, ResponseCompletion,
    ResponseMediaPolicy, ResponseMetadata, ResponseWriter, StatusCode,
};

struct TestTransport {
    endpoint: EndpointIdentity<'static>,
    expected_target: &'static str,
    fail: bool,
    calls: AtomicU32,
}

impl BoundTransport for TestTransport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        Ok(self.endpoint)
    }
}

impl TestTransport {
    fn record(&self, request: AuthenticatedRequest<'_, '_>) {
        let target = request.transport_request().target();
        assert_eq!(target.as_str(), self.expected_target);
        assert!(matches!(
            target.query(),
            crate::transport::RequestQuery::ProviderLink(_)
        ));
        let mut output = [0xa5_u8; 128];
        assert_eq!(
            RequestTarget::assemble(
                RequestPath::new("/v2/account").unwrap_or_else(|_| unreachable!()),
                target.query(),
                &mut output,
            ),
            Err(crate::transport::RequestTargetError::ProviderLinkQueryCannotAssemble)
        );
        assert_eq!(output, [0xa5; 128]);
        self.calls.fetch_add(1, Ordering::Relaxed);
    }

    fn commit_empty(response: &mut ResponseWriter<'_>) -> Result<(), &'static str> {
        let mut attempt = response.begin_attempt().map_err(|_| "response rejected")?;
        attempt
            .commit(StatusCode::NO_CONTENT, 0, ResponseMetadata::EMPTY)
            .map_err(|_| "response rejected")
    }
}

impl BlockingAuthenticatedTransport for TestTransport {
    type Error = &'static str;

    fn send_authenticated(
        &self,
        request: AuthenticatedRequest<'_, '_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        self.record(request);
        if self.fail {
            Err("secret transport detail")
        } else {
            Self::commit_empty(response)
        }
    }
}

impl AsyncAuthenticatedTransport for TestTransport {
    type Error = &'static str;

    async fn send_authenticated<'transport, 'request, 'policy, 'writer, 'buffer>(
        &'transport self,
        request: AuthenticatedRequest<'request, 'policy>,
        _response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
        'policy: 'writer,
        'buffer: 'writer,
    {
        self.record(request);
        if self.fail {
            Err("secret transport detail")
        } else {
            Ok(ResponseCompletion::new(
                StatusCode::NO_CONTENT,
                0,
                ResponseMetadata::EMPTY,
            ))
        }
    }
}

struct UnboundTransport;

impl BoundTransport for UnboundTransport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        Err(EndpointIdentityError::UnboundTransport)
    }
}

impl BlockingAuthenticatedTransport for UnboundTransport {
    type Error = ();

    fn send_authenticated(
        &self,
        _request: AuthenticatedRequest<'_, '_>,
        _response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        unreachable!()
    }
}

fn operation(value: &'static str) -> OperationId {
    OperationId::new(value).unwrap_or_else(|_| unreachable!())
}

fn binding() -> ProviderLinkBinding<'static> {
    let endpoint = endpoint();
    let path = RequestPath::new("/v2/droplets").unwrap_or_else(|_| unreachable!());
    ProviderLinkBinding::new(endpoint, Method::Get, operation("list_droplets"), path)
}

fn endpoint() -> EndpointIdentity<'static> {
    EndpointIdentity::new(EndpointScheme::Https, "api.digitalocean.com", 443, "/v2")
        .unwrap_or_else(|_| unreachable!())
}

fn transport(expected_target: &'static str) -> TestTransport {
    TestTransport {
        endpoint: endpoint(),
        expected_target,
        fail: false,
        calls: AtomicU32::new(0),
    }
}

fn limits() -> PaginationLimits {
    PaginationLimits::new(8, 1_000, 512).unwrap_or_else(|_| unreachable!())
}

fn authentication() -> AuthenticationScopePolicy<'static> {
    AuthenticationScopePolicy::new(
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    )
}

fn response_policy() -> RawResponsePolicy<'static> {
    RawResponsePolicy::new(
        0,
        0,
        ResponseMediaPolicy::Forbidden,
        ResponseMediaPolicy::Forbidden,
        &[],
        0,
    )
    .unwrap_or_else(|_| unreachable!())
}

#[test]
fn digitalocean_absolute_link_preserves_raw_query_order_duplicates_and_percent_encoding() {
    let expected = "/v2/droplets?tag_name=a%2fb&filter=a+b==&raw=%41&page=2&page=3";
    let mut source =
        *b"https://api.digitalocean.com/v2/droplets?tag_name=a%2fb&filter=a+b==&raw=%41&page=2&page=3";
    let mut storage = [0xa5_u8; 128];
    let transport = transport(expected);
    let mut response_storage = [];
    let mut header_storage = [];
    let mut response = ResponseBuffer::new(&mut response_storage, 0, &mut header_storage);
    {
        let link =
            ValidatedProviderLink::transfer_from(&mut source, &mut storage, binding(), limits())
                .unwrap_or_else(|_| unreachable!());
        let observed = link.execute_blocking(
            &transport,
            Method::Get,
            operation("list_droplets"),
            authentication(),
            response_policy(),
            response.writer(),
        );
        assert_eq!(observed, Ok(()));
        assert_eq!(transport.calls.load(Ordering::Relaxed), 1);
        assert_redacted(&link);
    }
    assert!(source.iter().all(|byte| *byte == 0));
    assert_eq!(storage, [0; 128]);
}

#[test]
fn origin_form_link_remains_operation_bound() {
    let mut source = *b"/v2/droplets?page=2&per_page=20";
    let mut storage = [0_u8; 64];
    let link = ValidatedProviderLink::transfer_from(&mut source, &mut storage, binding(), limits())
        .unwrap_or_else(|_| unreachable!());
    let transport = transport("/v2/droplets?page=2&per_page=20");
    let mut response_storage = [];
    let mut header_storage = [];
    let mut response = ResponseBuffer::new(&mut response_storage, 0, &mut header_storage);
    assert_eq!(
        link.execute_blocking(
            &transport,
            Method::Get,
            operation("list_droplets"),
            authentication(),
            response_policy(),
            response.writer(),
        ),
        Ok(())
    );
    assert_eq!(
        link.execute_blocking(
            &transport,
            Method::Post,
            operation("list_droplets"),
            authentication(),
            response_policy(),
            response.writer(),
        ),
        Err(ProviderLinkExecutionError::Pagination(
            PaginationError::ProviderLinkMethodChanged
        ))
    );
    assert_eq!(
        link.execute_blocking(
            &transport,
            Method::Get,
            operation("delete_droplet"),
            authentication(),
            response_policy(),
            response.writer(),
        ),
        Err(ProviderLinkExecutionError::Pagination(
            PaginationError::ProviderLinkOperationChanged
        ))
    );
    assert_eq!(transport.calls.load(Ordering::Relaxed), 1);
}

#[test]
fn rejects_use_through_a_different_bound_transport_endpoint() {
    let mut source = *b"/v2/droplets?page=2";
    let mut storage = [0_u8; 64];
    let link = ValidatedProviderLink::transfer_from(&mut source, &mut storage, binding(), limits())
        .unwrap_or_else(|_| unreachable!());
    let other = EndpointIdentity::new(EndpointScheme::Https, "api.example.com", 443, "/v2")
        .unwrap_or_else(|_| unreachable!());
    let other = TestTransport {
        endpoint: other,
        expected_target: "/v2/droplets?page=2",
        fail: false,
        calls: AtomicU32::new(0),
    };
    let mut response_storage = [];
    let mut header_storage = [];
    let mut response = ResponseBuffer::new(&mut response_storage, 0, &mut header_storage);

    assert_eq!(
        link.execute_blocking(
            &other,
            Method::Get,
            operation("list_droplets"),
            authentication(),
            response_policy(),
            response.writer(),
        ),
        Err(ProviderLinkExecutionError::Pagination(
            PaginationError::ProviderLinkAuthorityChanged
        ))
    );
    assert_eq!(other.calls.load(Ordering::Relaxed), 0);
    assert_eq!(
        link.execute_blocking(
            &UnboundTransport,
            Method::Get,
            operation("list_droplets"),
            authentication(),
            response_policy(),
            response.writer(),
        ),
        Err(ProviderLinkExecutionError::Pagination(
            PaginationError::ProviderLinkAuthorityChanged
        ))
    );
}

#[test]
fn async_execution_couples_endpoint_validation_and_dispatch() {
    let mut source = *b"/v2/droplets?page=2";
    let mut storage = [0_u8; 64];
    let link = ValidatedProviderLink::transfer_from(&mut source, &mut storage, binding(), limits())
        .unwrap_or_else(|_| unreachable!());
    let transport = transport("/v2/droplets?page=2");
    let mut response_storage = [];
    let mut header_storage = [];
    let mut response = ResponseBuffer::new(&mut response_storage, 0, &mut header_storage);
    let mut future = core::pin::pin!(link.execute_async(
        &transport,
        Method::Get,
        operation("list_droplets"),
        authentication(),
        response_policy(),
        response.writer(),
    ));
    let mut context = Context::from_waker(Waker::noop());

    assert_eq!(future.as_mut().poll(&mut context), Poll::Ready(Ok(())));
    assert_eq!(transport.calls.load(Ordering::Relaxed), 1);
}

#[test]
fn send_transport_provider_links_execute_through_local_async() {
    let mut source = *b"/v2/droplets?page=2";
    let mut storage = [0_u8; 64];
    let link = ValidatedProviderLink::transfer_from(&mut source, &mut storage, binding(), limits())
        .unwrap_or_else(|_| unreachable!());
    let transport = transport("/v2/droplets?page=2");
    let mut response_storage = [];
    let mut header_storage = [];
    let mut response = ResponseBuffer::new(&mut response_storage, 0, &mut header_storage);
    let mut future = core::pin::pin!(link.execute_local_async(
        &transport,
        Method::Get,
        operation("list_droplets"),
        authentication(),
        response_policy(),
        response.writer(),
    ));
    let mut context = Context::from_waker(Waker::noop());

    assert_eq!(future.as_mut().poll(&mut context), Poll::Ready(Ok(())));
    assert_eq!(transport.calls.load(Ordering::Relaxed), 1);
}

#[test]
fn execution_errors_flatten_and_redact_transport_details() {
    let mut source = *b"/v2/droplets?page=2";
    let mut storage = [0_u8; 64];
    let link = ValidatedProviderLink::transfer_from(&mut source, &mut storage, binding(), limits())
        .unwrap_or_else(|_| unreachable!());
    let transport = TestTransport {
        endpoint: endpoint(),
        expected_target: "/v2/droplets?page=2",
        fail: true,
        calls: AtomicU32::new(0),
    };
    let mut response_storage = [];
    let mut header_storage = [];
    let mut response = ResponseBuffer::new(&mut response_storage, 0, &mut header_storage);
    let result = link.execute_blocking(
        &transport,
        Method::Get,
        operation("list_droplets"),
        authentication(),
        response_policy(),
        response.writer(),
    );

    assert_eq!(
        result,
        Err(ProviderLinkExecutionError::Transport(
            "secret transport detail"
        ))
    );
    assert!(result.is_err());
    let error = result.err().unwrap_or_else(|| unreachable!());
    let mut debug = DebugBuffer::new();
    assert!(write!(&mut debug, "{error:?}").is_ok());
    assert_eq!(debug.as_str(), "Transport([redacted])");
    let mut display = DebugBuffer::new();
    assert!(write!(&mut display, "{error}").is_ok());
    assert_eq!(
        display.as_str(),
        "provider pagination link transport failed"
    );
    let _: &dyn core::error::Error = &error;
}

#[test]
fn execution_error_supports_non_debug_transport_errors() {
    struct NonDebugTransportError;

    fn require_error<T: core::error::Error>() {}

    require_error::<ProviderLinkExecutionError<NonDebugTransportError>>();
}

#[test]
fn rejects_scheme_authority_userinfo_fragment_and_operation_path_changes() {
    let cases: [(&[u8], PaginationError); 5] = [
        (
            b"http://api.digitalocean.com/v2/droplets?page=2",
            PaginationError::ProviderLinkSchemeChanged,
        ),
        (
            b"https://evil.example/v2/droplets?page=2",
            PaginationError::ProviderLinkAuthorityChanged,
        ),
        (
            b"https://user@api.digitalocean.com/v2/droplets?page=2",
            PaginationError::ProviderLinkUserinfo,
        ),
        (
            b"https://api.digitalocean.com/v2/droplets?page=2#next",
            PaginationError::ProviderLinkFragment,
        ),
        (
            b"https://api.digitalocean.com/v2/account?page=2",
            PaginationError::ProviderLinkPathChanged,
        ),
    ];
    for (value, expected) in cases {
        let mut source = [0_u8; 96];
        let Some(source) = source.get_mut(..value.len()) else {
            return;
        };
        source.copy_from_slice(value);
        let mut destination = [0xa5_u8; 128];
        assert!(matches!(
            ValidatedProviderLink::transfer_from(source, &mut destination, binding(), limits()),
            Err(error) if error == expected
        ));
        assert!(source.iter().all(|byte| *byte == 0));
        assert_eq!(destination, [0; 128]);
    }
}

#[test]
fn accepts_explicit_default_port_and_rejects_insufficient_output_atomically() {
    let mut source = *b"https://api.digitalocean.com:443/v2/droplets?page=2";
    let mut storage = [0_u8; 64];
    assert!(
        ValidatedProviderLink::transfer_from(&mut source, &mut storage, binding(), limits(),)
            .is_ok()
    );

    let mut source = *b"/v2/droplets?page=2";
    let mut output = [0xa5_u8; 4];
    assert!(matches!(
        ValidatedProviderLink::transfer_from(&mut source, &mut output, binding(), limits()),
        Err(PaginationError::OutputTooSmall)
    ));
    assert!(source.iter().all(|byte| *byte == 0));
    assert_eq!(output, [0; 4]);
}

#[test]
fn accepts_equivalent_ipv6_authority_and_rejects_invalid_raw_queries() {
    let endpoint = EndpointIdentity::new(EndpointScheme::Https, "[2001:db8::1]", 443, "/v2")
        .unwrap_or_else(|_| unreachable!());
    let path = RequestPath::new("/v2/droplets").unwrap_or_else(|_| unreachable!());
    let binding = ProviderLinkBinding::new(endpoint, Method::Get, operation("list_droplets"), path);
    let mut source = *b"https://[2001:0db8:0:0:0:0:0:1]/v2/droplets?page=2";
    let mut storage = [0_u8; 64];
    assert!(
        ValidatedProviderLink::transfer_from(&mut source, &mut storage, binding, limits()).is_ok()
    );

    for value in [
        b"/v2/droplets?cursor=%".as_slice(),
        b"/v2/droplets?cursor=%00".as_slice(),
        b"/v2/droplets?cursor=a b".as_slice(),
        b"/v2/droplets?cursor=a\\b".as_slice(),
    ] {
        let mut source = [0_u8; 32];
        let Some(source) = source.get_mut(..value.len()) else {
            return;
        };
        source.copy_from_slice(value);
        let mut destination = [0xa5_u8; 64];
        assert!(matches!(
            ValidatedProviderLink::transfer_from(source, &mut destination, binding, limits()),
            Err(PaginationError::InvalidProviderLink)
        ));
        assert!(source.iter().all(|byte| *byte == 0));
        assert_eq!(destination, [0; 64]);
    }
}
