use cloud_sdk::transport::{EndpointIdentity, EndpointPolicy, EndpointScheme};

use super::test_timeouts;
use crate::asynchronous::{LinkLocalHttpEndpoint, RawLinkLocalAsyncClientBuilder, UserAgent};

#[test]
fn exact_link_local_endpoint_builds_only_the_raw_credential_free_client() {
    let identity = EndpointIdentity::new(EndpointScheme::Http, "169.254.169.254", 80, "/");
    let (Ok(identity), Some(timeouts), Ok(user_agent)) = (
        identity,
        test_timeouts(),
        UserAgent::new("cloud-sdk-metadata-test/0.97"),
    ) else {
        unreachable!("security fixture construction failed")
    };
    let endpoint = LinkLocalHttpEndpoint::new_with_policy(
        "http://169.254.169.254",
        EndpointPolicy::fixed(identity),
    );
    let Ok(endpoint) = endpoint else {
        unreachable!("security fixture construction failed")
    };
    assert!(
        RawLinkLocalAsyncClientBuilder::new(endpoint, user_agent, timeouts)
            .build()
            .is_ok()
    );
}
