use core::cell::Cell;
use core::future::Future;
use core::sync::atomic::{AtomicU8, Ordering};
use core::task::{Context, Poll, Waker};

use cloud_sdk::Method;
use cloud_sdk::transport::{
    AsyncRawHttpExecutor, AsyncResponseStaging, BlockingRawHttpExecutor, BoundTransport,
    EndpointIdentity, EndpointIdentityError, HeaderSensitivity, RawResponsePolicy, ResponseBuffer,
    ResponseCompletion, ResponseMediaPolicy, ResponseMetadata, ResponseWriter, StatusCode,
    TransportRequest,
};

use super::redirect::validated_target;
use super::{
    ApiRequestTarget, CratesIoEndpointError, DownloadExecutionError, DownloadRedirect,
    DownloadRedirectError, MAX_DOWNLOAD_REDIRECT_LOCATION_BYTES, OfficialCratesIoEndpoint,
    ProductionDownloadResponse,
};

const VALID_LOCATION: &[u8] = b"https://static.crates.io/crates/serde/serde-1.0.0.crate";

#[test]
fn checked_response_and_anonymous_execution_are_one_fail_closed_path() {
    let source = source_target("/api/v1/crates/serde/1.0.0/download");
    let mut checked_target = [0_u8; 128];
    let checked = check_response(
        source,
        found(),
        &[],
        &[("location", VALID_LOCATION)],
        &mut checked_target,
    );
    assert!(checked.is_ok());
    let Ok(checked) = checked else {
        unreachable!("checked redirect fixture failed");
    };
    let redirect = DownloadRedirect::from_verified(checked);

    let transport = RecordingRawTransport::new(OfficialCratesIoEndpoint::static_downloads());
    let mut body = [0_u8; 1];
    let mut headers = [0_u8; 64];
    let mut response = ResponseBuffer::new(&mut body, 0, &mut headers);
    let result = redirect.follow_blocking(&transport, empty_response_policy(), response.writer());
    assert_eq!(result, Ok(()));
    assert_eq!(transport.calls.get(), 1);
}

#[test]
fn anonymous_execution_rejects_the_wrong_transport_before_dispatch() {
    let source = source_target("/api/v1/crates/serde/1.0.0/download");
    let mut checked_target = [0_u8; 128];
    let checked = check_response(
        source,
        found(),
        &[],
        &[("location", VALID_LOCATION)],
        &mut checked_target,
    );
    let Ok(checked) = checked else {
        unreachable!("checked redirect fixture failed");
    };
    let redirect = DownloadRedirect::from_verified(checked);
    let wrong = RecordingRawTransport::new(OfficialCratesIoEndpoint::production_api());
    let mut body = [0_u8; 1];
    let mut headers = [0_u8; 64];
    let mut response = ResponseBuffer::new(&mut body, 0, &mut headers);
    assert!(matches!(
        redirect.follow_blocking(&wrong, empty_response_policy(), response.writer()),
        Err(DownloadExecutionError::InvalidDestinationEndpoint(
            CratesIoEndpointError::DestinationMismatch
        ))
    ));
    assert_eq!(wrong.calls.get(), 0);
}

#[test]
fn anonymous_execution_uses_the_same_empty_request_for_async_variants() {
    let source = source_target("/api/v1/crates/serde/1.0.0/download");
    let mut checked_target = [0_u8; 128];
    let checked = check_response(
        source,
        found(),
        &[],
        &[("location", VALID_LOCATION)],
        &mut checked_target,
    );
    let Ok(checked) = checked else {
        unreachable!("checked redirect fixture failed");
    };
    let redirect = DownloadRedirect::from_verified(checked);
    let transport = RecordingAsyncTransport::new();

    let mut send_body = [0_u8; 1];
    let mut send_headers = [0_u8; 64];
    let mut send_response = ResponseBuffer::new(&mut send_body, 0, &mut send_headers);
    assert!(matches!(
        poll_once(redirect.follow_async(
            &transport,
            empty_response_policy(),
            send_response.writer(),
        )),
        Poll::Ready(Ok(()))
    ));

    let mut local_body = [0_u8; 1];
    let mut local_headers = [0_u8; 64];
    let mut local_response = ResponseBuffer::new(&mut local_body, 0, &mut local_headers);
    assert!(matches!(
        poll_once(redirect.follow_local_async(
            &transport,
            empty_response_policy(),
            local_response.writer(),
        )),
        Poll::Ready(Ok(()))
    ));
    assert_eq!(transport.calls.load(Ordering::Relaxed), 2);
}

