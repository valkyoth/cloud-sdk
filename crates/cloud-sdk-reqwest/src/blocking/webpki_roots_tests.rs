use std::time::Duration;

use super::{BearerToken, BlockingClientBuilder, HttpsEndpoint, RequestTimeouts, UserAgent};

#[test]
fn deterministic_root_snapshot_is_nonempty_and_not_fips() {
    let result = super::config::test_webpki_roots_configuration();
    assert!(matches!(result, Ok((count, false)) if count > 0));
}

#[test]
fn deterministic_root_client_builds_without_platform_state() {
    let endpoint = HttpsEndpoint::new_custom("https://api.example.test");
    let token = BearerToken::new("test-token");
    let user_agent = UserAgent::new("cloud-sdk-test/0.24");
    let timeouts = RequestTimeouts::new(Duration::from_secs(2), Duration::from_secs(1));
    let (Ok(endpoint), Ok(token), Ok(user_agent), Ok(timeouts)) =
        (endpoint, token, user_agent, timeouts)
    else {
        return;
    };
    assert!(
        BlockingClientBuilder::new(endpoint, token, user_agent, timeouts)
            .build()
            .is_ok()
    );
}
