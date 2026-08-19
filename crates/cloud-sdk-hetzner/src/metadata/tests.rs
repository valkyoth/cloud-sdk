use core::future::Future;
use core::net::Ipv4Addr;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::{Context, Poll, Waker};

use cloud_sdk::Method;
use cloud_sdk::operation::{OperationImpact, RetryEligibility};
use cloud_sdk::transport::{
    AsyncRawHttpExecutor, AsyncResponseStaging, BlockingRawHttpExecutor, BoundTransport,
    EndpointIdentity, EndpointIdentityError, EndpointScheme, RawResponsePolicy, ResponseBuffer,
    ResponseCompletion, ResponseMetadata, ResponseWriter, StatusCode, TransportRequest,
};

use super::{
    MAX_METADATA_RESPONSE_BYTES, MetadataDecodeError, MetadataRequest, MetadataResponse,
    MetadataRoute, decode_metadata_body, decode_metadata_response, execute_metadata_async,
    execute_metadata_blocking, execute_metadata_local_async, metadata_endpoint_identity,
    verify_metadata_endpoint,
};

mod adversarial_tests;

const SUMMARY: &[u8] = b"availability-zone: hel1-dc2\nhostname: my-server\ninstance-id: 42\npublic-ipv4: 1.2.3.4\nregion: eu-central\n";
const NETWORKS: &[u8] = b"- ip: 10.0.0.2\n  alias_ips: [10.0.0.3, 10.0.0.4]\n  interface_num: 1\n  mac_address: 86:00:00:2a:7d:e0\n  network_id: 1234\n  network_name: nw-test1\n  network: 10.0.0.0/8\n  subnet: 10.0.0.0/24\n  gateway: 10.0.0.1\n- ip: 192.168.0.2\n  alias_ips: []\n  interface_num: 2\n  mac_address: 86:00:00:2a:7d:e1\n  network_id: 4321\n  network_name: nw-test2\n  network: 192.168.0.0/16\n  subnet: 192.168.0.0/24\n  gateway: 192.168.0.1\n";

#[test]
fn all_seven_routes_are_exact_bodyless_reads() {
    let cases = [
        (MetadataRoute::Summary, "/hetzner/v1/metadata"),
        (MetadataRoute::Hostname, "/hetzner/v1/metadata/hostname"),
        (
            MetadataRoute::InstanceId,
            "/hetzner/v1/metadata/instance-id",
        ),
        (
            MetadataRoute::PublicIpv4,
            "/hetzner/v1/metadata/public-ipv4",
        ),
        (
            MetadataRoute::PrivateNetworks,
            "/hetzner/v1/metadata/private-networks",
        ),
        (
            MetadataRoute::AvailabilityZone,
            "/hetzner/v1/metadata/availability-zone",
        ),
        (MetadataRoute::Region, "/hetzner/v1/metadata/region"),
    ];
    for (route, expected) in cases {
        let request = MetadataRequest::new(route);
        let wire = request
            .transport_request()
            .unwrap_or_else(|_| unreachable!("static route"));
        let metadata = request
            .operation_metadata()
            .unwrap_or_else(|_| unreachable!("static metadata"));
        assert_eq!(wire.method(), Method::Get);
        assert_eq!(wire.target().as_str(), expected);
        assert!(wire.body().is_empty());
        assert!(wire.headers().as_slice().is_empty());
        assert_eq!(metadata.impact(), OperationImpact::ReadOnly);
        assert_eq!(metadata.retry_eligibility(), RetryEligibility::Never);
        assert!(request.response_policy().is_ok());
    }
}

#[test]
fn summary_and_scalar_decoders_are_strict() {
    let response = decode_metadata_body(MetadataRoute::Summary, SUMMARY);
    let Ok(MetadataResponse::Summary(summary)) = response else {
        unreachable!("official summary fixture")
    };
    assert_eq!(summary.hostname(), "my-server");
    assert_eq!(summary.instance_id(), 42);
    assert_eq!(summary.public_ipv4(), Ipv4Addr::new(1, 2, 3, 4));
    assert_eq!(summary.availability_zone(), "hel1-dc2");
    assert_eq!(summary.region(), "eu-central");

    assert_eq!(
        decode_metadata_body(MetadataRoute::InstanceId, b"042\n"),
        Err(MetadataDecodeError::InvalidNumber)
    );
    assert_eq!(
        decode_metadata_body(MetadataRoute::PublicIpv4, b"1.2.3.04\n"),
        Err(MetadataDecodeError::InvalidIpv4)
    );
    let duplicate = b"availability-zone: hel1-dc2\nhostname: one\nhostname: two\ninstance-id: 42\npublic-ipv4: 1.2.3.4\nregion: eu-central\n";
    assert_eq!(
        decode_metadata_body(MetadataRoute::Summary, duplicate),
        Err(MetadataDecodeError::DuplicateField)
    );
    let unknown = b"availability-zone: hel1-dc2\nhostname: one\ninstance-id: 42\npublic-ipv4: 1.2.3.4\nregion: eu-central\ncredential: secret\n";
    assert_eq!(
        decode_metadata_body(MetadataRoute::Summary, unknown),
        Err(MetadataDecodeError::UnknownField)
    );
}

