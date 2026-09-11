use super::*;
use crate::{
    catalog::tests::{FIXTURES, requests},
    credentials::{ApiToken, CredentialOrigin},
    discovery::tests::Fixture as _,
    wire::{ScheduleError, TEST_GATE_LOCK, reset_test_gate},
};
use cloud_sdk::{
    Method,
    transport::{
        AsyncRawHttpExecutor, AsyncResponseStaging, BlockingRawHttpExecutor, EndpointIdentity,
        EndpointIdentityError, HeaderSensitivity, LocalAsyncRawHttpExecutor, RawResponsePolicy,
        ResponseCompletion, ResponseMetadata, ResponseWriter, StatusCode, TransportRequest,
    },
};
use core::{
    future::Future,
    sync::atomic::{AtomicUsize, Ordering},
    task::{Context, Poll, Waker},
};

struct Executor {
    index: usize,
    calls: AtomicUsize,
    pending: bool,
    retry: Option<&'static [u8]>,
    status: u16,
}
impl BoundTransport for Executor {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        Ok(crate::endpoint::OfficialCratesIoEndpoint::production_api()
            .identity()
            .fixture("endpoint"))
    }
}
impl BoundUserAgent for Executor {
    fn configured_user_agent(&self) -> &[u8] {
        b"tests/1 (tests@example.org)"
    }
}
impl Executor {
    fn verify(&self, request: TransportRequest<'_>, policy: RawResponsePolicy<'_>) {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.method(), Method::Get);
        let expected = [
            "/api/v1/crates",
            "/api/v1/crates/serde",
            "/api/v1/crates/new",
        ];
        assert_eq!(
            request.target().path().as_str(),
            *expected.get(self.index).fixture("route")
        );
        assert!(request.body().is_empty());
        assert!(request.headers().get("authorization").is_none());
        assert!(request.headers().get("cookie").is_none());
        assert_eq!(policy.max_body_bytes(), 65_536);
    }
    fn stage(
        &self,
        mut response: AsyncResponseStaging<'_, '_>,
    ) -> Result<ResponseCompletion, &'static str> {
        let bytes = FIXTURES.get(self.index).fixture("fixture").as_bytes();
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
                .map_err(|_| "retry")?;
        }
        Ok(ResponseCompletion::new(
            StatusCode::new(self.status).fixture("status"),
            bytes.len(),
            ResponseMetadata::EMPTY,
        ))
    }
}
impl BlockingRawHttpExecutor for Executor {
    type Error = &'static str;
    fn execute(
        &self,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        self.verify(request, policy);
        let mut attempt = response.begin_attempt().map_err(|_| "attempt")?;
        let bytes = FIXTURES.get(self.index).fixture("fixture").as_bytes();
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
                .map_err(|_| "retry")?;
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
impl AsyncRawHttpExecutor for Executor {
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
struct Local(Executor, core::cell::Cell<()>);
impl BoundTransport for Local {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        self.0.endpoint_identity()
    }
}
impl BoundUserAgent for Local {
    fn configured_user_agent(&self) -> &[u8] {
        self.0.configured_user_agent()
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
fn executor(index: usize) -> Executor {
    Executor {
        index,
        calls: AtomicUsize::new(0),
        pending: false,
        retry: None,
        status: 200,
    }
}
fn identity() -> IdentifyingUserAgent<'static> {
    IdentifyingUserAgent::new("tests/1 (tests@example.org)").fixture("identity")
}
fn ready<F: Future>(future: F) -> F::Output {
    let mut future = core::pin::pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(v) => v,
        Poll::Pending => unreachable!("fixture did not finish"),
    }
}
fn send<F: Future + Send>(future: F) -> F {
    future
}

#[test]
fn all_catalog_operations_run_with_blocking_local_and_send_parity() {
    let _serial = TEST_GATE_LOCK.lock().fixture("gate");
    for (index, request) in requests().into_iter().enumerate() {
        let executor = executor(index);
        let local = Local(self::executor(index), core::cell::Cell::new(()));
        let client = CatalogClient::production(&executor, identity(), 65_536).fixture("client");
        let local_client = CatalogClient::production(&local, identity(), 65_536).fixture("client");
        let mut body = [0xa5; 65_536];
        let mut headers = [0xa5; 512];
        reset_test_gate();
        let a = client
            .execute(request, &mut body, &mut headers)
            .fixture("blocking");
        assert!(body.iter().all(|v| *v == 0));
        assert!(headers.iter().all(|v| *v == 0));
        reset_test_gate();
        let b =
            ready(send(client.execute_async(request, &mut body, &mut headers))).fixture("async");
        assert!(body.iter().all(|v| *v == 0));
        assert!(headers.iter().all(|v| *v == 0));
        reset_test_gate();
        let c =
            ready(local_client.execute_local(request, &mut body, &mut headers)).fixture("local");
        assert!(body.iter().all(|v| *v == 0));
        assert!(headers.iter().all(|v| *v == 0));
        assert_eq!(core::mem::discriminant(&a), core::mem::discriminant(&b));
        assert_eq!(core::mem::discriminant(&a), core::mem::discriminant(&c));
        assert_eq!(executor.calls.load(Ordering::SeqCst), 2);
    }
}
#[test]
fn catalog_shares_rate_gate_and_cancellation_cleanup_with_discovery() {
    let _serial = TEST_GATE_LOCK.lock().fixture("gate");
    reset_test_gate();
    let executor = Executor {
        pending: true,
        ..executor(0)
    };
    let client = CatalogClient::production(&executor, identity(), 65_536).fixture("client");
    let mut body = [0xa5; 65_536];
    let mut headers = [0xa5; 512];
    let request = CatalogRequest::list(&[]).fixture("request");
    drop(client.execute_async(request, &mut body, &mut headers));
    assert!(body.iter().all(|v| *v == 0));
    assert!(headers.iter().all(|v| *v == 0));
    assert_eq!(executor.calls.load(Ordering::SeqCst), 0);
    {
        let mut future = core::pin::pin!(client.execute_async(request, &mut body, &mut headers));
        assert!(
            future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending()
        );
        assert!(matches!(
            crate::wire::OfficialApiGate::new(identity())
                .try_call::<(), ()>(|_| unreachable!("in-flight bypass")),
            Err(crate::wire::OfficialCallError::Schedule(
                ScheduleError::Unavailable
            ))
        ));
    }
    assert!(body.iter().all(|v| *v == 0));
    assert!(headers.iter().all(|v| *v == 0));
    assert!(matches!(
        client.execute(request, &mut body, &mut headers),
        Err(CatalogExecutionError::Schedule(ScheduleError::Wait(_)))
    ));
    assert_eq!(executor.calls.load(Ordering::SeqCst), 1);
}
#[test]
fn optional_token_only_enters_list_and_secret_storage_always_clears() {
    let _serial = TEST_GATE_LOCK.lock().fixture("gate");
    reset_test_gate();
    let executor = executor(0);
    let client = CatalogClient::production(&executor, identity(), 65_536).fixture("client");
    // Public non-production material, used only by an in-memory adapter.
    let mut source = alloc::format!("fixture{}", line!()).into_bytes();
    let token =
        ApiToken::from_mut_bytes(CredentialOrigin::Production, &mut source).fixture("token");
    assert!(source.iter().all(|v| *v == 0));
    let mut secret = [0xa5; 1024];
    let mut body = [0xa5; 65_536];
    let mut headers = [0xa5; 512];
    let params = [crate::query::Parameter::Following];
    let request = CatalogRequest::list(&params).fixture("following");
    assert!(matches!(
        client.execute(request, &mut body, &mut headers),
        Err(CatalogExecutionError::Model(CatalogError::Binding))
    ));
    assert!(ready(client.execute_async(request, &mut body, &mut headers)).is_err());
    assert_eq!(executor.calls.load(Ordering::SeqCst), 0);
    client
        .execute_with_token(
            request,
            &token,
            &mut secret,
            &mut body,
            &mut headers,
            |executor, material, wire, policy, response| {
                assert!(material.authorization().is_some());
                assert!(material.json_body().is_none());
                assert_eq!(material.method(), wire.method());
                assert_eq!(material.target(), wire.target());
                BlockingRawHttpExecutor::execute(executor, wire, policy, response)
            },
        )
        .fixture("token adapter");
    assert!(secret.iter().all(|v| *v == 0));
    assert!(body.iter().all(|v| *v == 0));
    assert!(headers.iter().all(|v| *v == 0));
    for request in [
        *requests().get(1).fixture("metadata"),
        CatalogRequest::cargo_search(&[]).fixture("cargo"),
    ] {
        reset_test_gate();
        assert!(
            client
                .execute_with_token::<()>(
                    request,
                    &token,
                    &mut secret,
                    &mut body,
                    &mut headers,
                    |_, _, _, _, _| unreachable!("token disclosure to anonymous-only endpoint")
                )
                .is_err()
        );
    }
    reset_test_gate();
    let mut source = alloc::format!("fixture{}", line!()).into_bytes();
    let wrong = ApiToken::from_mut_bytes(CredentialOrigin::Staging, &mut source).fixture("token");
    assert!(
        client
            .execute_with_token::<()>(
                request,
                &wrong,
                &mut secret,
                &mut body,
                &mut headers,
                |_, _, _, _, _| unreachable!("wrong origin")
            )
            .is_err()
    );
    assert!(secret.iter().all(|v| *v == 0));
    assert!(body.iter().all(|v| *v == 0));
    assert!(headers.iter().all(|v| *v == 0));
}

#[test]
fn catalog_and_token_delays_are_bounded_and_clear_every_buffer() {
    let _serial = TEST_GATE_LOCK.lock().fixture("gate");
    let mut source = alloc::format!("fixture{}", line!()).into_bytes();
    let token =
        ApiToken::from_mut_bytes(CredentialOrigin::Production, &mut source).fixture("token");
    let request = CatalogRequest::list(&[]).fixture("list");
    for status in [200, 503] {
        for retry in [
            b"86400".as_slice(),
            b"86401",
            b"18446744073709551615",
            b"Fri, 31 Dec 9999 23:59:59 GMT",
        ] {
            let executor = Executor {
                retry: Some(retry),
                status,
                ..executor(0)
            };
            let local = Local(
                Executor {
                    retry: Some(retry),
                    status,
                    ..self::executor(0)
                },
                core::cell::Cell::new(()),
            );
            let client = CatalogClient::production(&executor, identity(), 65_536).fixture("client");
            let local_client =
                CatalogClient::production(&local, identity(), 65_536).fixture("local");
            let mut body = [0; 65_536];
            let mut headers = [0; 512];
            let mut secret = [0; 1024];
            for mode in 0..4 {
                reset_test_gate();
                body.fill(0xa5);
                headers.fill(0xa5);
                secret.fill(0xa5);
                let result = match mode {
                    0 => client.execute(request, &mut body, &mut headers),
                    1 => ready(send(client.execute_async(request, &mut body, &mut headers))),
                    2 => ready(local_client.execute_local(request, &mut body, &mut headers)),
                    _ => client.execute_with_token(
                        request,
                        &token,
                        &mut secret,
                        &mut body,
                        &mut headers,
                        |executor, _, wire, policy, response| {
                            BlockingRawHttpExecutor::execute(executor, wire, policy, response)
                        },
                    ),
                };
                if retry == b"86400" {
                    if status == 200 {
                        assert!(result.is_ok());
                    } else {
                        assert!(matches!(
                            result,
                            Err(CatalogExecutionError::Wire(
                                crate::wire::CratesIoWireError::Provider(_)
                            ))
                        ));
                    }
                } else {
                    assert!(matches!(
                        result,
                        Err(CatalogExecutionError::Schedule(ScheduleError::Overflow))
                    ));
                }
                assert!(body.iter().all(|v| *v == 0));
                assert!(headers.iter().all(|v| *v == 0));
                if mode == 3 {
                    assert!(secret.iter().all(|v| *v == 0));
                }
                assert!(matches!(client.execute(request, &mut body, &mut headers),
                    Err(CatalogExecutionError::Schedule(ScheduleError::Wait(delay)))
                    if delay <= crate::wire::MAX_PROVIDER_DELAY));
            }
        }
    }
    reset_test_gate();
}