#[test]
fn executed_response_checks_status_body_media_and_headers() {
    let source = source_target("/api/v1/crates/serde/1.0.0/download");
    let mut target = [0x5a_u8; 128];
    for (status, body, headers, expected) in [
        (
            StatusCode::OK,
            &[][..],
            &[("location", VALID_LOCATION)][..],
            DownloadRedirectError::InvalidSourceStatus,
        ),
        (
            found(),
            &[1_u8][..],
            &[("location", VALID_LOCATION)][..],
            DownloadRedirectError::InvalidSourceResponse,
        ),
        (
            found(),
            &[][..],
            &[("location", VALID_LOCATION), ("etag", b"public")][..],
            DownloadRedirectError::InvalidSourceResponse,
        ),
        (
            found(),
            &[][..],
            &[
                ("location", VALID_LOCATION),
                ("content-type", &b"text/plain"[..]),
            ][..],
            DownloadRedirectError::InvalidSourceResponse,
        ),
        (
            found(),
            &[][..],
            &[("etag", &b"public"[..])][..],
            DownloadRedirectError::MissingLocation,
        ),
        (
            found(),
            &[][..],
            &[("location", &[0xff_u8][..])][..],
            DownloadRedirectError::InvalidLocationEncoding,
        ),
    ] {
        target.fill(0x5a);
        let result = check_response(source, status, body, headers, &mut target);
        assert!(matches!(result, Err(error) if error == expected));
        assert!(target.iter().all(|byte| *byte == 0));
    }
}

#[test]
fn redirects_reject_authority_and_archive_confusion_from_checked_responses() {
    let source = source_target("/api/v1/crates/serde/1.0.0/download");
    for location in [
        "http://static.crates.io/crates/serde/serde-1.0.0.crate",
        "https://user@static.crates.io/crates/serde/serde-1.0.0.crate",
        "https://static.crates.io:443/crates/serde/serde-1.0.0.crate",
        "https://static.crates.io.evil.example/crates/serde/serde-1.0.0.crate",
        "https://static.crates.io@evil.example/crates/serde/serde-1.0.0.crate",
        "https://static.crates.io./crates/serde/serde-1.0.0.crate",
        "https://static.crat\u{e9}s.io/crates/serde/serde-1.0.0.crate",
        "https://static.crates.io/crates/other/other-1.0.0.crate",
        "https://static.crates.io/crates/serde/serde-2.0.0.crate",
        "https://static.crates.io/crates/serde/other-1.0.0.crate",
        "https://static.crates.io/crates/serde/serde-1.0.0.crate?token=secret",
        "https://static.crates.io/crates/serde/%2E%2E.crate",
    ] {
        let mut target = [0x5a_u8; 128];
        assert!(
            check_response(
                source,
                found(),
                &[],
                &[("location", location.as_bytes())],
                &mut target,
            )
            .is_err()
        );
        assert!(target.iter().all(|byte| *byte == 0));
    }
}

#[test]
fn source_location_and_caller_storage_bounds_fail_closed() {
    let wrong_source = source_target("/api/v1/crates/serde");
    let mut target = [0x5a_u8; 128];
    assert!(matches!(
        check_response(
            wrong_source,
            found(),
            &[],
            &[("location", VALID_LOCATION)],
            &mut target,
        ),
        Err(DownloadRedirectError::InvalidSourcePath)
    ));
    assert!(target.iter().all(|byte| *byte == 0));

    let source = source_target("/api/v1/crates/serde/1.0.0/download");
    let oversized = alloc::string::String::from_iter(core::iter::repeat_n(
        'a',
        MAX_DOWNLOAD_REDIRECT_LOCATION_BYTES + 1,
    ));
    assert!(matches!(
        validated_target(&oversized, "serde", "1.0.0"),
        Err(DownloadRedirectError::LocationTooLong)
    ));
    let mut too_small = [0x5a_u8; 8];
    assert!(matches!(
        check_response(
            source,
            found(),
            &[],
            &[("location", VALID_LOCATION)],
            &mut too_small,
        ),
        Err(DownloadRedirectError::TargetStorageTooSmall)
    ));
    assert!(too_small.iter().all(|byte| *byte == 0));
}

