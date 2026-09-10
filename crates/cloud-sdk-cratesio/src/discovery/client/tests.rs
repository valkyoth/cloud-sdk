use super::*;
use crate::discovery::tests::Fixture as _;
use crate::{
    discovery::tests::{FIXTURES, requests},
    wire::{TEST_GATE_LOCK, reset_test_gate},
};
use cloud_sdk::transport::{
    AsyncRawHttpExecutor, AsyncResponseStaging, BlockingRawHttpExecutor, EndpointIdentity,
    EndpointIdentityError, HeaderSensitivity, LocalAsyncRawHttpExecutor, ResponseCompletion,
    ResponseMetadata, ResponseWriter,
};
use core::sync::atomic::{AtomicUsize, Ordering};
use core::{
    future::Future,
    task::{Context, Poll, Waker},
};

struct Fixture {
    index: usize,
    calls: AtomicUsize,
    pending: bool,
    foreign: bool,
    changed_identity: core::sync::atomic::AtomicBool,
    status: u16,
    retry: Option<&'static [u8]>,
}
impl BoundTransport for Fixture {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        EndpointIdentity::new(
            cloud_sdk::transport::EndpointScheme::Https,
            if self.foreign {
                "wrong.example"
            } else {
                "crates.io"
            },
            443,
            "/",
        )
    }
}
impl Fixture {
    fn verify(&self, request: TransportRequest<'_>, policy: RawResponsePolicy<'_>) {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.method(), Method::Get);
        assert!(request.body().is_empty());
        assert!(request.headers().get("authorization").is_none());
        assert!(request.headers().get("cookie").is_none());
        assert!(request.headers().get("user-agent").is_none());
        assert_eq!(self.configured_user_agent(), b"test/1 (tests@example.org)");
        let mut expected = [0; 4096];
        assert_eq!(
            request.target().as_str(),
            requests()
                .get(self.index)
                .fixture("request index")
                .write_target(&mut expected)
                .fixture("target")
                .as_str()
        );
        assert!(policy.admits_header("retry-after"));
        assert_eq!(policy.max_body_bytes(), 65_536);
    }
    fn stage(
        &self,
        mut response: AsyncResponseStaging<'_, '_>,
    ) -> Result<ResponseCompletion, &'static str> {
        let bytes = FIXTURES.get(self.index).fixture("fixture index").as_bytes();
        response
            .body_mut()
            .map_err(|_| "body")?
            .get_mut(..bytes.len())
            .ok_or("size")?
            .copy_from_slice(bytes);
        response
            .headers_mut()
            .map_err(|_| "headers")?
            .try_push(
                "content-type",
                b"application/json",
                HeaderSensitivity::Public,
            )
            .map_err(|_| "header")?;
        if let Some(retry) = self.retry {
            response
                .headers_mut()
                .map_err(|_| "headers")?
                .try_push("retry-after", retry, HeaderSensitivity::Public)
                .map_err(|_| "header")?;
        }
        Ok(ResponseCompletion::new(
            StatusCode::new(self.status).fixture("fixture status"),
            bytes.len(),
            ResponseMetadata::EMPTY,
        ))
    }
}
impl BoundUserAgent for Fixture {
    fn configured_user_agent(&self) -> &[u8] {
        if self.changed_identity.load(Ordering::SeqCst) {
            b"other/1 (tests@example.org)"
        } else {
            b"test/1 (tests@example.org)"
        }
    }
}
impl BlockingRawHttpExecutor for Fixture {
    type Error = &'static str;
    fn execute(
        &self,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        self.verify(request, policy);
        let mut attempt = response.begin_attempt().map_err(|_| "attempt")?;
        let bytes = FIXTURES.get(self.index).fixture("fixture index").as_bytes();
        attempt
            .body_mut()
            .map_err(|_| "body")?
            .get_mut(..bytes.len())
            .ok_or("size")?
            .copy_from_slice(bytes);
        attempt
            .headers_mut()
            .map_err(|_| "headers")?
            .try_push(
                "content-type",
                b"application/json",
                HeaderSensitivity::Public,
            )
            .map_err(|_| "header")?;
        if let Some(retry) = self.retry {
            attempt
                .headers_mut()
                .map_err(|_| "headers")?
                .try_push("retry-after", retry, HeaderSensitivity::Public)
                .map_err(|_| "header")?;
        }
        attempt
            .commit(
                StatusCode::new(self.status).fixture("status"),
                bytes.len(),
                ResponseMetadata::EMPTY,
            )
            .map_err(|_| "commit")
    }
}
impl AsyncRawHttpExecutor for Fixture {
    type Error = &'static str;
    async fn execute<'e, 'r, 'p, 'w, 'b>(
        &'e self,
        request: TransportRequest<'r>,
        policy: RawResponsePolicy<'p>,
        response: AsyncResponseStaging<'w, 'b>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'e: 'w,
        'r: 'w,
        'p: 'w,
        'b: 'w,
    {
        self.verify(request, policy);
        let done = self.stage(response)?;
        if self.pending {
            core::future::pending::<()>().await;
        }
        Ok(done)
    }
}
struct Local(Fixture, core::cell::Cell<()>);
impl BoundUserAgent for Local {
    fn configured_user_agent(&self) -> &[u8] {
        self.0.configured_user_agent()
    }
}
impl BoundTransport for Local {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        self.0.endpoint_identity()
    }
}
impl LocalAsyncRawHttpExecutor for Local {
    type Error = &'static str;
    async fn execute_local<'e, 'r, 'p, 'w, 'b>(
        &'e self,
        request: TransportRequest<'r>,
        policy: RawResponsePolicy<'p>,
        response: AsyncResponseStaging<'w, 'b>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'e: 'w,
        'r: 'w,
        'p: 'w,
        'b: 'w,
    {
        self.1.get();
        self.0.verify(request, policy);
        self.0.stage(response)
    }
}
fn fixture(index: usize) -> Fixture {
    Fixture {
        index,
        calls: AtomicUsize::new(0),
        pending: false,
        foreign: false,
        changed_identity: core::sync::atomic::AtomicBool::new(false),
        status: 200,
        retry: None,
    }
}
fn identity() -> IdentifyingUserAgent<'static> {
    IdentifyingUserAgent::new("test/1 (tests@example.org)").fixture("identity")
}
fn ready<F: Future>(future: F) -> F::Output {
    let mut future = core::pin::pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(result) => result,
        Poll::Pending => unreachable!("synchronous fixture did not complete"),
    }
}
fn send<F: Future + Send>(future: F) -> F {
    future
}

