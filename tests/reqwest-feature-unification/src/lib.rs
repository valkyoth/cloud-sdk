#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use cloud_sdk_reqwest::blocking::{
        BearerToken, BlockingClientBuilder, HttpsEndpoint, RequestTimeouts, UserAgent,
    };

    #[test]
    fn hardened_client_builds_with_hickory_and_http2_unified() {
        let endpoint = HttpsEndpoint::new("https://api.example.test/v1");
        let token = BearerToken::new("test-token");
        let user_agent = UserAgent::new("cloud-sdk-feature-unification-test/0.16");
        let timeouts = RequestTimeouts::new(Duration::from_secs(2), Duration::from_secs(1));
        assert!(endpoint.is_ok());
        assert!(token.is_ok());
        assert!(user_agent.is_ok());
        assert!(timeouts.is_ok());
        if let (Ok(endpoint), Ok(token), Ok(user_agent), Ok(timeouts)) =
            (endpoint, token, user_agent, timeouts)
        {
            assert!(
                BlockingClientBuilder::new(endpoint, token, user_agent, timeouts)
                    .build()
                    .is_ok()
            );
        }
    }
}