fn check_response<'storage>(
    source: ApiRequestTarget<'_>,
    status: StatusCode,
    body: &[u8],
    headers: &[(&str, &[u8])],
    target_storage: &'storage mut [u8],
) -> Result<ProductionDownloadResponse<'storage>, DownloadRedirectError> {
    let mut body_storage = [0_u8; 8];
    assert!(body.len() <= body_storage.len());
    let mut header_storage = [0_u8; 2_048];
    let mut response = ResponseBuffer::new(&mut body_storage, body.len(), &mut header_storage);
    {
        let attempt = response.writer().begin_attempt();
        let Ok(mut attempt) = attempt else {
            unreachable!("response fixture attempt failed");
        };
        let initialized = attempt.body_mut();
        let Ok(initialized) = initialized else {
            unreachable!("response fixture body access failed");
        };
        let destination = initialized.get_mut(..body.len());
        let Some(destination) = destination else {
            unreachable!("response fixture body exceeds admitted capacity");
        };
        destination.copy_from_slice(body);
        for (name, value) in headers {
            let retained = attempt.headers_mut();
            let Ok(retained) = retained else {
                unreachable!("response fixture header access failed");
            };
            assert!(
                retained
                    .try_push(name, value, HeaderSensitivity::Sensitive)
                    .is_ok()
            );
        }
        assert!(
            attempt
                .commit(status, body.len(), ResponseMetadata::EMPTY)
                .is_ok()
        );
    }
    let checked = response.with_response(|response| {
        ProductionDownloadResponse::from_executed_response(source, response, target_storage)
    });
    let Ok(checked) = checked else {
        unreachable!("committed response fixture was unavailable");
    };
    checked
}

fn source_target(value: &'static str) -> ApiRequestTarget<'static> {
    let target = ApiRequestTarget::new(value);
    let Ok(target) = target else {
        unreachable!("source target fixture failed");
    };
    target
}

fn found() -> StatusCode {
    let status = StatusCode::new(302);
    let Some(status) = status else {
        unreachable!("302 must remain a valid status");
    };
    status
}

fn poll_once<F: Future>(future: F) -> Poll<F::Output> {
    let mut future = core::pin::pin!(future);
    Future::poll(future.as_mut(), &mut Context::from_waker(Waker::noop()))
}

fn empty_response_policy() -> RawResponsePolicy<'static> {
    let policy = RawResponsePolicy::new(
        0,
        0,
        ResponseMediaPolicy::Forbidden,
        ResponseMediaPolicy::Forbidden,
        &[],
        0,
    );
    let Ok(policy) = policy else {
        unreachable!("empty response policy fixture failed");
    };
    policy
}

struct RecordingRawTransport {
    endpoint: EndpointIdentity<'static>,
    calls: Cell<u8>,
}

impl RecordingRawTransport {
    fn new(endpoint: OfficialCratesIoEndpoint) -> Self {
        let identity = endpoint.identity();
        let Ok(endpoint) = identity else {
            unreachable!("official endpoint fixture failed");
        };
        Self {
            endpoint,
            calls: Cell::new(0),
        }
    }
}

impl BoundTransport for RecordingRawTransport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        Ok(self.endpoint)
    }
}

impl BlockingRawHttpExecutor for RecordingRawTransport {
    type Error = ();

    fn execute(
        &self,
        request: TransportRequest<'_>,
        _policy: RawResponsePolicy<'_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        assert_anonymous_download_request(request);
        self.calls.set(self.calls.get().saturating_add(1));
        let mut attempt = response.begin_attempt().map_err(|_| ())?;
        attempt
            .commit(StatusCode::NO_CONTENT, 0, ResponseMetadata::EMPTY)
            .map_err(|_| ())
    }
}

struct RecordingAsyncTransport {
    endpoint: EndpointIdentity<'static>,
    calls: AtomicU8,
}

impl RecordingAsyncTransport {
    fn new() -> Self {
        let identity = OfficialCratesIoEndpoint::static_downloads().identity();
        let Ok(endpoint) = identity else {
            unreachable!("official endpoint fixture failed");
        };
        Self {
            endpoint,
            calls: AtomicU8::new(0),
        }
    }
}

impl BoundTransport for RecordingAsyncTransport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        Ok(self.endpoint)
    }
}

impl AsyncRawHttpExecutor for RecordingAsyncTransport {
    type Error = ();

    async fn execute<'executor, 'request, 'policy, 'writer, 'buffer>(
        &'executor self,
        request: TransportRequest<'request>,
        _policy: RawResponsePolicy<'policy>,
        _response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'executor: 'writer,
        'request: 'writer,
        'policy: 'writer,
        'buffer: 'writer,
    {
        assert_anonymous_download_request(request);
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(ResponseCompletion::new(
            StatusCode::NO_CONTENT,
            0,
            ResponseMetadata::EMPTY,
        ))
    }
}

fn assert_anonymous_download_request(request: TransportRequest<'_>) {
    assert_eq!(request.method(), Method::Get);
    assert_eq!(request.target().as_str(), "/crates/serde/serde-1.0.0.crate");
    assert!(request.body().is_empty());
    assert!(request.headers().as_slice().is_empty());
    for forbidden in [
        "authorization",
        "cookie",
        "proxy-authorization",
        "x-sensitive",
    ] {
        assert!(request.headers().get(forbidden).is_none());
    }
}