#[test]
fn all_operations_have_blocking_local_and_send_parity() {
    let _serial = TEST_GATE_LOCK.lock().fixture("test gate lock");
    for (index, request) in requests().into_iter().enumerate() {
        let fixture = fixture(index);
        let local = Local(self::fixture(index), core::cell::Cell::new(()));
        let client = DiscoveryClient::production(&fixture, identity(), 65_536).fixture("client");
        let local_client =
            DiscoveryClient::production(&local, identity(), 65_536).fixture("local client");
        let mut body = [0xa5; 65_536];
        let mut headers = [0xa5; 512];
        reset_test_gate();
        let blocking = client
            .execute(request, &mut body, &mut headers)
            .fixture("blocking decode");
        assert!(body.iter().all(|b| *b == 0));
        assert!(headers.iter().all(|b| *b == 0));
        reset_test_gate();
        let asynchronous = ready(send(client.execute_async(request, &mut body, &mut headers)))
            .fixture("async decode");
        assert!(body.iter().all(|b| *b == 0));
        assert!(headers.iter().all(|b| *b == 0));
        reset_test_gate();
        let local = ready(local_client.execute_local(request, &mut body, &mut headers))
            .fixture("local decode");
        assert!(body.iter().all(|b| *b == 0));
        assert!(headers.iter().all(|b| *b == 0));
        assert_eq!(
            core::mem::discriminant(&blocking),
            core::mem::discriminant(&asynchronous)
        );
        assert_eq!(
            core::mem::discriminant(&blocking),
            core::mem::discriminant(&local)
        );
        assert_eq!(fixture.calls.load(Ordering::SeqCst), 2);
    }
}

