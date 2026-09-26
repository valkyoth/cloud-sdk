use crate::{
    client::*,
    discovery::{DiscoveryExecutionError, tests::Fixture as _},
    wire::{IdentifyingUserAgent, reset_test_gate},
};
use alloc::rc::Rc;
use cloud_sdk::{Method, transport::*};
use core::{
    future::Future,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    task::{Context, Poll, Waker},
};
mod cargo;
mod catalog;
mod coverage;
mod lifecycle;
mod operations;
mod publish;
mod secret_paths;

pub(crate) const UA: &str = "unified-tests/1 (tests@example.org)";
pub(crate) fn identity() -> IdentifyingUserAgent<'static> {
    IdentifyingUserAgent::new(UA).fixture("identity")
}
pub(crate) struct Fixture<'a> {
    pub method: Method,
    pub target: &'a str,
    pub payload: &'a [u8],
    pub wire: &'a [u8],
    pub status: u16,
    pub auth: bool,
    pub expected_auth: Option<&'a [u8]>,
    pub media: bool,
    pub retry: Option<&'a [u8]>,
    pub encoding: Option<&'a [u8]>,
    pub pending: bool,
    pub fail: bool,
    pub changed: AtomicBool,
    pub calls: AtomicUsize,
}
impl<'a> Fixture<'a> {
    pub fn new(
        method: Method,
        target: &'a str,
        payload: &'a [u8],
        wire: &'a [u8],
        status: u16,
    ) -> Self {
        Self {
            method,
            target,
            payload,
            wire,
            status,
            auth: true,
            expected_auth: None,
            media: status != 204,
            retry: None,
            encoding: None,
            pending: false,
            fail: false,
            changed: AtomicBool::new(false),
            calls: AtomicUsize::new(0),
        }
    }
    fn verify(
        &self,
        request: TransportRequest<'_>,
        auth: Option<HeaderValue<'_>>,
        policy: RawResponsePolicy<'_>,
    ) {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.method(), self.method);
        assert_eq!(request.target().as_str(), self.target);
        assert!(request.body() == self.payload, "request body differs");
        assert_eq!(auth.is_some(), self.auth);
        if let Some(expected) = self.expected_auth {
            assert!(
                auth.fixture("explicit authorization").as_str().as_bytes() == expected,
                "authorization differs"
            );
        }
        assert!(request.headers().get("authorization").is_none());
        assert!(request.headers().get("cookie").is_none());
        assert_eq!(
            request.headers().get("content-type").is_some(),
            !self.payload.is_empty()
        );
        assert!(policy.admits_header("retry-after"));
    }
    fn stage(&self, mut output: AsyncResponseStaging<'_, '_>) -> Result<ResponseCompletion, ()> {
        output
            .body_mut()
            .map_err(|_| ())?
            .get_mut(..self.wire.len())
            .ok_or(())?
            .copy_from_slice(self.wire);
        for (name, value) in [
            (
                "content-type",
                self.media.then_some(b"application/json".as_slice()),
            ),
            ("retry-after", self.retry),
            ("content-encoding", self.encoding),
        ] {
            if let Some(value) = value {
                output
                    .headers_mut()
                    .map_err(|_| ())?
                    .try_push(name, value, HeaderSensitivity::Public)
                    .map_err(|_| ())?;
            }
        }
        if self.fail {
            return Err(());
        }
        Ok(ResponseCompletion::new(
            StatusCode::new(self.status).fixture("status"),
            self.wire.len(),
            ResponseMetadata::EMPTY,
        ))
    }
}
impl BoundTransport for Fixture<'_> {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        EndpointIdentity::new(
            EndpointScheme::Https,
            if self.changed.load(Ordering::SeqCst) {
                "wrong.invalid"
            } else {
                "crates.io"
            },
            443,
            "/",
        )
    }
}
impl BoundUserAgent for Fixture<'_> {
    fn configured_user_agent(&self) -> &[u8] {
        UA.as_bytes()
    }
}
impl BlockingRawHttpExecutor for Fixture<'_> {
    type Error = ();
    fn execute(
        &self,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), ()> {
        ready(drive_async_raw(self, request, policy, response)).map_err(|_| ())
    }
}
impl BlockingAuthorizedRawHttpExecutor for Fixture<'_> {
    fn execute_authorized(
        &self,
        expected: EndpointIdentity<'_>,
        authorization: HeaderValue<'_>,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), ()> {
        ready(drive_async_authorized_raw(
            self,
            expected,
            authorization,
            request,
            policy,
            response,
        ))
        .map_err(|_| ())
    }
}
impl AsyncRawHttpExecutor for Fixture<'_> {
    type Error = ();
    async fn execute<'e, 'r, 'p, 'w, 'b>(
        &'e self,
        request: TransportRequest<'r>,
        policy: RawResponsePolicy<'p>,
        response: AsyncResponseStaging<'w, 'b>,
    ) -> Result<ResponseCompletion, ()>
    where
        'e: 'w,
        'r: 'w,
        'p: 'w,
        'b: 'w,
    {
        self.verify(request, None, policy);
        let done = self.stage(response)?;
        if self.pending {
            core::future::pending::<()>().await;
        }
        Ok(done)
    }
}
impl AsyncAuthorizedRawHttpExecutor for Fixture<'_> {
    async fn execute_authorized<'e, 'r, 'p, 'w, 'b>(
        &'e self,
        expected: EndpointIdentity<'r>,
        authorization: HeaderValue<'r>,
        request: TransportRequest<'r>,
        policy: RawResponsePolicy<'p>,
        response: AsyncResponseStaging<'w, 'b>,
    ) -> Result<ResponseCompletion, ()>
    where
        'e: 'w,
        'r: 'w,
        'p: 'w,
        'b: 'w,
    {
        assert_eq!(expected, self.endpoint_identity().fixture("origin"));
        self.verify(request, Some(authorization), policy);
        let done = self.stage(response)?;
        if self.pending {
            core::future::pending::<()>().await;
        }
        Ok(done)
    }
}
pub(crate) struct Local<'a>(pub Fixture<'a>, pub Rc<()>);
impl BoundTransport for Local<'_> {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        self.0.endpoint_identity()
    }
}
impl BoundUserAgent for Local<'_> {
    fn configured_user_agent(&self) -> &[u8] {
        self.0.configured_user_agent()
    }
}
impl LocalAsyncRawHttpExecutor for Local<'_> {
    type Error = ();
    async fn execute_local<'e, 'r, 'p, 'w, 'b>(
        &'e self,
        request: TransportRequest<'r>,
        policy: RawResponsePolicy<'p>,
        response: AsyncResponseStaging<'w, 'b>,
    ) -> Result<ResponseCompletion, ()>
    where
        'e: 'w,
        'r: 'w,
        'p: 'w,
        'b: 'w,
    {
        self.0.verify(request, None, policy);
        let done = self.0.stage(response)?;
        if self.0.pending {
            core::future::pending::<()>().await;
        }
        assert_eq!(Rc::strong_count(&self.1), 1);
        Ok(done)
    }
}
impl LocalAuthorizedRawHttpExecutor for Local<'_> {
    async fn execute_authorized_local<'e, 'r, 'p, 'w, 'b>(
        &'e self,
        expected: EndpointIdentity<'r>,
        authorization: HeaderValue<'r>,
        request: TransportRequest<'r>,
        policy: RawResponsePolicy<'p>,
        response: AsyncResponseStaging<'w, 'b>,
    ) -> Result<ResponseCompletion, ()>
    where
        'e: 'w,
        'r: 'w,
        'p: 'w,
        'b: 'w,
    {
        assert_eq!(expected, self.endpoint_identity().fixture("origin"));
        self.0.verify(request, Some(authorization), policy);
        let done = self.0.stage(response)?;
        if self.0.pending {
            core::future::pending::<()>().await;
        }
        assert_eq!(Rc::strong_count(&self.1), 1);
        Ok(done)
    }
}
pub(crate) fn ready<F: Future>(future: F) -> F::Output {
    let mut future = core::pin::pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(value) => value,
        Poll::Pending => unreachable!("fixture unexpectedly pending"),
    }
}
pub(crate) fn send<F: Future + Send>(future: F) -> F {
    future
}
pub(crate) struct Buffers {
    credential: [u8; 32768],
    body: [u8; 4096],
    response: [u8; 16384],
    headers: [u8; 512],
}
impl Buffers {
    pub fn new() -> Self {
        Self {
            credential: [0xa5; 32768],
            body: [0xa5; 4096],
            response: [0xa5; 16384],
            headers: [0xa5; 512],
        }
    }
    pub fn parts(&mut self) -> RegistryBuffers<'_> {
        RegistryBuffers {
            credential: &mut self.credential,
            body: &mut self.body,
            response: &mut self.response,
            headers: &mut self.headers,
        }
    }
    pub fn cleared(&self) {
        for bytes in [
            &self.credential[..],
            &self.body[..],
            &self.response[..],
            &self.headers[..],
        ] {
            assert!(bytes.iter().all(|b| *b == 0));
        }
    }
}
pub(crate) fn parity<R>(make: impl Fn() -> R, fixture: Fixture<'_>)
where
    R: AsyncRegistryOperation
        + LocalRegistryOperation<Response = <R as AsyncRegistryOperation>::Response>
        + BlockingRegistryOperation<Response = <R as AsyncRegistryOperation>::Response>,
{
    let local = Local(fixture, Rc::new(()));
    let client = RegistryClient::production(&local.0, identity(), 16384).fixture("client");
    let local_client =
        RegistryClient::production(&local, identity(), 16384).fixture("local client");
    let mut buffers = Buffers::new();
    reset_test_gate();
    let first = client.execute(make(), buffers.parts()).fixture("blocking");
    buffers.cleared();
    reset_test_gate();
    let second = ready(send(client.execute_async(make(), buffers.parts()))).fixture("Send");
    buffers.cleared();
    reset_test_gate();
    let third = ready(local_client.execute_local(make(), buffers.parts())).fixture("local");
    buffers.cleared();
    assert_eq!(
        core::mem::discriminant(&first),
        core::mem::discriminant(&second)
    );
    assert_eq!(
        core::mem::discriminant(&first),
        core::mem::discriminant(&third)
    );
    assert_eq!(local.0.calls.load(Ordering::SeqCst), 3);
    coverage::record(local.0.method, local.0.target);
    reset_test_gate();
}