#[test]
fn private_network_yaml_is_bounded_and_cross_validated() {
    let response = decode_metadata_body(MetadataRoute::PrivateNetworks, NETWORKS);
    let networks = match response {
        Ok(MetadataResponse::PrivateNetworks(networks)) => networks,
        other => unreachable!("official private-network fixture failed: {other:?}"),
    };
    assert_eq!(networks.len(), 2);
    let first = networks
        .iter()
        .next()
        .unwrap_or_else(|| unreachable!("first network"));
    assert_eq!(first.ip(), Ipv4Addr::new(10, 0, 0, 2));
    assert_eq!(first.alias_ips().len(), 2);
    assert_eq!(first.interface_num(), 1);
    assert_eq!(first.network_id(), 1234);
    assert_eq!(first.network(), (Ipv4Addr::new(10, 0, 0, 0), 8));
    assert_eq!(first.subnet(), (Ipv4Addr::new(10, 0, 0, 0), 24));

    let outside = core::str::from_utf8(NETWORKS)
        .unwrap_or_else(|_| unreachable!("utf8"))
        .replace("10.0.0.3", "10.0.1.3");
    assert_eq!(
        decode_metadata_body(MetadataRoute::PrivateNetworks, outside.as_bytes()),
        Err(MetadataDecodeError::InconsistentNetwork)
    );
    let duplicate = core::str::from_utf8(NETWORKS)
        .unwrap_or_else(|_| unreachable!("utf8"))
        .replace(
            "  network_id: 1234",
            "  network_id: 1234\n  network_id: 1234",
        );
    assert_eq!(
        decode_metadata_body(MetadataRoute::PrivateNetworks, duplicate.as_bytes()),
        Err(MetadataDecodeError::DuplicateField)
    );
}

#[test]
fn endpoint_binding_rejects_every_destination_change() {
    let exact = FixedEndpoint(endpoint(EndpointScheme::Http, "169.254.169.254", 80, "/"));
    assert!(verify_metadata_endpoint(&exact).is_ok());
    for identity in [
        endpoint(EndpointScheme::Https, "169.254.169.254", 443, "/"),
        endpoint(EndpointScheme::Http, "169.254.169.253", 80, "/"),
        endpoint(EndpointScheme::Http, "169.254.169.254", 8080, "/"),
        endpoint(
            EndpointScheme::Http,
            "169.254.169.254",
            80,
            "/2009-04-04/meta-data",
        ),
    ] {
        assert!(verify_metadata_endpoint(&FixedEndpoint(identity)).is_err());
    }
}

#[test]
fn blocking_send_async_and_local_async_execute_the_same_exact_wire_request() {
    let executor = MetadataExecutor::new();
    execute_once_blocking(&executor);
    execute_once_async(&executor, false);
    execute_once_async(&executor, true);
    assert_eq!(executor.calls.load(Ordering::Acquire), 3);
}

#[test]
fn cancelled_metadata_execution_clears_staged_response_bytes() {
    let executor = PendingMetadataExecutor;
    let mut body = [0xa5; 32];
    let mut headers = [0xa5; 64];
    let mut response = ResponseBuffer::new(&mut body, 32, &mut headers);
    let future = execute_metadata_async(
        &executor,
        MetadataRequest::new(MetadataRoute::Hostname),
        response.writer(),
    );
    assert!(matches!(poll_once(future), Poll::Pending));
    assert!(response.writer().headers().is_empty());
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("response remained reusable"));
    assert!(
        attempt
            .body_mut()
            .is_ok_and(|output| output.iter().all(|byte| *byte == 0))
    );
}

fn execute_once_blocking(executor: &MetadataExecutor) {
    let mut body = [0xa5; MAX_METADATA_RESPONSE_BYTES];
    let mut headers = [0xa5; 128];
    let mut response = ResponseBuffer::new(&mut body, SUMMARY.len(), &mut headers);
    execute_metadata_blocking(
        executor,
        MetadataRequest::new(MetadataRoute::Summary),
        response.writer(),
    )
    .unwrap_or_else(|_| unreachable!("blocking metadata execution failed"));
    assert_summary(&response);
}