#[test]
fn cancellation_and_unpolled_drop_clear_storage_and_do_not_burst() {
    let _serial = TEST_GATE_LOCK.lock().fixture("test gate lock");
    reset_test_gate();
    let fixture = Fixture {
        pending: true,
        ..fixture(5)
    };
    let client = DiscoveryClient::production(&fixture, identity(), 65_536).fixture("client");
    let mut body = [0xa5; 65_536];
    let mut headers = [0xa5; 512];
    drop(client.execute_async(DiscoveryRequest::site_metadata(), &mut body, &mut headers));
    assert!(body.iter().all(|b| *b == 0));
    assert!(headers.iter().all(|b| *b == 0));
    assert_eq!(fixture.calls.load(Ordering::SeqCst), 0);
    {
        let mut future = core::pin::pin!(client.execute_async(
            DiscoveryRequest::site_metadata(),
            &mut body,
            &mut headers
        ));
        assert!(
            future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending()
        );
        assert!(matches!(
            OfficialApiGate::new(identity())
                .try_call::<(), ()>(|_| unreachable!("concurrent gate bypass")),
            Err(crate::wire::OfficialCallError::Schedule(
                ScheduleError::Unavailable
            ))
        ));
    }
    assert!(body.iter().all(|b| *b == 0));
    assert!(headers.iter().all(|b| *b == 0));
    assert_eq!(fixture.calls.load(Ordering::SeqCst), 1);
    assert!(matches!(
        client.execute(DiscoveryRequest::site_metadata(), &mut body, &mut headers),
        Err(DiscoveryExecutionError::Schedule(ScheduleError::Wait(_)))
    ));
    assert_eq!(fixture.calls.load(Ordering::SeqCst), 1);
}

#[test]
fn status_origin_response_bounds_and_retry_after_fail_closed() {
    let _serial = TEST_GATE_LOCK.lock().fixture("test gate lock");
    reset_test_gate();
    let wrong = Fixture {
        foreign: true,
        ..fixture(5)
    };
    assert!(DiscoveryClient::production(&wrong, identity(), 65_536).is_err());
    let fixture = Fixture {
        status: 503,
        retry: Some(b"60"),
        ..fixture(5)
    };
    let client = DiscoveryClient::production(&fixture, identity(), 65_536).fixture("client");
    let mut body = [0xa5; 65_536];
    let mut headers = [0xa5; 512];
    let result = client.execute(DiscoveryRequest::site_metadata(), &mut body, &mut headers);
    assert!(matches!(
        result,
        Err(DiscoveryExecutionError::Wire(CratesIoWireError::Provider(
            _
        )))
    ));
    assert!(
        matches!(client.execute(DiscoveryRequest::site_metadata(),&mut body,&mut headers),Err(DiscoveryExecutionError::Schedule(ScheduleError::Wait(d))) if d.as_secs() >= 59)
    );
    assert!(body.iter().all(|b| *b == 0));
    assert!(headers.iter().all(|b| *b == 0));
    reset_test_gate();
    let mut small = [0xa5; 2];
    assert!(
        client
            .execute(DiscoveryRequest::site_metadata(), &mut small, &mut headers)
            .is_err()
    );
    assert_eq!(small, [0, 0]);
    assert_eq!(wrong.calls.load(Ordering::SeqCst), 0);
    let error = DiscoveryExecutionError::Transport("secret fixture error");
    assert!(!alloc::format!("{error} {error:?}").contains("secret fixture error"));
    assert!(core::error::Error::source(&error).is_none());
}

#[test]
fn transport_user_agent_is_bound_before_construction_and_each_dispatch() {
    let _serial = TEST_GATE_LOCK.lock().fixture("test gate lock");
    reset_test_gate();
    let fixture = fixture(5);
    let other = IdentifyingUserAgent::new("other/1 (tests@example.org)").fixture("identity");
    assert!(DiscoveryClient::production(&fixture, other, 65_536).is_err());
    let client = DiscoveryClient::production(&fixture, identity(), 65_536).fixture("client");
    fixture.changed_identity.store(true, Ordering::SeqCst);
    let mut body = [0xa5; 65_536];
    let mut headers = [0xa5; 512];
    assert!(matches!(
        client.execute(DiscoveryRequest::site_metadata(), &mut body, &mut headers),
        Err(DiscoveryExecutionError::Model(DiscoveryError::Binding))
    ));
    assert!(matches!(
        ready(client.execute_async(DiscoveryRequest::site_metadata(), &mut body, &mut headers)),
        Err(DiscoveryExecutionError::Model(DiscoveryError::Binding))
    ));
    assert!(body.iter().all(|b| *b == 0));
    assert!(headers.iter().all(|b| *b == 0));
    assert_eq!(fixture.calls.load(Ordering::SeqCst), 0);
}
