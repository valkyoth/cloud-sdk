use super::*;
use crate::discovery::tests::Fixture as _;
use cloud_sdk::transport::{BoundTransport, BoundUserAgent, MAX_STREAM_BYTES};

fn identity() -> IdentifyingUserAgent<'static> {
    IdentifyingUserAgent::new("bundled-test/1 (test@example.org)").fixture("identity")
}
fn timeouts() -> RequestTimeouts {
    RequestTimeouts::new(
        core::time::Duration::from_secs(30),
        core::time::Duration::from_secs(5),
    )
    .fixture("timeouts")
}
fn verify(client: &(impl BoundTransport + BoundUserAgent), endpoint: OfficialCratesIoEndpoint) {
    assert_eq!(
        client.endpoint_identity().fixture("actual"),
        endpoint.identity().fixture("expected")
    );
    assert_eq!(
        client.configured_user_agent(),
        identity().as_str().as_bytes()
    );
}
#[cfg(feature = "blocking-rustls")]
#[test]
fn blocking_constructors_bind_three_distinct_authorities_without_io() {
    verify(
        &production_blocking(identity(), timeouts()).fixture("production"),
        OfficialCratesIoEndpoint::production_api(),
    );
    verify(
        &staging_blocking(identity(), timeouts()).fixture("staging"),
        OfficialCratesIoEndpoint::staging_api(),
    );
    verify(
        &ArtifactTransport::blocking(identity(), timeouts(), 1024).fixture("artifact"),
        OfficialCratesIoEndpoint::static_downloads(),
    );
    assert!(ArtifactTransport::blocking(identity(), timeouts(), 0).is_err());
    assert!(
        ArtifactTransport::blocking(
            identity(),
            timeouts(),
            MAX_STREAM_BYTES.checked_add(1).fixture("limit")
        )
        .is_err()
    );
}
#[cfg(feature = "async-rustls")]
#[test]
fn async_constructors_bind_three_distinct_authorities_without_io() {
    verify(
        &production_async(identity(), timeouts()).fixture("production"),
        OfficialCratesIoEndpoint::production_api(),
    );
    verify(
        &staging_async(identity(), timeouts()).fixture("staging"),
        OfficialCratesIoEndpoint::staging_api(),
    );
    verify(
        &ArtifactTransport::asynchronous(identity(), timeouts(), 1024).fixture("artifact"),
        OfficialCratesIoEndpoint::static_downloads(),
    );
    assert!(ArtifactTransport::asynchronous(identity(), timeouts(), 0).is_err());
    assert!(
        ArtifactTransport::asynchronous(
            identity(),
            timeouts(),
            MAX_STREAM_BYTES.checked_add(1).fixture("limit")
        )
        .is_err()
    );
}