fn execute_once_async(executor: &MetadataExecutor, local: bool) {
    let mut body = [0xa5; MAX_METADATA_RESPONSE_BYTES];
    let mut headers = [0xa5; 128];
    let mut response = ResponseBuffer::new(&mut body, SUMMARY.len(), &mut headers);
    if local {
        let future = execute_metadata_local_async(
            executor,
            MetadataRequest::new(MetadataRoute::Summary),
            response.writer(),
        );
        assert!(matches!(poll_once(future), Poll::Ready(Ok(()))));
    } else {
        let future = execute_metadata_async(
            executor,
            MetadataRequest::new(MetadataRoute::Summary),
            response.writer(),
        );
        assert!(matches!(poll_once(future), Poll::Ready(Ok(()))));
    }
    assert_summary(&response);
}

fn assert_summary(response: &ResponseBuffer<'_>) {
    let decoded = response.with_response(|response| {
        decode_metadata_response(MetadataRoute::Summary, response)
            .map(|value| matches!(value, MetadataResponse::Summary(summary) if summary.instance_id() == 42))
    });
    assert_eq!(decoded, Ok(Ok(true)));
}

fn poll_once<F: Future>(future: F) -> Poll<F::Output> {
    let mut future = core::pin::pin!(future);
    Future::poll(future.as_mut(), &mut Context::from_waker(Waker::noop()))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MetadataExecutorError;

struct MetadataExecutor {
    calls: AtomicUsize,
}

impl MetadataExecutor {
    const fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
        }
    }

    fn validate(&self, request: TransportRequest<'_>) -> Result<(), MetadataExecutorError> {
        if request.method() != Method::Get
            || request.target().as_str() != "/hetzner/v1/metadata"
            || !request.body().is_empty()
            || !request.headers().as_slice().is_empty()
        {
            return Err(MetadataExecutorError);
        }
        self.calls.fetch_add(1, Ordering::AcqRel);
        Ok(())
    }
}

impl core::fmt::Display for MetadataExecutorError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("metadata test executor rejected request")
    }
}

impl core::error::Error for MetadataExecutorError {}

impl BoundTransport for MetadataExecutor {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        metadata_endpoint_identity()
    }
}

impl BlockingRawHttpExecutor for MetadataExecutor {
    type Error = MetadataExecutorError;

    fn execute(
        &self,
        request: TransportRequest<'_>,
        _policy: RawResponsePolicy<'_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        self.validate(request)?;
        let mut attempt = response
            .begin_attempt()
            .map_err(|_| MetadataExecutorError)?;
        attempt
            .body_mut()
            .map_err(|_| MetadataExecutorError)?
            .get_mut(..SUMMARY.len())
            .ok_or(MetadataExecutorError)?
            .copy_from_slice(SUMMARY);
        attempt
            .commit_completion(ResponseCompletion::new(
                StatusCode::OK,
                SUMMARY.len(),
                ResponseMetadata::EMPTY,
            ))
            .map_err(|_| MetadataExecutorError)
    }
}

impl AsyncRawHttpExecutor for MetadataExecutor {
    type Error = MetadataExecutorError;

    async fn execute<'executor, 'request, 'policy, 'writer, 'buffer>(
        &'executor self,
        request: TransportRequest<'request>,
        _policy: RawResponsePolicy<'policy>,
        mut response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'executor: 'writer,
        'request: 'writer,
        'policy: 'writer,
        'buffer: 'writer,
    {
        self.validate(request)?;
        response
            .body_mut()
            .map_err(|_| MetadataExecutorError)?
            .get_mut(..SUMMARY.len())
            .ok_or(MetadataExecutorError)?
            .copy_from_slice(SUMMARY);
        Ok(ResponseCompletion::new(
            StatusCode::OK,
            SUMMARY.len(),
            ResponseMetadata::EMPTY,
        ))
    }
}

struct PendingMetadataExecutor;

impl BoundTransport for PendingMetadataExecutor {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        metadata_endpoint_identity()
    }
}

impl AsyncRawHttpExecutor for PendingMetadataExecutor {
    type Error = MetadataExecutorError;

    async fn execute<'executor, 'request, 'policy, 'writer, 'buffer>(
        &'executor self,
        _request: TransportRequest<'request>,
        _policy: RawResponsePolicy<'policy>,
        mut response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'executor: 'writer,
        'request: 'writer,
        'policy: 'writer,
        'buffer: 'writer,
    {
        response
            .body_mut()
            .map_err(|_| MetadataExecutorError)?
            .get_mut(..6)
            .ok_or(MetadataExecutorError)?
            .copy_from_slice(b"secret");
        core::future::pending::<()>().await;
        Err(MetadataExecutorError)
    }
}

fn endpoint(
    scheme: EndpointScheme,
    host: &'static str,
    port: u16,
    path: &'static str,
) -> EndpointIdentity<'static> {
    EndpointIdentity::new(scheme, host, port, path)
        .unwrap_or_else(|_| unreachable!("valid test endpoint"))
}

struct FixedEndpoint(EndpointIdentity<'static>);

impl BoundTransport for FixedEndpoint {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        Ok(self.0)
    }
}
