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

use super::{
    ApiRequestTarget, CratesIoEndpointError, DownloadProvenanceError, DownloadRedirectError,
    OfficialCratesIoEndpoint, ProductionDownloadResponse,
};

const SOURCE: &str = "/api/v1/crates/serde/1.0.0/download";
const LOCATION: &[u8] = b"https://static.crates.io/crates/serde/serde-1.0.0.crate";

#[test]
fn blocking_execution_mints_provenance_only_from_the_executing_transport() {
    let transport = SourceTransport::new(OfficialCratesIoEndpoint::production_api(), found());
    let mut body = [0_u8; 1];
    let mut headers = [0_u8; 256];
    let mut response = ResponseBuffer::new(&mut body, 0, &mut headers);
    let mut target = [0_u8; 128];
    let checked = ProductionDownloadResponse::execute_blocking(
        &transport,
        source(),
        &mut response,
        &mut target,
    );
    assert!(checked.is_ok());
    assert_eq!(transport.calls.load(Ordering::Relaxed), 1);
}

#[test]
fn endpoint_rejection_happens_before_source_dispatch() {
    let transport = SourceTransport::new(OfficialCratesIoEndpoint::staging_api(), found());
    let mut body = [0x5a_u8; 1];
    let mut headers = [0x5a_u8; 256];
    let mut response = ResponseBuffer::new(&mut body, 0, &mut headers);
    let mut target = [0x5a_u8; 128];
    assert!(matches!(
        ProductionDownloadResponse::execute_blocking(
            &transport,
            source(),
            &mut response,
            &mut target,
        ),
        Err(DownloadProvenanceError::InvalidSourceEndpoint(
            CratesIoEndpointError::DestinationMismatch
        ))
    ));
    assert_eq!(transport.calls.load(Ordering::Relaxed), 0);
    assert!(target.iter().all(|byte| *byte == 0));
}

#[test]
fn invalid_source_path_is_rejected_before_every_execution_mode_dispatches() {
    let transport = SourceTransport::new(OfficialCratesIoEndpoint::production_api(), found());
    let invalid_source = invalid_source();

    let mut blocking_body = [0_u8; 1];
    let mut blocking_headers = [0_u8; 256];
    let mut blocking_response = ResponseBuffer::new(&mut blocking_body, 0, &mut blocking_headers);
    let mut blocking_target = [0x5a_u8; 128];
    assert!(matches!(
        ProductionDownloadResponse::execute_blocking(
            &transport,
            invalid_source,
            &mut blocking_response,
            &mut blocking_target,
        ),
        Err(DownloadProvenanceError::InvalidRedirect(
            DownloadRedirectError::InvalidSourcePath
        ))
    ));

    let mut send_body = [0_u8; 1];
    let mut send_headers = [0_u8; 256];
    let mut send_response = ResponseBuffer::new(&mut send_body, 0, &mut send_headers);
    let mut send_target = [0x5a_u8; 128];
    assert!(matches!(
        poll_once(ProductionDownloadResponse::execute_async(
            &transport,
            invalid_source,
            &mut send_response,
            &mut send_target,
        )),
        Poll::Ready(Err(DownloadProvenanceError::InvalidRedirect(
            DownloadRedirectError::InvalidSourcePath
        )))
    ));

    let mut local_body = [0_u8; 1];
    let mut local_headers = [0_u8; 256];
    let mut local_response = ResponseBuffer::new(&mut local_body, 0, &mut local_headers);
    let mut local_target = [0x5a_u8; 128];
    assert!(matches!(
        poll_once(ProductionDownloadResponse::execute_local_async(
            &transport,
            invalid_source,
            &mut local_response,
            &mut local_target,
        )),
        Poll::Ready(Err(DownloadProvenanceError::InvalidRedirect(
            DownloadRedirectError::InvalidSourcePath
        )))
    ));

    assert_eq!(transport.calls.load(Ordering::Relaxed), 0);
    assert!(blocking_target.iter().all(|byte| *byte == 0));
    assert!(send_target.iter().all(|byte| *byte == 0));
    assert!(local_target.iter().all(|byte| *byte == 0));
}

#[test]
fn async_execution_modes_mint_only_their_own_committed_response() {
    let transport = SourceTransport::new(OfficialCratesIoEndpoint::production_api(), found());
    let mut send_body = [0_u8; 1];
    let mut send_headers = [0_u8; 256];
    let mut send_response = ResponseBuffer::new(&mut send_body, 0, &mut send_headers);
    let mut send_target = [0_u8; 128];
    assert!(matches!(
        poll_once(ProductionDownloadResponse::execute_async(
            &transport,
            source(),
            &mut send_response,
            &mut send_target,
        )),
        Poll::Ready(Ok(_))
    ));

    let mut local_body = [0_u8; 1];
    let mut local_headers = [0_u8; 256];
    let mut local_response = ResponseBuffer::new(&mut local_body, 0, &mut local_headers);
    let mut local_target = [0_u8; 128];
    assert!(matches!(
        poll_once(ProductionDownloadResponse::execute_local_async(
            &transport,
            source(),
            &mut local_response,
            &mut local_target,
        )),
        Poll::Ready(Ok(_))
    ));
    assert_eq!(transport.calls.load(Ordering::Relaxed), 2);
}

#[test]
fn uncommitted_and_structurally_invalid_source_responses_fail_closed() {
    let uncommitted = SourceTransport::without_commit();
    let mut body = [0_u8; 1];
    let mut headers = [0_u8; 256];
    let mut response = ResponseBuffer::new(&mut body, 0, &mut headers);
    let mut target = [0x5a_u8; 128];
    assert!(matches!(
        ProductionDownloadResponse::execute_blocking(
            &uncommitted,
            source(),
            &mut response,
            &mut target,
        ),
        Err(DownloadProvenanceError::ResponseWriter(_))
    ));
    assert!(target.iter().all(|byte| *byte == 0));

    let wrong_status =
        SourceTransport::new(OfficialCratesIoEndpoint::production_api(), StatusCode::OK);
    let mut second_body = [0_u8; 1];
    let mut second_headers = [0_u8; 256];
    let mut second_response = ResponseBuffer::new(&mut second_body, 0, &mut second_headers);
    assert!(matches!(
        ProductionDownloadResponse::execute_blocking(
            &wrong_status,
            source(),
            &mut second_response,
            &mut target,
        ),
        Err(DownloadProvenanceError::InvalidRedirect(
            DownloadRedirectError::InvalidSourceStatus
        ))
    ));
    assert!(target.iter().all(|byte| *byte == 0));
}

#[test]
fn precommitted_response_cannot_supply_stale_provenance() {
    let transport = SourceTransport::new(OfficialCratesIoEndpoint::production_api(), found());
    let mut body = [0_u8; 1];
    let mut headers = [0_u8; 256];
    let mut response = ResponseBuffer::new(&mut body, 0, &mut headers);
    commit_response(response.writer(), found());
    let mut target = [0x5a_u8; 128];
    assert!(matches!(
        ProductionDownloadResponse::execute_blocking(
            &transport,
            source(),
            &mut response,
            &mut target,
        ),
        Err(DownloadProvenanceError::ResponseWriter(
            cloud_sdk::transport::ResponseWriterError::AlreadyCommitted
        ))
    ));
    assert_eq!(transport.calls.load(Ordering::Relaxed), 0);
    assert!(target.iter().all(|byte| *byte == 0));
}

fn source() -> ApiRequestTarget<'static> {
    let source = ApiRequestTarget::new(SOURCE);
    let Ok(source) = source else {
        unreachable!("source fixture failed");
    };
    source
}

fn invalid_source() -> ApiRequestTarget<'static> {
    let source = ApiRequestTarget::new("/api/v1/crates/serde");
    let Ok(source) = source else {
        unreachable!("invalid download source fixture must remain a valid generic API target");
    };
    source
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

struct SourceTransport {
    endpoint: EndpointIdentity<'static>,
    status: StatusCode,
    commit: bool,
    calls: AtomicU8,
}

impl SourceTransport {
    fn new(endpoint: OfficialCratesIoEndpoint, status: StatusCode) -> Self {
        let identity = endpoint.identity();
        let Ok(endpoint) = identity else {
            unreachable!("official endpoint fixture failed");
        };
        Self {
            endpoint,
            status,
            commit: true,
            calls: AtomicU8::new(0),
        }
    }

    fn without_commit() -> Self {
        let mut transport = Self::new(OfficialCratesIoEndpoint::production_api(), found());
        transport.commit = false;
        transport
    }
}

impl BoundTransport for SourceTransport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        Ok(self.endpoint)
    }
}

impl BlockingRawHttpExecutor for SourceTransport {
    type Error = ();

    fn execute(
        &self,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        assert_source_request(request, policy, self.status);
        self.calls.fetch_add(1, Ordering::Relaxed);
        if !self.commit {
            return Ok(());
        }
        commit_response(response, self.status);
        Ok(())
    }
}

fn commit_response(response: &mut ResponseWriter<'_>, status: StatusCode) {
    let attempt = response.begin_attempt();
    let Ok(mut attempt) = attempt else {
        unreachable!("source response attempt failed");
    };
    let headers = attempt.headers_mut();
    let Ok(headers) = headers else {
        unreachable!("source response headers failed");
    };
    assert!(
        headers
            .try_push("location", LOCATION, HeaderSensitivity::Public)
            .is_ok()
    );
    assert!(attempt.commit(status, 0, ResponseMetadata::EMPTY).is_ok());
}

impl AsyncRawHttpExecutor for SourceTransport {
    type Error = ();

    async fn execute<'executor, 'request, 'policy, 'writer, 'buffer>(
        &'executor self,
        request: TransportRequest<'request>,
        policy: RawResponsePolicy<'policy>,
        mut response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'executor: 'writer,
        'request: 'writer,
        'policy: 'writer,
        'buffer: 'writer,
    {
        assert_source_request(request, policy, self.status);
        self.calls.fetch_add(1, Ordering::Relaxed);
        response
            .headers_mut()
            .map_err(|_| ())?
            .try_push("location", LOCATION, HeaderSensitivity::Public)
            .map_err(|_| ())?;
        Ok(ResponseCompletion::new(
            self.status,
            0,
            ResponseMetadata::EMPTY,
        ))
    }
}

fn assert_source_request(
    request: TransportRequest<'_>,
    policy: RawResponsePolicy<'_>,
    status: StatusCode,
) {
    assert_eq!(request.method(), Method::Get);
    assert_eq!(request.target().as_str(), SOURCE);
    assert!(request.body().is_empty());
    assert!(request.headers().as_slice().is_empty());
    assert_eq!(policy.max_body_bytes(), 0);
    assert_eq!(policy.media_policy(status), ResponseMediaPolicy::Forbidden);
    assert!(policy.admits_header("location"));
    assert!(!policy.admits_header("authorization"));
    assert!(!policy.admits_header("cookie"));
    assert!(!policy.admits_header("proxy-authorization"));
    assert_eq!(policy.informational_limit(), 0);
}
